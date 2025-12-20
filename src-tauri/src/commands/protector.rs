//! Protection and file locking functionality
//! Migrated from original eframe/egui main.rs

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Unset readonly attribute recursively
fn unset_readonly_recursive(path: &Path) -> Result<(), String> {
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if let Ok(meta) = fs::metadata(p) {
            let mut perms = meta.permissions();
            if perms.readonly() {
                perms.set_readonly(false);
                fs::set_permissions(p, perms).ok();
            }
        }
    }
    Ok(())
}

/// Create readonly blocker file
fn create_readonly(path: &Path) -> Result<(), String> {
    if path.exists() {
        unset_readonly_recursive(path).ok();
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| e.to_string())?;
        } else {
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }
    fs::write(path, "").map_err(|e| e.to_string())?;
    Command::new("attrib")
        .arg("+r")
        .arg(path)
        .output()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Lock configuration file
fn lock_configuration(apps_path: &Path) -> Result<(), String> {
    let config_path = apps_path.join("configure.ini");
    let content = if config_path.exists() {
        fs::read_to_string(&config_path).unwrap_or_default()
    } else {
        String::new()
    };

    let mut new_lines: Vec<String> = Vec::new();
    let mut found = false;

    for line in content.lines() {
        if line.trim().starts_with("last_version") {
            new_lines.push("last_version=1.0.0.0".to_string());
            found = true;
        } else {
            new_lines.push(line.to_string());
        }
    }

    if !found {
        new_lines.push("last_version=1.0.0.0".to_string());
    }

    fs::write(config_path, new_lines.join("\n")).map_err(|e| e.to_string())?;
    Ok(())
}

/// Create dummy blocker files
fn create_dummy_files(capcut_path: &Path, apps_path: &Path) -> Result<(), String> {
    let pinfo = apps_path.join("ProductInfo.xml");
    create_readonly(&pinfo)?;

    let download_dir = capcut_path.join("User Data").join("Download");
    fs::create_dir_all(&download_dir).map_err(|e| e.to_string())?;

    let update_exe = download_dir.join("update.exe");
    create_readonly(&update_exe)?;

    Ok(())
}

/// Protection result
#[derive(serde::Serialize)]
pub struct ProtectionResult {
    pub success: bool,
    pub error: Option<String>,
    pub logs: Vec<String>,
}

/// Delete specified version directories
#[tauri::command]
pub fn delete_versions(paths: Vec<String>) -> ProtectionResult {
    let mut logs: Vec<String> = Vec::new();

    for path_str in &paths {
        let path = PathBuf::from(path_str);
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        logs.push(format!("Deleting: {}", name));

        if let Err(e) = unset_readonly_recursive(&path) {
            logs.push(format!("[!] Warning: {}", e));
        }

        if let Err(e) = fs::remove_dir_all(&path) {
            return ProtectionResult {
                success: false,
                error: Some(format!("Failed to delete {}: {}", name, e)),
                logs,
            };
        }
    }

    if paths.is_empty() {
        logs.push("[OK] No versions to delete".to_string());
    } else {
        logs.push(format!("[OK] Deleted {} version(s)", paths.len()));
    }

    ProtectionResult {
        success: true,
        error: None,
        logs,
    }
}

/// Apply protection (lock config + create blockers)
#[tauri::command]
pub fn apply_protection() -> ProtectionResult {
    let apps_path = match std::env::var("LOCALAPPDATA") {
        Ok(p) => PathBuf::from(p).join("CapCut").join("Apps"),
        Err(_) => {
            return ProtectionResult {
                success: false,
                error: Some("Failed to get LOCALAPPDATA".to_string()),
                logs: vec![],
            }
        }
    };

    let capcut_root = apps_path.parent().unwrap_or(&apps_path).to_path_buf();
    let mut logs: Vec<String> = Vec::new();

    // Lock configuration
    logs.push("Modifying config...".to_string());
    if let Err(e) = lock_configuration(&apps_path) {
        return ProtectionResult {
            success: false,
            error: Some(e),
            logs,
        };
    }
    logs.push("[OK] Configuration locked".to_string());

    // Create blockers
    logs.push("Creating blockers...".to_string());
    if let Err(e) = create_dummy_files(&capcut_root, &apps_path) {
        return ProtectionResult {
            success: false,
            error: Some(e),
            logs,
        };
    }
    logs.push("[OK] Update blockers created".to_string());

    ProtectionResult {
        success: true,
        error: None,
        logs,
    }
}

