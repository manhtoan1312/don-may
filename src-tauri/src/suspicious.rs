//! Heuristic gắn cờ tiến trình đáng ngờ. Chỉ gắn cờ, không tự dừng tiến trình.

/// Tên tiến trình hệ thống Windows hay bị malware mạo danh.
const SYSTEM_NAMES: &[&str] = &[
    "svchost.exe",
    "lsass.exe",
    "csrss.exe",
    "winlogon.exe",
    "services.exe",
    "smss.exe",
    "wininit.exe",
    "spoolsv.exe",
    "taskhostw.exe",
    "dwm.exe",
    "explorer.exe",
    "conhost.exe",
    "rundll32.exe",
];

/// Dừng các tiến trình này sẽ gây treo hoặc màn hình xanh.
const CRITICAL: &[&str] = &[
    "system",
    "registry",
    "memory compression",
    "csrss.exe",
    "lsass.exe",
    "smss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
];

const RISKY_DIRS: &[&str] = &[
    "\\appdata\\local\\temp\\",
    "\\windows\\temp\\",
    "\\downloads\\",
    "\\users\\public\\",
    "$recycle.bin",
];

const DOC_EXTS: &[&str] = &[
    ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".jpg", ".png", ".txt", ".mp4",
];

pub const DOUBLE_EXT: &str = "Đuôi file kép (giả dạng tài liệu)";
pub const EXE_MISSING: &str = "File thực thi đã bị xóa khỏi ổ đĩa";
pub const RISKY_DIR: &str = "Chạy từ thư mục tạm / Downloads / Public";
pub const IMPERSONATE: &str = "Mạo danh tên tiến trình hệ thống";

/// Các dấu hiệu gần như chắc chắn là bất thường: AI không được hạ mức xuống dưới "cao".
pub fn is_strong(flag: &str) -> bool {
    [DOUBLE_EXT, EXE_MISSING, IMPERSONATE].contains(&flag)
}

pub fn is_critical(name: &str) -> bool {
    CRITICAL.contains(&name.to_lowercase().as_str())
}

pub fn flags(name: &str, exe: Option<&str>, exe_missing: bool, windir: &str) -> Vec<&'static str> {
    let name = name.to_lowercase();
    let mut out = Vec::new();
    if let Some(stem) = name
        .strip_suffix(".exe")
        .or_else(|| name.strip_suffix(".scr"))
    {
        if DOC_EXTS.iter().any(|e| stem.ends_with(e)) {
            out.push(DOUBLE_EXT);
        }
    }
    // Không đọc được đường dẫn thường là tiến trình hệ thống được bảo vệ, bỏ qua.
    let Some(exe) = exe.map(str::to_lowercase) else {
        return out;
    };
    if exe_missing {
        out.push(EXE_MISSING);
    }
    if RISKY_DIRS.iter().any(|d| exe.contains(d)) {
        out.push(RISKY_DIR);
    }
    if SYSTEM_NAMES.contains(&name.as_str()) && !exe.starts_with(&windir.to_lowercase()) {
        out.push(IMPERSONATE);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    const WIN: &str = "C:\\Windows";

    #[test]
    fn legit_svchost_is_clean() {
        assert!(
            flags(
                "svchost.exe",
                Some("C:\\Windows\\System32\\svchost.exe"),
                false,
                WIN
            )
            .is_empty()
        );
    }

    #[test]
    fn fake_svchost_in_temp() {
        let f = flags(
            "svchost.exe",
            Some("C:\\Users\\a\\AppData\\Local\\Temp\\svchost.exe"),
            false,
            WIN,
        );
        assert_eq!(f.len(), 2);
    }

    #[test]
    fn double_extension_and_missing_file() {
        let f = flags("invoice.pdf.exe", Some("D:\\x\\invoice.pdf.exe"), true, WIN);
        assert_eq!(f, vec![DOUBLE_EXT, EXE_MISSING]);
    }

    #[test]
    fn critical_names() {
        assert!(is_critical("LSASS.EXE") && is_critical("System") && !is_critical("chrome.exe"));
    }
}
