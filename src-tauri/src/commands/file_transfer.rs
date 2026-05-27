use tauri::command;
use tauri::Manager;
use crate::network::server::ConnectionPool;
use base64::Engine;
use std::fs;

#[command]
pub async fn send_file_to_peer(
    peer_id: String,
    file_path: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    let metadata = fs::metadata(&file_path).map_err(|e| e.to_string())?;
    let _file_size = metadata.len();
    
    if let Some(state) = app_handle.try_state::<ConnectionPool>() {
        let pool = state.inner();
        crate::network::client::send_file_in_chunks(
            pool, &peer_id, "", &file_name, &file_path,
        ).map_err(|e| e.to_string())?;
    } else {
        return Err("Connection pool not available".to_string());
    }
    
    Ok(())
}

#[command]
pub fn get_download_dir() -> Result<String, String> {
    let dir = dirs::download_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    Ok(dir.to_string_lossy().to_string())
}

#[command]
pub fn get_app_download_dir(app_handle: tauri::AppHandle) -> Result<String, String> {
    let dir = crate::config::get_effective_download_dir(&app_handle);
    Ok(dir.to_string_lossy().to_string())
}

#[command]
pub fn read_file_base64(file_path: String) -> Result<String, String> {
    let content = fs::read(&file_path).map_err(|e| format!("读取文件失败: {}", e))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&content))
}
