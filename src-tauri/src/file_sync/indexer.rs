use crate::models::FileInfo;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use blake3::Hasher;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn index_folder(folder_path: &str) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(folder_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let path = entry.path();
            let metadata = fs::metadata(path)?;
            let modified = metadata
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64;

            let mut hasher = Hasher::new();
            let content = fs::read(path)?;
            hasher.update(&content);
            let hash = hasher.finalize().to_hex().to_string();

            files.push(FileInfo {
                path: path
                    .strip_prefix(folder_path)?
                    .to_string_lossy()
                    .replace('\\', "/"),
                size: metadata.len(),
                modified,
                hash,
            });
        }
    }

    Ok(files)
}

pub fn read_file_base64(folder_path: &str, relative_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let full_path = Path::new(folder_path).join(relative_path);
    let content = fs::read(full_path)?;
    Ok(STANDARD.encode(&content))
}

pub fn write_file_from_base64(
    folder_path: &str,
    relative_path: &str,
    content_base64: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let full_path = Path::new(folder_path).join(relative_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = STANDARD.decode(content_base64)?;
    fs::write(full_path, content)?;
    Ok(())
}
