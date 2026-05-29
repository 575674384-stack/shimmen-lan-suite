use tauri::command;
use tauri::{Emitter, Manager};
use crate::network::server::ConnectionPool;
use crate::network::client;
use crate::models::NetworkMessage;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use image::{DynamicImage, ImageEncoder, imageops::FilterType};
use image::codecs::jpeg::JpegEncoder;
use base64::Engine;

static SHARING: AtomicBool = AtomicBool::new(false);
static TARGET_FPS: AtomicU64 = AtomicU64::new(10);
static TARGET_RES: AtomicU64 = AtomicU64::new(720); // 720p / 540p / 450p

#[command]
pub fn start_screen_share(
    app_handle: tauri::AppHandle,
    fps: Option<u64>,
    resolution: Option<u64>,
) -> Result<(), String> {
    // CAS 防止并发启动导致多个截屏线程
    if SHARING.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed).is_err() {
        return Err("已经在演示中".to_string());
    }
    
    let cfg = crate::config::load_config();
    let target_fps = fps.unwrap_or(cfg.screen_fps).clamp(5, 30);
    let target_res = resolution.unwrap_or(cfg.screen_resolution);
    TARGET_FPS.store(target_fps, Ordering::Relaxed);
    TARGET_RES.store(target_res, Ordering::Relaxed);
    
    thread::spawn(move || {
        // 确保无论正常退出还是 panic，SHARING 状态都被重置
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // 预先获取显示器列表，避免每次循环重新查询
            let monitor = match xcap::Monitor::all().ok().and_then(|m| m.into_iter().next()) {
                Some(m) => m,
                None => {
                    eprintln!("[screen_share] 无可用显示器");
                    return;
                }
            };
            let frame_interval = Duration::from_millis(1000 / target_fps);
            while SHARING.load(Ordering::Relaxed) {
                let start = Instant::now();
                match capture_and_broadcast(&app_handle, &monitor, &cfg.device_id) {
                    Ok(_) => {}
                    Err(e) => eprintln!("截屏失败: {:?}", e),
                }
                let elapsed = start.elapsed();
                if elapsed < frame_interval {
                    thread::sleep(frame_interval - elapsed);
                }
            }
        }));
        SHARING.store(false, Ordering::Relaxed);
        if let Err(e) = result {
            eprintln!("[screen_share] capture loop panicked: {:?}", e);
        }
    });
    
    Ok(())
}

#[command]
pub fn stop_screen_share() -> Result<(), String> {
    SHARING.store(false, Ordering::Relaxed);
    Ok(())
}

fn capture_and_broadcast(app_handle: &tauri::AppHandle, monitor: &xcap::Monitor, my_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let image = monitor.capture_image()?;
    
    let target_res = TARGET_RES.load(Ordering::Relaxed) as u32;
    let (target_w, target_h) = match target_res {
        450 => (800, 450),
        540 => (960, 540),
        _ => (1280, 720),
    };
    
    let scaled = DynamicImage::ImageRgba8(image).resize(target_w, target_h, FilterType::Triangle);
    
    // JPEG 编码器不支持 RGBA8，必须先转为 RGB8
    let rgb_image = scaled.to_rgb8();
    
    // 高分辨率用较高质量，低分辨率用较低质量；若帧过大则自动降级
    let mut quality = if target_w >= 1280 { 30 } else { 20 };
    let mut buf = Vec::new();
    loop {
        buf.clear();
        let encoder = JpegEncoder::new_with_quality(&mut buf, quality);
        encoder.write_image(
            rgb_image.as_raw(),
            rgb_image.width(),
            rgb_image.height(),
            image::ExtendedColorType::Rgb8,
        )?;
        // 限制单帧 base64 不超过 1MB，避免 IPC 阻塞
        if buf.len() <= 768 * 1024 || quality <= 10 {
            break;
        }
        quality -= 5;
    }
    
    let base64 = base64::engine::general_purpose::STANDARD.encode(&buf);
    let frame = format!("data:image/jpeg;base64,{}", base64);
    
    let msg = NetworkMessage::ScreenShare {
        frame_base64: frame.clone(),
    };
    
    if let Some(pool) = app_handle.try_state::<ConnectionPool>() {
        client::broadcast_message(&pool, &msg);
    }
    
    // 同时推送给本地前端（演示者自己也能预览）
    let _ = app_handle.emit("screen-share", serde_json::json!({
        "peer_id": my_id,
        "frame": frame,
    }));
    
    Ok(())
}
