use tauri::command;
use tauri::Manager;
use crate::network::server::ConnectionPool;
use crate::models::NetworkMessage;
use std::fs;
use base64::Engine;

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
    
    let content = fs::read(&file_path).map_err(|e| e.to_string())?;
    let content_base64 = base64::engine::general_purpose::STANDARD.encode(&content);
    
    let msg = NetworkMessage::FileResponse {
        folder_id: "".to_string(),
        file_path: file_name,
        content_base64,
    };
    
    if let Some(state) = app_handle.try_state::<ConnectionPool>() {
        let pool = state.inner();
        let json = serde_json::to_vec(&msg).map_err(|e| e.to_string())?;
        let mut p = pool.lock().map_err(|e| e.to_string())?;
        if let Some(conn) = p.get_mut(&peer_id) {
            conn.send_message(&json).map_err(|e| e.to_string())?;
        } else {
            return Err("Peer not connected".to_string());
        }
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
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| e.to_string())?;
    let download_dir = app_dir.join("downloads");
    std::fs::create_dir_all(&download_dir).map_err(|e| e.to_string())?;
    Ok(download_dir.to_string_lossy().to_string())
}

#[command]
pub fn read_file_base64(file_path: String) -> Result<String, String> {
    let content = fs::read(&file_path).map_err(|e| format!("读取文件失败: {}", e))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&content))
}
