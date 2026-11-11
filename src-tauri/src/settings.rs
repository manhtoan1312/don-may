use std::{fs, path::PathBuf, sync::Mutex};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub alerts_enabled: bool,
    /// Ngưỡng % CPU / RAM.
    pub cpu_threshold: f32,
    pub ram_threshold: f32,
    /// Phải vượt ngưỡng liên tục bao lâu mới cảnh báo (tránh báo khi CPU chỉ nhảy vọt 1-2 giây).
    pub sustain_secs: u64,
    /// Khoảng cách tối thiểu giữa hai lần cảnh báo cùng loại.
    pub cooldown_mins: u64,
    pub ollama_model: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            alerts_enabled: true,
            cpu_threshold: 90.0,
            ram_threshold: 90.0,
            sustain_secs: 30,
            cooldown_mins: 10,
            ollama_model: "gemma4:e4b".into(),
        }
    }
}

impl Settings {
    /// Ép giá trị về khoảng hợp lệ, vì dữ liệu đến từ giao diện hoặc file có thể bị sửa tay.
    fn sanitized(mut self) -> Self {
        self.cpu_threshold = self.cpu_threshold.clamp(10.0, 100.0);
        self.ram_threshold = self.ram_threshold.clamp(10.0, 100.0);
        self.sustain_secs = self.sustain_secs.clamp(4, 600);
        self.cooldown_mins = self.cooldown_mins.clamp(1, 240);
        self.ollama_model = self.ollama_model.trim().to_string();
        if self.ollama_model.is_empty() {
            self.ollama_model = Settings::default().ollama_model;
        }
        self
    }
}

fn file_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| d.join("settings.json"))
}

pub fn load(app: &AppHandle) -> Settings {
    file_path(app)
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
        .unwrap_or_default()
        .sanitized()
}

#[tauri::command]
pub fn get_settings(state: State<Mutex<Settings>>) -> Settings {
    state.lock().unwrap().clone()
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<Mutex<Settings>>,
    settings: Settings,
) -> Result<Settings, String> {
    let settings = settings.sanitized();
    let path = file_path(&app).ok_or("Không tìm thấy thư mục cấu hình")?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| format!("Không lưu được cài đặt: {e}"))?;
    *state.lock().unwrap() = settings.clone();
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_clamps_and_fills_model() {
        let s = Settings {
            cpu_threshold: 500.0,
            ram_threshold: 1.0,
            sustain_secs: 0,
            ollama_model: "  ".into(),
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(
            (s.cpu_threshold, s.ram_threshold, s.sustain_secs),
            (100.0, 10.0, 4)
        );
        assert_eq!(s.ollama_model, "gemma4:e4b");
    }

    #[test]
    fn missing_fields_use_defaults() {
        let s: Settings = serde_json::from_str(r#"{"cpuThreshold": 70}"#).unwrap();
        assert_eq!((s.cpu_threshold, s.ram_threshold), (70.0, 90.0));
    }
}
