//! Dọn rác theo danh sách cố định do code định nghĩa. AI không tham gia quyết định xóa ở đây.

use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use serde::Serialize;

use crate::fsutil::{self, Stats};

struct Target {
    id: &'static str,
    name: &'static str,
    desc: &'static str,
    /// Bỏ qua file mới sửa gần đây (có thể đang được dùng).
    min_age_hours: u64,
    roots: fn() -> Vec<PathBuf>,
}

fn under(var: &str, rels: &[&str]) -> Vec<PathBuf> {
    // Thiếu biến môi trường thì bỏ qua cả nhóm, không bao giờ rơi về đường dẫn tương đối.
    std::env::var_os(var)
        .map(|base| rels.iter().map(|r| Path::new(&base).join(r)).collect())
        .unwrap_or_default()
}

fn subdirs(p: &Path) -> Vec<PathBuf> {
    fs::read_dir(p)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect()
        })
        .unwrap_or_default()
}

fn browser_caches() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for user_data in under(
        "LOCALAPPDATA",
        &[
            "Google\\Chrome\\User Data",
            "Microsoft\\Edge\\User Data",
            "BraveSoftware\\Brave-Browser\\User Data",
        ],
    ) {
        for profile in subdirs(&user_data) {
            out.extend(["Cache", "Code Cache", "GPUCache"].map(|c| profile.join(c)));
        }
    }
    for firefox in under("LOCALAPPDATA", &["Mozilla\\Firefox\\Profiles"]) {
        out.extend(subdirs(&firefox).into_iter().map(|p| p.join("cache2")));
    }
    out
}

const WER: [&str; 2] = [
    "Microsoft\\Windows\\WER\\ReportArchive",
    "Microsoft\\Windows\\WER\\ReportQueue",
];

const TARGETS: &[Target] = &[
    Target {
        id: "temp_user",
        name: "File tạm của bạn",
        desc: "Thư mục %TEMP%. Bỏ qua file sửa trong 24 giờ gần đây.",
        min_age_hours: 24,
        roots: || under("TEMP", &[""]),
    },
    Target {
        id: "temp_win",
        name: "File tạm của Windows",
        desc: "C:\\Windows\\Temp. Cần chạy app bằng quyền Administrator để xóa hết.",
        min_age_hours: 24,
        roots: || under("SystemRoot", &["Temp"]),
    },
    Target {
        id: "browser",
        name: "Cache trình duyệt",
        desc: "Chrome, Edge, Brave, Firefox. Không đụng tới mật khẩu, lịch sử, cookie. Nên đóng trình duyệt trước.",
        min_age_hours: 0,
        roots: browser_caches,
    },
    Target {
        id: "shader",
        name: "Cache shader (DirectX / NVIDIA / AMD)",
        desc: "Game tự tạo lại; lần đầu mở game sau khi dọn có thể hơi giật.",
        min_age_hours: 0,
        roots: || {
            under(
                "LOCALAPPDATA",
                &[
                    "D3DSCache",
                    "NVIDIA\\DXCache",
                    "NVIDIA\\GLCache",
                    "AMD\\DxCache",
                ],
            )
        },
    },
    Target {
        id: "crash",
        name: "Báo cáo lỗi & crash dump",
        desc: "File Windows ghi lại khi app bị crash.",
        min_age_hours: 0,
        roots: || {
            [
                under("LOCALAPPDATA", &["CrashDumps"]),
                under("LOCALAPPDATA", &WER),
                under("ProgramData", &WER),
            ]
            .concat()
        },
    },
    Target {
        id: "win_update",
        name: "File tải về của Windows Update",
        desc: "Bỏ qua file trong 7 ngày gần đây. Cần quyền Administrator.",
        min_age_hours: 24 * 7,
        roots: || under("SystemRoot", &["SoftwareDistribution\\Download"]),
    },
];

fn run(t: &Target, delete: bool) -> Stats {
    let mut st = Stats::default();
    let min_age = Duration::from_secs(t.min_age_hours * 3600);
    for root in (t.roots)().iter().filter(|r| fsutil::is_safe_root(r)) {
        fsutil::walk(root, min_age, delete, &mut st);
    }
    st
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JunkItem {
    id: &'static str,
    name: &'static str,
    desc: &'static str,
    #[serde(flatten)]
    stats: Stats,
}

#[tauri::command(async)]
pub fn scan_junk() -> Vec<JunkItem> {
    thread::scope(|s| {
        let handles: Vec<_> = TARGETS
            .iter()
            .map(|t| s.spawn(move || run(t, false)))
            .collect();
        TARGETS
            .iter()
            .zip(handles)
            .map(|(t, h)| JunkItem {
                id: t.id,
                name: t.name,
                desc: t.desc,
                stats: h.join().unwrap_or_default(),
            })
            .collect()
    })
}

/// `ids` đến từ giao diện: id lạ bị bỏ qua, chỉ dọn đúng các nhóm có trong TARGETS.
#[tauri::command(async)]
pub fn clean_junk(ids: Vec<String>) -> Stats {
    let mut total = Stats::default();
    for t in TARGETS.iter().filter(|t| ids.iter().any(|i| i == t.id)) {
        total.add(run(t, true));
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_root_is_safe() {
        for t in TARGETS {
            for r in (t.roots)() {
                assert!(fsutil::is_safe_root(&r), "{}: {}", t.id, r.display());
            }
        }
    }

    #[test]
    fn unknown_ids_clean_nothing() {
        assert_eq!(clean_junk(vec!["../../Windows".into()]), Stats::default());
    }
}
