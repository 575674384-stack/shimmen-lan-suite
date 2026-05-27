use chrono::{DateTime, Duration, Utc};
use std::fs;
use std::path::Path;

pub fn create_snapshot(
    folder_path: &str,
    relative_path: &str,
    history_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = Path::new(folder_path).join(relative_path);
    if !source.exists() {
        return Ok(());
    }

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let snapshot_path = Path::new(history_dir)
        .join(".shimmen_history")
        .join(&timestamp)
        .join(relative_path);

    if let Some(parent) = snapshot_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(source, snapshot_path)?;
    Ok(())
}

pub fn cleanup_old_snapshots(history_dir: &str, days: i64) -> Result<(), Box<dyn std::error::Error>> {
    let base_path = Path::new(history_dir).join(".shimmen_history");
    if !base_path.exists() {
        return Ok(());
    }

    let cutoff = Utc::now() - Duration::days(days);

    for entry in fs::read_dir(base_path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        if let Ok(dt) = DateTime::parse_from_str(&format!("{} +0000", name), "%Y%m%d_%H%M%S %z") {
            if dt < cutoff {
                let _ = fs::remove_dir_all(entry.path());
            }
        }
    }

    Ok(())
}
