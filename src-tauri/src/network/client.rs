use crate::network::connection::Connection;
use crate::network::server::ConnectionPool;
use base64::Engine;
use std::thread;

const CHUNK_SIZE: usize = 256 * 1024; // 256KB

/// 将本地文件切分成 256KB 块逐块发送给指定 Peer
pub fn send_file_in_chunks(
    pool: &ConnectionPool,
    peer_id: &str,
    folder_id: &str,
    file_path: &str,
    local_full_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(local_full_path)?;
    let total_size = file.metadata()?.len() as usize;
    let total_chunks = ((total_size + CHUNK_SIZE - 1) / CHUNK_SIZE) as u32;
    let mut reader = std::io::BufReader::new(file);
    let mut buffer = vec![0u8; CHUNK_SIZE];

    for chunk_index in 0..total_chunks {
        let bytes_read = std::io::Read::read(&mut reader, &mut buffer)?;
        if bytes_read == 0 { break; }
        let data_base64 = base64::engine::general_purpose::STANDARD.encode(&buffer[..bytes_read]);
        let msg = crate::models::NetworkMessage::FileChunk {
            folder_id: folder_id.to_string(),
            file_path: file_path.to_string(),
            chunk_index,
            total_chunks,
            data_base64,
        };
        send_to_peer(pool, peer_id, &msg)?;
    }
    Ok(())
}

/// 将本地文件切分成 256KB 块广播给所有 Peer（经 Leader 转发）
pub fn broadcast_file_in_chunks(
    pool: &ConnectionPool,
    folder_id: &str,
    file_path: &str,
    local_full_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(local_full_path)?;
    let total_size = file.metadata()?.len() as usize;
    let total_chunks = ((total_size + CHUNK_SIZE - 1) / CHUNK_SIZE) as u32;
    let mut reader = std::io::BufReader::new(file);
    let mut buffer = vec![0u8; CHUNK_SIZE];

    for chunk_index in 0..total_chunks {
        let bytes_read = std::io::Read::read(&mut reader, &mut buffer)?;
        if bytes_read == 0 { break; }
        let data_base64 = base64::engine::general_purpose::STANDARD.encode(&buffer[..bytes_read]);
        let msg = crate::models::NetworkMessage::FileChunk {
            folder_id: folder_id.to_string(),
            file_path: file_path.to_string(),
            chunk_index,
            total_chunks,
            data_base64,
        };
        broadcast_message(pool, &msg);
    }
    Ok(())
}

pub fn connect_to_peer(
    peer_id: String,
    peer_ip: String,
    port: u16,
    pool: ConnectionPool,
    my_id: String,
    app_handle: tauri::AppHandle,
) {
    // 星型拓扑：只连接 Leader，不连接其他 Peer
    if !crate::network::leader::should_connect_to(&peer_id, &my_id) {
        return;
    }

    // 避免双向连接竞争：device_id 字典序大的一方等 2 秒，给小的一方先连的机会
    if my_id > peer_id {
        std::thread::sleep(std::time::Duration::from_secs(2));
        let already_connected = {
            let p = pool.lock().unwrap_or_else(|e| e.into_inner());
            p.contains_key(&peer_id)
        };
        if already_connected {
            eprintln!("[network] skipping outbound connect to {} (already have inbound)", peer_id);
            return;
        }
    }

    thread::spawn(move || {
        let peer_id_for_cleanup = peer_id.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let addr = format!("{}:{}", peer_ip, port);

            let stream = match std::net::TcpStream::connect(&addr) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[network] connect_to_peer {} failed: {}", addr, e);
                    return None;
                }
            };

            stream.set_nonblocking(false).ok();
            let conn = Connection::new(peer_id.clone(), stream);

            let handshake = serde_json::json!({"peer_id": my_id});
            if conn.send_message(&handshake).is_err() {
                eprintln!("[network] handshake to {} failed", peer_id);
                return Some(conn);
            }

            {
                let mut p = pool.lock().unwrap_or_else(|e| e.into_inner());
                p.remove(&peer_id);
                p.insert(peer_id.clone(), conn.clone());
            }

            // Read loop: handle messages sent back on this outbound connection
            loop {
                match conn.read_message() {
                    Ok(data) => {
                        if !crate::network::server::check_json_depth(&data, 50) {
                            eprintln!("[network] JSON nested too deeply from {}, dropping message", peer_id);
                            continue;
                        }
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            crate::network::server::process_message(
                                &peer_id,
                                &data,
                                &app_handle,
                                &pool,
                            );
                        }));
                        if let Err(e) = result {
                            eprintln!("[server] process_message panicked: {:?}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("[network] outbound read error from {}: {}", peer_id, e);
                        if crate::network::leader::get_leader_id().as_ref() == Some(&peer_id) {
                            crate::network::leader::set_leader_id(None);
                            eprintln!("[network] leader {} disconnected, will re-elect", peer_id);
                        }
                        break;
                    }
                }
            }

            Some(conn)
        }));
        // 无论是否 panic，都执行 cleanup
        let mut p = pool.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(c) = p.get(&peer_id_for_cleanup) {
            if result.as_ref().map_or(true, |opt_conn| opt_conn.as_ref().map_or(true, |conn| conn.id == c.id)) {
                p.remove(&peer_id_for_cleanup);
            }
        }
        crate::network::server::cleanup_peer_chunks(&peer_id_for_cleanup);
        if let Err(e) = result {
            eprintln!("[network] outbound connection thread panicked: {:?}", e);
        }
    });
}

