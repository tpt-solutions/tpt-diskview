use std::path::Path;

use crate::detector::{CleanupCandidate, CleanupCategory};

pub fn detect_stale_docker_volumes(root: &Path) -> Vec<CleanupCandidate> {
    let mut candidates = Vec::new();

    if !root.exists() {
        return candidates;
    }

    if cfg!(windows) {
        let docker_path = root.join("AppData").join("Local").join("Docker");
        if docker_path.exists() {
            collect_docker_artifacts(&docker_path, &mut candidates);
        }
    } else {
        let docker_paths = vec![
            root.join("var").join("lib").join("docker"),
            root.join(".docker"),
            root.join(".docker").join("desktop"),
        ];

        if let Some(home) = std::env::var_os("HOME") {
            let docker_desktop = std::path::PathBuf::from(home).join(".docker").join("desktop");
            if docker_desktop.exists() {
                collect_docker_artifacts(&docker_desktop, &mut candidates);
            }
        }

        let extra_linux_paths = vec![
            "/var/lib/docker/desktop",
            "/var/lib/docker/containers",
        ];
        for p in extra_linux_paths {
            let path = std::path::Path::new(p);
            if path.exists() {
                collect_docker_artifacts(path, &mut candidates);
            }
        }

        for path in docker_paths {
            if path.exists() {
                collect_docker_artifacts(&path, &mut candidates);
            }
        }
    }

    candidates
}

fn collect_docker_artifacts(dir: &Path, candidates: &mut Vec<CleanupCandidate>) {
    let stale_indicators = vec!["volumes", "overlay2", "image"];

    for indicator in &stale_indicators {
        let path = dir.join(indicator);
        if path.exists() {
            if let Ok(_meta) = path.metadata() {
                candidates.push(CleanupCandidate {
                    path: path.to_string_lossy().to_string(),
                    size: estimate_dir_size(&path),
                    category: CleanupCategory::DockerVolumes,
                    description: format!("Docker {} directory", indicator),
                });
            }
        }
    }
}

fn estimate_dir_size(dir: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total += meta.len();
                } else if meta.is_dir() {
                    total += estimate_dir_size(&entry.path());
                }
            }
        }
    }
    total
}
