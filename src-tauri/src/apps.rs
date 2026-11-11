//! Đọc danh sách app đã cài từ registry (giống "Apps & features" của Windows).

use std::collections::HashSet;

use serde::Serialize;
use winreg::{
    RegKey,
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ},
};

const UNINSTALL_ROOTS: [(&str, &str); 3] = [
    (
        "HKLM",
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
    ),
    (
        "HKLM",
        "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
    ),
    (
        "HKCU",
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
    ),
];

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct App {
    /// "HKLM|<đường dẫn root>|<tên key>", dùng để tra lại registry thay vì tin dữ liệu từ giao diện.
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub publisher: Option<String>,
    /// Dạng YYYYMMDD như Windows lưu.
    pub install_date: Option<String>,
    pub size_kb: u64,
    pub install_location: Option<String>,
    #[serde(skip)]
    pub uninstall: Option<String>,
    #[serde(skip)]
    pub visible: bool,
}

fn hive(name: &str) -> RegKey {
    RegKey::predef(if name == "HKLM" {
        HKEY_LOCAL_MACHINE
    } else {
        HKEY_CURRENT_USER
    })
}

fn get(k: &RegKey, name: &str) -> Option<String> {
    k.get_value::<String, _>(name)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Ẩn giống Windows: thành phần hệ thống, bản cập nhật, mục không có lệnh gỡ.
fn is_visible(
    uninstall: bool,
    system_component: u32,
    parent: bool,
    release_type: Option<&str>,
) -> bool {
    uninstall
        && system_component != 1
        && !parent
        && !matches!(release_type, Some("Update" | "Hotfix" | "Security Update"))
}

fn read_key(hive_name: &str, root: &str, key_name: &str) -> Option<App> {
    let k = hive(hive_name)
        .open_subkey_with_flags(format!("{root}\\{key_name}"), KEY_READ)
        .ok()?;
    let name = get(&k, "DisplayName")?;
    let uninstall = get(&k, "UninstallString");
    let visible = is_visible(
        uninstall.is_some(),
        k.get_value("SystemComponent").unwrap_or(0),
        get(&k, "ParentKeyName").is_some(),
        get(&k, "ReleaseType").as_deref(),
    );
    Some(App {
        id: format!("{hive_name}|{root}|{key_name}"),
        name,
        version: get(&k, "DisplayVersion"),
        publisher: get(&k, "Publisher"),
        install_date: get(&k, "InstallDate"),
        size_kb: k.get_value::<u32, _>("EstimatedSize").unwrap_or(0) as u64,
        install_location: get(&k, "InstallLocation").map(|p| p.trim_matches('"').to_string()),
        uninstall,
        visible,
    })
}

/// Tất cả mục trong registry, kể cả mục bị ẩn (dùng để đối chiếu đồ còn sót cho an toàn).
pub fn read_all() -> Vec<App> {
    UNINSTALL_ROOTS
        .iter()
        .flat_map(|(h, root)| {
            let keys: Vec<String> = hive(h)
                .open_subkey_with_flags(root, KEY_READ)
                .map(|k| k.enum_keys().flatten().collect())
                .unwrap_or_default();
            keys.into_iter()
                .filter_map(move |key| read_key(h, root, &key))
        })
        .collect()
}

/// Tra lại một app theo id; id phải trỏ vào đúng một trong các root Uninstall.
pub fn find(id: &str) -> Option<App> {
    let mut parts = id.splitn(3, '|');
    let (h, root, key) = (parts.next()?, parts.next()?, parts.next()?);
    let known = UNINSTALL_ROOTS
        .iter()
        .any(|(kh, kr)| *kh == h && *kr == root);
    (known && !key.contains('\\'))
        .then(|| read_key(h, root, key))
        .flatten()
}

#[tauri::command(async)]
pub fn list_apps() -> Vec<App> {
    let mut seen = HashSet::new();
    let mut apps: Vec<App> = read_all()
        .into_iter()
        .filter(|a| a.visible && seen.insert((a.name.clone(), a.version.clone())))
        .collect();
    apps.sort_by_key(|a| a.name.to_lowercase());
    apps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_like_windows() {
        assert!(is_visible(true, 0, false, None));
        assert!(!is_visible(false, 0, false, None));
        assert!(!is_visible(true, 1, false, None));
        assert!(!is_visible(true, 0, true, None));
        assert!(!is_visible(true, 0, false, Some("Hotfix")));
    }

    #[test]
    fn find_rejects_unknown_roots() {
        assert!(find("HKLM|SOFTWARE\\Evil|x").is_none());
        assert!(
            find("HKCU|SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall|a\\..\\b").is_none()
        );
        assert!(find("garbage").is_none());
    }

    #[test]
    fn reads_real_registry() {
        assert!(
            !list_apps().is_empty(),
            "máy Windows nào cũng có ít nhất một app"
        );
    }
}
