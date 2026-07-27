use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::State;

use tpt_diskview_core::{ScanOptions, ScanProgress, Scanner, Tree};
use tpt_diskview_cleanup::detector::CleanupCandidate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanState {
    pub is_scanning: bool,
    pub current_path: String,
    pub files_scanned: u64,
    pub bytes_scanned: u64,
    pub result: Option<Tree>,
    pub partial_tree: Option<Tree>,
    pub error: Option<String>,
}

pub struct AppState {
    pub scan_state: Mutex<ScanState>,
    pub scanner: Scanner,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            scan_state: Mutex::new(ScanState {
                is_scanning: false,
                current_path: String::new(),
                files_scanned: 0,
                bytes_scanned: 0,
                result: None,
                partial_tree: None,
                error: None,
            }),
            scanner: Scanner::new(),
        }
    }
}

#[tauri::command]
pub fn start_scan(
    state: State<'_, Arc<AppState>>,
    options: ScanOptions,
) -> Result<(), String> {
    let mut scan_state = state.scan_state.lock().map_err(|e| e.to_string())?;
    if scan_state.is_scanning {
        return Err("Scan already in progress".to_string());
    }

    scan_state.is_scanning = true;
    scan_state.current_path = String::new();
    scan_state.files_scanned = 0;
    scan_state.bytes_scanned = 0;
    scan_state.result = None;
    scan_state.error = None;

    let state_clone = state.inner().clone();
    let _handle = state.scanner.start(options, move |progress| {
        if let Ok(mut scan_state) = state_clone.scan_state.lock() {
            match progress {
                ScanProgress::Scanning { path, files_scanned, bytes_scanned } => {
                    scan_state.current_path = path.clone();
                    scan_state.files_scanned = *files_scanned;
                    scan_state.bytes_scanned = *bytes_scanned;
                }
                ScanProgress::PartialTree(tree) => {
                    scan_state.partial_tree = Some(tree.clone());
                }
                ScanProgress::Completed(tree) => {
                    scan_state.is_scanning = false;
                    scan_state.result = Some(tree.clone());
                }
                ScanProgress::Error(msg) => {
                    scan_state.is_scanning = false;
                    scan_state.error = Some(msg.clone());
                }
            }
        }
    })
    .map_err(|e| {
        if let Ok(mut scan_state) = state.scan_state.lock() {
            scan_state.is_scanning = false;
            scan_state.error = Some(e.to_string());
        }
        e.to_string()
    })?;

    Ok(())
}

#[tauri::command]
pub fn cancel_scan(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut scan_state = state.scan_state.lock().map_err(|e| e.to_string())?;
    scan_state.is_scanning = false;
    Ok(())
}

#[tauri::command]
pub fn scan_status(state: State<'_, Arc<AppState>>) -> Result<ScanState, String> {
    state.scan_state.lock().map(|s| s.clone()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_scan_results(state: State<'_, Arc<AppState>>) -> Result<Option<Tree>, String> {
    state.scan_state.lock().map(|s| s.result.clone()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn detect_cleanup_candidates(
    path: String,
) -> Result<Vec<CleanupCandidate>, String> {
    let root = std::path::Path::new(&path);
    let mut candidates = Vec::new();

    candidates.extend(tpt_diskview_cleanup::temp::detect_temp_files(root));
    candidates.extend(tpt_diskview_cleanup::temp::detect_browser_cache(root));
    candidates.extend(tpt_diskview_cleanup::duplicate::detect_duplicates(root));
    candidates.extend(tpt_diskview_cleanup::docker::detect_stale_docker_volumes(root));

    Ok(candidates)
}

#[tauri::command]
#[allow(dead_code)]
pub fn cleanup_selected(paths: Vec<String>) -> Result<serde_json::Value, String> {
    let mut removed = 0;
    let mut freed = 0u64;

    for path_str in paths {
        let path = PathBuf::from(&path_str);
        if !path.exists() {
            continue;
        }

        // Get file size before deletion
        let size = if path.is_file() {
            std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
        } else {
            // For directories, calculate total size
            let mut total = 0u64;
            if let Ok(entries) = std::fs::read_dir(&path) {
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
        };

        // Use trash crate for recycle bin deletion
        if let Err(e) = trash::delete(&path) {
            eprintln!("Failed to move to trash: {} - {}", path_str, e);
            // Fallback to regular delete if trash fails
            if let Err(e) = std::fs::remove_file(&path).or_else(|_| std::fs::remove_dir_all(&path)) {
                eprintln!("Failed to delete: {} - {}", path_str, e);
                continue;
            }
        }

        removed += 1;
        freed += size;
    }

    Ok(serde_json::json!({ "removed": removed, "freed": freed }))
}

#[allow(dead_code)]
fn estimate_dir_size(dir: &PathBuf) -> u64 {
    let mut total = 0u64;
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
