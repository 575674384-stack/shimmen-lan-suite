use tauri::command;
use tauri::Emitter;
use tauri::Manager;
use crate::db::DbPool;
use crate::models::{AiConfig, NetworkMessage};
use crate::network::server::ConnectionPool;
use crate::network::client::broadcast_message;

#[command]
pub fn get_ai_config(db: tauri::State<DbPool>) -> Result<AiConfig, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT api_key, base_url, model, updated_at FROM ai_config WHERE id = 1"
    ).map_err(|e| e.to_string())?;
    
    let config = stmt.query_row([], |row| {
        Ok(AiConfig {
            api_key: row.get(0)?,
            base_url: row.get(1)?,
            model: row.get(2)?,
            updated_at: row.get(3)?,
        })
    });
    
    match config {
        Ok(c) => Ok(c),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AiConfig {
            api_key: String::new(),
            base_url: String::new(),
            model: String::new(),
            updated_at: 0,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[command]
pub fn set_ai_config(
    api_key: String,
    base_url: String,
    model: String,
    db: tauri::State<DbPool>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    
    conn.execute(
        "INSERT OR REPLACE INTO ai_config (id, api_key, base_url, model, updated_at) VALUES (1, ?1, ?2, ?3, ?4)",
        [&api_key, &base_url, &model, &now.to_string()],
    ).map_err(|e| e.to_string())?;
    
    let config = AiConfig {
        api_key,
        base_url,
        model,
        updated_at: now,
    };
    
    let msg = NetworkMessage::StateSync {
        table: "ai_config".to_string(),
        data: serde_json::to_value(&config).unwrap_or_default(),
        version: serde_json::json!({"updated_at": now}),
    };
    
    if let Some(state) = app_handle.try_state::<ConnectionPool>() {
        broadcast_message(state.inner(), &msg);
    }
    
    let _ = app_handle.emit("network-message", serde_json::json!({
        "peer_id": "system",
        "message": msg,
    }));
    
    Ok(())
}
