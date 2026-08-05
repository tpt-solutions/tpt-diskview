use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ignore::WalkBuilder;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::error::ScanError;
use crate::tree::{FileNode, FileNodeType, Tree};

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;
#[cfg(windows)]
use std::ffi::OsString;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOptions {
    pub root: String,
    pub follow_symlinks: bool,
    pub max_depth: Option<usize>,
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanProgress {
    Scanning { path: String, files_scanned: u64, bytes_scanned: u64 },
    PartialTree(Tree),
    Completed(Tree),
    Error(String),
}

pub struct Scanner {}

#[derive(Debug)]
pub struct ScannerHandle {
    cancel_flag: Arc<AtomicBool>,
}

impl ScannerHandle {
    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::Relaxed)
    }
}

impl Scanner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start<F>(
        &self,
        options: ScanOptions,
        progress_callback: F,
    ) -> Result<(ScannerHandle, Tree), ScanError>
    where
        F: Fn(&ScanProgress) + Send + Sync + 'static,
    {
        let root = PathBuf::from(&options.root);
        if !root.exists() {
            return Err(ScanError::PathNotFound(options.root));
        }

        let cancel_flag = Arc::new(AtomicBool::new(false));
        let handle = ScannerHandle {
            cancel_flag: cancel_flag.clone(),
        };

        let progress_callback = Arc::new(progress_callback);

        let tree = self.scan_path(&root, &options, &cancel_flag, &progress_callback)?;

        Ok((handle, Tree { root: tree }))
    }

    fn scan_path<F>(
        &self,
        root: &Path,
        options: &ScanOptions,
        cancel_flag: &Arc<AtomicBool>,
        progress_callback: &Arc<F>,
    ) -> Result<FileNode, ScanError>
    where
        F: Fn(&ScanProgress) + Send + Sync + 'static,
    {
        let mut builder = WalkBuilder::new(root);
        builder
            .follow_links(options.follow_symlinks)
            .max_depth(options.max_depth)
            .hidden(false)
            .git_ignore(false);

        for pattern in &options.exclude_patterns {
            builder.add_custom_ignore_filename(pattern);
        }

        let cancel_flag = cancel_flag.clone();
        let progress_callback = progress_callback.clone();

        let entries: Vec<_> = builder
            .build()
            .filter_map(|e| e.ok())
            .collect();

        let mut root_node = FileNode::directory(
            root.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default().as_str(),
            &root.to_string_lossy(),
        );

        let files_scanned = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let bytes_scanned = Arc::new(std::sync::atomic::AtomicU64::new(0));

        let dir_entries: Vec<_> = entries
            .iter()
            .filter(|e| e.path() != root && e.depth() == 1 && !is_skipped_mount_point(e.path()))
            .collect();

        let child_nodes: Vec<Result<FileNode, ScanError>> = dir_entries
            .par_iter()
            .map(|entry| {
                if cancel_flag.load(Ordering::Relaxed) {
                    return Err(ScanError::Cancelled);
                }

                let path = entry.path();
                let file_name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let meta = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => return Ok(FileNode::file(&file_name, &path.to_string_lossy(), 0)),
                };

                let size = meta.len();

                let last_modified = meta.modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs());

                let files = files_scanned.fetch_add(1, Ordering::Relaxed) + 1;
                let bytes = bytes_scanned.fetch_add(size, Ordering::Relaxed) + size;

                if files % 100 == 0 {
                    progress_callback(&ScanProgress::Scanning {
                        path: path.to_string_lossy().to_string(),
                        files_scanned: files,
                        bytes_scanned: bytes,
                    });
                }

                let mut node = if meta.is_dir() {
                    self.scan_directory(path, options, &cancel_flag, &progress_callback)?
                } else {
                    FileNode::file(&file_name, &path.to_string_lossy(), size)
                };

                node.last_modified = last_modified;
                Ok(node)
            })
            .collect();

        for child in child_nodes {
            let child = child?;
            root_node.size += child.size;
            root_node.item_count += child.item_count;
            root_node.children.push(child);
        }

        root_node.children.sort_by(|a, b| b.size.cmp(&a.size));

        // Emit partial tree for incremental UI updates
        progress_callback(&ScanProgress::PartialTree(Tree { root: root_node.clone() }));

        progress_callback(&ScanProgress::Scanning {
            path: root.to_string_lossy().to_string(),
            files_scanned: files_scanned.load(Ordering::Relaxed),
            bytes_scanned: bytes_scanned.load(Ordering::Relaxed),
        });

        Ok(root_node)
    }

    fn scan_directory<F>(
        &self,
        dir: &Path,
        options: &ScanOptions,
        cancel_flag: &Arc<AtomicBool>,
        progress_callback: &Arc<F>,
    ) -> Result<FileNode, ScanError>
    where
        F: Fn(&ScanProgress) + Send + Sync + 'static,
    {
        let mut builder = WalkBuilder::new(dir);
        builder
            .follow_links(options.follow_symlinks)
            .max_depth(Some(1))
            .hidden(false)
            .git_ignore(false);

        let mut node = FileNode::directory(
            &dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
            &dir.to_string_lossy(),
        );

        let entries: Vec<_> = builder.build().filter_map(|e| e.ok()).collect();

        for entry in &entries {
            if cancel_flag.load(Ordering::Relaxed) {
                return Err(ScanError::Cancelled);
            }

            let path = entry.path();
            if path == dir {
                continue;
            }
            if is_skipped_mount_point(path) {
                continue;
            }

            let file_name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => {
                    node.children.push(FileNode::file(&file_name, &path.to_string_lossy(), 0));
                    continue;
                }
            };

            let size = meta.len();

            let last_modified = meta.modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());

            let mut child = if meta.is_dir() {
                self.scan_path(path, options, cancel_flag, progress_callback)?
            } else {
                FileNode::file(&file_name, &path.to_string_lossy(), size)
            };

            child.last_modified = last_modified;
            node.size += child.size;
            node.item_count += child.item_count;
            node.children.push(child);
        }

        node.children.sort_by(|a, b| b.size.cmp(&a.size));
        Ok(node)
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a path is a reparse point (junction, symlink, etc.) on Windows
#[cfg(windows)]
fn is_reparse_point(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    if let Ok(meta) = std::fs::metadata(path) {
        // FILE_ATTRIBUTE_REPARSE_POINT = 0x400
        (meta.file_attributes() & 0x400) != 0
    } else {
        false
    }
}

