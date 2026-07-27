use std::sync::Arc;

mod commands;

use commands::{AppState, start_scan, cancel_scan, scan_status, get_scan_results, detect_cleanup_candidates};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Arc::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            start_scan,
            cancel_scan,
            scan_status,
            get_scan_results,
            detect_cleanup_candidates,
        ])
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
