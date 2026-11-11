//! Thống kê ổ đĩa / thư mục: thư mục con nặng nhất, 50 file lớn nhất, dung lượng theo loại file.

use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};

use serde::Serialize;
use sysinfo::Disks;

use crate::{
    file_rules::{self, Verdict},
    fsutil,
};

const TOP_FILES: usize = 50;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Drive {
    mount: String,
    name: String,
    total: u64,
    available: u64,
}

#[tauri::command(async)]
pub fn list_drives() -> Vec<Drive> {
    Disks::new_with_refreshed_list()
        .iter()
        .map(|d| Drive {
            mount: d.mount_point().to_string_lossy().into_owned(),
            name: d.name().to_string_lossy().into_owned(),
            total: d.total_space(),
            available: d.available_space(),
        })
        .collect()
}

#[derive(Default)]
struct Acc {
    bytes: u64,
    top: BinaryHeap<Reverse<(u64, PathBuf)>>,
    types: HashMap<&'static str, (u64, u64)>,
}

impl Acc {
    fn add(&mut self, path: &Path, len: u64) {
        self.bytes += len;
        let t = self.types.entry(file_rules::kind_of(path)).or_default();
        t.0 += len;
        t.1 += 1;
        self.push_top(len, path.to_path_buf());
    }
    fn push_top(&mut self, len: u64, path: PathBuf) {
        if self.top.len() < TOP_FILES || self.top.peek().is_some_and(|Reverse((b, _))| len > *b) {
            self.top.push(Reverse((len, path)));
            if self.top.len() > TOP_FILES {
                self.top.pop();
            }
        }
    }
    fn merge(&mut self, o: Acc) {
        self.bytes += o.bytes;
        for (k, (b, n)) in o.types {
            let t = self.types.entry(k).or_default();
            t.0 += b;
            t.1 += n;
        }
        for Reverse((len, p)) in o.top {
            self.push_top(len, p);
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Child {
    path: String,
    name: String,
    is_dir: bool,
    bytes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BigFile {
    path: String,
    bytes: u64,
    age_days: u64,
    verdict: Verdict,
    reason: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scan {
    path: String,
    total: u64,
    children: Vec<Child>,
    big_files: Vec<BigFile>,
    types: Vec<(&'static str, u64, u64)>,
}

fn walk_dir(dir: &Path) -> Acc {
    let mut acc = Acc::default();
    fsutil::visit_files(dir, &mut |p, m| acc.add(p, m.len()));
    acc
}

#[tauri::command(async)]
pub fn scan_path(path: String) -> Result<Scan, String> {
    let root = PathBuf::from(&path);
    if !root.is_absolute() || !root.is_dir() {
        return Err("Thư mục không hợp lệ".into());
    }
    let mut total = Acc::default();
    let mut children = Vec::new();
    let mut dirs = Vec::new();
    for e in fs::read_dir(&root).map_err(|e| e.to_string())?.flatten() {
        let Ok(meta) = e.metadata() else { continue };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            dirs.push(e.path());
        } else {
            total.add(&e.path(), meta.len());
            children.push(Child {
                path: e.path().to_string_lossy().into(),
                name: e.file_name().to_string_lossy().into(),
                is_dir: false,
                bytes: meta.len(),
            });
        }
    }
    // Chia thư mục con cho số luồng = số lõi CPU (không tạo một luồng cho mỗi thư mục).
    let next = AtomicUsize::new(0);
    let workers = thread::available_parallelism().map_or(4, |n| n.get());
    let results: Vec<(usize, Acc)> = thread::scope(|s| {
        let hs: Vec<_> = (0..workers)
            .map(|_| {
                s.spawn(|| {
                    let mut out = Vec::new();
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(d) = dirs.get(i) else { break out };
                        out.push((i, walk_dir(d)));
                    }
                })
            })
            .collect();
        hs.into_iter()
            .flat_map(|h| h.join().unwrap_or_default())
            .collect()
    });
    for (i, acc) in results {
        let d = &dirs[i];
        children.push(Child {
            path: d.to_string_lossy().into(),
            name: d.file_name().unwrap_or_default().to_string_lossy().into(),
            is_dir: true,
            bytes: acc.bytes,
        });
        total.merge(acc);
    }
    children.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    children.truncate(100);

    let system = file_rules::system_dirs();
    let mut big_files: Vec<BigFile> = total
        .top
        .into_sorted_vec()
        .into_iter()
        .map(|Reverse((bytes, p))| {
            let age = fs::metadata(&p).map(|m| fsutil::age_days(&m)).unwrap_or(0);
            let (verdict, reason) = file_rules::classify(&p, bytes, age, &system);
            BigFile {
                path: p.to_string_lossy().into(),
                bytes,
                age_days: age,
                verdict,
                reason,
            }
        })
        .collect();
    big_files.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let mut types: Vec<_> = total
        .types
        .into_iter()
        .map(|(k, (b, n))| (k, b, n))
        .collect();
    types.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(Scan {
        path,
        total: total.bytes,
        children,
        big_files,
        types,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_largest_files() {
        let mut acc = Acc::default();
        for i in 0..200u64 {
            acc.add(Path::new(&format!("f{i}.mp4")), i);
        }
        let sizes: Vec<u64> = acc
            .top
            .into_sorted_vec()
            .into_iter()
            .map(|Reverse((b, _))| b)
            .collect();
        assert_eq!(sizes.len(), TOP_FILES);
        assert_eq!(*sizes.iter().min().unwrap(), 150);
        assert_eq!(acc.types["Video"], ((0..200).sum(), 200));
    }
}

