use std::{fs, path::Path, time::Duration};

use serde::Serialize;

#[derive(Default, Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub bytes: u64,
    pub files: u64,
    /// File không xóa được (đang được dùng hoặc thiếu quyền).
    pub failed: u64,
}

impl Stats {
    pub fn add(&mut self, o: Stats) {
        self.bytes += o.bytes;
        self.files += o.files;
        self.failed += o.failed;
    }
}

/// Chặn đường dẫn rỗng/tương đối/quá nông như `C:\` hoặc `C:\Windows`, phòng khi biến môi trường bị thiếu hoặc sai.
pub fn is_safe_root(p: &Path) -> bool {
    p.is_absolute() && p.components().count() >= 4
}

/// Gọi `f` cho mọi file trong `dir` (đệ quy), không đi theo symlink/junction.
pub fn visit_files(dir: &Path, f: &mut dyn FnMut(&Path, &fs::Metadata)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        // Trên Windows, metadata của DirEntry lấy sẵn từ lúc liệt kê thư mục và không đi theo link -> nhanh.
        let Ok(meta) = entry.metadata() else { continue };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            visit_files(&entry.path(), f);
        } else {
            f(&entry.path(), &meta);
        }
    }
}

/// Số ngày kể từ lần sửa cuối.
pub fn age_days(meta: &fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|m| m.elapsed().ok())
        .map_or(0, |d| d.as_secs() / 86_400)
}

/// Duyệt `dir`, cộng dồn file cũ hơn `min_age`; nếu `delete` thì xóa luôn và dọn thư mục con rỗng.
/// Thư mục gốc `dir` luôn được giữ lại.
pub fn walk(dir: &Path, min_age: Duration, delete: bool, st: &mut Stats) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        // Không đi theo symlink/junction, để không bao giờ xóa lan ra ngoài thư mục đích.
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            walk(&path, min_age, delete, st);
            if delete {
                let _ = fs::remove_dir(&path); // chỉ thành công khi đã rỗng
            }
            continue;
        }
        let too_new = meta
            .modified()
            .ok()
            .and_then(|m| m.elapsed().ok())
            .is_some_and(|age| age < min_age);
        if too_new {
            continue;
        }
        if delete && fs::remove_file(&path).is_err() {
            st.failed += 1;
            continue;
        }
        st.bytes += meta.len();
        st.files += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_then_delete_keeps_root_and_new_files() {
        let root = std::env::temp_dir().join(format!("don-may-test-{}", std::process::id()));
        fs::create_dir_all(root.join("sub/deeper")).unwrap();
        fs::write(root.join("a.tmp"), b"12345").unwrap();
        fs::write(root.join("sub/deeper/b.tmp"), b"123").unwrap();

        let mut st = Stats::default();
        walk(&root, Duration::from_secs(3600), false, &mut st);
        assert_eq!(st, Stats::default(), "file mới tạo phải được bỏ qua");

        let mut st = Stats::default();
        walk(&root, Duration::ZERO, false, &mut st);
        assert_eq!((st.bytes, st.files), (8, 2));
        assert!(root.join("a.tmp").exists(), "chỉ quét thì không được xóa");

        let mut st = Stats::default();
        walk(&root, Duration::ZERO, true, &mut st);
        assert_eq!((st.bytes, st.files, st.failed), (8, 2, 0));
        assert!(root.exists() && fs::read_dir(&root).unwrap().next().is_none());
        fs::remove_dir(&root).unwrap();
    }

    #[test]
    fn rejects_shallow_or_relative_roots() {
        assert!(!is_safe_root(Path::new("C:\\")));
        assert!(!is_safe_root(Path::new("C:\\Windows")));
        assert!(!is_safe_root(Path::new("Temp")));
        assert!(is_safe_root(Path::new("C:\\Windows\\Temp")));
    }
}
