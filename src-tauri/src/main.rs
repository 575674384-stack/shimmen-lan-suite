#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

mod commands;
mod db;
mod models;
mod network;
mod sync;
mod file_sync;
mod system;
mod config;
mod webview2_check;
mod file_index;

fn main() {
    webview2_check::ensure_webview2();

    // 初始化 tracing 日志：stdout + 日志文件
    let log_dir = dirs::data_dir()
        .map(|d| d.join("shimmen-lan-suite"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let _ = std::fs::create_dir_all(&log_dir);
    let log_file = tracing_appender::rolling::daily(&log_dir, "shimmen.log");
    let subscriber = tracing_subscriber::fmt()
        .with_writer(log_file)
        .with_ansi(false)
        .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let db_pool = db::init_db(&app_dir)?;
            app.manage(db_pool.clone());
            
            // setup tray
            system::tray::setup_tray(app.app_handle())?;
            
            // check --minimized
            let args: Vec<String> = std::env::args().collect();
            if args.contains(&"--minimized".to_string()) {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            
            // load config
            let config = config::load_config();
            let my_id = config.device_id.clone();
            
            // Peers map must exist before both TCP server and discovery
            let peers = network::peer::create_peer_map();
            app.manage(peers.clone());
            
            // start TCP server FIRST (before discovery so port is ready when peers try to connect)
            let pool = network::server::create_connection_pool();
            app.manage(pool.clone());
            
            let folder_cache = network::folder_cache::create_folder_cache();
            app.manage(folder_cache);
            let pool_for_server = pool.clone();
            let peers_for_server = peers.clone();
            let app_handle = app.app_handle().clone();
            let app_handle_for_reconnect = app_handle.clone();
            std::thread::spawn(move || {
                let _ = network::server::start_server(
                    config::CONTROL_PORT,
                    peers_for_server,
                    pool_for_server,
                    app_handle,
                );
            });
            
            // start discovery AFTER TCP server is bound
            let peers_for_discovery = peers.clone();
            let config_for_discovery = config.clone();
            let app_handle_for_discovery = app.app_handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(1));
                let _ = network::discovery::start_discovery(config_for_discovery, peers_for_discovery, app_handle_for_discovery);
            });
            
            // create and manage sync engine
            let sync_engine = std::sync::Arc::new(file_sync::engine::SyncEngine::new(
                db_pool.clone(),
                pool.clone(),
                app_dir.clone(),
            ));
            app.manage(sync_engine.clone());
            
            // start monitoring existing shared folders
            let db_for_monitor = db_pool.clone();
            let engine_for_monitor = sync_engine.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(1));
                if let Ok(conn) = db_for_monitor.lock() {
                    let mut stmt = match conn.prepare(
                        "SELECT id, local_path FROM shared_folders WHERE sync_status = 'syncing'"
                    ) {
                        Ok(s) => s,
                        Err(_) => return,
                    };
                    let rows = stmt.query_map([], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    });
                    if let Ok(rows) = rows {
                        for row in rows.flatten() {
                            let (folder_id, local_path) = row;
                            let _ = engine_for_monitor.start_monitoring(folder_id, local_path);
                        }
                    }
                }
            });
            
            // 广播自己的共享文件夹
            let db_for_broadcast = db_pool.clone();
            let pool_for_broadcast = pool.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(2));
                if let Ok(conn) = db_for_broadcast.lock() {
                    if let Ok(mut stmt) = conn.prepare("SELECT id, owner_id, owner_name, local_path, name, sync_status FROM shared_folders") {
                        if let Ok(rows) = stmt.query_map([], |row| {
                            Ok(models::SharedFolder {
                                id: row.get(0)?,
                                owner_id: row.get(1)?,
                                owner_name: row.get(2)?,
                                local_path: row.get(3)?,
                                name: row.get(4)?,
                                sync_status: match row.get::<_, String>(5)?.as_str() {
                                    "syncing" => models::SyncStatus::Syncing,
                                    "paused" => models::SyncStatus::Paused,
                                    "error" => models::SyncStatus::Error,
                                    _ => models::SyncStatus::Paused,
                                },
                            })
                        }) {
                            let folders: Vec<_> = rows.flatten().collect();
                            if !folders.is_empty() {
                                if let Ok(data) = serde_json::to_value(&folders) {
                                    let msg = models::NetworkMessage::StateSync {
                                        table: "shared_folders".to_string(),
                                        data,
                                        version: serde_json::json!({}),
                                    };
                                    network::client::broadcast_message(&pool_for_broadcast, &msg);
                                }
                            }
                        }
                    }
                }
            });
            
            // 启动文件索引扫描并广播
            let db_for_index = db_pool.clone();
            let pool_for_index = pool.clone();
            let my_id_for_index = my_id.clone();
            let config_for_index = config.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(5));
                let default_paths = vec![
                    dirs::desktop_dir().map(|p| p.to_string_lossy().to_string()),
                    dirs::document_dir().map(|p| p.to_string_lossy().to_string()),
                    dirs::download_dir().map(|p| p.to_string_lossy().to_string()),
                ].into_iter().flatten().collect::<Vec<_>>();
                
                if let Ok(count) = file_index::indexer::scan_directories(default_paths, &my_id_for_index, &config_for_index.username, &db_for_index) {
                    println!("文件索引完成: {} 个文件", count);
                    file_index::network::broadcast_index(&db_for_index, &pool_for_index, &my_id_for_index, &config_for_index.username);
                }
            });

            // 星型拓扑：只连接 Leader，Leader 不主动 outbound
            let pool_for_connect = pool.clone();
            let peers_for_connect = peers.clone();
            let my_id_for_connect = my_id.clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    network::leader::ensure_leader(&peers_for_connect, &my_id_for_connect);
                    let leader_id = network::leader::get_leader_id();
                    
                    let pool_ids: std::collections::HashSet<String> = {
                        let p = pool_for_connect.lock().unwrap();
                        p.keys().cloned().collect()
                    };
                    
                    // 断开所有非 Leader 的连接
                    if let Some(ref lid) = leader_id {
                        let to_remove: Vec<String> = pool_ids.iter()
                            .filter(|id| *id != lid)
                            .cloned()
                            .collect();
                        if !to_remove.is_empty() {
                            let mut p = pool_for_connect.lock().unwrap();
                            for id in to_remove {
                                p.remove(&id);
                            }
                        }
                    }
                    
                    // Leader 不需要主动连接任何人
                    if network::leader::is_leader(&my_id_for_connect) {
                        continue;
                    }
                    
                    // 非 Leader：确保已连接 Leader（直接检查 pool，不依赖 pending）
                    if let Some(ref lid) = leader_id {
                        if !pool_ids.contains(lid) {
                            let leader_ip = {
                                let peers_map = peers_for_connect.lock().unwrap();
                                peers_map.get(lid).map(|p| p.user.ip.clone())
                            };
                            if let Some(ip) = leader_ip {
                                network::client::connect_to_peer(
                                    lid.clone(),
                                    ip,
                                    config::CONTROL_PORT,
                                    pool_for_connect.clone(),
                                    my_id_for_connect.clone(),
                                    app_handle_for_reconnect.clone(),
                                );
                            }
                        }
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::system::enable_autostart,
            commands::system::disable_autostart,
            commands::chat::send_chat_message,
            commands::chat::send_chat_file,
            commands::chat::clear_chat_screen,
            commands::chat::get_chat_history,
            commands::folder_sync::get_my_shared_folders,
            commands::folder_sync::get_remote_shared_folders,
            commands::folder_sync::create_shared_folder,
            commands::folder_sync::subscribe_shared_folder,
            commands::folder_sync::list_folder_files,
            commands::folder_sync::delete_shared_folder,
            commands::vault::get_passwords,
            commands::vault::save_password,
            commands::vault::delete_password,
            commands::announcement::get_announcements,
            commands::announcement::save_announcement,
            commands::announcement::delete_announcement,
            commands::board::get_tasks,
            commands::board::save_task,
            commands::board::delete_task,
            commands::board::update_task_status,
            commands::board::archive_task,
            commands::config::get_config,
            commands::config::set_username,
            commands::config::set_download_dir,
            commands::config::set_sync_interval,
            commands::config::set_autostart,
            commands::config::get_autostart_status,
            commands::config::set_screen_fps,
            commands::config::set_screen_resolution,
            commands::config::set_auto_update,
            commands::dialog::select_folder,
            commands::dialog::select_file,
            commands::file_transfer::send_file_to_peer,
            commands::file_transfer::get_download_dir,
            commands::file_transfer::get_app_download_dir,
            commands::file_transfer::read_file_base64,
            commands::screen_share::start_screen_share,
            commands::screen_share::stop_screen_share,
            commands::network::get_online_users_cmd,
            commands::ai::get_ai_config,
            commands::ai::set_ai_config,
            commands::avatar::set_avatar,
            commands::avatar::get_avatar,
            commands::avatar::set_avatar_preset,
            commands::tools::get_system_info,
            commands::tools::get_network_interfaces,
            commands::tools::set_dns,
            commands::tools::activate_windows,
            commands::tools::check_powershell7,
            commands::tools::install_powershell7,
            commands::tools::check_utf8,
            commands::tools::set_utf8,
            commands::tools::run_optimize,
            commands::tools::search_files,
            commands::tools::open_file_location,
            commands::tools::list_files_in_dir,
            commands::tools::preview_rename,
            commands::tools::execute_rename,
            commands::tools::get_installed_software,
            commands::tools::install_software,
            commands::tools::get_printers,
            commands::tools::get_print_jobs,
            commands::tools::clear_print_queue,
            commands::tools::get_network_details,
            commands::tools::get_network_status,
            commands::file_index::get_indexed_directories,
            commands::file_index::rebuild_file_index,
            commands::file_index::search_files_network,
            commands::file_index::clear_remote_index,
            commands::file_index::request_file_from_peer,
            commands::update::check_update,
            commands::update::download_and_install,
            commands::update::exit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
