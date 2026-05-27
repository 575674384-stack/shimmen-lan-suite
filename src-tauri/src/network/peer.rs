use crate::models::User;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub type PeerMap = Arc<Mutex<HashMap<String, PeerInfo>>>;

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub user: User,
    pub last_seen: Instant,
}

pub fn create_peer_map() -> PeerMap {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn update_peer(peers: &PeerMap, user: User) {
    let mut map = peers.lock().unwrap();
    map.insert(user.id.clone(), PeerInfo {
        user,
        last_seen: Instant::now(),
    });
}

pub fn get_online_users(peers: &PeerMap) -> Vec<User> {
    let map = peers.lock().unwrap();
    map.values()
        .filter(|p| p.last_seen.elapsed() < Duration::from_secs(6))
        .map(|p| p.user.clone())
        .collect()
}

pub fn remove_stale_peers(peers: &PeerMap) {
    let mut map = peers.lock().unwrap();
    let stale: Vec<String> = map.iter()
        .filter(|(_, p)| p.last_seen.elapsed() >= Duration::from_secs(6))
        .map(|(k, _)| k.clone())
        .collect();
    for id in stale {
        map.remove(&id);
    }
}
