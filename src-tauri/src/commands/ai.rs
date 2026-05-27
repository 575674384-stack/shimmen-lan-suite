use tauri::command;
use tauri::Emitter;
use crate::db::DbPool;
use crate::models::AiConfig;

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
    
    // ⚠️ 安全：ai_config 包含 API 密钥，绝不可广播到网络
    // 仅本地保存，不通过 StateSync 同步
    let _ = app_handle.emit("network-message", serde_json::json!({
        "peer_id": "system",
        "message": {
            "type": "StateSync",
            "payload": {
                "table": "ai_config",
                "data": {
                    "base_url": &base_url,
                    "model": &model,
                    "updated_at": now,
                },
            },
        },
    }));
    
    Ok(())
}
