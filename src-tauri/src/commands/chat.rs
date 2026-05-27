use tauri::command;
use tauri::Emitter;
use tauri::Manager;
use crate::models::NetworkMessage;
use crate::network::server::ConnectionPool;
use crate::config::load_config;
use crate::db::DbPool;
use base64::Engine;

#[command]
pub fn send_chat_message(
    content: String,
    message_type: String,
    app_handle: tauri::AppHandle,
    db: tauri::State<DbPool>,
) -> Result<(), String> {
    if content.len() > 5_000_000 {
        return Err("图片太大，请发送小于 5MB 的图片".to_string());
    }
    let config = load_config();
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp();
    let msg = NetworkMessage::ChatMessage {
        id: id.clone(),
        sender_id: config.device_id.clone(),
        sender_name: config.username.clone(),
        content: content.clone(),
        message_type: message_type.clone(),
    };
    
    // 保存到数据库
    if let Ok(conn) = db.lock() {
        let _ = conn.execute(
            "INSERT INTO chat_messages (id, sender_id, sender_name, message_type, content, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [&id, &config.device_id, &config.username, &message_type, &content, &timestamp.to_string()],
        );
    }
    
    // 广播给所有 peers
    if let Some(state) = app_handle.try_state::<ConnectionPool>() {
        crate::network::client::broadcast_message(state.inner(), &msg);
    }
    
    // 也推送给前端自己（本地显示）
    let _ = app_handle.emit("network-message", serde_json::json!({
        "peer_id": config.device_id,
        "message": msg
    }));
    
    Ok(())
}

#[command]
pub fn clear_chat_screen(app_handle: tauri::AppHandle, db: tauri::State<DbPool>) -> Result<(), String> {
    let msg = NetworkMessage::ClearScreen;
    
    // 清空数据库
    if let Ok(conn) = db.lock() {
        let _ = conn.execute("DELETE FROM chat_messages", []);
    }
    
    if let Some(state) = app_handle.try_state::<ConnectionPool>() {
        crate::network::client::broadcast_message(state.inner(), &msg);
    }
    
    let _ = app_handle.emit("network-message", serde_json::json!({
        "peer_id": "system",
        "message": msg
    }));
    
    Ok(())
}

#[command]
pub fn send_chat_file(
    file_path: String,
    app_handle: tauri::AppHandle,
    db: tauri::State<DbPool>,
) -> Result<(), String> {
    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    // 读取文件内容
    let content = std::fs::read(&file_path).map_err(|e| format!("读取文件失败: {}", e))?;
    let content_base64 = base64::engine::general_purpose::STANDARD.encode(&content);
    
    // 保存到本地下载目录
    let download_dir = crate::config::get_effective_download_dir(&app_handle);
    let local_path = download_dir.join(&file_name);
    std::fs::write(&local_path, &content).ok();
    
    // 先发送 file 类型的聊天消息
    let config = load_config();
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp();
    let chat_msg = NetworkMessage::ChatMessage {
        id: id.clone(),
        sender_id: config.device_id.clone(),
        sender_name: config.username.clone(),
        content: file_name.clone(),
        message_type: "file".to_string(),
    };
    
    // 保存到数据库
    if let Ok(conn) = db.lock() {
        let _ = conn.execute(
            "INSERT INTO chat_messages (id, sender_id, sender_name, message_type, content, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [&id, &config.device_id, &config.username, "file", &file_name, &timestamp.to_string()],
        );
    }
    
    // 广播 FileResponse（让接收方保存文件）
    let file_msg = NetworkMessage::FileResponse {
        folder_id: "".to_string(),
        file_path: file_name,
        content_base64,
    };
    
    if let Some(state) = app_handle.try_state::<ConnectionPool>() {
        crate::network::client::broadcast_message(state.inner(), &file_msg);
        crate::network::client::broadcast_message(state.inner(), &chat_msg);
    }
    
    // 本地显示
    let _ = app_handle.emit("network-message", serde_json::json!({
        "peer_id": config.device_id,
        "message": chat_msg
    }));
    
    Ok(())
}

#[command]
pub fn get_chat_history(db: tauri::State<DbPool>) -> Result<Vec<serde_json::Value>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, sender_id, sender_name, message_type, content, timestamp FROM chat_messages ORDER BY timestamp ASC LIMIT 200"
    ).map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "sender_id": row.get::<_, String>(1)?,
            "sender_name": row.get::<_, String>(2)?,
            "message_type": row.get::<_, String>(3)?,
            "content": row.get::<_, String>(4)?,
            "timestamp": row.get::<_, i64>(5)?,
        }))
    }).map_err(|e| e.to_string())?;
    
    let result: Result<Vec<_>, _> = rows.collect();
    result.map_err(|e| e.to_string())
}