pub fn broadcast_message<T: serde::Serialize>(pool: &ConnectionPool, msg: &T) {
    let my_id = crate::config::cached_device_id();
    if crate::network::leader::is_leader(&my_id) {
        // Leader: 广播给所有已连接 Peer
        let conns: Vec<Connection> = {
            let p = pool.lock().unwrap_or_else(|e| e.into_inner());
            p.values().cloned().collect()
        };
        let mut to_remove: Vec<String> = Vec::new();
        for conn in &conns {
            if let Err(e) = conn.send_message(msg) {
                eprintln!("[network] broadcast to {} failed: {}", conn.peer_id, e);
                to_remove.push(conn.peer_id.clone());
            }
        }
        if !to_remove.is_empty() {
            let mut p = pool.lock().unwrap_or_else(|e| e.into_inner());
            for peer_id in to_remove {
                if let Some(c) = p.get(&peer_id) {
                    if conns.iter().any(|conn| conn.peer_id == peer_id && conn.id == c.id) {
                        p.remove(&peer_id);
                    }
                }
            }
        }
    } else {
        // 非 Leader: 只发给 Leader
        if let Some(leader_id) = crate::network::leader::get_leader_id() {
            if let Err(e) = send_to_peer(pool, &leader_id, msg) {
                eprintln!("[network] send to leader {} failed: {}", leader_id, e);
            }
        } else {
            eprintln!("[network] no leader elected, dropping broadcast");
        }
    }
}

pub fn send_to_peer<T: serde::Serialize>(
    pool: &ConnectionPool,
    peer_id: &str,
    msg: &T,
) -> std::io::Result<()> {
    let my_id = crate::config::cached_device_id();
    if crate::network::leader::is_leader(&my_id) {
        // Leader: 直接发给目标 Peer
        let conn = {
            let p = pool.lock().unwrap_or_else(|e| e.into_inner());
            p.get(peer_id).cloned()
        };
        if let Some(conn) = conn {
            conn.send_message(msg)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "peer not connected",
            ))
        }
    } else {
        // 非 Leader
        if let Some(leader_id) = crate::network::leader::get_leader_id() {
            if leader_id == peer_id {
                // 目标就是 Leader，直接发
                let conn = {
                    let p = pool.lock().unwrap_or_else(|e| e.into_inner());
                    p.get(peer_id).cloned()
                };
                if let Some(conn) = conn {
                    conn.send_message(msg)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "leader not connected",
                    ))
                }
            } else {
                // 目标不是 Leader，通过 broadcast 经 Leader 转发
                // 先检查 Leader 是否已连接，避免静默丢消息
                let leader_connected = crate::network::leader::get_leader_id()
                    .map(|lid| {
                        let p = pool.lock().unwrap_or_else(|e| e.into_inner());
                        p.contains_key(&lid)
                    })
                    .unwrap_or(false);
                if !leader_connected {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "leader not connected, cannot relay message",
                    ));
                }
                broadcast_message(pool, msg);
                Ok(())
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no leader elected",
            ))
        }
    }
}
