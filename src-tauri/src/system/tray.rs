use tauri::{
    tray::TrayIconBuilder,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    Manager, AppHandle, Wry, image::Image
};

pub fn setup_tray(app: &AppHandle<Wry>) -> Result<(), Box<dyn std::error::Error>> {
    // 创建菜单项
    let show_i = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(app, &[&show_i, &separator, &quit_i])?;

    let icon = match app.default_window_icon() {
        Some(icon) => icon.clone(),
        None => Image::from_bytes(&[0u8; 1])?,
    };

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip("水门内网协同")
        .menu(&menu)
        .on_menu_event(|app, event| {
            match event.id.0.as_str() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click { button_state, .. } = event {
                if button_state == tauri::tray::MouseButtonState::Up {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(true) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
