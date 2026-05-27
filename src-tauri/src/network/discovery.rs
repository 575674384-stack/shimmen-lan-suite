use crate::config::{AppConfig, DISCOVERY_PORT};
use crate::models::{User, UserStatus};
use crate::network::peer::{PeerMap, update_peer};
use crate::network::protocol::{DiscoveryPacket, DISCOVERY_INTERVAL_SECS, PEER_TIMEOUT_SECS};
use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::Emitter;

pub fn start_discovery(config: AppConfig, peers: PeerMap, app_handle: tauri::AppHandle) -> std::io::Result<()> {
    let socket = Arc::new(UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT))?);
    socket.set_broadcast(true)?;
    
    let my_id = config.device_id.clone();
    let my_username = config.username.clone();
    let my_version = crate::config::APP_VERSION.to_string();
    
    // 获取本机局域网 IP
    let local_ip = get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
    let local_ip_send = local_ip.clone();
    let local_ip_recv = local_ip;
    
    // 发送线程：广播发现包
    let send_socket = socket.clone();
    thread::spawn(move || {
        let broadcast_addr = format!("255.255.255.255:{}", DISCOVERY_PORT);
        loop {
            let packet = DiscoveryPacket {
                id: my_id.clone(),
                username: my_username.clone(),
                ip: local_ip_send.clone(),
                version: my_version.clone(),
                timestamp: chrono::Utc::now().timestamp(),
            };
            let json = serde_json::to_string(&packet).unwrap();
            let _ = send_socket.send_to(json.as_bytes(), &broadcast_addr);
            thread::sleep(Duration::from_secs(DISCOVERY_INTERVAL_SECS));
        }
    });
    
    // 接收线程：处理发现包
    let recv_socket = socket.clone();
    let recv_peers = peers.clone();
    let recv_id = config.device_id.clone();
    let recv_username = config.username.clone();
    let recv_version = crate::config::APP_VERSION.to_string();
    let app_handle_for_recv = app_handle.clone();
    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match recv_socket.recv_from(&mut buf) {
                Ok((len, addr)) => {
                    let msg = String::from_utf8_lossy(&buf[..len]);
                    if let Ok(packet) = serde_json::from_str::<DiscoveryPacket>(&msg) {
                        if packet.id != recv_id {
                            if packet.version != recv_version {
                                let _ = app_handle_for_recv.emit("version-mismatch", serde_json::json!({
                                    "peer_name": packet.username,
                                    "peer_ip": packet.ip,
                                    "peer_version": packet.version,
                                    "my_version": recv_version,
                                }));
                            }
                            let user = User {
                                id: packet.id,
                                username: packet.username,
                                ip: addr.ip().to_string(),
                                status: UserStatus::Online,
                                version: packet.version,
                            };
                            update_peer(&recv_peers, user);
                            // 回复发现响应
                            let response = DiscoveryPacket {
                                id: recv_id.clone(),
                                username: recv_username.clone(),
                                ip: local_ip_recv.clone(),
                                version: recv_version.clone(),
                                timestamp: chrono::Utc::now().timestamp(),
                            };
                            let json = serde_json::to_string(&response).unwrap();
                            let _ = recv_socket.send_to(json.as_bytes(), addr);
                        }
                    }
                }
                Err(_) => {}
            }
        }
    });
    
    // 清理线程：移除超时节点
    let clean_peers = peers.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(PEER_TIMEOUT_SECS));
            crate::network::peer::remove_stale_peers(&clean_peers);
        }
    });
    
    Ok(())
}

fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    // Fast path: try external route (works when internet is available)
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                let ip = addr.ip().to_string();
                if ip != "127.0.0.1" && !ip.starts_with("169.254.") {
                    return Some(ip);
                }
            }
        }
    }
    // Fallback: LAN-only environments — connect to broadcast to discover local IP
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("255.255.255.255:1").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                let ip = addr.ip().to_string();
                if ip != "127.0.0.1" && !ip.starts_with("169.254.") {
                    return Some(ip);
                }
            }
        }
    }
    None
}
