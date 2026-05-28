use tauri::command;
use tauri::Manager;
use crate::db::DbPool;
use crate::models::{SharedFolder, SyncStatus};
use crate::network::folder_cache::RemoteFolderCache;

#[command]
pub fn get_my_shared_folders(db: tauri::State<DbPool>) -> Result<Vec<SharedFolder>, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, owner_id, owner_name, local_path, name, sync_status FROM shared_folders"
    ).map_err(|e| e.to_string())?;
    
    let folders = stmt.query_map([], |row| {
        Ok(SharedFolder {
            id: row.get(0)?,
            owner_id: row.get(1)?,
            owner_name: row.get(2)?,
            local_path: row.get(3)?,
            name: row.get(4)?,
            sync_status: match row.get::<_, String>(5)?.as_str() {
                "syncing" => SyncStatus::Syncing,
                "paused" => SyncStatus::Paused,
                "error" => SyncStatus::Error,
                _ => SyncStatus::Paused,
            },
        })
    }).map_err(|e| e.to_string())?;
    
    let result: Result<Vec<_>, _> = folders.collect();
    result.map_err(|e| e.to_string())
}

#[command]
pub fn get_remote_shared_folders(cache: tauri::State<RemoteFolderCache>) -> Result<Vec<SharedFolder>, String> {
    let c = cache.lock().map_err(|e| e.to_string())?;
    let mut all = Vec::new();
    for (_, folders) in c.iter() {
        all.extend(folders.clone());
    }
    Ok(all)
}

#[command]
pub fn create_shared_folder(
    name: String,
    local_path: String,
    db: tauri::State<DbPool>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let config = crate::config::load_config();
    let conn = db.get().map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT INTO shared_folders (id, owner_id, owner_name, local_path, name, subscribers, sync_status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            config.device_id,
            config.username,
            local_path,
            name,
            "[]",
            "syncing",
        ],
    ).map_err(|e| e.to_string())?;
    drop(conn);
    
    // Broadcast updated folder list to all peers
    if let Ok(folders) = get_my_shared_folders(db) {
        let msg = crate::models::NetworkMessage::StateSync {
            table: "shared_folders".to_string(),
            data: serde_json::to_value(&folders).unwrap_or(serde_json::Value::Null),
            version: serde_json::json!({}),
        };
        if let Some(state) = app_handle.try_state::<crate::network::server::ConnectionPool>() {
            crate::network::client::broadcast_message(state.inner(), &msg);
        }
    }
    
    Ok(())
}

#[command]
pub fn subscribe_shared_folder(
    folder_id: String,
    local_path: String,
    _db: tauri::State<DbPool>,
) -> Result<(), String> {
    // TODO: 订阅逻辑，后续 Phase 7b 实现
    let _ = folder_id;
    let _ = local_path;
    Ok(())
}

#[command]
pub fn list_folder_files(path: String) -> Result<Vec<serde_json::Value>, String> {
    let mut files = Vec::new();
    let entries = std::fs::read_dir(&path).map_err(|e| e.to_string())?;
    
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() };
        let modified = metadata.modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        
        files.push(serde_json::json!({
            "name": name,
            "is_dir": is_dir,
            "size": size,
            "modified": modified,
        }));
    }
    
    Ok(files)
}

#[command]
pub fn delete_shared_folder(id: String, db: tauri::State<DbPool>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM shared_folders WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;
    drop(conn);
    
    // Broadcast deletion to all peers
    if let Ok(folders) = get_my_shared_folders(db) {
        let msg = crate::models::NetworkMessage::StateSync {
            table: "shared_folders".to_string(),
            data: serde_json::to_value(&folders).unwrap_or(serde_json::Value::Null),
            version: serde_json::json!({}),
        };
        if let Some(state) = app_handle.try_state::<crate::network::server::ConnectionPool>() {
            crate::network::client::broadcast_message(state.inner(), &msg);
        }
    }
    Ok(())
}
