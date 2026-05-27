use serde::{Deserialize, Serialize};

pub const DISCOVERY_PORT: u16 = 23333;
pub const CONTROL_PORT: u16 = 23334;
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub username: String,
    pub device_id: String,
    #[serde(default)]
    pub avatar_preset: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            username: whoami::username(),
            device_id: uuid::Uuid::new_v4().to_string(),
            avatar_preset: String::new(),
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

    // Ensure both stores are synced
    let _ = save_config(&cfg);
    if let Some(p) = id_file {
        let _ = std::fs::create_dir_all(p.parent().unwrap_or(&p));
        let _ = std::fs::write(&p, &cfg.device_id);
    }

    cfg
}

pub fn save_config(cfg: &AppConfig) -> Result<(), confy::ConfyError> {
    confy::store("shimmen-lan-suite", "config", cfg)
}
