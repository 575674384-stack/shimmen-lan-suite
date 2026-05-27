use crate::db::DbPool;
use crate::file_sync::history;
use crate::file_sync::indexer;
use crate::models::{FileInfo, NetworkMessage};
use crate::network::client::{broadcast_message, send_to_peer};
use crate::network::server::ConnectionPool;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct SyncEngine {
    db: DbPool,
    pool: ConnectionPool,
    app_dir: PathBuf,
    watchers: Arc<Mutex<HashMap<String, super::watcher::FolderWatcher>>>,
}

impl SyncEngine {
    pub fn new(db: DbPool, pool: ConnectionPool, app_dir: PathBuf) -> Self {
        Self {
            db,
            pool,
            app_dir,
            watchers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start_monitoring(
        &self,
        folder_id: String,
        folder_path: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let interval = crate::config::load_config().sync_interval_secs;
        let pool = self.pool.clone();
        let fid = folder_id.clone();
        let fpath = folder_path.clone();

        if interval == 0 {
            // 实时模式：文件系统事件监听
            let watcher = super::watcher::FolderWatcher::new(
                folder_path,
                Box::new(move || {
                    if let Ok(files) = indexer::index_folder(&fpath) {
                        let msg = NetworkMessage::FileList {
                            folder_id: fid.clone(),
                            files,
                        };
                        let _ = broadcast_message(&pool, &msg);
                    }
                }),
            )?;
            let mut w = self.watchers.lock().map_err(|e| e.to_string())?;
            w.insert(folder_id, watcher);
        } else {
            // 定时模式：按配置间隔轮询扫描
            let pool = self.pool.clone();
            std::thread::spawn(move || {
                let duration = std::time::Duration::from_secs(interval);
                loop {
                    std::thread::sleep(duration);
                    if let Ok(files) = indexer::index_folder(&fpath) {
                        let msg = NetworkMessage::FileList {
                            folder_id: fid.clone(),
                            files,
                        };
                        let _ = broadcast_message(&pool, &msg);
                    }
                }
            });
        }

        Ok(())
    }

    pub fn handle_file_list(
        &self,
        folder_id: &str,
        files: &[FileInfo],
        local_sync_path: &str,
        peer_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. 扫描本地同步路径的当前状态
        let local_files = indexer::index_folder(local_sync_path)?;
        let local_map: HashMap<String, &FileInfo> =
            local_files.iter().map(|f| (f.path.clone(), f)).collect();

        // 2. 对比差异
        for remote_file in files {
            let local = local_map.get(&remote_file.path);
            let need_update = match local {
                Some(l) => l.hash != remote_file.hash,
                None => true,
            };

            if need_update {
                // 如果本地文件存在且较新，保留冲突副本
                if local.map(|l| l.modified > remote_file.modified).unwrap_or(false) {
                    let _conflict_path = format!("{} (冲突)", remote_file.path);
                    // 未来：发送 FileRequest 请求源文件，保存为冲突副本
                } else {
                    // 请求更新
                    let msg = NetworkMessage::FileRequest {
                        folder_id: folder_id.to_string(),
                        file_path: remote_file.path.clone(),
                    };
                    let _ = send_to_peer(&self.pool, peer_id, &msg);
                }
            }
        }

        Ok(())
    }

    pub fn handle_file_request(
        &self,
        folder_id: &str,
        file_path: &str,
        folder_local_path: &str,
    ) -> Result<NetworkMessage, Box<dyn std::error::Error>> {
        // 创建快照
        let history_base = self.app_dir.to_string_lossy().to_string();
        let _ = history::create_snapshot(folder_local_path, file_path, &history_base);

        let content_base64 = indexer::read_file_base64(folder_local_path, file_path)?;

        Ok(NetworkMessage::FileResponse {
            folder_id: folder_id.to_string(),
            file_path: file_path.to_string(),
            content_base64,
        })
    }

    pub fn handle_file_response(
        &self,
        _folder_id: &str,
        file_path: &str,
        content_base64: &str,
        local_sync_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 写入前先创建快照
        let history_base = self.app_dir.to_string_lossy().to_string();
        let _ = history::create_snapshot(local_sync_path, file_path, &history_base);

        indexer::write_file_from_base64(local_sync_path, file_path, content_base64)?;
        Ok(())
    }

    pub fn cleanup_history(&self) -> Result<(), Box<dyn std::error::Error>> {
        let history_base = self.app_dir.to_string_lossy().to_string();
        history::cleanup_old_snapshots(&history_base, 15)
    }

    /// 尝试处理远程文件请求：查询本地路径并构造 FileResponse
    pub fn try_handle_file_request(
        &self,
        folder_id: &str,
        file_path: &str,
    ) -> Option<NetworkMessage> {
        let conn = self.db.lock().ok()?;
        let local_path: String = conn
            .query_row(
                "SELECT local_path FROM shared_folders WHERE id = ?1",
                rusqlite::params![folder_id],
                |row| row.get(0),
            )
            .ok()?;
        self.handle_file_request(folder_id, file_path, &local_path).ok()
    }
}
