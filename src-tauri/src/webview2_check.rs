use winreg::enums::*;
use winreg::RegKey;

const WEBVIEW2_GUID: &str = "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

pub fn is_webview2_installed() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let paths = [
        format!(r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{}", WEBVIEW2_GUID),
        format!(r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{}", WEBVIEW2_GUID),
    ];

    for base in [&hklm, &hkcu] {
        for path in &paths {
            if let Ok(key) = base.open_subkey_with_flags(path, KEY_READ) {
                if let Ok(version) = key.get_value::<String, _>("pv") {
                    if !version.is_empty() {
                        return true;
                    }
                }
            }
        }
    }

    false
}

pub fn ensure_webview2() {
    if is_webview2_installed() {
        return;
    }

    let choice = rfd::MessageDialog::new()
        .set_title("需要安装 WebView2 运行时")
        .set_description(
            "本应用需要 Microsoft WebView2 运行时才可启动，检测到当前系统未安装。\n\n\
             点击「自动安装」将下载并静默安装（需要联网，约30秒）。\n\
             点击「退出」将关闭应用，您可以手动从以下地址下载：\n\
             https://developer.microsoft.com/microsoft-edge/webview2"
        )
        .set_buttons(rfd::MessageButtons::OkCancelCustom(
            "自动安装".to_string(),
            "退出".to_string(),
        ))
        .show();

    match choice {
        rfd::MessageDialogResult::Custom(label) if label == "自动安装" => {
            match download_and_install() {
                Ok(_) => {
                    rfd::MessageDialog::new()
                        .set_title("安装程序已启动")
                        .set_description("WebView2 安装程序正在运行，安装完成后请重新启动本应用。")
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                }
                Err(e) => {
                    let open_browser = rfd::MessageDialog::new()
                        .set_title("自动安装失败")
                        .set_description(&format!(
                            "错误: {}\n\n是否打开浏览器手动下载？",
                            e
                        ))
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .show();

                    if open_browser == rfd::MessageDialogResult::Yes {
                        let _ = std::process::Command::new("cmd")
                            .args(["/c", "start", "", "https://developer.microsoft.com/microsoft-edge/webview2"])
                            .spawn();
                    }
                }
            }
        }
        _ => {}
    }

    std::process::exit(0);
}

fn download_and_install() -> Result<(), String> {
    let temp_dir = std::env::temp_dir();
    let installer = temp_dir.join("MicrosoftEdgeWebview2Setup.exe");

    let resp = ureq::get("https://go.microsoft.com/fwlink/p/?LinkId=2124703")
        .timeout(std::time::Duration::from_secs(60))
        .call()
        .map_err(|e| e.to_string())?;

    let mut file = std::fs::File::create(&installer)
        .map_err(|e| e.to_string())?;

    let mut reader = resp.into_reader();
    std::io::copy(&mut reader, &mut file)
        .map_err(|e| e.to_string())?;

    std::process::Command::new(&installer)
        .arg("/silent")
        .arg("/install")
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok(())
}
