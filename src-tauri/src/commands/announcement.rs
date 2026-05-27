use tauri::command;
use tauri::Manager;
use crate::db::DbPool;
use crate::models::{Announcement, NetworkMessage};
use crate::network::server::ConnectionPool;
use crate::network::client::broadcast_message;
use crate::config::load_config;

#[command]
pub fn get_announcements(db: tauri::State<DbPool>) -> Result<Vec<Announcement>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, title, content, is_pinned, created_by, updated_at FROM announcements ORDER BY is_pinned DESC, updated_at DESC"
    ).map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([], |row| {
        Ok(Announcement {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            is_pinned: row.get::<_, i32>(3)? != 0,
            created_by: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;
    
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[command]
pub fn save_announcement(
    announcement: Announcement,
    db: tauri::State<DbPool>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let config = load_config();
    let now = chrono::Utc::now().timestamp();
    let conn = db.lock().map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT OR REPLACE INTO announcements (id, title, content, is_pinned, created_by, updated_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        [
            announcement.id.clone(),
            announcement.title.clone(),
            announcement.content.clone(),
            if announcement.is_pinned { "1" } else { "0" }.to_string(),
            if announcement.created_by.is_empty() { config.device_id } else { announcement.created_by.clone() },
            now.to_string(),
            "[]".to_string(),
        ],
    ).map_err(|e| e.to_string())?;
    
    let msg = NetworkMessage::StateSync {
        table: "announcements".to_string(),
        data: serde_json::to_value(&announcement).unwrap_or_default(),
        version: serde_json::json!({"updated_at": now}),
    };
    
    if let Some(state) = app_handle.try_state::<ConnectionPool>() {
        broadcast_message(state.inner(), &msg);
    }
    
    Ok(())
}

#[command]
pub fn delete_announcement(id: String, db: tauri::State<DbPool>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM announcements WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    drop(conn);
    
    // Broadcast deletion to all peers
    if let Ok(list) = get_announcements(db) {
        let msg = crate::models::NetworkMessage::StateSync {
            table: "announcements".to_string(),
            data: serde_json::to_value(&list).unwrap_or(serde_json::Value::Null),
            version: serde_json::json!({}),
        };
        if let Some(state) = app_handle.try_state::<crate::network::server::ConnectionPool>() {
            crate::network::client::broadcast_message(state.inner(), &msg);
        }
    }
    
    Ok(())
}
