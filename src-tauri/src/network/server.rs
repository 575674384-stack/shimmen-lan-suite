use crate::models::NetworkMessage;
use crate::network::connection::Connection;
use crate::network::folder_cache::RemoteFolderCache;
use crate::network::peer::PeerMap;
use base64::Engine;
use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use tauri::{Emitter, Manager};

const CHUNK_SIZE: usize = 256 * 1024;

struct ChunkReceive {
    file: std::fs::File,
    received: HashSet<u32>,
    total_chunks: u32,
}

fn chunk_receives() -> &'static Mutex<HashMap<String, ChunkReceive>> {
    static INSTANCE: OnceLock<Mutex<HashMap<String, ChunkReceive>>> = OnceLock::new();
    INSTANCE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 检查文件路径是否包含路径遍历组件（..）或绝对路径
pub(crate) fn is_path_safe(path: &str) -> bool {
    !path.contains("..") && !std::path::Path::new(path).is_absolute()
}

pub(crate) fn cleanup_peer_chunks(peer_id: &str) {
    let mut map = chunk_receives().lock().unwrap_or_else(|e| e.into_inner());
    let keys_to_remove: Vec<String> = map.keys()
        .filter(|k| k.starts_with(&format!("{}:", peer_id)))
        .cloned()
        .collect();
    for key in keys_to_remove {
        map.remove(&key);
    }
}

const MAX_CONNECTIONS: usize = 128;

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
    let active_count = Arc::new(AtomicUsize::new(0));

    // Leader 心跳线程：每 2 秒给所有已连接 Peer 发送 Heartbeat
    let pool_for_heartbeat = pool.clone();
    thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            loop {
                thread::sleep(std::time::Duration::from_secs(2));
                let my_id = crate::config::cached_device_id();
                if crate::network::leader::is_leader(&my_id) {
                    let heartbeat = NetworkMessage::Heartbeat;
                    let conns: Vec<Connection> = {
                        let p = pool_for_heartbeat.lock().unwrap_or_else(|e| e.into_inner());
                        p.values().cloned().collect()
                    };
                    for conn in conns {
                        let _ = conn.send_message(&heartbeat);
                    }
                }
            }
        }));
        if let Err(e) = result {
            eprintln!("[network] heartbeat thread panicked: {:?}", e);
        }
    });

    thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let current = active_count.fetch_add(1, Ordering::Relaxed);
                        if current >= MAX_CONNECTIONS {
                            active_count.fetch_sub(1, Ordering::Relaxed);
                            eprintln!("[network] connection limit reached ({}), dropping new connection", MAX_CONNECTIONS);
                            continue;
                        }
                        let pool = pool.clone();
                        let app_handle = app_handle.clone();
                        let active_count = active_count.clone();
                        thread::spawn(move || {
                            // 确保无论 handle_incoming 是否正常返回或 panic，计数器都正确递减
                            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                handle_incoming(stream, pool, app_handle);
                            }));
                            active_count.fetch_sub(1, Ordering::Relaxed);
                            if let Err(e) = result {
                                eprintln!("[network] handle_incoming panicked: {:?}", e);
                            }
                        });
                    }
                    Err(_) => {}
                }
            }
        }));
        if let Err(e) = result {
            eprintln!("[network] listener thread panicked: {:?}", e);
        }
    });

    Ok(())
}

