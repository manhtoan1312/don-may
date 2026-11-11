//! AI giải thích một file lớn (chỉ dựa vào tên, đường dẫn, kích thước, ngày sửa; không đọc nội dung).

use std::{fs, path::PathBuf, sync::Mutex};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

use crate::{
    ai,
    file_rules::{self, Verdict},
    fsutil,
    settings::Settings,
};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
enum Safe {
    Co,
    Khong,
    KhongRo,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileVerdict {
    what: String,
    safe_to_delete: Safe,
    reason: String,
}

#[tauri::command]
pub async fn explain_file(
    path: String,
    settings: State<'_, Mutex<Settings>>,
) -> Result<FileVerdict, String> {
    let p = PathBuf::from(&path);
    let meta = fs::metadata(&p).map_err(|_| "Không tìm thấy file")?;
    let age = fsutil::age_days(&meta);
    let (rule, rule_reason) = file_rules::classify(&p, meta.len(), age, &file_rules::system_dirs());
    let model = settings.lock().unwrap().ollama_model.clone();
    let prompt = format!(
        "Bạn là chuyên gia Windows. Dựa vào tên và vị trí file, cho biết đây là file gì (what), xóa có an toàn \
không (safeToDelete = co / khong / khong_ro) và lý do ngắn (reason). File cá nhân (ảnh, video, tài liệu tự tạo) \
thì chọn khong_ro vì chỉ người dùng biết có cần không. {}\n\n--- DỮ LIỆU ---\nĐường dẫn: {}\nKích thước: {:.1} MB\n\
Sửa lần cuối: {} ngày trước\nLoại: {}",
        ai::BASE_RULES,
        p.display(),
        meta.len() as f64 / 1_048_576.0,
        age,
        file_rules::kind_of(&p),
    );
    let schema = json!({
        "type": "object",
        "properties": {
            "what": { "type": "string" },
            "safeToDelete": { "enum": ["co", "khong", "khong_ro"] },
            "reason": { "type": "string" }
        },
        "required": ["what", "safeToDelete", "reason"]
    });
    let mut v: FileVerdict = ai::generate(model, prompt, schema).await?;
    if rule == Verdict::KhongDuocXoa {
        v.safe_to_delete = Safe::Khong;
        v.reason = format!("{rule_reason} {}", v.reason);
    }
    Ok(v)
}
