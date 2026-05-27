use tauri::command;
use crate::config::{AppConfig, load_config, save_config};

#[command]
pub fn get_config() -> Result<AppConfig, String> {
    Ok(load_config())
}

#[command]
pub fn set_username(username: String) -> Result<(), String> {
    let mut config = load_config();
    config.username = username;
    save_config(&config).map_err(|e| e.to_string())
}

#[command]
pub fn set_download_dir(download_dir: String) -> Result<(), String> {
    let mut config = load_config();
    config.download_dir = download_dir;
    save_config(&config).map_err(|e| e.to_string())
}

#[command]
pub fn set_sync_interval(sync_interval_secs: u64) -> Result<(), String> {
    let mut config = load_config();
    config.sync_interval_secs = sync_interval_secs;
    save_config(&config).map_err(|e| e.to_string())
}

#[command]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let mut config = load_config();
    config.autostart = enabled;
    if enabled {
        crate::system::autostart::setup_autostart().map_err(|e| e.to_string())?;
    } else {
        crate::system::autostart::remove_autostart().map_err(|e| e.to_string())?;
    }
    save_config(&config).map_err(|e| e.to_string())
}

#[command]
pub fn get_autostart_status() -> Result<bool, String> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_READ) {
            return Ok(key.get_value::<String, _>("ShimmenLanSuite").is_ok());
        }
    }
    Ok(false)
}

#[command]
pub fn set_screen_fps(fps: u64) -> Result<(), String> {
    let mut config = load_config();
    config.screen_fps = fps.clamp(5, 30);
    save_config(&config).map_err(|e| e.to_string())
}

#[command]
pub fn set_screen_resolution(resolution: u64) -> Result<(), String> {
    let mut config = load_config();
    config.screen_resolution = resolution;
    save_config(&config).map_err(|e| e.to_string())
}


