//! Version scanning functionality
//! Migrated from original eframe/egui main.rs

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

// Re-export path functions from paths module for backwards compatibility
pub use super::paths::{get_capcut_apps_path, get_capcut_root_path};

/// Complete list of all CapCut versions up to 5.4.0 Beta 6
/// Format: "Label|BaseVersion|URL"
const ALL_VERSIONS_DATA: &str = r#"
5.4.0 (Beta6)|5.4.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_4_0_1991_beta6_capcutpc_beta_creatortool.exe
5.4.0 (Beta5)|5.4.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_4_0_1988_beta5_capcutpc_beta_creatortool.exe
5.4.0 (Beta4)|5.4.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_4_0_1982_beta4_capcutpc_beta_creatortool.exe
5.4.0 (Beta3)|5.4.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_4_0_1979_beta3_capcutpc_beta_creatortool.exe
5.4.0 (Beta2)|5.4.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_4_0_1978_beta2_capcutpc_beta_creatortool.exe
5.4.0 (Beta1)|5.4.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_4_0_1976_beta1_capcutpc_beta_creatortool.exe
5.3.0 (Latest)|5.3.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_3_0_1964_capcutpc_0_creatortool.exe
5.3.0 (Test2)|5.3.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_3_0_1961_capcutpc_0_creatortool.exe
5.3.0 (Test1)|5.3.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_3_0_1957_capcutpc_0_creatortool.exe
5.3.0 (Beta5)|5.3.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_3_0_1962_beta5_capcutpc_beta_creatortool.exe
5.3.0 (Beta4)|5.3.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_3_0_1956_beta4_capcutpc_beta_creatortool.exe
5.3.0 (Beta3)|5.3.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_3_0_1952_beta3_capcutpc_beta_creatortool.exe
5.3.0 (Beta2)|5.3.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_3_0_1949_beta2_capcutpc_beta_creatortool.exe
5.3.0 (Test1 Beta2)|5.3.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_3_0_1947_beta2_capcutpc_beta_creatortool.exe
5.3.0 (Beta1)|5.3.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_3_0_1942_beta1_capcutpc_beta_creatortool.exe
5.3.0 (Test1 Beta1)|5.3.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_3_0_1941_beta1_capcutpc_beta_creatortool.exe
5.2.0 (Latest)|5.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_2_0_1950_capcutpc_0_creatortool.exe
5.2.0 (Test3)|5.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_2_0_1946_capcutpc_0_creatortool.exe
5.2.0 (Test2)|5.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_2_0_1940_capcutpc_0_creatortool.exe
5.2.0 (Test1)|5.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_2_0_1939_capcutpc_0_creatortool.exe
5.2.0 (Beta8)|5.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_2_0_1945_beta8_capcutpc_beta_creatortool.exe
5.2.0 (Beta7)|5.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_2_0_1937_beta7_capcutpc_beta_creatortool.exe
5.2.0 (Beta6)|5.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_2_0_1934_beta6_capcutpc_beta_creatortool.exe
5.2.0 (Beta5)|5.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_2_0_1933_beta5_capcutpc_beta_creatortool.exe
5.2.0 (Beta4)|5.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_2_0_1929_beta4_capcutpc_beta_creatortool.exe
5.2.0 (Beta3)|5.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_2_0_1928_beta3_capcutpc_beta_creatortool.exe
5.2.0 (Beta2)|5.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_2_0_1925_beta2_capcutpc_beta_creatortool.exe
5.2.0 (Beta1)|5.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_2_0_1923_beta1_capcutpc_beta_creatortool.exe
5.1.0 (Latest)|5.1.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_1_0_1926_capcutpc_0_creatortool.exe
5.1.0 (Test2)|5.1.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_1_0_1922_capcutpc_0_creatortool.exe
5.1.0 (Test1)|5.1.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_1_0_1919_capcutpc_0_creatortool.exe
5.1.0 (Beta7)|5.1.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_1_0_1924_beta7_capcutpc_beta_creatortool.exe
5.1.0 (Beta6)|5.1.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_1_0_1920_beta6_capcutpc_beta_creatortool.exe
5.1.0 (Beta5)|5.1.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_1_0_1918_beta5_capcutpc_beta_creatortool.exe
5.1.0 (Beta4)|5.1.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_1_0_1916_beta4_capcutpc_beta_creatortool.exe
5.1.0 (Beta3)|5.1.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_1_0_1913_beta3_capcutpc_beta_creatortool.exe
5.1.0 (Beta2)|5.1.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_1_0_1910_beta2_capcutpc_beta_creatortool.exe
5.1.0 (Beta1)|5.1.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_1_0_1907_beta1_capcutpc_beta_creatortool.exe
5.0.0 (Latest)|5.0.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_0_0_1908_capcutpc_0_creatortool.exe
5.0.0 (Latest v2)|5.0.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_0_0_1903_capcutpc_0_creatortool.exe
5.0.0 (Test1)|5.0.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_0_0_1899_capcutpc_0_creatortool.exe
5.0.0 (Beta6)|5.0.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_0_0_1906_beta6_capcutpc_beta_creatortool.exe
5.0.0 (Beta5)|5.0.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_0_0_1905_beta5_capcutpc_beta_creatortool.exe
5.0.0 (Beta4)|5.0.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_0_0_1902_beta4_capcutpc_beta_creatortool.exe
5.0.0 (Beta3)|5.0.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_0_0_1901_beta3_capcutpc_beta_creatortool.exe
5.0.0 (Beta2)|5.0.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_0_0_1898_beta2_capcutpc_beta_creatortool.exe
5.0.0 (Beta1)|5.0.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_5_0_0_1897_beta1_capcutpc_beta_creatortool.exe
4.7.0 (Latest)|4.7.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_4_7_0_1869_capcutpc_0_creatortool.exe
4.6.0 (Latest)|4.6.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_4_6_0_1842_capcutpc_0_creatortool.exe
4.5.0 (Latest)|4.5.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_4_5_0_1815_capcutpc_0_creatortool.exe
4.4.0 (Latest)|4.4.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_4_4_0_1783_capcutpc_0_creatortool.exe
4.3.0 (Latest)|4.3.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_4_3_0_1754_capcutpc_0_creatortool.exe
4.2.0 (Latest)|4.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_4_2_0_1728_capcutpc_0_creatortool.exe
4.1.0 (Latest)|4.1.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_4_1_0_1706_capcutpc_0_creatortool.exe
4.0.0 (Latest)|4.0.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_4_0_0_1680_capcutpc_0_creatortool.exe
3.9.0 (Latest)|3.9.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_3_9_0_1663_capcutpc_0_creatortool.exe
3.8.0 (Latest)|3.8.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_3_8_0_1638_capcutpc_0_creatortool.exe
3.7.0 (Latest)|3.7.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_3_7_0_1622_capcutpc_0_creatortool.exe
3.6.0 (Latest)|3.6.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_3_6_0_1596_capcutpc_0_creatortool.exe
3.5.0 (Latest)|3.5.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_3_5_0_1578_capcutpc_0_creatortool.exe
3.4.0 (Latest)|3.4.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_3_4_0_1559_capcutpc_0_creatortool.exe
3.3.0 (Latest)|3.3.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_3_3_0_1535_capcutpc_0_creatortool.exe
3.2.0 (Latest)|3.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_3_2_0_1516_capcutpc_0_creatortool.exe
3.1.0 (Latest)|3.1.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_3_1_0_1497_capcutpc_0_creatortool.exe
3.0.0 (Latest)|3.0.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_3_0_0_1478_capcutpc_0_creatortool.exe
2.9.0 (Latest)|2.9.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_2_9_0_1457_capcutpc_0_creatortool.exe
2.8.0 (Latest)|2.8.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_2_8_0_1441_capcutpc_0_creatortool.exe
2.7.0 (Latest)|2.7.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_2_7_0_1435_capcutpc_0_creatortool.exe
2.6.0 (Latest)|2.6.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_2_6_0_1269_capcutpc_0.exe
2.5.0 (Latest)|2.5.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_2_5_0_1222_capcutpc_0.exe
2.4.0 (Latest)|2.4.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_2_4_0_1186_capcutpc_0.exe
2.3.0 (Latest)|2.3.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_2_3_0_1158_capcutpc_0.exe
2.2.0 (Latest)|2.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_2_2_0_1112_capcutpc_0.exe
2.1.0 (Latest)|2.1.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_2_1_0_1038_capcutpc_0.exe
2.0.0 (Latest)|2.0.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_2_0_0_822_capcutpc_0.exe
1.9.0 (Latest)|1.9.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_1_9_0_699_capcutpc_0.exe
1.8.0 (Latest)|1.8.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_1_8_0_633_capcutpc_0.exe
1.7.0 (Latest)|1.7.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_1_7_0_587_capcutpc_0.exe
1.6.0 (Latest)|1.6.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_1_6_0_519_capcutpc_0.exe
1.5.0 (Latest)|1.5.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_1_5_0_433_capcutpc_0.exe
1.4.0 (Latest)|1.4.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_1_4_0_334_capcutpc_0.exe
1.3.0 (Latest)|1.3.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_1_3_0_262_capcutpc_0.exe
1.2.0 (Latest)|1.2.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_1_2_0_213_capcutpc_0.exe
1.1.0 (Latest)|1.1.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_1_1_0_155_capcutpc_0.exe
1.0.0 (Latest)|1.0.0|https://lf16-capcut.faceulv.com/obj/capcutpc-packages-us/packages/CapCut_1_0_0_44_capcutpc_0.exe
"#;


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

