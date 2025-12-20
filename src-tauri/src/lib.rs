//! CC Version Guard - Tauri Backend
//! Lock your CapCut version and prevent auto-updates

mod commands;

use commands::{cleaner, process, protector, scanner, switcher};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Scanner commands
            scanner::get_archive_versions,
            scanner::scan_versions,
            scanner::get_capcut_paths,
            // Process commands
            process::is_capcut_running,
            process::perform_precheck,
            // Cleaner commands
            cleaner::calculate_cache_size,
            cleaner::clean_cache,
            // Protector commands
            protector::delete_versions,
            protector::apply_protection,
            protector::run_full_protection,
            protector::check_protection_status,
            protector::remove_protection,
            // Switcher commands
            switcher::switch_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
