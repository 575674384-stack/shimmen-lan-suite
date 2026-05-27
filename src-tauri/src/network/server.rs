use crate::models::NetworkMessage;
use crate::network::connection::Connection;
use crate::network::folder_cache::RemoteFolderCache;
use crate::network::peer::PeerMap;
use base64::Engine;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{Emitter, Manager};

pub type ConnectionPool = Arc<Mutex<HashMap<String, Connection>>>;

pub fn create_connection_pool() -> ConnectionPool {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn start_server(
    port: u16,
    _peers: PeerMap,
    pool: ConnectionPool,
    app_handle: tauri::AppHandle,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let pool = pool.clone();
                    let app_handle = app_handle.clone();
                    thread::spawn(move || {
                        handle_incoming(stream, pool, app_handle);
                    });
                }
                Err(_) => {}
            }
        }
    });

    Ok(())
}

fn handle_incoming(stream: TcpStream, pool: ConnectionPool, app_handle: tauri::AppHandle) {
    stream.set_nonblocking(false).ok();

    let mut conn = Connection::new(String::new(), stream);

    match conn.read_message() {
        Ok(data) => {
            if let Ok(msg) = serde_json::from_slice::<serde_json::Value>(&data) {
                if let Some(peer_id) = msg.get("peer_id").and_then(|v| v.as_str()) {
                    let peer_id = peer_id.to_string();
                    conn.peer_id = peer_id.clone();

                    {
                        let mut p = pool.lock().unwrap();
                        p.remove(&peer_id); // close any stale connection first
                        p.insert(peer_id.clone(), conn.clone());
                    }

                    loop {
                        match conn.read_message() {
                            Ok(data) => {
                                process_message(&peer_id, &data, &app_handle, &pool);
                            }
                            Err(e) => {
                                eprintln!("[network] read error from {}: {}", peer_id, e);
                                break;
                            }
                        }
                    }

                    let mut p = pool.lock().unwrap();
                    if let Some(c) = p.get(&peer_id) {
                        if c.id == conn.id {
                            p.remove(&peer_id);
                        }
                    }
                }
            }
        }
        Err(_) => {}
    }
}

