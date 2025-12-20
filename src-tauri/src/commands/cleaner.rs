//! Cache cleaning functionality
//! Migrated from original eframe/egui main.rs

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Get cache directories for CapCut
fn get_cache_dirs(capcut_root: &Path) -> Vec<PathBuf> {
    let user_data = capcut_root.join("User Data");
    vec![
        user_data.join("Cache"),
        user_data.join("Shadow_Cache"),
        user_data.join("Smart_Crop"),
    ]
}

/// Calculate directory size
fn calculate_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

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

/// Calculate total cache size in MB
#[tauri::command]
pub fn calculate_cache_size() -> f64 {
    let capcut_root = match std::env::var("LOCALAPPDATA") {
        Ok(p) => PathBuf::from(p).join("CapCut"),
        Err(_) => return 0.0,
    };

    let dirs = get_cache_dirs(&capcut_root);
    let total_bytes: u64 = dirs
        .iter()
        .filter(|d| d.exists())
        .map(|d| calculate_dir_size(d))
        .sum();

    total_bytes as f64 / (1024.0 * 1024.0)
}

/// Cache cleaning result
#[derive(serde::Serialize)]
pub struct CacheCleanResult {
    pub success: bool,
    pub cleaned_mb: f64,
    pub logs: Vec<String>,
}

/// Clean cache directories
#[tauri::command]
pub fn clean_cache() -> CacheCleanResult {
    let capcut_root = match std::env::var("LOCALAPPDATA") {
        Ok(p) => PathBuf::from(p).join("CapCut"),
        Err(_) => {
            return CacheCleanResult {
                success: false,
                cleaned_mb: 0.0,
                logs: vec!["Failed to get LOCALAPPDATA".to_string()],
            }
        }
    };

    let dirs = get_cache_dirs(&capcut_root);
    let mut total_cleaned: u64 = 0;
    let mut logs: Vec<String> = Vec::new();

    for dir in dirs {
        if dir.exists() {
            let name = dir.file_name().unwrap_or_default().to_string_lossy();
            let size = calculate_dir_size(&dir);
            logs.push(format!(
                "Cleaning: {} ({:.1} MB)",
                name,
                size as f64 / (1024.0 * 1024.0)
            ));

            if let Err(e) = unset_readonly_recursive(&dir) {
                logs.push(format!("[!] Warning: {}", e));
            }

            if let Err(e) = fs::remove_dir_all(&dir) {
                logs.push(format!("[!] Failed to clean {}: {}", name, e));
            } else {
                total_cleaned += size;
            }
        }
    }

    let cleaned_mb = total_cleaned as f64 / (1024.0 * 1024.0);
    logs.push(format!("[OK] Cleaned {:.1} MB of cache", cleaned_mb));

    CacheCleanResult {
        success: true,
        cleaned_mb,
        logs,
    }
}
