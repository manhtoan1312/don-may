//! Luồng nền theo dõi CPU/RAM và bắn thông báo Windows khi vượt ngưỡng.

use std::{
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

use sysinfo::{Process, ProcessesToUpdate, System};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::settings::Settings;

const TICK: Duration = Duration::from_secs(2);

/// Theo dõi một chỉ số: chỉ báo khi vượt ngưỡng liên tục `sustain` và đã qua `cooldown` từ lần báo trước.
#[derive(Default)]
struct Tracker {
    over_since: Option<Instant>,
    last_alert: Option<Instant>,
}

impl Tracker {
    fn update(
        &mut self,
        value: f32,
        threshold: f32,
        now: Instant,
        sustain: Duration,
        cooldown: Duration,
    ) -> bool {
        if value < threshold {
            self.over_since = None;
            return false;
        }
        let since = *self.over_since.get_or_insert(now);
        let cooled = self.last_alert.is_none_or(|t| now - t >= cooldown);
        if now - since >= sustain && cooled {
            self.last_alert = Some(now);
            return true;
        }
        false
    }
}

fn top_by(sys: &System, key: impl Fn(&Process) -> f32) -> Option<(String, f32)> {
    sys.processes()
        .values()
        .filter(|p| p.pid().as_u32() != 0) // bỏ "System Idle Process"
        .map(|p| (p.name().to_string_lossy().into_owned(), key(p)))
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

pub fn notify(app: &AppHandle, title: &str, body: &str) {
    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        eprintln!("Không gửi được thông báo: {e}");
    }
}

#[tauri::command]
pub fn test_notification(app: AppHandle) {
    notify(&app, "Dọn Máy", "Thông báo hoạt động bình thường.");
}

pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        let mut sys = System::new();
        let (mut cpu, mut ram) = (Tracker::default(), Tracker::default());
        loop {
            thread::sleep(TICK);
            let s = app.state::<Mutex<Settings>>().lock().unwrap().clone();
            if !s.alerts_enabled {
                continue;
            }
            sys.refresh_cpu_usage();
            sys.refresh_memory();
            let cpu_pct = sys.global_cpu_usage();
            let ram_pct = sys.used_memory() as f32 / sys.total_memory().max(1) as f32 * 100.0;
            // Chỉ quét tiến trình khi đang quá tải, lúc bình thường luồng này gần như không tốn gì.
            if cpu_pct >= s.cpu_threshold || ram_pct >= s.ram_threshold {
                sys.refresh_processes(ProcessesToUpdate::All, true);
            }
            let now = Instant::now();
            let sustain = Duration::from_secs(s.sustain_secs);
            let cooldown = Duration::from_secs(s.cooldown_mins * 60);
            let cores = sys.cpus().len().max(1) as f32;

            if cpu.update(cpu_pct, s.cpu_threshold, now, sustain, cooldown) {
                let top = top_by(&sys, |p| p.cpu_usage() / cores)
                    .map(|(n, v)| format!("Nặng nhất: {n} ({v:.0}%)"))
                    .unwrap_or_default();
                notify(&app, &format!("CPU đang ở {cpu_pct:.0}%"), &top);
            }
            if ram.update(ram_pct, s.ram_threshold, now, sustain, cooldown) {
                let top = top_by(&sys, |p| p.memory() as f32)
                    .map(|(n, v)| format!("Nặng nhất: {n} ({:.1} GB)", v / 1_073_741_824.0))
                    .unwrap_or_default();
                notify(&app, &format!("RAM đang ở {ram_pct:.0}%"), &top);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alerts_only_after_sustain_and_respects_cooldown() {
        let (s, c) = (Duration::from_secs(10), Duration::from_secs(60));
        let t0 = Instant::now();
        let at = |secs| t0 + Duration::from_secs(secs);
        let mut t = Tracker::default();
        assert!(!t.update(95.0, 90.0, at(0), s, c)); // mới vượt
        assert!(!t.update(95.0, 90.0, at(5), s, c)); // chưa đủ 10s
        assert!(t.update(95.0, 90.0, at(10), s, c)); // báo
        assert!(!t.update(95.0, 90.0, at(30), s, c)); // đang cooldown
        assert!(!t.update(50.0, 90.0, at(40), s, c)); // hạ xuống, reset
        assert!(!t.update(95.0, 90.0, at(75), s, c)); // vượt lại, đếm lại từ đầu
        assert!(t.update(95.0, 90.0, at(85), s, c)); // đủ 10s và hết cooldown
    }
}
