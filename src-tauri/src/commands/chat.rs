use tauri::command;
use tauri::Emitter;
use tauri::Manager;
use tracing::{info, error, warn};
use crate::models::NetworkMessage;
use crate::network::server::ConnectionPool;
use crate::config::load_config;
use crate::db::DbPool;

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
    info!("send_chat_message called, content_len={}, type={}", content.len(), message_type);
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
    
    // 保存到数据库（加详细日志以便排查）
    {
        let conn = match db.lock() {
            Ok(c) => c,
            Err(e) => {
                error!("db.lock() failed: {}", e);
                return Err(format!("数据库锁获取失败: {}", e));
            }
        };
        if let Err(e) = conn.execute(
            "INSERT INTO chat_messages (id, sender_id, sender_name, message_type, content, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [&id, &config.device_id, &config.username, &message_type, &content, &timestamp.to_string()],
        ) {
            error!("INSERT chat_messages failed: {}", e);
            return Err(format!("保存消息到数据库失败: {}", e));
        }
        info!("chat message saved to db, id={}", id);
    }
    
    // 广播给所有 peers
    if let Some(state) = app_handle.try_state::<ConnectionPool>() {
        let pool = state.inner();
        let count = {
            let p = pool.lock().map_err(|e| e.to_string())?;
            p.len()
        };
        info!("broadcasting chat message to {} peers", count);
        crate::network::client::broadcast_message(pool, &msg);
    } else {
        warn!("ConnectionPool not found, cannot broadcast");
    }
    
    // 也推送给前端自己（本地显示）
    let _ = app_handle.emit("network-message", serde_json::json!({
        "peer_id": config.device_id,
        "message": msg
    }));
    info!("send_chat_message completed successfully");
    Ok(())
}

#[command]
pub fn clear_chat_screen(app_handle: tauri::AppHandle, db: tauri::State<DbPool>) -> Result<(), String> {
    let msg = NetworkMessage::ClearScreen;
    
    // 清空数据库（锁 poison 时直接报错）
    {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM chat_messages", [])
            .map_err(|e| format!("清空聊天记录失败: {}", e))?;
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
    
    // 保存到本地下载目录（直接复制，避免大文件载入内存）
    let download_dir = crate::config::get_effective_download_dir(&app_handle);
    let local_path = download_dir.join(&file_name);
    if let Err(e) = std::fs::copy(&file_path, &local_path) {
        return Err(format!("复制文件到下载目录失败: {}", e));
    }
    
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
    
    // 保存到数据库（锁 poison 时直接报错）
    {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO chat_messages (id, sender_id, sender_name, message_type, content, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [&id, &config.device_id, &config.username, "file", &file_name, &timestamp.to_string()],
        ).map_err(|e| format!("保存文件消息到数据库失败: {}", e))?;
    }
    
    // 分块广播文件（经 Leader 转发给所有人）
    if let Some(state) = app_handle.try_state::<ConnectionPool>() {
        let pool = state.inner();
        let _ = crate::network::client::broadcast_file_in_chunks(
            pool, "", &file_name, &file_path,
        );
        crate::network::client::broadcast_message(pool, &chat_msg);
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
