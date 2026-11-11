//! Chạy trình gỡ cài đặt gốc của app (giống bấm "Uninstall" trong Settings của Windows).

use std::{os::windows::process::CommandExt, process::Command};

use crate::apps;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Tách UninstallString thành (exe, tham số). Hỗ trợ đường dẫn có ngoặc kép và đường dẫn có dấu cách không ngoặc.
fn split_command(cmd: &str) -> Option<(String, String)> {
    let s = cmd.trim();
    if let Some(rest) = s.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some((rest[..end].to_string(), rest[end + 1..].trim().to_string()));
    }
    // to_ascii_lowercase giữ nguyên độ dài byte nên dùng được index cho chuỗi gốc.
    let end = s
        .to_ascii_lowercase()
        .find(".exe")
        .map(|i| i + 4)
        .or_else(|| s.find(' '))
        .unwrap_or(s.len());
    Some((s[..end].to_string(), s[end..].trim().to_string()))
}

/// MsiExec /I{GUID} là "sửa cài đặt"; đổi thành /X{GUID} để gỡ.
fn fix_msi(exe: &str, args: String) -> String {
    let is_msi = exe
        .to_ascii_lowercase()
        .trim_end_matches(".exe")
        .ends_with("msiexec");
    if is_msi && args.to_ascii_uppercase().starts_with("/I") {
        format!("/X{}", &args[2..])
    } else {
        args
    }
}

fn build(cmd: &str) -> Result<(String, String), String> {
    let (exe, args) = split_command(cmd).ok_or("Lệnh gỡ không hợp lệ")?;
    // Lệnh chạy qua cmd: chặn ký tự điều khiển để chuỗi trong registry không thể chèn thêm lệnh khác.
    if exe.is_empty() || exe.contains('"') || args.contains(['&', '|', '<', '>', '^', '\n']) {
        return Err("Lệnh gỡ có ký tự lạ. Hãy gỡ app này trong Settings của Windows.".into());
    }
    let args = fix_msi(&exe, args);
    Ok((exe, args))
}

/// Trả về true nếu app vẫn còn trong danh sách sau khi trình gỡ đóng
/// (người dùng bấm Hủy, hoặc trình gỡ vẫn chạy tiếp ở tiến trình khác).
#[tauri::command(async)]
pub fn uninstall_app(id: String) -> Result<bool, String> {
    let app = apps::find(&id).ok_or("Không tìm thấy app (có thể đã được gỡ)")?;
    let (exe, args) = build(app.uninstall.as_deref().ok_or("App không có lệnh gỡ")?)?;
    // `start /wait` dùng ShellExecute nên Windows tự hỏi quyền Administrator (UAC) khi trình gỡ cần.
    Command::new("cmd")
        .raw_arg(format!("/C start \"\" /wait \"{exe}\" {args}"))
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| format!("Không chạy được trình gỡ: {e}"))?;
    Ok(apps::find(&id).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_quoted_unquoted_and_msi() {
        assert_eq!(
            build(r#""C:\Program Files\App\unins000.exe" /SILENT"#).unwrap(),
            (
                r"C:\Program Files\App\unins000.exe".into(),
                "/SILENT".into()
            )
        );
        assert_eq!(
            build(r"C:\Program Files (x86)\Old App\uninstall.exe /S").unwrap(),
            (
                r"C:\Program Files (x86)\Old App\uninstall.exe".into(),
                "/S".into()
            )
        );
        assert_eq!(
            build("MsiExec.exe /I{1234-ABCD}").unwrap(),
            ("MsiExec.exe".into(), "/X{1234-ABCD}".into())
        );
    }

    #[test]
    fn rejects_command_injection() {
        assert!(build(r#""C:\a\unins.exe" & del C:\x"#).is_err());
        assert!(build(r"C:\a\unins.exe /S | calc").is_err());
    }
}
