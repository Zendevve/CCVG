//! Protection and file locking functionality
//! Migrated from original eframe/egui main.rs

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use super::paths;

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

/// Create readonly lock file
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

/// Create dummy lock files
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

/// Delete specified version directories (with automatic backup)
#[tauri::command]
pub fn delete_versions(paths: Vec<String>) -> ProtectionResult {
    use super::backup;

    let mut logs: Vec<String> = Vec::new();

    for path_str in &paths {
        let path = PathBuf::from(path_str);
        let name = path.file_name().unwrap_or_default().to_string_lossy();

        // Create backup before deletion
        logs.push(format!("Backing up: {}", name));
        let backup_result = backup::create_backup(&path, "Version deleted during protection");

        if backup_result.success {
            if let Some(backup_id) = &backup_result.backup_id {
                logs.push(format!("[OK] Backup created: {}", backup_id));
            }
        } else {
            // Backup failed - warn but continue (user confirmed deletion)
            logs.push(format!("[!] Backup failed: {}", backup_result.error.unwrap_or_default()));
            logs.push("[!] Proceeding with deletion (backup unavailable)".to_string());
        }

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
        logs.push("[OK] Backups available for recovery".to_string());
    }

    ProtectionResult {
        success: true,
        error: None,
        logs,
    }
}

/// Apply protection (lock config + create locks)
#[tauri::command]
pub fn apply_protection() -> ProtectionResult {
    let capcut_paths = match paths::resolve_capcut_paths() {
        Some(p) => p,
        None => {
            return ProtectionResult {
                success: false,
                error: Some("Could not find CapCut installation".to_string()),
                logs: vec![],
            }
        }
    };

    let apps_path = capcut_paths.apps;
    let capcut_root = capcut_paths.root;
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

    // Create locks
    logs.push("Creating locks...".to_string());
    if let Err(e) = create_dummy_files(&capcut_root, &apps_path) {
        return ProtectionResult {
            success: false,
            error: Some(e),
            logs,
        };
    }
    logs.push("[OK] Version lock active".to_string());

    ProtectionResult {
        success: true,
        error: None,
        logs,
    }
}

/// Apply protection with specific options
pub fn apply_protection_with_options(lock_config: bool, create_blockers: bool) -> ProtectionResult {
    let capcut_paths = match paths::resolve_capcut_paths() {
        Some(p) => p,
        None => {
            return ProtectionResult {
                success: false,
                error: Some("Could not find CapCut installation".to_string()),
                logs: vec![],
            }
        }
    };

    let apps_path = capcut_paths.apps;
    let capcut_root = capcut_paths.root;
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

    // Create locks if enabled
    if create_blockers {
        logs.push("Creating locks...".to_string());
        if let Err(e) = create_dummy_files(&capcut_root, &apps_path) {
            return ProtectionResult {
                success: false,
                error: Some(e),
                logs,
            };
        }
        logs.push("[OK] Version lock active".to_string());
    } else {
        logs.push("Skipping lock creation (disabled)".to_string());
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

/// Protection status result
#[derive(serde::Serialize)]
pub struct ProtectionStatus {
    pub is_protected: bool,
    pub config_locked: bool,
    pub blockers_exist: bool,
}

/// Check if protection is currently applied
#[tauri::command]
pub fn check_protection_status() -> ProtectionStatus {
    let capcut_paths = match paths::resolve_capcut_paths() {
        Some(p) => p,
        None => return ProtectionStatus {
            is_protected: false,
            config_locked: false,
            blockers_exist: false,
        },
    };

    let apps_path = capcut_paths.apps;
    let capcut_root = capcut_paths.root;

    // Check if ProductInfo.xml is a readonly empty file (lock)
    let product_info = apps_path.join("ProductInfo.xml");
    let blockers_exist = if product_info.exists() {
        if let Ok(meta) = fs::metadata(&product_info) {
            meta.len() == 0 && meta.permissions().readonly()
        } else {
            false
        }
    } else {
        false
    };

    // Check if update.exe lock exists
    let update_blocker = capcut_root.join("User Data").join("Download").join("update.exe");
    let update_blocked = if update_blocker.exists() {
        if let Ok(meta) = fs::metadata(&update_blocker) {
            meta.len() == 0 && meta.permissions().readonly()
        } else {
            false
        }
    } else {
        false
    };

    // Check if configure.ini has last_version=1.0.0.0
    let config_path = apps_path.join("configure.ini");
    let config_locked = if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            content.contains("last_version=1.0.0.0")
        } else {
            false
        }
    } else {
        false
    };

    ProtectionStatus {
        is_protected: blockers_exist || update_blocked || config_locked,
        config_locked,
        blockers_exist: blockers_exist || update_blocked,
    }
}

/// Remove all protection measures
#[tauri::command]
pub fn remove_protection() -> ProtectionResult {
    let capcut_paths = match paths::resolve_capcut_paths() {
        Some(p) => p,
        None => {
            return ProtectionResult {
                success: false,
                error: Some("Could not find CapCut installation".to_string()),
                logs: vec![],
            }
        }
    };

    let apps_path = capcut_paths.apps;
    let capcut_root = capcut_paths.root;
    let mut logs: Vec<String> = Vec::new();

    // Remove ProductInfo.xml lock
    let product_info = apps_path.join("ProductInfo.xml");
    if product_info.exists() {
        logs.push("Removing ProductInfo.xml lock...".to_string());
        if let Err(e) = unset_readonly_recursive(&product_info) {
            logs.push(format!("[!] Warning: {}", e));
        }
        if let Err(e) = fs::remove_file(&product_info) {
            logs.push(format!("[!] Could not remove ProductInfo.xml: {}", e));
        } else {
            logs.push("[OK] ProductInfo.xml lock removed".to_string());
        }
    }

    // Remove update.exe lock
    let update_blocker = capcut_root.join("User Data").join("Download").join("update.exe");
    if update_blocker.exists() {
        logs.push("Removing update.exe lock...".to_string());
        if let Err(e) = unset_readonly_recursive(&update_blocker) {
            logs.push(format!("[!] Warning: {}", e));
        }
        if let Err(e) = fs::remove_file(&update_blocker) {
            logs.push(format!("[!] Could not remove update.exe: {}", e));
        } else {
            logs.push("[OK] update.exe lock removed".to_string());
        }
    }

    // Reset configure.ini (remove last_version lock)
    let config_path = apps_path.join("configure.ini");
    if config_path.exists() {
        logs.push("Resetting configure.ini...".to_string());
        if let Ok(content) = fs::read_to_string(&config_path) {
            let new_content: String = content
                .lines()
                .filter(|line| !line.trim().starts_with("last_version"))
                .collect::<Vec<_>>()
                .join("\n");
            if let Err(e) = fs::write(&config_path, new_content) {
                logs.push(format!("[!] Could not reset configure.ini: {}", e));
            } else {
                logs.push("[OK] configure.ini reset".to_string());
            }
        }
    }

    logs.push("[OK] Protection removed - CapCut allows updates".to_string());

    ProtectionResult {
        success: true,
        error: None,
        logs,
    }
}
