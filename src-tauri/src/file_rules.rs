//! Quy tắc đánh giá một file có xóa được không. Code quyết định; AI chỉ giải thích file "không rõ".

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::fsutil;

#[derive(Serialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    KhongDuocXoa,
    CoTheXoa,
    CanNhac,
    KhongRo,
}

const INSTALLER_LIKE: &[&str] = &["Bộ cài", "File nén", "Ảnh đĩa / máy ảo"];

pub fn kind_of(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" | "flv" | "m4v" => "Video",
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "heic" | "bmp" | "raw" | "psd" => "Ảnh",
        "mp3" | "flac" | "wav" | "m4a" | "aac" | "ogg" => "Nhạc",
        "zip" | "rar" | "7z" | "tar" | "gz" | "xz" => "File nén",
        "exe" | "msi" | "msix" | "appx" => "Bộ cài",
        "iso" | "img" | "vhd" | "vhdx" | "vmdk" | "vdi" => "Ảnh đĩa / máy ảo",
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" => "Tài liệu",
        _ => "Khác",
    }
}

/// Thư mục hệ thống lấy từ biến môi trường (chữ thường).
pub fn system_dirs() -> Vec<String> {
    [
        "SystemRoot",
        "ProgramFiles",
        "ProgramFiles(x86)",
        "ProgramData",
    ]
    .iter()
    .filter_map(|v| std::env::var(v).ok())
    .map(|d| d.to_lowercase())
    .collect()
}

pub fn classify(
    path: &Path,
    bytes: u64,
    age_days: u64,
    system: &[String],
) -> (Verdict, &'static str) {
    let p = path.to_string_lossy().to_lowercase();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let in_dir = |d: &str| p.starts_with(&format!("{}\\", d.trim_end_matches('\\')));
    match name.as_str() {
        "hiberfil.sys" => {
            return (
                Verdict::KhongDuocXoa,
                "File ngủ đông. Muốn bỏ thì chạy 'powercfg /h off' bằng quyền Administrator, Windows sẽ tự xóa.",
            );
        }
        "pagefile.sys" | "swapfile.sys" => {
            return (
                Verdict::KhongDuocXoa,
                "Bộ nhớ ảo của Windows. Chỉ chỉnh trong System Properties > Virtual memory.",
            );
        }
        _ => {}
    }
    // Thư mục hệ thống ở gốc mọi ổ, ví dụ "D:\System Volume Information\...".
    let root_system = [
        "system volume information",
        "$recycle.bin",
        "recovery",
        "boot",
    ]
    .iter()
    .any(|d| p.contains(&format!(":\\{d}\\")) || p.ends_with(&format!(":\\{d}")));
    if system.iter().any(|d| in_dir(d)) || root_system {
        return (
            Verdict::KhongDuocXoa,
            "Thuộc Windows hoặc app đã cài. Muốn bỏ app thì dùng tab Gỡ app.",
        );
    }
    if p.contains("\\appdata\\") {
        return (
            Verdict::CanNhac,
            "Dữ liệu của ứng dụng. Xem tab Dọn rác > AppData.",
        );
    }
    if p.contains("\\downloads\\") && INSTALLER_LIKE.contains(&kind_of(path)) && age_days > 30 {
        return (
            Verdict::CoTheXoa,
            "Bộ cài / file nén cũ hơn 30 ngày trong Downloads, có thể tải lại khi cần.",
        );
    }
    if age_days > 180 && bytes > 500 * 1024 * 1024 {
        return (Verdict::CanNhac, "File lớn, hơn 6 tháng không sửa.");
    }
    (
        Verdict::KhongRo,
        "Chưa có quy tắc cho file này, hãy hỏi AI.",
    )
}

/// Chuyển một file vào Thùng rác, sau khi kiểm tra lại quy tắc ở backend.
#[tauri::command(async)]
pub fn delete_file(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    let meta = fs::symlink_metadata(&p).map_err(|_| "Không tìm thấy file")?;
    if !p.is_absolute() || !meta.is_file() {
        return Err("Chỉ xóa được file thường".into());
    }
    let (verdict, reason) = classify(&p, meta.len(), fsutil::age_days(&meta), &system_dirs());
    if verdict == Verdict::KhongDuocXoa {
        return Err(reason.into());
    }
    trash::delete(&p).map_err(|e| format!("Không chuyển được vào Thùng rác: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sys() -> Vec<String> {
        vec!["c:\\windows".into(), "c:\\program files".into()]
    }
    fn v(p: &str, bytes: u64, age: u64) -> Verdict {
        classify(Path::new(p), bytes, age, &sys()).0
    }

    #[test]
    fn classifies() {
        assert_eq!(v("C:\\hiberfil.sys", 1 << 33, 0), Verdict::KhongDuocXoa);
        assert_eq!(
            v("C:\\Windows\\System32\\big.dll", 1 << 30, 999),
            Verdict::KhongDuocXoa
        );
        assert_eq!(
            v("C:\\Program Files\\App\\a.bin", 1, 999),
            Verdict::KhongDuocXoa
        );
        assert_eq!(
            v("C:\\System Volume Information\\x", 1, 0),
            Verdict::KhongDuocXoa
        );
        assert_eq!(
            v("C:\\Users\\a\\AppData\\Local\\x.db", 1, 0),
            Verdict::CanNhac
        );
        assert_eq!(
            v("C:\\Users\\a\\Downloads\\setup.exe", 1, 40),
            Verdict::CoTheXoa
        );
        assert_eq!(
            v("C:\\Users\\a\\Downloads\\setup.exe", 1, 3),
            Verdict::KhongRo,
            "file mới tải chưa được gợi ý xóa"
        );
        assert_eq!(v("D:\\Movies\\old.mkv", 1 << 30, 400), Verdict::CanNhac);
        assert_eq!(v("D:\\Movies\\new.mkv", 1 << 30, 10), Verdict::KhongRo);
        // "C:\Windowsxyz" không được coi là thư mục Windows.
        assert_eq!(v("C:\\Windowsxyz\\a.bin", 1, 0), Verdict::KhongRo);
    }

    #[test]
    fn kinds() {
        assert_eq!(kind_of(Path::new("a.MKV")), "Video");
        assert_eq!(kind_of(Path::new("noext")), "Khác");
    }

    #[test]
    fn delete_refuses_system_files() {
        let windir = std::env::var("SystemRoot").unwrap();
        assert!(delete_file(format!("{windir}\\explorer.exe")).is_err());
        assert!(delete_file("relative.txt".into()).is_err());
    }
}
