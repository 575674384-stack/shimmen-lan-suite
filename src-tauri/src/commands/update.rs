use tauri::{AppHandle, Emitter};

#[derive(serde::Serialize)]
pub struct UpdateInfo {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub release_notes: String,
}

#[tauri::command]
pub fn check_update() -> Result<UpdateInfo, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();

    let resp = ureq::get(
        "https://api.github.com/repos/575674384-stack/shimmen-lan-suite/releases/latest",
    )
    .set("User-Agent", "shimmen-lan-suite")
    .call()
    .map_err(|e| e.to_string())?;

    let json: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;

    let latest = json["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches('v')
        .to_string();
    let release_notes = json["body"].as_str().unwrap_or("").to_string();

    let mut download_url = String::new();
    if let Some(assets) = json["assets"].as_array() {
        for asset in assets {
            let name = asset["name"].as_str().unwrap_or("");
            if name.ends_with("setup.exe") || name.ends_with("-setup.exe") {
                download_url = asset["browser_download_url"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                break;
            }
        }
    }

    let has_update = version_greater(&latest, &current);

    Ok(UpdateInfo {
        has_update,
        current_version: current,
        latest_version: latest,
        download_url,
        release_notes,
    })
}

fn version_greater(a: &str, b: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> {
        s.split('.').filter_map(|p| p.parse().ok()).collect()
    };
    let va = parse(a);
    let vb = parse(b);
    for i in 0..va.len().max(vb.len()) {
        let a_val = va.get(i).copied().unwrap_or(0);
        let b_val = vb.get(i).copied().unwrap_or(0);
        if a_val != b_val {
            return a_val > b_val;
        }
    }
    false
}

#[tauri::command]
pub fn download_and_install(download_url: String, app_handle: AppHandle) -> Result<String, String> {
    if download_url.is_empty() {
        return Err("下载链接为空".to_string());
    }

    std::thread::spawn(move || {
        let tmp_dir = std::env::temp_dir();
        let installer_path = tmp_dir.join("shimmen-lan-suite-update.exe");
        let bat_path = tmp_dir.join("shimmen-update.bat");

        // 下载安装包
        let download_result = (|| -> Result<u64, String> {
            let resp = ureq::get(&download_url)
                .set("User-Agent", "shimmen-lan-suite")
                .call()
                .map_err(|e| format!("下载请求失败: {}", e))?;

            let mut file = std::fs::File::create(&installer_path)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            let mut reader = resp.into_reader();
            let bytes = std::io::copy(&mut reader, &mut file)
                .map_err(|e| format!("写入文件失败: {}", e))?;
            Ok(bytes)
        })();

        match download_result {
            Ok(bytes) if bytes > 1_048_576 => {} // 至少 1MB
            Ok(_) => {
                let _ = app_handle.emit("update-error", "下载文件过小，可能不完整".to_string());
                return;
            }
            Err(e) => {
                let _ = app_handle.emit("update-error", e);
                return;
            }
        }

        // 创建自删除的批处理脚本：等待旧进程退出 -> 启动安装程序 -> 删除自己
        let bat = format!(
            "@echo off\r\ntimeout /t 2 /nobreak >nul\r\nstart \"\" \"{}\"\r\ndel \"%~f0\"\r\n",
            installer_path.display()
        );
        if let Err(e) = std::fs::write(&bat_path, bat) {
            let _ = app_handle.emit("update-error", format!("创建启动脚本失败: {}", e));
            return;
        }

        // 启动批处理（独立进程，最小化窗口）
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", "/min", bat_path.to_str().unwrap_or("")])
            .spawn();

        // 给批处理一点时间启动
        std::thread::sleep(std::time::Duration::from_millis(800));

        // 通知前端准备就绪，然后退出
        let _ = app_handle.emit("update-ready", ());
        std::thread::sleep(std::time::Duration::from_millis(500));
        app_handle.exit(0);
    });

    Ok("已开始下载更新".to_string())
}

#[tauri::command]
pub fn exit_app(app_handle: AppHandle) {
    app_handle.exit(0);
}