pub(crate) fn process_message(
    peer_id: &str,
    data: &[u8],
    app_handle: &tauri::AppHandle,
    pool: &ConnectionPool,
) {
    if let Ok(msg) = serde_json::from_slice::<NetworkMessage>(data) {
        match &msg {
            NetworkMessage::ChatMessage { id: _, sender_id, sender_name, content, message_type } => {
                if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                    if let Ok(conn) = db.lock() {
                        let _ = conn.execute(
                            "INSERT INTO chat_messages (sender_id, sender_name, content, message_type, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
                            rusqlite::params![sender_id, sender_name, content, message_type, chrono::Utc::now().timestamp()],
                        );
                    }
                }
                let _ = app_handle.emit(
                    "network-message",
                    serde_json::json!({
                        "peer_id": peer_id,
                        "message": msg
                    }),
                );
            }
            NetworkMessage::ClearScreen { .. } => {
                let _ = app_handle.emit(
                    "network-message",
                    serde_json::json!({
                        "peer_id": peer_id,
                        "message": msg
                    }),
                );
            }
            NetworkMessage::FileList { folder_id, files } => {
                let _ = app_handle.emit(
                    "file-sync",
                    serde_json::json!({
                        "type": "file_list",
                        "folder_id": folder_id,
                        "files": files,
                        "peer_id": peer_id,
                    }),
                );
            }
            NetworkMessage::FileRequest {
                folder_id,
                file_path,
            } => {
                // 尝试通过 SyncEngine 响应文件请求
                if let Some(engine) = app_handle.try_state::<std::sync::Arc<crate::file_sync::engine::SyncEngine>>() {
                    if let Some(response) = engine.try_handle_file_request(folder_id, file_path) {
                        let _ = crate::network::client::send_to_peer(pool, peer_id, &response);
                    }
                }
                let _ = app_handle.emit(
                    "file-sync",
                    serde_json::json!({
                        "type": "file_request",
                        "folder_id": folder_id,
                        "file_path": file_path,
                        "peer_id": peer_id,
                    }),
                );
            }
            NetworkMessage::FileResponse {
                folder_id,
                file_path,
                content_base64,
            } => {
                if folder_id.is_empty() {
                    // 点对点文件传输
                    let app_dir = app_handle.path().app_data_dir().unwrap_or_default();
                    let download_dir = app_dir.join("downloads");
                    std::fs::create_dir_all(&download_dir).ok();
                    
                    let file_name = std::path::Path::new(&file_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    let file_path_full = download_dir.join(file_name);
                    if let Ok(content) = base64::engine::general_purpose::STANDARD.decode(content_base64) {
                        std::fs::write(&file_path_full, content).ok();
                    }
                    
                    // 通知前端有新文件到达
                    let _ = app_handle.emit("file-received", serde_json::json!({
                        "file_name": file_path,
                        "download_path": file_path_full.to_string_lossy().to_string(),
                        "peer_id": peer_id,
                    }));
                } else {
                    // 共享文件夹同步（已有逻辑）
                    let _ = app_handle.emit(
                        "file-sync",
                        serde_json::json!({
                            "type": "file_response",
                            "folder_id": folder_id,
                            "file_path": file_path,
                            "content_base64": content_base64,
                        }),
                    );
                }
            }
            NetworkMessage::ScreenShare { frame_base64 } => {
                let _ = app_handle.emit("screen-share", serde_json::json!({
                    "peer_id": peer_id,
                    "frame": frame_base64,
                }));
            }
            NetworkMessage::FileIndexBroadcast { peer_id: sender_id, peer_name, files } => {
                if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                    crate::file_index::network::handle_index_broadcast(sender_id, peer_name, files.clone(), &db);
                }
                let _ = app_handle.emit("file-index-update", serde_json::json!({
                    "peer_id": sender_id,
                    "peer_name": peer_name,
                    "file_count": files.len(),
                }));
            }
            NetworkMessage::FileIndexRequest { requester_id } => {
                if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                    let my_id = crate::config::load_config().device_id;
                    let my_name = crate::config::load_config().username;
                    let files = match crate::file_index::indexer::search_local("", &db) {
                        Ok(f) => f,
                        Err(_) => Vec::new(),
                    };
                    let msg = crate::models::NetworkMessage::FileIndexBroadcast {
                        peer_id: my_id.clone(),
                        peer_name: my_name,
                        files,
                    };
                    let _ = crate::network::client::send_to_peer(pool, requester_id, &msg);
                }
            }
            NetworkMessage::FileSearchRequest { requester_id, query } => {
                if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                    let my_id = crate::config::load_config().device_id;
                    crate::file_index::network::handle_search_request(requester_id, &query, pool, &db, &my_id);
                }
            }
            NetworkMessage::FileSearchResponse { responder_id, results } => {
                if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                    crate::file_index::network::handle_search_response(responder_id, results.clone(), &db, app_handle);
                }
            }
            NetworkMessage::FileTransferRequest { requester_id, file_path } => {
                crate::file_index::network::handle_transfer_request(requester_id, file_path, pool);
            }
            NetworkMessage::StateSync { table, data, .. } => {
                if table == "shared_folders" {
                    if let Ok(folders) = serde_json::from_value::<Vec<crate::models::SharedFolder>>(data.clone()) {
                        if let Some(cache) = app_handle.try_state::<RemoteFolderCache>() {
                            if let Ok(mut c) = cache.lock() {
                                c.insert(peer_id.to_string(), folders);
                            }
                        }
                    }
                } else if table == "ai_config" {
                    if let Ok(config) = serde_json::from_value::<crate::models::AiConfig>(data.clone()) {
                        if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                            if let Ok(conn) = db.lock() {
                                let _ = conn.execute(
                                    "INSERT OR REPLACE INTO ai_config (id, api_key, base_url, model, updated_at) VALUES (1, ?1, ?2, ?3, ?4)",
                                    [&config.api_key, &config.base_url, &config.model, &config.updated_at.to_string()],
                                );
                            }
                        }
                    }
                }
                let _ = app_handle.emit(
                    "network-message",
                    serde_json::json!({
                        "peer_id": peer_id,
                        "message": msg
                    }),
                );
            }

        }
    }
}
