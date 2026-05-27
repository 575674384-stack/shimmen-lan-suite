use tauri::command;
use rfd::FileDialog;

#[command]
pub async fn select_folder() -> Result<Option<String>, String> {
    let folder = FileDialog::new().pick_folder();
    Ok(folder.map(|p| p.to_string_lossy().to_string()))
}

#[command]
pub async fn select_file() -> Result<Option<String>, String> {
    let file = rfd::AsyncFileDialog::new().pick_file().await;
    Ok(file.map(|f| f.path().to_string_lossy().to_string()))
}
