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
    if cfg.device_id.is_empty() {
        // Fallback: try dedicated device ID file
        if let Some(app_dir) = dirs::data_dir().map(|d| d.join("shimmen-lan-suite")) {
            let id_file = app_dir.join(".device_id");
            if let Ok(id) = std::fs::read_to_string(&id_file) {
                cfg.device_id = id.trim().to_string();
            }
        }
        if cfg.device_id.is_empty() {
            cfg.device_id = uuid::Uuid::new_v4().to_string();
            // Save fallback for next time
            if let Some(app_dir) = dirs::data_dir().map(|d| d.join("shimmen-lan-suite")) {
                let _ = std::fs::create_dir_all(&app_dir);
                let _ = std::fs::write(app_dir.join(".device_id"), &cfg.device_id);
            }
        }
    }
    cfg
}

pub fn save_config(cfg: &AppConfig) -> Result<(), confy::ConfyError> {
    confy::store("shimmen-lan-suite", "config", cfg)
}
