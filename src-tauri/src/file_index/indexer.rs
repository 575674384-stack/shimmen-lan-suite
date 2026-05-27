use std::path::Path;
use crate::db::DbPool;

#[derive(serde::Serialize)]
pub struct FileIndexEntry {
    pub id: i64,
    pub peer_id: String,
    pub peer_name: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub modified_at: i64,
    pub is_local: bool,
}

const DEFAULT_EXCLUDES: &[&str] = &[
    "Windows", "Program Files", "Program Files (x86)", "ProgramData",
    "AppData", "Temp", "$Recycle.Bin", "System Volume Information",
    ".git", "node_modules", "target", "__pycache__", ".next", "dist", "build",
];

pub fn is_excluded(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    DEFAULT_EXCLUDES.iter().any(|ex| path_str.contains(ex))
}

pub fn scan_directories(paths: Vec<String>, peer_id: &str, peer_name: &str, db: &DbPool) -> Result<usize, String> {
    let mut count = 0;
    let conn = db.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM file_index WHERE peer_id = ?1 AND is_local = 1", [peer_id])
        .map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "INSERT INTO file_index (peer_id, peer_name, file_name, file_path, file_size, modified_at, is_local)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)"
    ).map_err(|e| e.to_string())?;

    for path_str in paths {
        let base_path = Path::new(&path_str);
        if !base_path.exists() {
            continue;
        }

        for entry in walkdir::WalkDir::new(base_path)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let full_path = entry.path();
            if is_excluded(full_path) {
                continue;
            }

            let file_name = entry.file_name().to_string_lossy().to_string();
            let path_str = full_path.to_string_lossy().to_string();
            let metadata = entry.metadata().ok();
            let file_size = metadata.as_ref().map(|m| m.len() as i64).unwrap_or(0);
            let modified_at = metadata
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            stmt.execute([
                peer_id, peer_name, &file_name, &path_str,
                &file_size.to_string(), &modified_at.to_string(),
            ]).ok();
            count += 1;
        }
    }

    Ok(count)
}

pub fn search_all(query: &str, db: &DbPool) -> Result<Vec<FileIndexEntry>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let pattern = format!("%{}%", query);

    let mut stmt = conn.prepare(
        "SELECT id, peer_id, peer_name, file_name, file_path, file_size, modified_at, is_local
         FROM file_index
         WHERE file_name LIKE ?1
         ORDER BY is_local DESC, file_name
         LIMIT 200"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([&pattern], |row| {
        Ok(FileIndexEntry {
            id: row.get(0)?,
            peer_id: row.get(1)?,
            peer_name: row.get(2)?,
            file_name: row.get(3)?,
            file_path: row.get(4)?,
            file_size: row.get(5)?,
            modified_at: row.get(6)?,
            is_local: row.get::<_, i64>(7)? != 0,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for row in rows {
        if let Ok(entry) = row {
            results.push(entry);
        }
    }
    Ok(results)
}

pub fn clear_peer_index(peer_id: &str, db: &DbPool) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM file_index WHERE peer_id = ?1 AND is_local = 0", [peer_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn insert_remote_files(peer_id: &str, peer_name: &str, files: &[crate::models::RemoteFileInfo], db: &DbPool) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM file_index WHERE peer_id = ?1 AND is_local = 0", [peer_id])
        .map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "INSERT INTO file_index (peer_id, peer_name, file_name, file_path, file_size, modified_at, is_local)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)"
    ).map_err(|e| e.to_string())?;

    for file in files {
        stmt.execute([
            peer_id, peer_name, &file.file_name, &file.file_path,
            &(file.file_size as i64).to_string(), &file.modified_at.to_string(),
        ]).ok();
    }

    Ok(())
}

pub fn search_local(query: &str, db: &DbPool) -> Result<Vec<crate::models::RemoteFileInfo>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let pattern = format!("%{}%", query);

    let mut stmt = conn.prepare(
        "SELECT file_name, file_path, file_size, modified_at
         FROM file_index
         WHERE is_local = 1 AND file_name LIKE ?1
         LIMIT 100"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([&pattern], |row| {
        Ok(crate::models::RemoteFileInfo {
            file_name: row.get(0)?,
            file_path: row.get(1)?,
            file_size: row.get::<_, i64>(2)? as u64,
            modified_at: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for row in rows {
        if let Ok(entry) = row {
            results.push(entry);
        }
    }
    Ok(results)
}