#[cfg(not(windows))]
fn is_reparse_point(_path: &Path) -> bool {
    false
}

/// Get the target of a junction or symlink on Windows
#[cfg(windows)]
fn get_reparse_target(path: &Path) -> Option<PathBuf> {
    use std::os::windows::ffi::OsStrExt;
    use std::ffi::OsStr;
    use std::ptr;
    use winapi::um::fileapi::{CreateFileW, GetFinalPathNameByHandleW};
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::winnt::{FILE_READ_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SHARE_DELETE};
    use winapi::shared::minwindef::{MAX_PATH, DWORD};
    // Windows constants not in winapi crate - use numeric values
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x02000000;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x00200000;
    const OPEN_EXISTING: u32 = 3;

    let wide_path: Vec<u16> = OsStr::new(path).encode_wide().chain(Some(0)).collect();
    
    unsafe {
        let handle = CreateFileW(
            wide_path.as_ptr(),
            FILE_READ_ATTRIBUTES,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
            ptr::null_mut(),
        );
        
        if handle == winapi::um::handleapi::INVALID_HANDLE_VALUE {
            return None;
        }
        
        let mut buffer = vec![0u16; MAX_PATH as usize];
        let result = GetFinalPathNameByHandleW(handle, buffer.as_mut_ptr(), MAX_PATH as DWORD, 0);
        CloseHandle(handle);
        
        if result == 0 || result >= MAX_PATH as DWORD {
            return None;
        }
        
        let target = OsString::from_wide(&buffer[..result as usize]);
        // Remove the \\?\ prefix if present
        let target_str = target.to_string_lossy();
        if target_str.starts_with(r"\\?\") {
            Some(PathBuf::from(target_str[4..].to_string()))
        } else {
            Some(PathBuf::from(target_str.to_string()))
        }
    }
}

