use tauri::command;
use tauri::Emitter;
use crate::db::DbPool;
use crate::models::PasswordEntry;
use crate::config::load_config;

#[command]
pub fn get_passwords(db: tauri::State<DbPool>) -> Result<Vec<PasswordEntry>, String> {
    let config = load_config();
    let conn = db.get().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, name, account, password, note, created_by, updated_at FROM password_entries ORDER BY updated_at DESC"
    ).map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([], |row| {
        let encrypted_pw: String = row.get(3)?;
        let decrypted_pw = crate::system::crypto::decrypt_password(&encrypted_pw, &config.device_id)
            .unwrap_or_else(|_| String::new());
        
        Ok(PasswordEntry {
            id: row.get(0)?,
            name: row.get(1)?,
            account: row.get(2)?,
            password: decrypted_pw,
            note: row.get(4)?,
            created_by: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;
    
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[command]
pub fn save_password(
    entry: PasswordEntry,
    db: tauri::State<DbPool>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let config = load_config();
    let encrypted_pw = crate::system::crypto::encrypt_password(&entry.password, &config.device_id)
        .map_err(|e| e.to_string())?;
    
    let now = chrono::Utc::now().timestamp();
    let conn = db.get().map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT OR REPLACE INTO password_entries (id, name, account, password, note, created_by, updated_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        [
            entry.id.clone(),
            entry.name.clone(),
            entry.account.clone(),
            encrypted_pw,
            entry.note.clone(),
            config.device_id.clone(),
            now.to_string(),
            "[]".to_string(),
        ],
    ).map_err(|e| e.to_string())?;
    
    // 安全：密码绝不通过网络广播。仅本地保存。
    // 如需同步，应使用端到端加密或仅同步非敏感元数据。
    let _ = app_handle.emit("password-saved", serde_json::json!({
        "id": entry.id,
        "name": entry.name,
    }));
    
    Ok(())
}

#[command]
pub fn delete_password(id: String, db: tauri::State<DbPool>) -> Result<(), String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM password_entries WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
