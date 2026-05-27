use tauri::{command, AppHandle, Manager};
use crate::config::{load_config, save_config};

#[command]
pub fn set_avatar(app_handle: AppHandle, path: String) -> Result<String, String> {
    let config = load_config();
    let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let avatars_dir = app_dir.join("avatars");
    std::fs::create_dir_all(&avatars_dir).map_err(|e| e.to_string())?;

    let file_path = avatars_dir.join(format!("{}.png", config.device_id));

    let base64_data = if path.contains(',') {
        path.split(',').nth(1).unwrap_or(&path).to_string()
    } else {
        path
    };

    let decoded = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        base64_data.trim()
    ).map_err(|e| e.to_string())?;

    std::fs::write(&file_path, decoded).map_err(|e| e.to_string())?;

    Ok(file_path.to_string_lossy().to_string())
}

#[command]
pub fn get_avatar(app_handle: AppHandle) -> Result<String, String> {
    let config = load_config();
    let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let file_path = app_dir.join("avatars").join(format!("{}.png", config.device_id));

    if !file_path.exists() {
        return Ok(String::new());
    }

    let data = std::fs::read(&file_path).map_err(|e| e.to_string())?;
    let base64_str = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &data
    );

    Ok(format!("data:image/png;base64,{}", base64_str))
}

#[command]
pub fn set_avatar_preset(preset: String, app_handle: AppHandle) -> Result<(), String> {
    let mut config = load_config();
    config.avatar_preset = preset.clone();
    save_config(&config).map_err(|e| e.to_string())?;
    
    // 如果切换到预设头像，删除旧自定义头像避免覆盖
    if !preset.is_empty() {
        let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
        let file_path = app_dir.join("avatars").join(format!("{}.png", config.device_id));
        let _ = std::fs::remove_file(&file_path);
    }
    
    Ok(())
}
