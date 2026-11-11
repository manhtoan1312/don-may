//! Tìm file trùng nội dung: gom theo kích thước -> băm 64KB đầu -> so từng byte để chắc chắn.

use std::{
    collections::HashMap,
    fs::File,
    hash::{DefaultHasher, Hasher},
    io::{self, BufReader, Read},
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::fsutil;

const MIN_SIZE: u64 = 1024 * 1024;
const MAX_GROUPS: usize = 50;

fn head_hash(p: &Path) -> io::Result<u64> {
    let mut buf = vec![0; 64 * 1024];
    let n = File::open(p)?.read(&mut buf)?;
    let mut h = DefaultHasher::new();
    h.write(&buf[..n]);
    Ok(h.finish())
}

fn same_content(a: &Path, b: &Path) -> io::Result<bool> {
    let (mut ra, mut rb) = (
        BufReader::new(File::open(a)?),
        BufReader::new(File::open(b)?),
    );
    let (mut ba, mut bb) = (vec![0; 64 * 1024], vec![0; 64 * 1024]);
    loop {
        let n = ra.read(&mut ba)?;
        if n == 0 {
            return Ok(rb.read(&mut bb)? == 0);
        }
        rb.read_exact(&mut bb[..n])?;
        if ba[..n] != bb[..n] {
            return Ok(false);
        }
    }
}

/// Tách nhóm cùng kích thước thành các cụm có nội dung giống hệt nhau.
fn clusters(files: Vec<PathBuf>) -> Vec<Vec<PathBuf>> {
    let mut by_head: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for f in files {
        if let Ok(h) = head_hash(&f) {
            by_head.entry(h).or_default().push(f);
        }
    }
    let mut out = Vec::new();
    for mut rest in by_head.into_values().filter(|g| g.len() > 1) {
        while rest.len() > 1 {
            let rep = rest.remove(0);
            let (same, other): (Vec<_>, Vec<_>) = rest
                .into_iter()
                .partition(|f| same_content(&rep, f).unwrap_or(false));
            if !same.is_empty() {
                out.push([vec![rep], same].concat());
            }
            rest = other;
        }
    }
    out
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DupFile {
    path: String,
    age_days: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DupGroup {
    bytes_each: u64,
    files: Vec<DupFile>,
}

fn duplicates_in(root: &Path, min_size: u64) -> Vec<DupGroup> {
    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    fsutil::visit_files(root, &mut |p, m| {
        if m.len() >= min_size {
            by_size.entry(m.len()).or_default().push(p.to_path_buf());
        }
    });
    let mut groups: Vec<DupGroup> = by_size
        .into_iter()
        .filter(|(_, v)| v.len() > 1)
        .flat_map(|(size, v)| clusters(v).into_iter().map(move |c| (size, c)))
        .map(|(size, c)| DupGroup {
            bytes_each: size,
            files: c
                .into_iter()
                .map(|p| DupFile {
                    age_days: std::fs::metadata(&p)
                        .map(|m| fsutil::age_days(&m))
                        .unwrap_or(0),
                    path: p.to_string_lossy().into(),
                })
                .collect(),
        })
        .collect();
    // Nhóm lãng phí nhiều nhất lên đầu.
    groups.sort_by_key(|g| std::cmp::Reverse(g.bytes_each * (g.files.len() as u64 - 1)));
    groups.truncate(MAX_GROUPS);
    groups
}

#[tauri::command(async)]
pub fn find_duplicates(path: String) -> Result<Vec<DupGroup>, String> {
    let root = PathBuf::from(path);
    if !root.is_absolute() || !root.is_dir() {
        return Err("Thư mục không hợp lệ".into());
    }
    Ok(duplicates_in(&root, MIN_SIZE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_only_identical_files() {
        let root = std::env::temp_dir().join(format!("don-may-dupes-{}", std::process::id()));
        std::fs::create_dir_all(root.join("sub")).unwrap();
        let big = |fill: u8, tail: u8| {
            let mut v = vec![fill; 100_000];
            v.push(tail); // khác nhau ở byte cuối, sau vùng 64KB đầu
            v
        };
        std::fs::write(root.join("a.bin"), big(1, 9)).unwrap();
        std::fs::write(root.join("sub/b.bin"), big(1, 9)).unwrap();
        std::fs::write(root.join("c.bin"), big(1, 7)).unwrap(); // cùng size, cùng 64KB đầu, khác cuối
        std::fs::write(root.join("d.bin"), big(2, 9)).unwrap();
        let groups = duplicates_in(&root, 1);
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(groups.len(), 1);
        let mut names: Vec<_> = groups[0]
            .files
            .iter()
            .map(|f| Path::new(&f.path).file_name().unwrap().to_owned())
            .collect();
        names.sort();
        assert_eq!(names, ["a.bin", "b.bin"]);
    }
}
