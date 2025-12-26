//! CC Version Guard - Tauri Backend
//! Lock your CapCut version and prevent auto-updates

mod commands;

use commands::{autostart, backup, cleaner, paths, process, protector, scanner, switcher};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

/// Setup the system tray with menu
fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Create menu items
    let show_i = MenuItem::with_id(app, "show", "Show Version Guard", true, None::<&str>)?;
    let check_i = MenuItem::with_id(app, "check_status", "Check Protection Status", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Build menu
    let menu = Menu::with_items(app, &[&show_i, &check_i, &quit_i])?;

    // Build tray icon
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("CC Version Guard")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "check_status" => {
                    // Emit event to frontend to show status
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.emit("tray-check-status", ());
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            // Left click shows the window
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize system tray
            setup_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Scanner commands
            scanner::get_archive_versions,
            scanner::scan_versions,
            scanner::get_capcut_paths,
            // Path resolution commands
            paths::get_path_info,
            paths::validate_custom_capcut_path,
            // Backup commands
            backup::list_backups,
            backup::restore_version_backup,
            backup::delete_backup,
            backup::get_backup_size,
            backup::clear_all_backups,
            // Autostart commands
            autostart::get_autostart_enabled,
            autostart::set_autostart_enabled,
            // Process commands
            process::is_capcut_running,
            process::perform_precheck,
            process::launch_capcut,
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
