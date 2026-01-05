//! Backup and rollback functionality for CapCut versions
//! Creates snapshots before destructive operations to enable recovery

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Backup metadata stored alongside each backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Original version name (e.g., "4.5.0.1234")
    pub version_name: String,
    /// Original path before backup
    pub original_path: String,
    /// Timestamp when backup was created (Unix timestamp)
    pub created_at: u64,
    /// Size of the backup in bytes
    pub size_bytes: u64,
    /// Reason for backup
    pub reason: String,
}

/// Result of a backup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub success: bool,
    pub backup_id: Option<String>,
    pub error: Option<String>,
}

/// Result of a restore operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    pub success: bool,
    pub restored_path: Option<String>,
    pub error: Option<String>,
}

/// Get the backup directory path
fn get_backup_dir() -> Option<PathBuf> {
    std::env::var("LOCALAPPDATA")
        .ok()
        .map(|p| PathBuf::from(p).join("CCVersionGuard").join("Backups"))
}

/// Calculate directory size
fn calc_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

/// Create a backup of a version directory before deletion
pub fn create_backup(version_path: &Path, reason: &str) -> BackupResult {
    let backup_dir = match get_backup_dir() {
        Some(d) => d,
        None => {
            return BackupResult {
                success: false,
                backup_id: None,
                error: Some("Could not determine backup directory".to_string()),
            }
        }
    };

    // Create backup directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&backup_dir) {
        return BackupResult {
            success: false,
            backup_id: None,
            error: Some(format!("Failed to create backup directory: {}", e)),
        };
    }

    // Generate backup ID from timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let version_name = version_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let backup_id = format!("{}_{}", version_name, timestamp);
    let backup_path = backup_dir.join(&backup_id);

    // Copy the version directory to backup location
    if let Err(e) = copy_dir_recursive(version_path, &backup_path) {
        return BackupResult {
            success: false,
            backup_id: None,
            error: Some(format!("Failed to copy directory: {}", e)),
        };
    }

    // Calculate size and create metadata
    let size_bytes = calc_dir_size(&backup_path);
    let metadata = BackupMetadata {
        version_name,
        original_path: version_path.to_string_lossy().to_string(),
        created_at: timestamp,
        size_bytes,
        reason: reason.to_string(),
    };

    // Save metadata
    let metadata_path = backup_path.join("_backup_metadata.json");
    if let Err(e) = fs::write(
        &metadata_path,
        serde_json::to_string_pretty(&metadata).unwrap_or_default(),
    ) {
        // Non-fatal error, backup still exists
        eprintln!("Warning: Could not save metadata: {}", e);
    }

    BackupResult {
        success: true,
        backup_id: Some(backup_id),
        error: None,
    }
}

/// Copy directory recursively
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.is_dir() {
        return Err("Source is not a directory".to_string());
    }

    fs::create_dir_all(dst).map_err(|e| e.to_string())?;

    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        let relative = entry_path.strip_prefix(src).map_err(|e| e.to_string())?;
        let target = dst.join(relative);

        if entry_path.is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(entry_path, &target).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

/// Restore a backup to the original location
pub fn restore_backup(backup_id: &str) -> RestoreResult {
    let backup_dir = match get_backup_dir() {
        Some(d) => d,
        None => {
            return RestoreResult {
                success: false,
                restored_path: None,
                error: Some("Could not determine backup directory".to_string()),
            }
        }
    };

    let backup_path = backup_dir.join(backup_id);
    if !backup_path.exists() {
        return RestoreResult {
            success: false,
            restored_path: None,
            error: Some(format!("Backup not found: {}", backup_id)),
        };
    }

    // Read metadata to get original path
    let metadata_path = backup_path.join("_backup_metadata.json");
    let metadata: BackupMetadata = match fs::read_to_string(&metadata_path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(m) => m,
            Err(e) => {
                return RestoreResult {
                    success: false,
                    restored_path: None,
                    error: Some(format!("Invalid metadata: {}", e)),
                }
            }
        },
        Err(e) => {
            return RestoreResult {
                success: false,
                restored_path: None,
                error: Some(format!("Could not read metadata: {}", e)),
            }
        }
    };

    let original_path = PathBuf::from(&metadata.original_path);

    // Ensure parent directory exists
    if let Some(parent) = original_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return RestoreResult {
                success: false,
                restored_path: None,
                error: Some(format!("Could not create parent directory: {}", e)),
            };
        }
    }

    // Remove existing directory at original path if it exists
    if original_path.exists() {
        if let Err(e) = fs::remove_dir_all(&original_path) {
            return RestoreResult {
                success: false,
                restored_path: None,
                error: Some(format!("Could not remove existing directory: {}", e)),
            };
        }
    }

    // Copy backup to original location (excluding metadata file)
    if let Err(e) =
        copy_dir_recursive_filtered(&backup_path, &original_path, "_backup_metadata.json")
    {
        return RestoreResult {
            success: false,
            restored_path: None,
            error: Some(format!("Failed to restore: {}", e)),
        };
    }

    RestoreResult {
        success: true,
        restored_path: Some(metadata.original_path),
        error: None,
    }
}

