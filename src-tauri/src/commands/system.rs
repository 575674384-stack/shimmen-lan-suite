use tauri::command;

#[command]
pub fn enable_autostart() -> Result<(), String> {
    crate::system::autostart::setup_autostart()
        .map_err(|e| e.to_string())
}

#[command]
pub fn disable_autostart() -> Result<(), String> {
    crate::system::autostart::remove_autostart()
        .map_err(|e| e.to_string())
}
