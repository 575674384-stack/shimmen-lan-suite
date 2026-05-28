use tauri::command;
use crate::db::DbPool;
use crate::file_index::indexer::FileIndexEntry;

#[command]
pub fn get_indexed_directories() -> Result<Vec<String>, String> {
    // Return default directories
    let home = dirs::home_dir().unwrap_or_default();
    let home_str = home.to_string_lossy().to_string();
    Ok(vec![
        format!("{}\\Desktop", home_str),
        format!("{}\\Documents", home_str),
        format!("{}\\Downloads", home_str),
    ])
}

#[command]
pub fn rebuild_file_index(paths: Vec<String>, peer_id: String, peer_name: String, db: tauri::State<DbPool>) -> Result<usize, String> {
    crate::file_index::indexer::scan_directories(paths, &peer_id, &peer_name, &db)
}

#[command]
pub fn search_files_network(query: String, db: tauri::State<DbPool>) -> Result<Vec<FileIndexEntry>, String> {
    crate::file_index::indexer::search_all(&query, &db)
}

#[command]
pub fn clear_remote_index(db: tauri::State<DbPool>) -> Result<(), String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM file_index WHERE is_local = 0", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn request_file_from_peer(peer_id: String, file_path: String, pool: tauri::State<crate::network::server::ConnectionPool>) -> Result<(), String> {
    let my_id = crate::config::load_config().device_id;
    let msg = crate::models::NetworkMessage::FileTransferRequest {
        requester_id: my_id,
        file_path,
    };
    crate::network::client::send_to_peer(&pool, &peer_id, &msg)
        .map_err(|e| e.to_string())
}
