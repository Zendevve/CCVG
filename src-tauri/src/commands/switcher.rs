use std::path::PathBuf;
use std::fs;
use crate::commands::scanner::{get_capcut_apps_path, get_capcut_root_path};
use serde::Serialize;

#[derive(Serialize)]
pub struct SwitchResult {
    pub success: bool,
    pub message: String,
    pub logs: Vec<String>,
}

#[tauri::command]
pub fn switch_version(target_path: String) -> SwitchResult {
    let mut logs = Vec::new();
    let target_dir = PathBuf::from(&target_path);

    logs.push(format!("Initiating switch to version at: {:?}", target_dir));

    if !target_dir.exists() {
        logs.push("[!] Target directory does not exist".to_string());
        return SwitchResult {
            success: false,
            message: "Target version not found".to_string(),
            logs,
        };
    }

    // Identify version from path (e.g. .../3.1.0.100)
    let version_name = target_dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    logs.push(format!("Detected version: {}", version_name));

    // 1. Update ProductInfo.xml to point to this version
    // This is how CapCut launcher knows which EXE to run
    if let Some(root_path) = get_capcut_root_path() {
        let product_info_path = root_path.join("Apps").join("ProductInfo.xml");
        logs.push(format!("Updating ProductInfo at: {:?}", product_info_path));

        let target_exe = target_dir.join("CapCut.exe");

        // Simple XML replacer (robust enough for this specific file)
        let new_content = format!(
            r#"<?xml version="1.0" charset="utf-8"?>
<ProductInfo>
  <InstallPath>{}</InstallPath>
  <Version>{}</Version>
</ProductInfo>"#,
            target_exe.to_string_lossy(),
            version_name
        );

        // Remove Read-Only if present
        if let Ok(metadata) = fs::metadata(&product_info_path) {
            let mut perms = metadata.permissions();
            if perms.readonly() {
                perms.set_readonly(false);
                let _ = fs::set_permissions(&product_info_path, perms);
                logs.push("Removed Read-Only attribute from ProductInfo.xml".to_string());
            }
        }

        match fs::write(&product_info_path, new_content) {
            Ok(_) => logs.push("[OK] Updated ProductInfo.xml".to_string()),
            Err(e) => logs.push(format!("[!] Failed to write ProductInfo.xml: {}", e)),
        }
    }

    // 2. Update configure.ini (last_version)
    if let Some(apps_path) = get_capcut_apps_path() {
        let config_path = apps_path.join("configure.ini");
        logs.push(format!("Updating configure.ini at: {:?}", config_path));

        let config_content = format!("[Configure]\r\nlast_version={}\r\n", version_name);

        // Remove Read-Only if present
        if let Ok(metadata) = fs::metadata(&config_path) {
             let mut perms = metadata.permissions();
             if perms.readonly() {
                 perms.set_readonly(false);
                 let _ = fs::set_permissions(&config_path, perms);
             }
        }

        match fs::write(&config_path, config_content) {
             Ok(_) => logs.push("[OK] Updated configure.ini".to_string()),
             Err(e) => logs.push(format!("[!] Failed to write configure.ini: {}", e)),
        }
    }

    // 3. Ensure target is not renamed to backup
    // Some older versions might have been renamed by the protector previously
    // This function assumes we are dealing with standard folders, but we could add logic
    // to rename "_backup" folders back to normal if needed.

    SwitchResult {
        success: true,
        message: format!("Successfully switched to v{}", version_name),
        logs,
    }
}
