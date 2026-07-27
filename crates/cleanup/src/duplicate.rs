use std::collections::HashMap;
use std::path::Path;

use blake3::Hasher;

use crate::detector::{CleanupCandidate, CleanupCategory};

#[derive(Debug, Clone)]
struct FileHash {
    path: String,
    size: u64,
}

pub fn detect_duplicates(root: &Path) -> Vec<CleanupCandidate> {
    let mut candidates = Vec::new();

    if !root.exists() {
        return candidates;
    }

    let mut size_groups: HashMap<u64, Vec<String>> = HashMap::new();
    collect_sizes(root, &mut size_groups);

    let mut hash_groups: HashMap<[u8; 32], Vec<FileHash>> = HashMap::new();

    for (size, paths) in size_groups {
        if paths.len() < 2 || size == 0 {
            continue;
        }

        for path in &paths {
            if let Ok(hash) = hash_file(path) {
                hash_groups.entry(hash).or_default().push(FileHash {
                    path: path.clone(),
                    size,
                });
            }
        }
    }

    for (_, files) in hash_groups {
        if files.len() < 2 {
            continue;
        }

        let mut iter = files.into_iter();
        let original = iter.next().unwrap();

        for dup in iter {
            candidates.push(CleanupCandidate {
                path: dup.path.clone(),
                size: dup.size,
                category: CleanupCategory::Duplicates,
                description: format!(
                    "Duplicate of {} ({} bytes)",
                    original.path, original.size
                ),
            });
        }
    }

    candidates.sort_by(|a, b| b.size.cmp(&a.size));
    candidates
}

fn collect_sizes(dir: &Path, groups: &mut HashMap<u64, Vec<String>>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(meta) = path.metadata() {
                if meta.is_file() {
                    groups.entry(meta.len()).or_default().push(path.to_string_lossy().to_string());
                } else if meta.is_dir() {
                    collect_sizes(&path, groups);
                }
            }
        }
    }
}

fn hash_file(path: &str) -> Result<[u8; 32], std::io::Error> {
    let data = std::fs::read(path)?;
    let mut hasher = Hasher::new();
    hasher.update(&data);
    Ok(*hasher.finalize().as_bytes())
}