/// Copy directory recursively, excluding a specific file
fn copy_dir_recursive_filtered(src: &Path, dst: &Path, exclude_file: &str) -> Result<(), String> {
    if !src.is_dir() {
        return Err("Source is not a directory".to_string());
    }

    fs::create_dir_all(dst).map_err(|e| e.to_string())?;

    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let entry_path = entry.path();

        // Skip the excluded file
        if entry_path
            .file_name()
            .map(|n| n == exclude_file)
            .unwrap_or(false)
        {
            continue;
        }

        let relative = entry_path.strip_prefix(src).map_err(|e| e.to_string())?;
        let target = dst.join(relative);

        if entry_path.is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(entry_path, &target).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

/// List all available backups
#[tauri::command]
pub fn list_backups() -> Vec<BackupMetadata> {
    let backup_dir = match get_backup_dir() {
        Some(d) if d.exists() => d,
        _ => return Vec::new(),
    };

    let mut backups: Vec<BackupMetadata> = fs::read_dir(&backup_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let metadata_path = e.path().join("_backup_metadata.json");
            fs::read_to_string(&metadata_path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
        })
        .collect();

    // Sort by creation time (newest first)
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    backups
}

/// Restore a specific backup
#[tauri::command]
pub fn restore_version_backup(backup_id: String) -> RestoreResult {
    restore_backup(&backup_id)
}

/// Delete a specific backup
#[tauri::command]
pub fn delete_backup(backup_id: String) -> BackupResult {
    let backup_dir = match get_backup_dir() {
        Some(d) => d,
        None => {
            return BackupResult {
                success: false,
                backup_id: None,
                error: Some("Could not determine backup directory".to_string()),
            }
        }
    };

    let backup_path = backup_dir.join(&backup_id);
    if !backup_path.exists() {
        return BackupResult {
            success: false,
            backup_id: None,
            error: Some(format!("Backup not found: {}", backup_id)),
        };
    }

    if let Err(e) = fs::remove_dir_all(&backup_path) {
        return BackupResult {
            success: false,
            backup_id: None,
            error: Some(format!("Failed to delete backup: {}", e)),
        };
    }

    BackupResult {
        success: true,
        backup_id: Some(backup_id),
        error: None,
    }
}

/// Get total backup size
#[tauri::command]
pub fn get_backup_size() -> u64 {
    let backup_dir = match get_backup_dir() {
        Some(d) if d.exists() => d,
        _ => return 0,
    };

    calc_dir_size(&backup_dir)
}

/// Clear all backups
#[tauri::command]
pub fn clear_all_backups() -> BackupResult {
    let backup_dir = match get_backup_dir() {
        Some(d) => d,
        None => {
            return BackupResult {
                success: false,
                backup_id: None,
                error: Some("Could not determine backup directory".to_string()),
            }
        }
    };

    if !backup_dir.exists() {
        return BackupResult {
            success: true,
            backup_id: None,
            error: None,
        };
    }

    if let Err(e) = fs::remove_dir_all(&backup_dir) {
        return BackupResult {
            success: false,
            backup_id: None,
            error: Some(format!("Failed to clear backups: {}", e)),
        };
    }

    BackupResult {
        success: true,
        backup_id: None,
        error: None,
    }
}