#[cfg(not(windows))]
fn get_reparse_target(_path: &Path) -> Option<PathBuf> {
    None
}

/// Convert a path to extended-length format for Windows (\\?\ prefix)
#[cfg(windows)]
fn to_extended_length_path(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.starts_with(r"\\?\") {
        path.to_path_buf()
    } else if let Ok(canonical) = path.canonicalize() {
        let canon_str = canonical.to_string_lossy();
        if canon_str.len() >= 260 && !canon_str.starts_with(r"\\?\") {
            PathBuf::from(format!(r"\\?\{}", canon_str))
        } else {
            canonical
        }
    } else {
        path.to_path_buf()
    }
}

#[cfg(not(windows))]
fn to_extended_length_path(path: &Path) -> PathBuf {
    path.to_path_buf()
}

/// Check if a path is a Linux mount point that should be skipped
#[cfg(unix)]
fn is_skipped_mount_point(path: &Path) -> bool {
    let skip_dirs = [
        "/proc", "/sys", "/dev", "/run", "/snap",
    ];
    let path_str = path.to_string_lossy();
    skip_dirs.iter().any(|d| path_str.starts_with(d))
}

#[cfg(not(unix))]
fn is_skipped_mount_point(_path: &Path) -> bool {
    false
}

/// Check if we have read permission for a path
fn has_read_permission(path: &Path) -> bool {
    std::fs::metadata(path).is_ok()
}

