//! Thống kê thư mục lớn trong AppData, đánh dấu đồ còn sót (theo quy tắc của code) và nhờ AI giải thích.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

use crate::{
    ai, apps,
    fsutil::{self, Stats},
    orphans,
    settings::Settings,
};

const ROOTS: [(&str, &str); 2] = [("Local", "LOCALAPPDATA"), ("Roaming", "APPDATA")];
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    path: String,
    name: String,
    root: &'static str,
    bytes: u64,
    /// None = đồ còn sót, được phép chuyển vào Thùng rác.
    blocked: Option<&'static str>,
}

#[tauri::command(async)]
pub fn appdata_folders() -> Vec<Folder> {
    let dirs: Vec<(&str, PathBuf)> = ROOTS
        .iter()
        .filter_map(|(label, var)| std::env::var_os(var).map(|b| (*label, PathBuf::from(b))))
        .flat_map(|(label, base)| {
            fs::read_dir(base)
                .into_iter()
                .flatten()
                .flatten()
                .map(move |e| (label, e.path()))
        })
        .filter(|(_, p)| p.is_dir())
        .collect();
    let mut out: Vec<Folder> = thread::scope(|s| {
        let handles: Vec<_> = dirs
            .iter()
            .map(|(_, p)| {
                s.spawn(move || {
                    let mut st = Stats::default();
                    fsutil::walk(p, Duration::ZERO, false, &mut st);
                    st.bytes
                })
            })
            .collect();
        dirs.iter()
            .zip(handles)
            .map(|((root, p), h)| Folder {
                path: p.to_string_lossy().into_owned(),
                name: p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                root,
                bytes: h.join().unwrap_or(0),
                blocked: None,
            })
            .collect()
    });
    out.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    out.truncate(20);
    let (apps, running) = (apps::read_all(), orphans::running_exes());
    for f in &mut out {
        f.blocked = orphans::block_reason(Path::new(&f.path), &apps, &running, true);
    }
    out
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    Cache,
    DuLieu,
    CauHinh,
    KhongRo,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
enum Safe {
    Co,
    Khong,
    KhongRo,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderVerdict {
    app: String,
    kind: Kind,
    safe_to_delete: Safe,
    reason: String,
}

/// Quy tắc của code ghi đè AI: thư mục Windows -> không; AI tự mâu thuẫn ("dữ liệu" mà "xóa được") -> không rõ.
fn apply_rules(name: &str, mut v: FolderVerdict) -> FolderVerdict {
    if orphans::is_protected(name) {
        v.safe_to_delete = Safe::Khong;
        v.reason = format!("Thư mục của Windows, không được xóa. {}", v.reason);
    } else if matches!(v.kind, Kind::DuLieu) && v.safe_to_delete == Safe::Co {
        v.safe_to_delete = Safe::KhongRo;
    }
    v
}

#[tauri::command]
pub async fn explain_folder(
    path: String,
    settings: State<'_, Mutex<Settings>>,
) -> Result<FolderVerdict, String> {
    let path = PathBuf::from(path);
    // Chỉ nhận thư mục con trực tiếp của AppData, không cho giao diện gửi đường dẫn tùy ý.
    let parent_ok = ROOTS.iter().any(|(_, var)| {
        std::env::var_os(var).is_some_and(|b| path.parent() == Some(Path::new(&b)))
    });
    if !parent_ok || !path.is_dir() {
        return Err("Đường dẫn không hợp lệ".into());
    }
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let children: Vec<String> = fs::read_dir(&path)
        .into_iter()
        .flatten()
        .flatten()
        .take(25)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let model = settings.lock().unwrap().ollama_model.clone();
    let prompt = format!(
        "Bạn là chuyên gia Windows. Cho biết thư mục AppData dưới đây thuộc app nào (app), loại dữ liệu \
(kind = cache / du_lieu / cau_hinh / khong_ro), xóa cả thư mục có an toàn không (safeToDelete = co / khong / khong_ro) \
và lý do ngắn (reason). Dữ liệu người dùng (game save, tin nhắn, ví, profile) thì không được xóa. {}\n\n\
--- DỮ LIỆU ---\nThư mục: {}\nCác mục con: {}",
        ai::BASE_RULES,
        path.display(),
        children.join(", ")
    );
    let schema = json!({
        "type": "object",
        "properties": {
            "app": { "type": "string" },
            "kind": { "enum": ["cache", "du_lieu", "cau_hinh", "khong_ro"] },
            "safeToDelete": { "enum": ["co", "khong", "khong_ro"] },
            "reason": { "type": "string" }
        },
        "required": ["app", "kind", "safeToDelete", "reason"]
    });
    let v: FolderVerdict = ai::generate(model, prompt, schema).await?;
    Ok(apply_rules(&name, v))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(kind: Kind, safe: Safe) -> FolderVerdict {
        FolderVerdict {
            app: "x".into(),
            kind,
            safe_to_delete: safe,
            reason: String::new(),
        }
    }

    #[test]
    fn rules_override_ai() {
        assert_eq!(
            apply_rules("Microsoft", v(Kind::Cache, Safe::Co)).safe_to_delete,
            Safe::Khong
        );
        assert_eq!(
            apply_rules("SomeGame", v(Kind::DuLieu, Safe::Co)).safe_to_delete,
            Safe::KhongRo
        );
        assert_eq!(
            apply_rules("SomeApp", v(Kind::Cache, Safe::Co)).safe_to_delete,
            Safe::Co
        );
    }
}
