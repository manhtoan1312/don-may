//! AI đánh giá một tiến trình. Code tự thu thập dữ liệu, AI chỉ đánh giá, quy tắc có quyền nâng mức.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::json;
use sysinfo::{Pid, System};
use tauri::State;

use crate::{ai, processes, settings::Settings, suspicious};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    Thap,
    TrungBinh,
    Cao,
}

#[derive(Deserialize)]
struct AiAnswer {
    program: String,
    risk: Risk,
    reasons: Vec<String>,
    advice: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Verdict {
    /// Đúng dữ liệu đã gửi cho AI, để người dùng tự đối chiếu.
    info: String,
    program: String,
    risk: Risk,
    ai_risk: Risk,
    reasons: Vec<String>,
    advice: String,
}

/// Mức tối thiểu theo quy tắc: dấu hiệu mạnh -> cao, có dấu hiệu bất kỳ -> trung bình.
fn rule_floor(flags: &[&str]) -> Risk {
    if flags.iter().any(|f| suspicious::is_strong(f)) {
        Risk::Cao
    } else if flags.is_empty() {
        Risk::Thap
    } else {
        Risk::TrungBinh
    }
}

fn describe(p: &processes::ProcInfo, parent: Option<&str>) -> String {
    format!(
        "Tên: {}\nĐường dẫn: {}\nDòng lệnh: {}\nTiến trình cha: {}\nCPU: {:.1}%, RAM: {:.0} MB\nDấu hiệu app phát hiện: {}",
        p.name,
        p.exe.as_deref().unwrap_or("không đọc được"),
        if p.cmd.is_empty() {
            "(trống)"
        } else {
            &p.cmd
        },
        parent.unwrap_or("không rõ"),
        p.cpu,
        p.memory as f64 / 1_048_576.0,
        if p.flags.is_empty() {
            "không có".to_string()
        } else {
            p.flags.join("; ")
        },
    )
}

#[tauri::command]
pub async fn analyze_process(
    pid: u32,
    sys: State<'_, Mutex<System>>,
    settings: State<'_, Mutex<Settings>>,
) -> Result<Verdict, String> {
    let (info, floor) = {
        let sys = sys.lock().unwrap();
        let p = sys
            .process(Pid::from_u32(pid))
            .ok_or("Tiến trình đã thoát")?;
        let parent = p
            .parent()
            .and_then(|pp| sys.process(pp))
            .map(|pp| pp.name().to_string_lossy().into_owned());
        let info = processes::proc_info(p, &sys);
        (describe(&info, parent.as_deref()), rule_floor(&info.flags))
    };
    let model = settings.lock().unwrap().ollama_model.clone();
    let prompt = format!(
        "Bạn là chuyên gia bảo mật Windows. Đánh giá tiến trình dưới đây: program = đây có thể là chương trình gì; \
risk = thap / trung_binh / cao; reasons = 1-3 lý do ngắn; advice = người dùng nên làm gì. {}\n\n--- DỮ LIỆU ---\n{info}",
        ai::BASE_RULES
    );
    let schema = json!({
        "type": "object",
        "properties": {
            "program": { "type": "string" },
            "risk": { "enum": ["thap", "trung_binh", "cao"] },
            "reasons": { "type": "array", "items": { "type": "string" } },
            "advice": { "type": "string" }
        },
        "required": ["program", "risk", "reasons", "advice"]
    });
    let a: AiAnswer = ai::generate(model, prompt, schema).await?;
    Ok(Verdict {
        info,
        program: a.program,
        risk: a.risk.max(floor),
        ai_risk: a.risk,
        reasons: a.reasons,
        advice: a.advice,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_raise_ai_risk() {
        assert_eq!(rule_floor(&[]), Risk::Thap);
        assert_eq!(rule_floor(&[suspicious::RISKY_DIR]), Risk::TrungBinh);
        assert_eq!(
            rule_floor(&[suspicious::RISKY_DIR, suspicious::IMPERSONATE]),
            Risk::Cao
        );
        assert_eq!(Risk::Thap.max(Risk::Cao), Risk::Cao);
    }

    #[test]
    fn ai_answer_rejects_unknown_risk() {
        let bad = r#"{"program":"x","risk":"rat_cao","reasons":[],"advice":""}"#;
        assert!(serde_json::from_str::<AiAnswer>(bad).is_err());
    }
}
