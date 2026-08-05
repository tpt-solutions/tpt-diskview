use std::path::Path;

use crate::detector::{CleanupCandidate, CleanupCategory};

pub fn detect_temp_files(root: &Path) -> Vec<CleanupCandidate> {
    let mut candidates = Vec::new();

    if !root.exists() {
        return candidates;
    }

    let temp_dirs: Vec<&str> = if cfg!(windows) {
        vec!["Temp", "tmp"]
    } else {
        vec!["tmp", "var/tmp"]
    };

    for dir_name in &temp_dirs {
        let temp_path = root.join(dir_name);
        if temp_path.exists() {
            collect_files(&temp_path, CleanupCategory::TempFiles, &mut candidates);
        }
    }

    #[cfg(unix)]
    {
        let linux_temp_dirs = vec![
            "/tmp",
            "/var/tmp",
            "/dev/shm",
        ];
        for dir in linux_temp_dirs {
            let p = std::path::Path::new(dir);
            if p.exists() && p.is_dir() {
                collect_files(p, CleanupCategory::TempFiles, &mut candidates);
            }
        }
        if let Some(home) = std::env::var_os("HOME") {
            let cache_dir = std::path::PathBuf::from(home).join(".cache");
            if cache_dir.exists() {
                collect_files(&cache_dir, CleanupCategory::TempFiles, &mut candidates);
            }
        }
    }

    candidates
}

pub fn detect_browser_cache(root: &Path) -> Vec<CleanupCandidate> {
    let mut candidates = Vec::new();

    if !root.exists() {
        return candidates;
    }

    let cache_patterns: Vec<&str> = if cfg!(windows) {
        vec!["Cache", "cache", "Code Cache"]
    } else {
        vec!["cache", ".cache"]
    };

    for pattern in &cache_patterns {
        let cache_path = root.join(pattern);
        if cache_path.exists() {
            collect_files(&cache_path, CleanupCategory::BrowserCache, &mut candidates);
        }
    }

    candidates
}

fn collect_files(dir: &Path, category: CleanupCategory, candidates: &mut Vec<CleanupCandidate>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(meta) = path.metadata() {
                if meta.is_file() {
                    candidates.push(CleanupCandidate {
                        path: path.to_string_lossy().to_string(),
                        size: meta.len(),
                        category: category.clone(),
                        description: format!("Temporary/cache file: {}", path.file_name().unwrap_or_default().to_string_lossy()),
                    });
                }
            }
        }
    }
}
