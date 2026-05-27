use tauri::command;
use tauri::Manager;
use crate::db::DbPool;
use crate::models::{Task, NetworkMessage};
use crate::network::server::ConnectionPool;
use crate::network::client::broadcast_message;
use crate::config::load_config;

#[command]
pub fn get_tasks(db: tauri::State<DbPool>) -> Result<Vec<Task>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, title, project, deadline, contact, priority, description, status, creator_id, assignee_id, is_team_visible, attached_files, archived_to_folder_id, created_at, updated_at FROM tasks ORDER BY updated_at DESC"
    ).map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([], |row| {
        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            project: row.get(2)?,
            deadline: row.get(3)?,
            contact: row.get(4)?,
            priority: row.get::<_, String>(5)?.parse().unwrap_or(crate::models::Priority::Medium),
            description: row.get(6)?,
            status: row.get::<_, String>(7)?.parse().unwrap_or(crate::models::Status::Todo),
            creator_id: row.get(8)?,
            assignee_id: row.get(9)?,
            is_team_visible: row.get::<_, i32>(10)? != 0,
            attached_files: serde_json::from_str(&row.get::<_, String>(11)?).unwrap_or_default(),
            archived_to_folder_id: row.get(12)?,
            created_at: row.get(13)?,
            updated_at: row.get(14)?,
        })
    }).map_err(|e| e.to_string())?;
    
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[command]
pub fn save_task(task: Task, db: tauri::State<DbPool>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let config = load_config();
    let now = chrono::Utc::now().timestamp();
    let conn = db.lock().map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT OR REPLACE INTO tasks (id, title, project, deadline, contact, priority, description, status, creator_id, assignee_id, is_team_visible, attached_files, archived_to_folder_id, created_at, updated_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        [
            task.id.clone(),
            task.title.clone(),
            task.project.clone(),
            task.deadline.clone().unwrap_or_default(),
            task.contact.clone(),
            task.priority.to_string(),
            task.description.clone(),
            task.status.to_string(),
            config.device_id,
            task.assignee_id.clone().unwrap_or_default(),
            if task.is_team_visible { "1".to_string() } else { "0".to_string() },
            serde_json::to_string(&task.attached_files).unwrap_or_default(),
            task.archived_to_folder_id.clone().unwrap_or_default(),
            now.to_string(),
            now.to_string(),
            "[]".to_string(),
        ],
    ).map_err(|e| e.to_string())?;
    
    // 广播同步
    if task.is_team_visible {
        let msg = NetworkMessage::StateSync {
            table: "tasks".to_string(),
            data: serde_json::to_value(&task).unwrap_or_default(),
            version: serde_json::json!({"updated_at": now}),
        };
        
        if let Some(state) = app_handle.try_state::<ConnectionPool>() {
            broadcast_message(state.inner(), &msg);
        }
    }
    
    Ok(())
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[command]
pub fn delete_task(id: String, db: tauri::State<DbPool>) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM tasks WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn update_task_status(id: String, status: String, db: tauri::State<DbPool>) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
        [status.clone(), now.to_string(), id.clone()],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn archive_task(
    task_id: String,
    folder_id: String,
    db: tauri::State<DbPool>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    
    // Get task
    let task: Task = conn.query_row(
        "SELECT id, title, project, deadline, contact, priority, description, status, creator_id, assignee_id, is_team_visible, attached_files, archived_to_folder_id, created_at, updated_at FROM tasks WHERE id = ?1",
        [&task_id],
        |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                project: row.get(2)?,
                deadline: row.get(3)?,
                contact: row.get(4)?,
                priority: row.get::<_, String>(5)?.parse().unwrap_or(crate::models::Priority::Medium),
                description: row.get(6)?,
                status: row.get::<_, String>(7)?.parse().unwrap_or(crate::models::Status::Todo),
                creator_id: row.get(8)?,
                assignee_id: row.get(9)?,
                is_team_visible: row.get::<_, i32>(10)? != 0,
                attached_files: serde_json::from_str(&row.get::<_, String>(11)?).unwrap_or_default(),
                archived_to_folder_id: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        },
    ).map_err(|e| e.to_string())?;
    
    // Get folder path
    let folder_path: String = conn.query_row(
        "SELECT local_path FROM shared_folders WHERE id = ?1",
        [&folder_id],
        |row| row.get(0),
    ).map_err(|e| format!("找不到共享文件夹: {}", e))?;
    
    // Create archive directory
    let archive_dir = std::path::Path::new(&folder_path).join(format!("archive_{}", task_id));
    std::fs::create_dir_all(&archive_dir).map_err(|e| format!("创建归档目录失败: {}", e))?;
    
    // Copy attached files and folders
    for file_path in &task.attached_files {
        let src = std::path::Path::new(file_path);
        if src.exists() {
            let file_name = src.file_name().unwrap_or_default();
            let dst = archive_dir.join(file_name);
            if src.is_dir() {
                let _ = copy_dir_all(src, &dst);
            } else {
                let _ = std::fs::copy(src, dst);
            }
        }
    }
    
    // Create summary file
    let summary = format!(
        "任务归档\n========\n标题: {}\n项目: {}\n联系人: {}\n优先级: {}\n描述: {}\n归档时间: {}\n",
        task.title,
        task.project,
        task.contact,
        task.priority,
        task.description,
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
    );
    let summary_path = archive_dir.join("归档说明.txt");
    std::fs::write(&summary_path, summary).map_err(|e| format!("写入归档说明失败: {}", e))?;
    
    // Update task archived_to_folder_id
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE tasks SET archived_to_folder_id = ?1, updated_at = ?2 WHERE id = ?3",
        [&folder_id, &now.to_string(), &task_id],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}
