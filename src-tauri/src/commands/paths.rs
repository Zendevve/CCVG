//! Path resolution utilities with registry lookup support
//! Handles custom CapCut installation paths beyond the default LOCALAPPDATA location

use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;

/// CapCut installation paths
#[derive(Debug, Clone, serde::Serialize)]
pub struct CapCutPaths {
    /// Root CapCut directory (e.g., C:\Users\X\AppData\Local\CapCut)
    pub root: PathBuf,
    /// Apps directory containing versions (e.g., ...\CapCut\Apps)
    pub apps: PathBuf,
    /// Source of the path detection
    pub source: PathDetectionSource,
}

/// How the path was detected
#[derive(Debug, Clone, serde::Serialize)]
pub enum PathDetectionSource {
    Registry,
    DefaultLocation,
    Custom(String),
}

/// Registry keys to check for CapCut installation
const REGISTRY_PATHS: &[(&str, &str)] = &[
    // Standard uninstall location
    (r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\CapCut", "InstallLocation"),
    // 64-bit uninstall location
    (r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\CapCut", "InstallLocation"),
    // App Paths
    (r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\CapCut.exe", "Path"),
];

/// Try to find CapCut installation path from Windows Registry
fn find_from_registry() -> Option<PathBuf> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    for (key_path, value_name) in REGISTRY_PATHS {
        // Try HKLM first
        if let Ok(key) = hklm.open_subkey(key_path) {
            let path_result: Result<String, _> = key.get_value(value_name);
            if let Ok(path) = path_result {
                let path_buf = PathBuf::from(&path);
                if path_buf.exists() {
                    return Some(path_buf);
                }
            }
        }
        // Then try HKCU
        if let Ok(key) = hkcu.open_subkey(key_path) {
            let path_result: Result<String, _> = key.get_value(value_name);
            if let Ok(path) = path_result {
                let path_buf = PathBuf::from(&path);
                if path_buf.exists() {
                    return Some(path_buf);
                }
            }
        }
    }

    None
}

/// Get the default CapCut installation path (LOCALAPPDATA)
fn get_default_path() -> Option<PathBuf> {
    std::env::var("LOCALAPPDATA")
        .ok()
        .map(|p| PathBuf::from(p).join("CapCut"))
}

/// Resolve CapCut installation paths with fallback logic
/// 1. Check Windows Registry for custom install paths
/// 2. Fall back to default LOCALAPPDATA location
/// 3. Return None if not found
pub fn resolve_capcut_paths() -> Option<CapCutPaths> {
    // Try registry first
    if let Some(root) = find_from_registry() {
        let apps = if root.join("Apps").exists() {
            root.join("Apps")
        } else {
            // Some installations may have Apps directly in the root
            root.clone()
        };

        if apps.exists() {
            return Some(CapCutPaths {
                root,
                apps,
                source: PathDetectionSource::Registry,
            });
        }
    }

    // Fall back to default location
    if let Some(root) = get_default_path() {
        let apps = root.join("Apps");
        if root.exists() || apps.exists() {
            return Some(CapCutPaths {
                root,
                apps,
                source: PathDetectionSource::DefaultLocation,
            });
        }
    }

    None
}

/// Get CapCut Apps path (convenience function)
pub fn get_capcut_apps_path() -> Option<PathBuf> {
    resolve_capcut_paths().map(|p| p.apps)
}

/// Get CapCut root path (convenience function)
pub fn get_capcut_root_path() -> Option<PathBuf> {
    resolve_capcut_paths().map(|p| p.root)
}

/// Set a custom CapCut path (for user-specified installations)
/// Returns the CapCutPaths if valid, None otherwise
pub fn validate_custom_path(custom_path: &str) -> Option<CapCutPaths> {
    let root = PathBuf::from(custom_path);
    if !root.exists() {
        return None;
    }

    // Check if this is a valid CapCut installation
    let apps = if root.join("Apps").exists() {
        root.join("Apps")
    } else if root.file_name().map(|n| n == "Apps").unwrap_or(false) {
        // User specified the Apps folder directly
        root.clone()
    } else {
        return None;
    };

    Some(CapCutPaths {
        root: if apps == root { apps.parent()?.to_path_buf() } else { root },
        apps,
        source: PathDetectionSource::Custom(custom_path.to_string()),
    })
}

/// Get path detection info for frontend display
#[tauri::command]
pub fn get_path_info() -> Option<CapCutPaths> {
    resolve_capcut_paths()
}

/// Validate a custom CapCut path provided by user
#[tauri::command]
pub fn validate_custom_capcut_path(path: String) -> Option<CapCutPaths> {
    validate_custom_path(&path)
}
