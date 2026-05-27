use crate::network::peer::{get_online_users, PeerMap};
use std::sync::{Mutex, OnceLock};

/// 当前 Leader 的 device_id（全局单例）
fn current_leader() -> &'static Mutex<Option<String>> {
    static INSTANCE: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    INSTANCE.get_or_init(|| Mutex::new(None))
}

pub fn get_leader_id() -> Option<String> {
    current_leader().lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub fn set_leader_id(id: Option<String>) {
    *current_leader().lock().unwrap_or_else(|e| e.into_inner()) = id;
}

/// 从在线 peer 中选举 device_id 字典序最小者为 Leader
pub fn elect_leader(peers: &PeerMap, my_id: &str) -> Option<String> {
    let online = get_online_users(peers);
    if online.is_empty() {
        // 只有自己在线，自己就是 Leader
        return Some(my_id.to_string());
    }
    let mut candidates: Vec<String> = online.into_iter().map(|u| u.id).collect();
    candidates.push(my_id.to_string());
    candidates.sort();
    candidates.into_iter().next()
}

/// 检查当前自己是否为 Leader
pub fn is_leader(my_id: &str) -> bool {
    get_leader_id()
        .map(|leader| leader == my_id)
        .unwrap_or(false)
}

/// 判断是否应该主动连接某个 peer
/// 规则：只连接当前 Leader；如果自己就是 Leader，谁都不连
pub fn should_connect_to(target_peer_id: &str, my_id: &str) -> bool {
    if is_leader(my_id) {
        return false; // Leader 只接受 inbound，不主动 outbound
    }
    match get_leader_id() {
        Some(leader_id) => leader_id == target_peer_id,
        None => false,
    }
}

/// 获取当前 Leader 的 device_id，每次都重新计算（确保旧 Leader 离线后能自动切换）
pub fn ensure_leader(peers: &PeerMap, my_id: &str) -> Option<String> {
    let elected = elect_leader(peers, my_id);
    set_leader_id(elected.clone());
    elected
}
