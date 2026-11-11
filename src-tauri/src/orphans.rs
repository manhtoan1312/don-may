//! Xác định thư mục còn sót của app đã gỡ. Code quyết định; nghi ngờ thì luôn coi là "có chủ" (không cho xóa).

use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use sysinfo::{ProcessesToUpdate, System};

use crate::apps::App;

/// Thư mục của Windows/hệ thống: không bao giờ được xóa (cộng với mọi thứ bắt đầu bằng "microsoft"/"windows").
const PROTECTED: &[&str] = &[
    "packages",
    "programs",
    "temp",
    "comms",
    "connecteddevicesplatform",
    "common files",
    "internet explorer",
    "dotnet",
    "package cache",
    "reference assemblies",
    "msbuild",
    "uninstall information",
    "modifiablewindowsapps",
    "virtualstore",
    "placeholdertilelogofolder",
    "publishers",
    "voiceaccess",
    "application data",
    "history",
    "temporary internet files",
];
pub fn is_protected(name: &str) -> bool {
    let lower = name.to_lowercase();
    PROTECTED.contains(&lower.as_str())
        || lower.starts_with("microsoft")
        || lower.starts_with("windows")
}

const RECENT: Duration = Duration::from_secs(30 * 24 * 3600);

fn norm(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn name_of(p: &Path) -> String {
    p.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// Thư mục có thể thuộc về app này không (thiên về "có", để không xóa nhầm).
pub fn belongs(folder: &Path, app: &App) -> bool {
    let f = norm(&name_of(folder));
    if f.len() < 3 {
        return true; // tên quá ngắn, không đoán được
    }
    let loc = app.install_location.as_deref().map(|l| l.to_lowercase());
    let folder_l = folder.to_string_lossy().to_lowercase();
    let n = norm(&app.name);
    let p = norm(app.publisher.as_deref().unwrap_or(""));
    // "docker-secrets-engine" -> token "docker" khớp app "Docker Desktop".
    let token_hit = name_of(folder)
        .split(|c: char| !c.is_alphanumeric())
        .map(str::to_lowercase)
        .filter(|t| t.len() >= 4)
        .any(|t| n.starts_with(&t) || p.starts_with(&t));
    loc.is_some_and(|l| !l.is_empty() && (l.starts_with(&folder_l) || folder_l.starts_with(&l)))
        || token_hit
        || n.contains(&f)
        || (p.len() >= 3 && (p.contains(&f) || f.contains(&p)))
        || (n.len() >= 4 && f.contains(&n))
}

/// Sửa đổi gần đây ở thư mục hoặc mục con trực tiếp -> vẫn đang được dùng.
fn recently_used(folder: &Path) -> bool {
    let fresh = |p: &Path| {
        p.metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| SystemTime::now().duration_since(t).ok())
            .is_some_and(|age| age < RECENT)
    };
    fresh(folder)
        || std::fs::read_dir(folder)
            .into_iter()
            .flatten()
            .flatten()
            .any(|e| fresh(&e.path()))
}

pub fn running_exes() -> Vec<PathBuf> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    sys.processes()
        .values()
        .filter_map(|p| p.exe().map(Path::to_path_buf))
        .collect()
}

/// Lý do không được xóa, hoặc None nếu là đồ còn sót đủ điều kiện chuyển vào Thùng rác.
/// `check_recent` = false khi vừa gỡ app: thư mục của app vừa gỡ đương nhiên mới được dùng gần đây.
pub fn block_reason(
    folder: &Path,
    apps: &[App],
    running: &[PathBuf],
    check_recent: bool,
) -> Option<&'static str> {
    let is_link = std::fs::symlink_metadata(folder).is_ok_and(|m| m.file_type().is_symlink());
    if is_link {
        Some("Là liên kết (junction) của Windows")
    } else if is_protected(&name_of(folder)) {
        Some("Thư mục của Windows")
    } else if apps.iter().any(|a| belongs(folder, a)) {
        Some("Khớp với một app đang cài")
    } else if running.iter().any(|e| e.starts_with(folder)) {
        Some("Có chương trình đang chạy từ thư mục này")
    } else if check_recent && recently_used(folder) {
        Some("Mới được sửa trong 30 ngày gần đây")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(name: &str, publisher: Option<&str>, loc: Option<&str>) -> App {
        App {
            id: String::new(),
            name: name.into(),
            version: None,
            publisher: publisher.map(Into::into),
            install_date: None,
            size_kb: 0,
            install_location: loc.map(Into::into),
            uninstall: None,
            visible: true,
        }
    }

    #[test]
    fn matches_by_name_publisher_and_location() {
        let base = Path::new("C:\\Users\\a\\AppData\\Local");
        assert!(belongs(
            &base.join("Google"),
            &app("Google Chrome", Some("Google LLC"), None)
        ));
        assert!(belongs(
            &base.join("Discord"),
            &app("Discord", Some("Discord Inc."), None)
        ));
        assert!(belongs(
            &base.join("Programs\\VSCode"),
            &app(
                "Code",
                None,
                Some("C:\\Users\\a\\AppData\\Local\\Programs\\VSCode")
            )
        ));
        assert!(
            belongs(&base.join("X"), &app("Anything", None, None)),
            "tên quá ngắn phải coi là có chủ"
        );
        assert!(!belongs(
            &base.join("OldGameStudio"),
            &app("Google Chrome", Some("Google LLC"), None)
        ));
    }

    #[test]
    fn blocks_protected_and_running() {
        let base = Path::new("C:\\Users\\a\\AppData\\Local");
        assert_eq!(
            block_reason(&base.join("Microsoft"), &[], &[], true),
            Some("Thư mục của Windows")
        );
        let running = vec![base.join("Leftover\\run.exe")];
        assert_eq!(
            block_reason(&base.join("Leftover"), &[], &running, true),
            Some("Có chương trình đang chạy từ thư mục này")
        );
    }

    #[test]
    fn real_windows_junction_is_blocked() {
        // Windows tạo sẵn junction "Application Data" trong LocalAppData.
        let j = Path::new(&std::env::var_os("LOCALAPPDATA").unwrap()).join("Application Data");
        if j.exists() {
            assert_eq!(
                block_reason(&j, &[], &[], false),
                Some("Là liên kết (junction) của Windows")
            );
        }
    }
}
