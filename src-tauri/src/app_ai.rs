//! AI tư vấn có nên gỡ một app không. Quy tắc của code giữ lại runtime/driver bất kể AI nói gì.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

use crate::{ai, apps, settings::Settings};

/// Thư viện dùng chung hoặc driver: gỡ đi có thể làm app khác / phần cứng hỏng.
const KEEP: &[&str] = &[
    "visual c++",
    "redistributable",
    ".net",
    "runtime",
    "driver",
    "directx",
    "webview2",
    "chipset",
    "realtek",
    "intel(r)",
    "nvidia",
    "amd software",
    "windows sdk",
    "vulkan",
    "physx",
    "java",
];

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
enum Advice {
    Giu,
    CoTheGo,
    NenGo,
    KhongRo,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppVerdict {
    purpose: String,
    advice: Advice,
    reason: String,
    #[serde(default)]
    overridden: bool,
}

fn apply_rules(name: &str, mut v: AppVerdict) -> AppVerdict {
    let lower = name.to_lowercase();
    if v.advice != Advice::Giu && KEEP.iter().any(|k| lower.contains(k)) {
        v.advice = Advice::Giu;
        v.overridden = true;
        v.reason = format!(
            "Thư viện dùng chung / driver: app khác hoặc phần cứng có thể cần. {}",
            v.reason
        );
    }
    v
}

#[tauri::command]
pub async fn explain_app(
    id: String,
    settings: State<'_, Mutex<Settings>>,
) -> Result<AppVerdict, String> {
    let app = apps::find(&id).ok_or("Không tìm thấy app")?;
    let model = settings.lock().unwrap().ollama_model.clone();
    let prompt = format!(
        "Bạn là chuyên gia Windows. Cho biết app dưới đây dùng để làm gì (purpose), lời khuyên \
(advice = giu / co_the_go / nen_go / khong_ro) và lý do ngắn (reason). nen_go chỉ dành cho bloatware, \
adware, phần mềm dùng thử cài kèm. {}\n\n--- DỮ LIỆU ---\nTên: {}\nNhà phát hành: {}\nPhiên bản: {}\nThư mục cài: {}",
        ai::BASE_RULES,
        app.name,
        app.publisher.as_deref().unwrap_or("không rõ"),
        app.version.as_deref().unwrap_or("không rõ"),
        app.install_location.as_deref().unwrap_or("không rõ"),
    );
    let schema = json!({
        "type": "object",
        "properties": {
            "purpose": { "type": "string" },
            "advice": { "enum": ["giu", "co_the_go", "nen_go", "khong_ro"] },
            "reason": { "type": "string" }
        },
        "required": ["purpose", "advice", "reason"]
    });
    let mut v: AppVerdict = ai::generate(model, prompt, schema).await?;
    v.overridden = false; // chỉ code được bật cờ này
    Ok(apply_rules(&app.name, v))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(advice: Advice) -> AppVerdict {
        AppVerdict {
            purpose: String::new(),
            advice,
            reason: String::new(),
            overridden: false,
        }
    }

    #[test]
    fn keeps_runtimes_and_drivers() {
        let r = apply_rules(
            "Microsoft Visual C++ 2015-2022 Redistributable (x64)",
            v(Advice::NenGo),
        );
        assert!(r.advice == Advice::Giu && r.overridden);
        assert_eq!(
            apply_rules("NVIDIA Graphics Driver 581.42", v(Advice::CoTheGo)).advice,
            Advice::Giu
        );
        let r = apply_rules("McAfee WebAdvisor", v(Advice::NenGo));
        assert!(r.advice == Advice::NenGo && !r.overridden);
    }
}
