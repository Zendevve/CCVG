//! Process detection functionality
//! Migrated from original eframe/egui main.rs

use sysinfo::System;

/// Check if CapCut is currently running
#[tauri::command]
pub fn is_capcut_running() -> bool {
    let mut sys = System::new();
    sys.refresh_processes();

    sys.processes_by_name("CapCut".as_ref()).next().is_some()
        || sys.processes_by_name("CapCut.exe".as_ref()).next().is_some()
}

/// System pre-check results
#[derive(serde::Serialize)]
pub struct PreCheckResult {
    pub capcut_found: bool,
    pub capcut_running: bool,
    pub apps_path: Option<String>,
}

/// Perform system pre-check
#[tauri::command]
pub fn perform_precheck() -> PreCheckResult {
    let apps_path = std::env::var("LOCALAPPDATA")
        .ok()
        .map(|p| std::path::PathBuf::from(p).join("CapCut").join("Apps"));

    let capcut_found = apps_path.as_ref().map(|p| p.exists()).unwrap_or(false);
    let capcut_running = is_capcut_running();

    PreCheckResult {
        capcut_found,
        capcut_running,
        apps_path: apps_path.map(|p| p.to_string_lossy().to_string()),
    }
}
