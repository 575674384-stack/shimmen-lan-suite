use tauri::command;
use tauri::Manager;
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
    if SHARING.load(Ordering::Relaxed) {
        return Err("已经在演示中".to_string());
    }
    
    let cfg = crate::config::load_config();
    let target_fps = fps.unwrap_or(cfg.screen_fps).clamp(5, 30);
    let target_res = resolution.unwrap_or(cfg.screen_resolution);
    TARGET_FPS.store(target_fps, Ordering::Relaxed);
    TARGET_RES.store(target_res, Ordering::Relaxed);
    SHARING.store(true, Ordering::Relaxed);
    
    thread::spawn(move || {
        let frame_interval = Duration::from_millis(1000 / target_fps);
        while SHARING.load(Ordering::Relaxed) {
            let start = Instant::now();
            match capture_and_broadcast(&app_handle) {
                Ok(_) => {}
                Err(e) => eprintln!("截屏失败: {:?}", e),
            }
            let elapsed = start.elapsed();
            if elapsed < frame_interval {
                thread::sleep(frame_interval - elapsed);
            }
        }
    });
    
    Ok(())
}

#[command]
pub fn stop_screen_share() -> Result<(), String> {
    SHARING.store(false, Ordering::Relaxed);
    Ok(())
}

fn capture_and_broadcast(app_handle: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let monitors = xcap::Monitor::all()?;
    let monitor = monitors.first().ok_or("无屏幕")?;
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
    
    // 高分辨率用较高质量，低分辨率用较低质量
    let quality = if target_w >= 1280 { 40 } else { 30 };
    let mut buf = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut buf, quality);
    encoder.write_image(
        rgb_image.as_raw(),
        rgb_image.width(),
        rgb_image.height(),
        image::ExtendedColorType::Rgb8,
    )?;
    
    let base64 = base64::engine::general_purpose::STANDARD.encode(&buf);
    
    let msg = NetworkMessage::ScreenShare {
        frame_base64: format!("data:image/jpeg;base64,{}", base64),
    };
    
    if let Some(pool) = app_handle.try_state::<ConnectionPool>() {
        client::broadcast_message(&pool, &msg);
    }
    
    Ok(())
}