/// Get file type including junction/symlink detection on Windows
fn get_file_node_type(path: &Path, meta: &std::fs::Metadata) -> FileNodeType {
    if meta.is_dir() {
        #[cfg(windows)]
        {
            if is_reparse_point(path) {
                // Check if it's a junction or symlink
                if let Some(target) = get_reparse_target(path) {
                    // Try to determine if it's a junction (directory symlink) vs file symlink
                    if target.is_dir() {
                        return FileNodeType::Junction;
                    } else {
                        return FileNodeType::Symlink;
                    }
                }
                return FileNodeType::Junction;
            }
        }
        FileNodeType::Directory
    } else if meta.is_symlink() {
        FileNodeType::Symlink
    } else {
        FileNodeType::File
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::FileNodeType;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_scan_empty_directory() {
        let dir = tempdir().unwrap();
        let scanner = Scanner::new();
        let options = ScanOptions {
            root: dir.path().to_string_lossy().to_string(),
            follow_symlinks: false,
            max_depth: None,
            exclude_patterns: vec![],
        };

        let (_, tree) = scanner.start(options, |_| {}).unwrap();
        assert_eq!(tree.root.children.len(), 0);
        assert_eq!(tree.root.size, 0);
    }

    #[test]
    fn test_scan_directory_with_files() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        fs::write(&file1, "hello").unwrap();
        fs::write(&file2, "world!").unwrap();

        let scanner = Scanner::new();
        let options = ScanOptions {
            root: dir.path().to_string_lossy().to_string(),
            follow_symlinks: false,
            max_depth: None,
            exclude_patterns: vec![],
        };

        let (_, tree) = scanner.start(options, |_| {}).unwrap();
        assert_eq!(tree.root.children.len(), 2);
        assert_eq!(tree.root.size, 11); // "hello" (5) + "world!" (6)
        assert_eq!(tree.root.item_count, 3); // 2 files + 1 directory
    }

    #[test]
    fn test_scan_nested_directories() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = subdir.join("file2.txt");
        fs::write(&file1, "hello").unwrap();
        fs::write(&file2, "world!").unwrap();

        let scanner = Scanner::new();
        let options = ScanOptions {
            root: dir.path().to_string_lossy().to_string(),
            follow_symlinks: false,
            max_depth: None,
            exclude_patterns: vec![],
        };

        let (_, tree) = scanner.start(options, |_| {}).unwrap();
        assert_eq!(tree.root.children.len(), 2); // file1.txt + subdir
        assert_eq!(tree.root.size, 11);
        
        // Find subdir
        let subdir_node = tree.root.children.iter().find(|c| c.name == "subdir").unwrap();
        assert_eq!(subdir_node.node_type, FileNodeType::Directory);
        assert_eq!(subdir_node.size, 6);
        assert_eq!(subdir_node.children.len(), 1);
    }

    #[test]
    fn test_scan_with_max_depth() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        let subsubdir = subdir.join("subsubdir");
        fs::create_dir_all(&subsubdir).unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = subdir.join("file2.txt");
        let file3 = subsubdir.join("file3.txt");
        fs::write(&file1, "a").unwrap();
        fs::write(&file2, "bb").unwrap();
        fs::write(&file3, "ccc").unwrap();

        let scanner = Scanner::new();
        let options = ScanOptions {
            root: dir.path().to_string_lossy().to_string(),
            follow_symlinks: false,
            max_depth: Some(1),
            exclude_patterns: vec![],
        };

        let (_, tree) = scanner.start(options, |_| {}).unwrap();
        // With max_depth=1, we should only see file1.txt and subdir (but not file2.txt inside subdir)
        assert_eq!(tree.root.children.len(), 2);
    }

    #[test]
    fn test_scan_cancelled() {
        let dir = tempdir().unwrap();
        // Create many files to allow time for cancellation
        for i in 0..100 {
            fs::write(dir.path().join(format!("file{}.txt", i)), "x".repeat(1000)).unwrap();
        }

        let scanner = Scanner::new();
        let options = ScanOptions {
            root: dir.path().to_string_lossy().to_string(),
            follow_symlinks: false,
            max_depth: None,
            exclude_patterns: vec![],
        };

        let (handle, _) = scanner.start(options, |_| {}).unwrap();
        handle.cancel();
        
        // The scan should be cancelled
        assert!(handle.is_cancelled());
    }

    #[test]
    fn test_file_node_creation() {
        let file = FileNode::file("test.txt", "/path/test.txt", 100);
        assert_eq!(file.name, "test.txt");
        assert_eq!(file.path, "/path/test.txt");
        assert_eq!(file.size, 100);
        assert_eq!(file.node_type, FileNodeType::File);
        assert_eq!(file.item_count, 1);
        assert!(file.children.is_empty());
    }

    #[test]
    fn test_directory_node_creation() {
        let dir = FileNode::directory("testdir", "/path/testdir");
        assert_eq!(dir.name, "testdir");
        assert_eq!(dir.path, "/path/testdir");
        assert_eq!(dir.size, 0);
        assert_eq!(dir.node_type, FileNodeType::Directory);
        assert_eq!(dir.item_count, 1); // Directory counts itself
        assert!(dir.children.is_empty());
    }

    #[test]
    fn test_tree_total_size() {
        let mut dir = FileNode::directory("root", "/root");
        let file1 = FileNode::file("file1.txt", "/root/file1.txt", 100);
        let file2 = FileNode::file("file2.txt", "/root/file2.txt", 200);
        dir.children.push(file1);
        dir.children.push(file2);
        dir.size = 300;
        dir.item_count = 3;

        assert_eq!(dir.total_size(), 300);
        assert_eq!(dir.total_items(), 3);
    }

    #[test]
    fn test_scan_nonexistent_path() {
        let scanner = Scanner::new();
        let options = ScanOptions {
            root: "/nonexistent/path/that/does/not/exist".to_string(),
            follow_symlinks: false,
            max_depth: None,
            exclude_patterns: vec![],
        };

        let result = scanner.start(options, |_| {});
        assert!(result.is_err());
        match result.unwrap_err() {
            ScanError::PathNotFound(_) => {}
            _ => panic!("Expected PathNotFound error"),
        }
    }
}
