use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub ip: String,
    pub status: UserStatus,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Online,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    Syncing,
    Paused,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl std::str::FromStr for Priority {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            _ => Ok(Priority::Medium),
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Low => write!(f, "low"),
            Priority::Medium => write!(f, "medium"),
            Priority::High => write!(f, "high"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Todo,
    Doing,
    Done,
}

impl std::str::FromStr for Status {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "todo" => Ok(Status::Todo),
            "doing" => Ok(Status::Doing),
            "done" => Ok(Status::Done),
            _ => Ok(Status::Todo),
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Todo => write!(f, "todo"),
            Status::Doing => write!(f, "doing"),
            Status::Done => write!(f, "done"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharedFolder {
    pub id: String,
    pub owner_id: String,
    pub owner_name: String,
    pub local_path: String,
    pub name: String,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub modified: i64,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntry {
    pub id: String,
    pub name: String,
    pub account: String,
    pub password: String,
    pub note: String,
    pub created_by: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Announcement {
    pub id: String,
    pub title: String,
    pub content: String,
    pub is_pinned: bool,
    pub created_by: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub project: String,
    pub deadline: Option<String>,
    pub contact: String,
    pub priority: Priority,
    pub description: String,
    pub status: Status,
    pub creator_id: String,
    pub assignee_id: Option<String>,
    pub is_team_visible: bool,
    #[serde(default)]
    pub attached_files: Vec<String>,
    #[serde(default)]
    pub archived_to_folder_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteFileInfo {
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub modified_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum NetworkMessage {
    ChatMessage { id: String, sender_id: String, sender_name: String, content: String, message_type: String },
    ClearScreen,
    StateSync { table: String, data: serde_json::Value, version: serde_json::Value },
    FileList { folder_id: String, files: Vec<FileInfo> },
    FileRequest { folder_id: String, file_path: String },
    FileResponse { folder_id: String, file_path: String, content_base64: String },
    ScreenShare { frame_base64: String },
    FileIndexBroadcast { peer_id: String, peer_name: String, files: Vec<RemoteFileInfo> },
    FileIndexRequest { requester_id: String },
    FileSearchRequest { requester_id: String, query: String },
    FileSearchResponse { responder_id: String, results: Vec<RemoteFileInfo> },
    FileTransferRequest { requester_id: String, file_path: String },
    FileChunk { folder_id: String, file_path: String, chunk_index: u32, total_chunks: u32, data_base64: String },
    Heartbeat,
    Relay { origin_peer_id: String, payload: Box<NetworkMessage> },
}