/// Apply protection with specific options
pub fn apply_protection_with_options(lock_config: bool, create_blockers: bool) -> ProtectionResult {
    let apps_path = match std::env::var("LOCALAPPDATA") {
        Ok(p) => PathBuf::from(p).join("CapCut").join("Apps"),
        Err(_) => {
            return ProtectionResult {
                success: false,
                error: Some("Failed to get LOCALAPPDATA".to_string()),
                logs: vec![],
            }
        }
    };

    let capcut_root = apps_path.parent().unwrap_or(&apps_path).to_path_buf();
    let mut logs: Vec<String> = Vec::new();

    // Lock configuration if enabled
    if lock_config {
        logs.push("Modifying config...".to_string());
        if let Err(e) = lock_configuration(&apps_path) {
            return ProtectionResult {
                success: false,
                error: Some(e),
                logs,
            };
        }
        logs.push("[OK] Configuration locked".to_string());
    } else {
        logs.push("Skipping config lock (disabled)".to_string());
    }

    // Create blockers if enabled
    if create_blockers {
        logs.push("Creating blockers...".to_string());
        if let Err(e) = create_dummy_files(&capcut_root, &apps_path) {
            return ProtectionResult {
                success: false,
                error: Some(e),
                logs,
            };
        }
        logs.push("[OK] Update blockers created".to_string());
    } else {
        logs.push("Skipping blocker creation (disabled)".to_string());
    }

    ProtectionResult {
        success: true,
        error: None,
        logs,
    }
}

/// Full protection sequence
#[derive(serde::Deserialize)]
pub struct ProtectionParams {
    pub versions_to_delete: Vec<String>,
    pub clean_cache: bool,
    pub lock_config: bool,
    pub create_blockers: bool,
}

#[tauri::command]
pub fn run_full_protection(params: ProtectionParams) -> ProtectionResult {
    use crate::commands::cleaner;
    use crate::commands::process;

    let mut all_logs: Vec<String> = Vec::new();

    // Check if CapCut is running
    all_logs.push("Checking system state...".to_string());
    if process::is_capcut_running() {
        return ProtectionResult {
            success: false,
            error: Some("CapCut is still running. Please close it.".to_string()),
            logs: all_logs,
        };
    }
    all_logs.push("[OK] No running instances".to_string());

    // Delete versions
    let delete_result = delete_versions(params.versions_to_delete);
    all_logs.extend(delete_result.logs);
    if !delete_result.success {
        return ProtectionResult {
            success: false,
            error: delete_result.error,
            logs: all_logs,
        };
    }

    // Clean cache if enabled
    if params.clean_cache {
        all_logs.push("Cleaning cache directories...".to_string());
        let cache_result = cleaner::clean_cache();
        all_logs.extend(cache_result.logs);
    } else {
        all_logs.push("Skipping cache cleaning (disabled)".to_string());
    }

    // Apply protection (conditionally based on flags)
    if params.lock_config || params.create_blockers {
        let protect_result = apply_protection_with_options(params.lock_config, params.create_blockers);
        all_logs.extend(protect_result.logs);
        if !protect_result.success {
            return ProtectionResult {
                success: false,
                error: protect_result.error,
                logs: all_logs,
            };
        }
    } else {
        all_logs.push("Skipping protection (all options disabled)".to_string());
    }

    ProtectionResult {
        success: true,
        error: None,
        logs: all_logs,
    }
}
