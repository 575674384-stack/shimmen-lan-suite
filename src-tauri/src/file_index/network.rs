use crate::db::DbPool;
use crate::models::{NetworkMessage, RemoteFileInfo};
use crate::network::client;
use crate::network::server::ConnectionPool;
use base64::Engine;
use tauri::Emitter;

pub fn broadcast_index(db: &DbPool, pool: &ConnectionPool, my_id: &str, my_name: &str) {
    let files = match crate::file_index::indexer::search_local("", db) {
        Ok(f) => f,
        Err(_) => return,
    };

    if files.is_empty() {
        return;
    }

    let msg = NetworkMessage::FileIndexBroadcast {
        peer_id: my_id.to_string(),
        peer_name: my_name.to_string(),
        files,
    };

    let _ = client::broadcast_message(pool, &msg);
}

pub fn handle_index_broadcast(
    peer_id: &str,
    peer_name: &str,
    files: Vec<RemoteFileInfo>,
    db: &DbPool,
) {
    let _ = crate::file_index::indexer::insert_remote_files(peer_id, peer_name, &files, db);
}

pub fn handle_search_request(
    requester_id: &str,
    query: &str,
    pool: &ConnectionPool,
    db: &DbPool,
    my_id: &str,
) {
    let results = match crate::file_index::indexer::search_local(query, db) {
        Ok(r) => r,
        Err(_) => return,
    };

    let msg = NetworkMessage::FileSearchResponse {
        responder_id: my_id.to_string(),
        results,
    };

    let _ = client::send_to_peer(pool, requester_id, &msg);
}

pub fn handle_search_response(
    responder_id: &str,
    results: Vec<RemoteFileInfo>,
    db: &DbPool,
    app_handle: &tauri::AppHandle,
) {
    let peer_name = responder_id.clone();
    let _ = crate::file_index::indexer::insert_remote_files(responder_id, &peer_name, &results, db);

    let _ = app_handle.emit("file-search-response", serde_json::json!({
        "responder_id": responder_id,
        "results": results,
    }));
}

pub fn handle_transfer_request(
    requester_id: &str,
    file_path: &str,
    pool: &ConnectionPool,
) {
    if let Ok(content) = std::fs::read(file_path) {
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();

        let content_base64 = base64::engine::general_purpose::STANDARD.encode(content);

        let msg = NetworkMessage::FileResponse {
            folder_id: "".to_string(),
            file_path: file_name,
            content_base64,
        };

        let _ = client::send_to_peer(pool, requester_id, &msg);
    }
}
