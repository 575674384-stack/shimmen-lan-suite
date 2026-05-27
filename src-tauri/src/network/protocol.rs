use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryPacket {
    pub id: String,
    pub username: String,
    pub ip: String,
    pub version: String,
    pub timestamp: i64,
}

// HeartbeatPacket reserved for future use
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct HeartbeatPacket {
    pub id: String,
    pub timestamp: i64,
}

pub const DISCOVERY_INTERVAL_SECS: u64 = 2;
pub const PEER_TIMEOUT_SECS: u64 = 6;
