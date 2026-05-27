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
    confy::load("shimmen-lan-suite", "config").unwrap_or_default()
}

pub fn save_config(cfg: &AppConfig) -> Result<(), confy::ConfyError> {
    confy::store("shimmen-lan-suite", "config", cfg)
}
