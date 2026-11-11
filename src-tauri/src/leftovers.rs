//! Tìm và chuyển vào Thùng rác phần còn sót sau khi gỡ app.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use serde::Serialize;

use crate::{
    apps::{self, App},
    fsutil::{self, Stats},
    orphans,
};

const ROOT_VARS: [&str; 5] = [
    "LOCALAPPDATA",
    "APPDATA",
    "ProgramData",
    "ProgramFiles",
    "ProgramFiles(x86)",
];

fn roots() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = ROOT_VARS
        .iter()
        .filter_map(|v| std::env::var_os(v).map(PathBuf::from))
        .collect();
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        out.push(Path::new(&local).join("Programs"));
    }
    out
}

/// Chỉ cho xóa thư mục con trực tiếp của các root trên, hoặc "Vendor\App" trong Program Files.
fn allowed_location(p: &Path, roots: &[PathBuf]) -> bool {
    let parent = p.parent();
    let in_root = |x: Option<&Path>| x.is_some_and(|x| roots.iter().any(|r| r == x));
    let program_files: Vec<PathBuf> = ["ProgramFiles", "ProgramFiles(x86)"]
        .iter()
        .filter_map(|v| std::env::var_os(v).map(PathBuf::from))
        .collect();
    fsutil::is_safe_root(p)
        && (in_root(parent)
            || parent
                .and_then(Path::parent)
                .is_some_and(|gp| program_files.iter().any(|r| r == gp)))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Leftover {
    path: String,
    bytes: u64,
    /// None = được phép chuyển vào Thùng rác.
    blocked: Option<&'static str>,
}

/// `name`/`publisher`/`install_location` lấy từ danh sách app trước khi gỡ (lúc này app đã khỏi registry).
#[tauri::command(async)]
pub fn find_leftovers(
    name: String,
    publisher: Option<String>,
    install_location: Option<String>,
) -> Vec<Leftover> {
    let removed = App {
        id: String::new(),
        name,
        version: None,
        publisher,
        install_date: None,
        size_kb: 0,
        install_location,
        uninstall: None,
        visible: true,
    };
    let installed = apps::read_all();
    let running = orphans::running_exes();
    let rs = roots();
    let mut candidates: Vec<PathBuf> = rs
        .iter()
        .flat_map(|r| {
            std::fs::read_dir(r)
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.path())
        })
        .filter(|p| p.is_dir() && orphans::belongs(p, &removed))
        .collect();
    if let Some(loc) = removed
        .install_location
        .as_deref()
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
    {
        if !candidates.contains(&loc) {
            candidates.push(loc);
        }
    }
    candidates
        .into_iter()
        .map(|p| {
            let mut st = Stats::default();
            fsutil::walk(&p, Duration::ZERO, false, &mut st);
            let blocked = if allowed_location(&p, &rs) {
                orphans::block_reason(&p, &installed, &running, false)
            } else {
                Some("Nằm ngoài các thư mục app được phép dọn")
            };
            Leftover {
                path: p.to_string_lossy().into_owned(),
                bytes: st.bytes,
                blocked,
            }
        })
        .collect()
}

/// Kiểm tra lại toàn bộ quy tắc ở đây (không tin giao diện), rồi chuyển vào Thùng rác để còn khôi phục được.
#[tauri::command(async)]
pub fn delete_leftover(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.is_dir() || !allowed_location(&p, &roots()) {
        return Err("Đường dẫn không được phép xóa".into());
    }
    if let Some(reason) =
        orphans::block_reason(&p, &apps::read_all(), &orphans::running_exes(), false)
    {
        return Err(format!("Không xóa: {reason}"));
    }
    trash::delete(&p).map_err(|e| format!("Không chuyển được vào Thùng rác: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_direct_children_of_known_roots() {
        let roots = vec![PathBuf::from("C:\\Users\\a\\AppData\\Local")];
        assert!(allowed_location(
            Path::new("C:\\Users\\a\\AppData\\Local\\OldApp"),
            &roots
        ));
        assert!(!allowed_location(
            Path::new("C:\\Users\\a\\AppData\\Local\\OldApp\\sub"),
            &roots
        ));
        assert!(!allowed_location(
            Path::new("C:\\Users\\a\\AppData"),
            &roots
        ));
        assert!(!allowed_location(Path::new("D:\\Games\\X"), &roots));
    }

    #[test]
    fn delete_rejects_system_paths() {
        assert!(delete_leftover("C:\\Windows".into()).is_err());
        assert!(delete_leftover("C:\\Windows\\System32".into()).is_err());
    }
}
