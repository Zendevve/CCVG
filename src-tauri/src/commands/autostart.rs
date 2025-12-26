//! Windows startup/autostart management
//! Adds/removes the app from Windows startup via Registry

use winreg::enums::*;
use winreg::RegKey;
use std::env;

const STARTUP_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "CCVersionGuard";

/// Check if auto-start is enabled
#[tauri::command]
pub fn get_autostart_enabled() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    if let Ok(key) = hkcu.open_subkey(STARTUP_KEY) {
        let result: Result<String, _> = key.get_value(APP_NAME);
        return result.is_ok();
    }

    false
}

/// Enable or disable auto-start
#[tauri::command]
pub fn set_autostart_enabled(enabled: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    if enabled {
        // Get current executable path
        let exe_path = env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;

        let exe_str = exe_path.to_string_lossy().to_string();

        let key = hkcu
            .open_subkey_with_flags(STARTUP_KEY, winreg::enums::KEY_WRITE)
            .map_err(|e| format!("Failed to open registry key: {}", e))?;

        key.set_value(APP_NAME, &exe_str)
            .map_err(|e| format!("Failed to set registry value: {}", e))?;
    } else {
        let key = hkcu
            .open_subkey_with_flags(STARTUP_KEY, winreg::enums::KEY_WRITE)
            .map_err(|e| format!("Failed to open registry key: {}", e))?;

        // Ignore error if key doesn't exist
        let _ = key.delete_value(APP_NAME);
    }

    Ok(())
}