/// Get ALL archive versions (complete list)
/// This includes every version up to 5.4.0 Beta 6 (last version where CC Version Guard works)
#[tauri::command]
pub fn get_all_archive_versions() -> Vec<ArchiveVersion> {
    // Parse version data from compact format
    // Format: "Label|BaseVersion|URL"
    ALL_VERSIONS_DATA
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 3 {
                return None;
            }

            let label = parts[0].trim();
            let version = parts[1].trim();
            let url = parts[2].trim();

            // Determine risk level based on version
            let risk_level = if version.starts_with('5') || version.starts_with('4') {
                "High"
            } else if version.starts_with('3') {
                "Medium"
            } else {
                "Low"
            };

            // Special description for the last compatible version
            let description = if label.contains("5.4.0") && label.contains("Beta6") {
                "Last version compatible with CC Version Guard"
            } else if label.contains("Beta") || label.contains("Test") {
                "Beta/Test release"
            } else if label.contains("Latest") {
                "Stable release"
            } else {
                "Release version"
            };

            Some(ArchiveVersion {
                persona: label.to_string(),
                version: version.to_string(),
                description: description.to_string(),
                features: vec![],
                download_url: url.to_string(),
                risk_level: risk_level.to_string(),
            })
        })
        .collect()
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
        Some((
            apps.to_string_lossy().to_string(),
            root.to_string_lossy().to_string(),
        ))
    } else {
        None
    }
}
