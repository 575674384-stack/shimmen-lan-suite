use serde::{Deserialize, Serialize};
use tauri::Manager;

pub const DISCOVERY_PORT: u16 = 23333;
pub const CONTROL_PORT: u16 = 23334;
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub username: String,
    pub device_id: String,
    #[serde(default)]
    pub avatar_preset: String,
    /// 文件接收保存路径（空字符串表示使用默认 app_data_dir/downloads）
    #[serde(default)]
    pub download_dir: String,
    /// 共享文件夹同步间隔（秒），0 表示实时
    #[serde(default = "default_sync_interval")]
    pub sync_interval_secs: u64,
    /// 开机自启
    #[serde(default)]
    pub autostart: bool,
    /// 截屏分享帧率
    #[serde(default = "default_fps")]
    pub screen_fps: u64,
    /// 截屏分享分辨率
    #[serde(default = "default_resolution")]
    pub screen_resolution: u64,
    /// 自动检查更新
    #[serde(default = "default_auto_update")]
    pub auto_update: bool,
}

fn default_fps() -> u64 {
    10
}

fn default_resolution() -> u64 {
    720
}

fn default_auto_update() -> bool {
    true
}

fn default_sync_interval() -> u64 {
    0 // 默认实时同步
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            username: whoami::username(),
            device_id: uuid::Uuid::new_v4().to_string(),
            avatar_preset: String::new(),
            download_dir: String::new(),
            sync_interval_secs: 0,
            autostart: false,
            screen_fps: 10,
            screen_resolution: 720,
            auto_update: true,
        }
    }
}

pub fn load_config() -> AppConfig {
    let mut cfg: AppConfig = confy::load("shimmen-lan-suite", "config").unwrap_or_default();

    // Device ID is the source of identity — keep it stable across config corruption
    let id_file = dirs::data_dir()
        .map(|d| d.join("shimmen-lan-suite"))
        .map(|d| d.join(".device_id"));

    let id_from_file = id_file.as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if let Some(id) = id_from_file {
        cfg.device_id = id;
    } else if cfg.device_id.is_empty() {
        cfg.device_id = uuid::Uuid::new_v4().to_string();
    }

    // 只在配置或 device_id 发生变化时才写盘，避免高频 I/O
    let need_save_config = confy::load::<AppConfig>("shimmen-lan-suite", "config")
        .map(|saved| saved.device_id != cfg.device_id || saved.username != cfg.username)
        .unwrap_or(true);
    if need_save_config {
        if let Err(e) = save_config(&cfg) {
            eprintln!("[config] failed to save config during load: {}", e);
        }
    }
    if let Some(p) = id_file {
        let need_write_id = std::fs::read_to_string(&p).ok().map(|s| s.trim() != cfg.device_id).unwrap_or(true);
        if need_write_id {
            if let Err(e) = std::fs::create_dir_all(p.parent().unwrap_or(&p)) {
                eprintln!("[config] failed to create device_id dir: {}", e);
            }
            if let Err(e) = std::fs::write(&p, &cfg.device_id) {
                eprintln!("[config] failed to write device_id file: {}", e);
            }
        }
    }

    cfg
}

pub fn save_config(cfg: &AppConfig) -> Result<(), confy::ConfyError> {
    confy::store("shimmen-lan-suite", "config", cfg)
}

/// 获取实际使用的下载目录：优先用户配置， fallback 到 app_data_dir/downloads
pub fn get_effective_download_dir(app_handle: &tauri::AppHandle) -> std::path::PathBuf {
    let cfg = load_config();
    if !cfg.download_dir.is_empty() {
        let p = std::path::PathBuf::from(&cfg.download_dir);
        let _ = std::fs::create_dir_all(&p);
        return p;
    }
    let app_dir = app_handle.path().app_data_dir().unwrap_or_default();
    let p = app_dir.join("downloads");
    let _ = std::fs::create_dir_all(&p);
    p
}