fn handle_incoming(stream: TcpStream, pool: ConnectionPool, app_handle: tauri::AppHandle) {
    stream.set_nonblocking(false).ok();

    let peer_addr = stream.peer_addr().ok();
    let mut conn = Connection::new(String::new(), stream);

    match conn.read_message() {
        Ok(data) => {
            if let Ok(msg) = serde_json::from_slice::<serde_json::Value>(&data) {
                if let Some(peer_id) = msg.get("peer_id").and_then(|v| v.as_str()) {
                    if peer_id.len() > 128 {
                        eprintln!("[network] peer_id too long ({} bytes), rejecting connection from {:?}", peer_id.len(), peer_addr);
                        return;
                    }
                    let peer_id = peer_id.to_string();
                    conn.peer_id = peer_id.clone();

                    {
                        let mut p = pool.lock().unwrap_or_else(|e| e.into_inner());
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

                    let mut p = pool.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(c) = p.get(&peer_id) {
                        if c.id == conn.id {
                            p.remove(&peer_id);
                        }
                    }
                    // 清理该 peer 未完成的文件分块接收，避免句柄泄漏
                    cleanup_peer_chunks(&peer_id);
                } else {
                    eprintln!("[network] handshake missing peer_id from {:?}", peer_addr);
                }
            } else {
                eprintln!("[network] handshake JSON parse failed from {:?}", peer_addr);
            }
        }
        Err(e) => {
            eprintln!("[network] handshake read failed: {}", e);
        }
    }
}

pub(crate) fn process_message(
    peer_id: &str,
    data: &[u8],
    app_handle: &tauri::AppHandle,
    pool: &ConnectionPool,
) {
    let msg = match serde_json::from_slice::<NetworkMessage>(data) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[server] failed to parse NetworkMessage from {}: {}", peer_id, e);
            return;
        }
    };
    // Leader 转发逻辑：将来自 Peer 的原始消息 Relay 给其他所有 Peer（Heartbeat 不转发）
        let my_id = crate::config::cached_device_id();
        // Leader 转发：排除 Heartbeat（维持连接）、FileIndexRequest/FileSearchRequest（Leader-only，无需转发）
        if crate::network::leader::is_leader(&my_id) && peer_id != my_id
            && !matches!(msg, NetworkMessage::Heartbeat)
            && !matches!(msg, NetworkMessage::FileIndexRequest { .. })
            && !matches!(msg, NetworkMessage::FileSearchRequest { .. })
        {
            // 拒绝嵌套 Relay（防止恶意 peer 构造嵌套包）
            if matches!(msg, NetworkMessage::Relay { .. }) {
                eprintln!("[server] dropping nested Relay from {}", peer_id);
                return;
            }
            let relay = NetworkMessage::Relay {
                origin_peer_id: peer_id.to_string(),
                payload: Box::new(msg.clone()),
            };
            let conns: Vec<Connection> = {
                let p = pool.lock().unwrap_or_else(|e| e.into_inner());
                p.values().cloned().collect()
            };
            for conn in conns {
                if conn.peer_id != peer_id {
                    if let Err(e) = conn.send_message(&relay) {
                        eprintln!("[network] leader relay to {} failed: {}", conn.peer_id, e);
                        let mut p = pool.lock().unwrap_or_else(|e| e.into_inner());
                        if let Some(c) = p.get(&conn.peer_id) {
                            if c.id == conn.id {
                                p.remove(&conn.peer_id);
                            }
                        }
                    }
                }
            }
        }

        match &msg {
            NetworkMessage::ChatMessage { id, sender_id: _claimed_sender_id, sender_name: _claimed_sender_name, content, message_type } => {
                // 安全：以 TCP 连接 handshake 中的 peer_id 作为可信来源，
                // 拒绝消息中伪造的 sender_id/sender_name
                let trusted_sender_id = peer_id.to_string();
                let trusted_sender_name = {
                    if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                        if let Ok(conn) = db.get() {
                            // 尝试从本地 chat_messages 找到该 peer 之前使用过的 sender_name
                            let name: Result<String, _> = conn.query_row(
                                "SELECT sender_name FROM chat_messages WHERE sender_id = ?1 ORDER BY timestamp DESC LIMIT 1",
                                [&trusted_sender_id],
                                |row| row.get(0),
                            );
                            name.ok()
                        } else { None }
                    } else { None }
                }.unwrap_or_else(|| trusted_sender_id.clone());
                
                if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                    match db.get() {
                        Ok(conn) => {
                            if let Err(e) = conn.execute(
                                "INSERT OR REPLACE INTO chat_messages (id, sender_id, sender_name, content, message_type, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                                rusqlite::params![id, &trusted_sender_id, &trusted_sender_name, content, message_type, chrono::Utc::now().timestamp()],
                            ) {
                                eprintln!("[server] failed to persist chat message to DB: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("[server] DB pool exhausted, chat message not persisted: {}", e);
                        }
                    }
                }
                let _ = app_handle.emit(
                    "network-message",
                    serde_json::json!({
                        "peer_id": peer_id,
                        "message": NetworkMessage::ChatMessage {
                            id: id.clone(),
                            sender_id: trusted_sender_id,
                            sender_name: trusted_sender_name,
                            content: content.clone(),
                            message_type: message_type.clone(),
                        }
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
                // 查询本地路径并直接分块发送
                let local_path_opt = if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                    if let Ok(conn) = db.get() {
                        conn.query_row(
                            "SELECT local_path FROM shared_folders WHERE id = ?1",
                            [folder_id],
                            |row| row.get::<_, String>(0),
                        ).ok()
                    } else { None }
                } else { None };

                if let Some(folder_local_path) = local_path_opt {
                    let full_path = std::path::Path::new(&folder_local_path).join(file_path);
                    if let Some(full_path_str) = full_path.to_str() {
                        let _ = crate::network::client::send_file_in_chunks(
                            pool, peer_id, folder_id, file_path, full_path_str,
                        );
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
                // 保留兼容：小文件或旧版本仍可能使用 FileResponse
                if folder_id.is_empty() {
                    let download_dir = crate::config::get_effective_download_dir(app_handle);
                    let file_name = std::path::Path::new(&file_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    let file_path_full = download_dir.join(file_name);
                    if let Ok(content) = base64::engine::general_purpose::STANDARD.decode(content_base64) {
                        std::fs::write(&file_path_full, content).ok();
                    }
                    let _ = app_handle.emit("file-received", serde_json::json!({
                        "file_name": file_path,
                        "download_path": file_path_full.to_string_lossy().to_string(),
                        "peer_id": peer_id,
                    }));
                } else {
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
            NetworkMessage::FileChunk { folder_id, file_path, chunk_index, total_chunks, data_base64 } => {
                // 安全：拒绝路径遍历和绝对路径
                if !is_path_safe(file_path) {
                    eprintln!("[server] FileChunk path traversal blocked: {}", file_path);
                    return;
                }
                // 安全：验证 chunk 参数合理性
                if *total_chunks == 0 || *total_chunks > 100_000 {
                    eprintln!("[file_chunk] invalid total_chunks: {}", total_chunks);
                    return;
                }
                if *chunk_index >= *total_chunks {
                    eprintln!("[file_chunk] chunk_index {} out of range (total: {})", chunk_index, total_chunks);
                    return;
                }
                let key = format!("{}:{}:{}", peer_id, folder_id, file_path);
                let mut map = chunk_receives().lock().unwrap_or_else(|e| e.into_inner());
                // 如果收到 chunk 0 且已有同 key 状态，说明是新传输，重置旧状态
                if *chunk_index == 0 && map.contains_key(&key) {
                    map.remove(&key);
                }
                if !map.contains_key(&key) {
                    // 限制每个 peer 的并发文件传输数（防 DoS）
                    let peer_chunk_count = map.keys().filter(|k| k.starts_with(&format!("{}:", peer_id))).count();
                    if peer_chunk_count >= 5 {
                        eprintln!("[file_chunk] peer {} has too many concurrent transfers, dropping chunk", peer_id);
                        return;
                    }
                    // 使用 .part 临时文件，避免多 peer 同时写同一目标文件导致损坏
                    let target_path = if folder_id.is_empty() {
                        crate::config::get_effective_download_dir(app_handle).join(file_path)
                    } else {
                        let folder_local = if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                            if let Ok(conn) = db.get() {
                                conn.query_row(
                                    "SELECT local_path FROM shared_folders WHERE id = ?1",
                                    [folder_id],
                                    |row| row.get::<_, String>(0),
                                ).ok()
                            } else { None }
                        } else { None };
                        match folder_local {
                            Some(p) => std::path::Path::new(&p).join(file_path),
                            None => {
                                eprintln!("[file_chunk] unknown folder_id: {}", folder_id);
                                return;
                            }
                        }
                    };
                    if let Some(parent) = target_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let part_path = target_path.with_extension("part");
                    match OpenOptions::new().write(true).create(true).truncate(true).open(&part_path) {
                        Ok(file) => {
                            map.insert(key.clone(), ChunkReceive {
                                file,
                                received: HashSet::new(),
                                total_chunks: *total_chunks,
                            });
                        }
                        Err(e) => {
                            eprintln!("[file_chunk] failed to open {}: {}", target_path.display(), e);
                            return;
                        }
                    }
                }
                // 将写入和完成检查放在闭包中，避免 state 借用与 map.remove 冲突
                let maybe_complete: Option<std::path::PathBuf> = {
                    let state = match map.get_mut(&key) {
                        Some(s) => s,
                        None => {
                            eprintln!("[file_chunk] chunk state missing for key: {}", key);
                            return;
                        }
                    };
                    match base64::engine::general_purpose::STANDARD.decode(data_base64) {
                        Ok(data) => {
                            let offset = (*chunk_index as u64) * (CHUNK_SIZE as u64);
                            if let Err(e) = state.file.seek(SeekFrom::Start(offset)) {
                                eprintln!("[file_chunk] seek failed: {}", e);
                                None
                            } else if let Err(e) = state.file.write_all(&data) {
                                eprintln!("[file_chunk] write failed: {}", e);
                                None
                            } else {
                                state.received.insert(*chunk_index);
                                if state.received.len() as u32 == state.total_chunks {
                                    let _ = state.file.flush();
                                    let final_target = if folder_id.is_empty() {
                                        crate::config::get_effective_download_dir(app_handle).join(file_path)
                                    } else {
                                        let folder_local = if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                                            if let Ok(conn) = db.get() {
                                                conn.query_row(
                                                    "SELECT local_path FROM shared_folders WHERE id = ?1",
                                                    [folder_id],
                                                    |row| row.get::<_, String>(0),
                                                ).ok()
                                            } else { None }
                                        } else { None };
                                        match folder_local {
                                            Some(p) => std::path::Path::new(&p).join(file_path),
                                            None => return,
                                        }
                                    };
                                    Some(final_target)
                                } else {
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[file_chunk] base64 decode failed: {}", e);
                            None
                        }
                    }
                };
                if let Some(final_target) = maybe_complete {
                    map.remove(&key);
                    drop(map);
                    let part_path = final_target.with_extension("part");
                    let _ = std::fs::rename(&part_path, &final_target);
                    if folder_id.is_empty() {
                        let _ = app_handle.emit("file-received", serde_json::json!({
                            "file_name": file_path,
                            "download_path": final_target.to_string_lossy().to_string(),
                            "peer_id": peer_id,
                        }));
                    } else {
                        let _ = app_handle.emit("file-sync", serde_json::json!({
                            "type": "file_response_complete",
                            "folder_id": folder_id,
                            "file_path": file_path,
                        }));
                    }
                }
            }
            NetworkMessage::ScreenShare { frame_base64 } => {
                let _ = app_handle.emit("screen-share", serde_json::json!({
                    "peer_id": peer_id,
                    "frame": frame_base64,
                }));
            }
            NetworkMessage::FileIndexBroadcast { peer_id: sender_id, peer_name, files } => {
                // 安全：验证消息中的 peer_id 与 TCP 连接来源一致，拒绝伪造
                if sender_id != peer_id {
                    eprintln!("[server] FileIndexBroadcast sender_id mismatch: expected {}, got {}", peer_id, sender_id);
                    return;
                }
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
                let my_id = crate::config::cached_device_id();
                // 星型拓扑：只有 Leader 有所有 Peer 的连接，非 Leader 无法直接响应
                if !crate::network::leader::is_leader(&my_id) {
                    return;
                }
                if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                    let my_name = crate::config::load_config().username;
                    let files = match crate::file_index::indexer::search_local("", &db) {
                        Ok(f) => f,
                        Err(_) => Vec::new(),
                    };
                    let msg = crate::models::NetworkMessage::FileIndexBroadcast {
                        peer_id: my_id.to_string(),
                        peer_name: my_name,
                        files,
                    };
                    let _ = crate::network::client::send_to_peer(pool, requester_id, &msg);
                }
            }
            NetworkMessage::FileSearchRequest { requester_id, query } => {
                let my_id = crate::config::cached_device_id();
                // 星型拓扑：只有 Leader 能直接响应请求者
                if !crate::network::leader::is_leader(&my_id) {
                    return;
                }
                if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                    crate::file_index::network::handle_search_request(requester_id, &query, pool, &db, &my_id);
                }
            }
            NetworkMessage::FileSearchResponse { responder_id, results } => {
                if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                    crate::file_index::network::handle_search_response(responder_id, results.clone(), &db, app_handle);
                }
            }
            NetworkMessage::FileTransferRequest { requester_id, file_path } => {
                if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                    crate::file_index::network::handle_transfer_request(requester_id, file_path, pool, &db);
                }
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
                    // 安全：ai_config 包含 API 密钥，拒绝通过网络同步覆盖
                    eprintln!("[network] rejected remote ai_config sync from {}", peer_id);
                } else if table == "tasks" {
                    if let Ok(tasks) = serde_json::from_value::<Vec<crate::models::Task>>(data.clone()) {
                        if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                            if let Ok(conn) = db.get() {
                                for task in tasks {
                                    let now_ts = chrono::Utc::now().timestamp();
                                    let updated_at = task.updated_at.unwrap_or(now_ts);
                                    // 版本检查：本地更新则拒绝旧版本覆盖
                                    let local_updated_at: i64 = conn.query_row(
                                        "SELECT updated_at FROM tasks WHERE id = ?1",
                                        [&task.id],
                                        |row| row.get(0),
                                    ).unwrap_or(0);
                                    if local_updated_at >= updated_at {
                                        continue;
                                    }
                                    let attached = serde_json::to_string(&task.attached_files).unwrap_or_default();
                                    let created_at = task.created_at.unwrap_or(now_ts);
                                    let _ = conn.execute(
                                        "INSERT OR REPLACE INTO tasks (id, title, project, deadline, contact, priority, description, status, creator_id, assignee_id, is_team_visible, attached_files, archived_to_folder_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                                        rusqlite::params![
                                            task.id, task.title, task.project, task.deadline, task.contact,
                                            task.priority.to_string(), task.description, task.status.to_string(),
                                            task.creator_id, task.assignee_id, task.is_team_visible as i32, attached,
                                            task.archived_to_folder_id, created_at, updated_at,
                                        ],
                                    );
                                }
                            }
                        }
                    }
                } else if table == "announcements" {
                    if let Ok(list) = serde_json::from_value::<Vec<crate::models::Announcement>>(data.clone()) {
                        if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
                            if let Ok(conn) = db.get() {
                                for item in list {
                                    // 版本检查：本地更新则拒绝旧版本覆盖
                                    let local_updated_at: i64 = conn.query_row(
                                        "SELECT updated_at FROM announcements WHERE id = ?1",
                                        [&item.id],
                                        |row| row.get(0),
                                    ).unwrap_or(0);
                                    if local_updated_at >= item.updated_at {
                                        continue;
                                    }
                                    let _ = conn.execute(
                                        "INSERT OR REPLACE INTO announcements (id, title, content, is_pinned, created_by, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                                        rusqlite::params![item.id, item.title, item.content, item.is_pinned as i32, item.created_by, item.updated_at],
                                    );
                                }
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
            NetworkMessage::Heartbeat => {
                // 非 Leader 收到 Heartbeat 后回复，维持 Leader 端连接活跃
                let my_id = crate::config::cached_device_id();
                if !crate::network::leader::is_leader(&my_id) {
                    if let Some(leader_id) = crate::network::leader::get_leader_id() {
                        if leader_id == peer_id {
                            let _ = crate::network::client::send_to_peer(pool, &leader_id, &NetworkMessage::Heartbeat);
                        }
                    }
                }
            }
            NetworkMessage::Relay { origin_peer_id, payload } => {
                // 拒绝嵌套 Relay，防止栈溢出
                if matches!(payload.as_ref(), NetworkMessage::Relay { .. }) {
                    eprintln!("[server] dropping nested Relay from {}", origin_peer_id);
                    return;
                }
                let payload_data = serde_json::to_vec(payload).unwrap_or_default();
                if !payload_data.is_empty() {
                    process_message(origin_peer_id, &payload_data, app_handle, pool);
                }
            }
        }
    }
