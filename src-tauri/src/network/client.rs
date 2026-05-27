use crate::network::connection::Connection;
use crate::network::server::ConnectionPool;
use std::thread;

pub fn connect_to_peer(peer_id: String, peer_ip: String, port: u16, pool: ConnectionPool, my_id: String) {
    thread::spawn(move || {
        let addr = format!("{}:{}", peer_ip, port);

        let stream = match std::net::TcpStream::connect(&addr) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[network] connect_to_peer {} failed: {}", addr, e);
                return;
            }
        };

        stream.set_nonblocking(false).ok();
        let conn = Connection::new(peer_id.clone(), stream);

        let handshake = serde_json::json!({"peer_id": my_id});
        if conn.send_message(&handshake).is_err() {
            eprintln!("[network] handshake to {} failed", peer_id);
            return;
        }

        {
            let mut p = pool.lock().unwrap();
            // Remove any existing connection for this peer before inserting the new one
            p.remove(&peer_id);
            p.insert(peer_id.clone(), conn.clone());
        }

        // Keep a read loop so the peer can send messages back on this connection
        loop {
            match conn.read_message() {
                Ok(_data) => {
                    // Outbound connections are used for sending; inbound server handles business messages.
                    // We just need to keep the read loop alive to detect EOF/disconnect.
                }
                Err(e) => {
                    eprintln!("[network] outbound read error from {}: {}", peer_id, e);
                    break;
                }
            }
        }

        let mut p = pool.lock().unwrap();
        p.remove(&peer_id);
    });
}

pub fn broadcast_message<T: serde::Serialize>(pool: &ConnectionPool, msg: &T) {
    let conns: Vec<Connection> = {
        let p = pool.lock().unwrap();
        p.values().cloned().collect()
    };
    for conn in conns {
        if let Err(e) = conn.send_message(msg) {
            eprintln!("[network] broadcast to {} failed: {}", conn.peer_id, e);
            let mut p = pool.lock().unwrap();
            p.remove(&conn.peer_id);
        }
    }
}

pub fn send_to_peer<T: serde::Serialize>(pool: &ConnectionPool, peer_id: &str, msg: &T) -> std::io::Result<()> {
    let conn = {
        let p = pool.lock().unwrap();
        p.get(peer_id).cloned()
    };
    if let Some(conn) = conn {
        conn.send_message(msg)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "peer not connected"))
    }
}
