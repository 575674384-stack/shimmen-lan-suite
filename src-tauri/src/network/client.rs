use crate::network::connection::Connection;
use crate::network::server::ConnectionPool;
use std::thread;
use std::time::Duration;

pub fn connect_to_peer(peer_id: String, peer_ip: String, port: u16, pool: ConnectionPool, my_id: String) {
    thread::spawn(move || {
        let addr = format!("{}:{}", peer_ip, port);

        let stream = loop {
            match std::net::TcpStream::connect(&addr) {
                Ok(s) => break s,
                Err(_) => {
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            }
        };

        stream.set_nonblocking(false).ok();
        let conn = Connection::new(peer_id.clone(), stream);

        let handshake = serde_json::json!({"peer_id": my_id});
        if conn.send_message(&handshake).is_err() {
            return;
        }

        {
            let mut p = pool.lock().unwrap();
            p.insert(peer_id.clone(), conn);
        }
    });
}

pub fn broadcast_message<T: serde::Serialize>(pool: &ConnectionPool, msg: &T) {
    let mut p = pool.lock().unwrap();
    for (_, conn) in p.iter_mut() {
        let _ = conn.send_message(msg);
    }
}

pub fn send_to_peer<T: serde::Serialize>(pool: &ConnectionPool, peer_id: &str, msg: &T) -> std::io::Result<()> {
    let mut p = pool.lock().unwrap();
    if let Some(conn) = p.get_mut(peer_id) {
        conn.send_message(msg)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "peer not connected"))
    }
}
