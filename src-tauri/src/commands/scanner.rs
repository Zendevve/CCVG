//! Version scanning functionality
//! Migrated from original eframe/egui main.rs

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

// Re-export path functions from paths module for backwards compatibility
pub use super::paths::{get_capcut_apps_path, get_capcut_root_path};

/// Information about an installed CapCut version
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub name: String,
    pub path: String,
    pub size_mb: f64,
}

/// Archive version from the curated list
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchiveVersion {
    pub persona: String,
    pub version: String,
    pub description: String,
    pub features: Vec<String>,
    pub download_url: String,
    pub risk_level: String,
}

/// Get curated archive versions
#[tauri::command]
pub fn get_archive_versions() -> Vec<ArchiveVersion> {
    vec![
        ArchiveVersion {
            persona: "Offline Purist".to_string(),
            version: "1.5.0".to_string(),
            description: "Zero cloud dependencies. Unrestricted 4K export.".to_string(),
            features: vec!["Clean UI".to_string(), "Offline Only".to_string(), "No Nags".to_string()],
            download_url: "https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_1_5_0_230_capcutpc_0.exe".to_string(),
            risk_level: "Low".to_string(),
        },
        ArchiveVersion {
            persona: "Audio Engineer".to_string(),
            version: "2.5.4".to_string(),
            description: "Multi-track audio & stable mixer. The golden era.".to_string(),
            features: vec!["Multi-Track".to_string(), "Audio Mixer".to_string(), "Keyframes".to_string()],
            download_url: "https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_2_5_4_810_capcutpc_0_creatortool.exe".to_string(),
            risk_level: "Low".to_string(),
        },
        ArchiveVersion {
            persona: "Classic Pro".to_string(),
            version: "2.9.0".to_string(),
            description: "Most free features before the generic paywalls.".to_string(),
            features: vec!["Max Free Features".to_string(), "Stable".to_string(), "Legacy UI".to_string()],
            download_url: "https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_2_9_0_966_capcutpc_0_creatortool.exe".to_string(),
            risk_level: "Medium".to_string(),
        },
        ArchiveVersion {
            persona: "Modern Stable".to_string(),
            version: "3.2.0".to_string(),
            description: "Good balance of modern features vs paywalls.".to_string(),
            features: vec!["Modern UI".to_string(), "Smooth".to_string(), "Balanced".to_string()],
            download_url: "https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_3_2_0_1106_capcutpc_0_creatortool.exe".to_string(),
            risk_level: "Medium".to_string(),
        },
        ArchiveVersion {
            persona: "Creator".to_string(),
            version: "3.9.0".to_string(),
            description: "Last version with free auto-captions (High Risk).".to_string(),
            features: vec!["Auto-Captions".to_string(), "AI Features".to_string(), "Effects".to_string()],
            download_url: "https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_3_9_0_1459_capcutpc_0_creatortool.exe".to_string(),
            risk_level: "High".to_string(),
        },
        ArchiveVersion {
            persona: "Power User".to_string(),
            version: "4.0.0".to_string(),
            description: "Track height adjustment & markers. Stricter paywall.".to_string(),
            features: vec!["Track Zoom".to_string(), "Markers".to_string(), "Adv Features".to_string()],
            download_url: "https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_4_0_0_1539_capcutpc_0_creatortool.exe".to_string(),
            risk_level: "Medium".to_string(),
        },
    ]
}

/// Calculate directory size recursively
fn calculate_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

/// Scan for installed CapCut versions
#[tauri::command]
pub async fn scan_versions() -> Vec<VersionInfo> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        let apps_path = match get_capcut_apps_path() {
            Some(p) if p.exists() => p,
            _ => return Vec::new(),
        };

        let mut versions: Vec<VersionInfo> = fs::read_dir(&apps_path)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .map(|p| {
                let name = p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let size_mb = calculate_dir_size(&p) as f64 / (1024.0 * 1024.0);
                VersionInfo {
                    name,
                    path: p.to_string_lossy().to_string(),
                    size_mb,
                }
            })
            .collect();

        // Sort by version name (oldest first) using simple string comparison
        versions.sort_by(|a, b| a.name.cmp(&b.name));
        versions
    })
    .await;

    result.unwrap_or_default()
}

/// Get CapCut installation paths
#[tauri::command]
pub fn get_capcut_paths() -> Option<(String, String)> {
    let apps = get_capcut_apps_path()?;
    let root = get_capcut_root_path()?;

    if apps.exists() {
        Some((apps.to_string_lossy().to_string(), root.to_string_lossy().to_string()))
    } else {
        None
    }
}
