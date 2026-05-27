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


