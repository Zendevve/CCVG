//! Process detection functionality
//! Migrated from original eframe/egui main.rs

use sysinfo::System;
use std::process::Command;

use super::paths;

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

/// Launch result
#[derive(serde::Serialize)]
pub struct LaunchResult {
    pub success: bool,
    pub error: Option<String>,
}

/// Launch CapCut application
#[tauri::command]
pub fn launch_capcut() -> LaunchResult {
    // Find CapCut executable
    let apps_path = match paths::get_capcut_apps_path() {
        Some(p) if p.exists() => p,
        _ => return LaunchResult {
            success: false,
            error: Some("CapCut installation not found".to_string()),
        },
    };

    // Look for versions and find CapCut.exe
    let versions: Vec<_> = std::fs::read_dir(&apps_path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    // Sort to get the latest version
    let mut version_paths: Vec<_> = versions.iter().map(|e| e.path()).collect();
    version_paths.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    for version_path in version_paths {
        let exe_path = version_path.join("CapCut.exe");
        if exe_path.exists() {
            match Command::new(&exe_path).spawn() {
                Ok(_) => return LaunchResult {
                    success: true,
                    error: None,
                },
                Err(e) => return LaunchResult {
                    success: false,
                    error: Some(format!("Failed to launch: {}", e)),
                },
            }
        }
    }

    LaunchResult {
        success: false,
        error: Some("CapCut.exe not found in any version".to_string()),
    }
}
