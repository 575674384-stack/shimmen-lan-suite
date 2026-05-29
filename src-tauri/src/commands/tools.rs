use std::path::Path;
use tauri::command;
use winreg::enums::*;
use winreg::RegKey;

// ============== 1. 本机环境检测 ==============

#[derive(serde::Serialize)]
pub struct SystemInfo {
    pub os_version: String,
    pub cpu: String,
    pub memory_total: String,
    pub memory_free: String,
    pub memory_used_percent: u8,
    pub disks: Vec<DiskInfo>,
    pub ip_address: String,
    pub hostname: String,
    pub username: String,
}

#[derive(serde::Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub total: String,
    pub free: String,
    pub used_percent: u8,
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}

#[command]
pub fn get_system_info() -> Result<SystemInfo, String> {
    use sysinfo::{Disks, Networks, System};

    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu = sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_else(|| "Unknown".to_string());
    let cpu = format!("{} x {}", cpu, sys.cpus().len());

    let total_mem = sys.total_memory();
    let free_mem = sys.free_memory();
    let used_percent = if total_mem > 0 {
        (((total_mem - free_mem) as f64 / total_mem as f64) * 100.0) as u8
    } else {
        0
    };

    let disks = Disks::new_with_refreshed_list()
        .iter()
        .map(|d| {
            let total = d.total_space();
            let free = d.available_space();
            let used_percent = if total > 0 {
                (((total - free) as f64 / total as f64) * 100.0) as u8
            } else {
                0
            };
            DiskInfo {
                name: d.mount_point().to_string_lossy().to_string(),
                total: format_bytes(total),
                free: format_bytes(free),
                used_percent,
            }
        })
        .collect();

    let networks = Networks::new_with_refreshed_list();
    let ip_address = networks
        .iter()
        .find(|(_, n)| !n.ip_networks().is_empty())
        .and_then(|(_, n)| n.ip_networks().first())
        .map(|ip| ip.addr.to_string())
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
    let username = whoami::username();

    // Windows version
    let os_version = std::process::Command::new("cmd")
        .args(["/c", "ver"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "Windows".to_string())
        .trim()
        .to_string();

    Ok(SystemInfo {
        os_version,
        cpu,
        memory_total: format_bytes(total_mem),
        memory_free: format_bytes(free_mem),
        memory_used_percent: used_percent,
        disks,
        ip_address,
        hostname,
        username,
    })
}

// ============== 2. DNS优选设置 ==============

#[derive(serde::Serialize, serde::Deserialize)]
pub struct NetworkInterface {
    #[serde(alias = "Description", alias = "description")]
    pub name: String,
    #[serde(alias = "IPAddress", alias = "ipAddress")]
    pub ip: String,
    #[serde(alias = "DNSServerSearchOrder", alias = "dnsServerSearchOrder")]
    pub dns_servers: Vec<String>,
}

#[command]
pub fn get_network_interfaces() -> Result<Vec<NetworkInterface>, String> {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile", "-ExecutionPolicy", "Bypass",
            "-Command",
            r#"Get-WmiObject Win32_NetworkAdapterConfiguration | Where-Object {$_.IPEnabled -eq $true} | Select-Object Description, IPAddress, DNSServerSearchOrder | ConvertTo-Json -Depth 3"#,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    let text = String::from_utf8_lossy(&output.stdout);
    let text = text.trim();
    if text.is_empty() || text == "null" {
        return Ok(Vec::new());
    }

    // Parse JSON output
    let interfaces: Vec<NetworkInterface> = if text.starts_with('[') {
        serde_json::from_str(text).map_err(|e| e.to_string())?
    } else {
        let single: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        vec![parse_adapter(&single)?]
    };

    Ok(interfaces)
}

fn parse_adapter(val: &serde_json::Value) -> Result<NetworkInterface, String> {
    let name = val["Description"]
        .as_str()
        .or_else(|| val["description"].as_str())
        .unwrap_or("Unknown")
        .to_string();

    let ip = val["IPAddress"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let dns_servers: Vec<String> = val["DNSServerSearchOrder"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Ok(NetworkInterface { name, ip, dns_servers })
}

#[command]
pub fn set_dns(interface: String, primary: String, secondary: Option<String>) -> Result<(), String> {
    let output = std::process::Command::new("netsh")
        .args([
            "interface", "ip", "set", "dns",
            &interface, "static", &primary,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("设置主DNS失败: {}", err));
    }

    if let Some(sec) = secondary {
        let output = std::process::Command::new("netsh")
            .args([
                "interface", "ip", "add", "dns",
                &interface, &sec, "index=2",
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("设置备用DNS失败: {}", err));
        }
    }

    Ok(())
}

// ============== 3. 一键激活Win10 ==============

const KMS_SERVERS: &[&str] = &["kms.03k.org", "kms.library.hk"];

#[derive(serde::Serialize)]
pub struct ActivateResult {
    pub success: bool,
    pub output: String,
}

#[command]
pub fn activate_windows(edition: String) -> Result<ActivateResult, String> {
    let key = match edition.as_str() {
        "home" => "TX9XD-98N7V-6WMQ6-BX7FG-H8Q99",
        "professional" => "W269N-WFGWX-YVC9B-4J6C9-T83GX",
        "enterprise" => "NPPR9-FWDCX-D2C8J-H872K-2YT43",
        _ => return Err("不支持的版本".to_string()),
    };

    let kms_server = KMS_SERVERS[0];
    let slmgr = r"C:\Windows\System32\slmgr.vbs";

    let mut outputs = Vec::new();

    // Use powershell to run cscript with UTF-8 encoding
    let run_slmgr = |args: &[&str]| -> Result<String, String> {
        let ps_cmd = format!(
            r#"[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; $out = cscript //nologo "{}" {}; Write-Output $out"#,
            slmgr,
            args.join(" ")
        );
        let out = std::process::Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_cmd])
            .output()
            .map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        let err = String::from_utf8_lossy(&out.stderr).to_string();
        if !err.trim().is_empty() && text.trim().is_empty() {
            return Ok(err);
        }
        Ok(text)
    };

    // Install key
    match run_slmgr(&["/ipk", key]) {
        Ok(o) => outputs.push(format!("安装密钥: {}", o)),
        Err(e) => outputs.push(format!("安装密钥失败: {}", e)),
    }

    // Set KMS server
    match run_slmgr(&["/skms", kms_server]) {
        Ok(o) => outputs.push(format!("设置KMS: {}", o)),
        Err(e) => outputs.push(format!("设置KMS失败: {}", e)),
    }

    // Activate
    match run_slmgr(&["/ato"]) {
        Ok(o) => {
            let success = o.contains("成功") || o.contains("successfully") || o.contains("activated");
            outputs.push(format!("激活: {}", o));
            Ok(ActivateResult {
                success,
                output: outputs.join("\n"),
            })
        }
        Err(e) => {
            outputs.push(format!("激活失败: {}", e));
            Ok(ActivateResult {
                success: false,
                output: outputs.join("\n"),
            })
        }
    }
}

// ============== 4. PowerShell7安装 + UTF8设置 ==============

#[command]
pub fn check_powershell7() -> Result<bool, String> {
    Ok(Path::new(r"C:\Program Files\PowerShell\7\pwsh.exe").exists())
}

#[command]
pub fn install_powershell7() -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let installer = temp_dir.join("PowerShell-7.4.6-win-x64.msi");

    // Download
    let resp = ureq::get("https://github.com/PowerShell/PowerShell/releases/download/v7.4.6/PowerShell-7.4.6-win-x64.msi")
        .timeout(std::time::Duration::from_secs(120))
        .call()
        .map_err(|e| e.to_string())?;

    let mut file = std::fs::File::create(&installer)
        .map_err(|e| e.to_string())?;
    let mut reader = resp.into_reader();
    std::io::copy(&mut reader, &mut file)
        .map_err(|e| e.to_string())?;

    // Install silently
    let output = std::process::Command::new("msiexec")
        .args([
            "/i", &installer.to_string_lossy(),
            "/qn", "/norestart",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok("PowerShell 7 安装成功".to_string())
    } else {
        Err(format!("安装失败: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

#[command]
pub fn check_utf8() -> Result<bool, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey_with_flags(
        r"SYSTEM\CurrentControlSet\Control\Nls\CodePage",
        KEY_READ,
    ).map_err(|e| e.to_string())?;

    let acp: String = key.get_value("ACP").unwrap_or_default();
    Ok(acp == "65001")
}

#[command]
pub fn set_utf8(enabled: bool) -> Result<(), String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey_with_flags(
        r"SYSTEM\CurrentControlSet\Control\Nls\CodePage",
        KEY_WRITE,
    ).map_err(|e| e.to_string())?;

    let value = if enabled { "65001" } else { "936" };
    key.set_value("ACP", &value).map_err(|e| e.to_string())?;
    key.set_value("OEMCP", &value).map_err(|e| e.to_string())?;
    key.set_value("MACCP", &value).map_err(|e| e.to_string())?;

    Ok(())
}

// ============== 5. 系统优化 ==============

#[derive(serde::Serialize)]
pub struct OptimizeResult {
    pub task: String,
    pub success: bool,
    pub output: String,
}

#[command]
pub fn run_optimize(tasks: Vec<String>) -> Result<Vec<OptimizeResult>, String> {
    let mut results = Vec::new();

    for task in tasks {
        let (success, output) = match task.as_str() {
            "clean_temp" => {
                let temp = std::env::temp_dir();
                let mut count = 0;
                let mut errors = 0;
                if let Ok(entries) = std::fs::read_dir(&temp) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if std::fs::remove_file(&path).is_ok() {
                                count += 1;
                            } else {
                                errors += 1;
                            }
                        } else if path.is_dir() {
                            if std::fs::remove_dir_all(&path).is_ok() {
                                count += 1;
                            } else {
                                errors += 1;
                            }
                        }
                    }
                }
                (errors < count + 1, format!("清理了 {} 个临时文件/目录，{} 个失败", count, errors))
            }
            "clean_recycle" => {
                let out = std::process::Command::new("cmd")
                    .args(["/c", "rd /s /q C:\\$Recycle.Bin 2>nul || echo 部分文件无法删除"])
                    .output();
                match out {
                    Ok(o) => (o.status.success(), "回收站已清空".to_string()),
                    Err(e) => (false, e.to_string()),
                }
            }
            "disable_hibernate" => {
                let out = std::process::Command::new("powercfg")
                    .args(["/hibernate", "off"])
                    .output();
                match out {
                    Ok(o) => (o.status.success(), String::from_utf8_lossy(&o.stdout).to_string()),
                    Err(e) => (false, e.to_string()),
                }
            }
            "disable_visual_effects" => {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                match hkcu.create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects") {
                    Ok((key, _)) => {
                        let _ = key.set_value("VisualFXSetting", &2u32);
                        (true, "已关闭视觉效果，注销后生效".to_string())
                    }
                    Err(e) => (false, e.to_string()),
                }
            }
            "optimize_services" => {
                let services = ["SysMain", "WSearch", "DiagTrack", "dmwappushservice"];
                let mut ok_count = 0;
                for svc in &services {
                    let out = std::process::Command::new("sc")
                        .args(["config", svc, "start=", "disabled"])
                        .output();
                    if let Ok(o) = out {
                        if o.status.success() {
                            ok_count += 1;
                        }
                    }
                    let _ = std::process::Command::new("sc")
                        .args(["stop", svc])
                        .output();
                }
                (ok_count > 0, format!("已优化 {} 个服务", ok_count))
            }
            "disable_telemetry" => {
                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                let mut ok = true;
                if let Ok((key, _)) = hklm.create_subkey(r"SOFTWARE\Policies\Microsoft\Windows\DataCollection") {
                    let _ = key.set_value("AllowTelemetry", &0u32);
                } else {
                    ok = false;
                }
                let _ = std::process::Command::new("sc")
                    .args(["stop", "DiagTrack"])
                    .output();
                let _ = std::process::Command::new("sc")
                    .args(["stop", "dmwappushservice"])
                    .output();
                (ok, "已禁用 Windows 遥测".to_string())
            }
            "disable_cortana" => {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                let mut ok = true;
                if let Ok((key, _)) = hkcu.create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Search") {
                    let _ = key.set_value("SearchboxTaskbarMode", &0u32);
                    let _ = key.set_value("BingSearchEnabled", &0u32);
                    let _ = key.set_value("CortanaConsent", &0u32);
                } else {
                    ok = false;
                }
                let _ = std::process::Command::new("taskkill")
                    .args(["/f", "/im", "SearchUI.exe"])
                    .output();
                (ok, "已关闭 Cortana".to_string())
            }
            "disable_onedrive" => {
                let _ = std::process::Command::new("taskkill")
                    .args(["/f", "/im", "OneDrive.exe"])
                    .output();
                let out = std::process::Command::new(r"C:\Windows\System32\OneDriveSetup.exe")
                    .args(["/uninstall"])
                    .output();
                let success = out.map(|o| o.status.success()).unwrap_or(true);
                (success, "已关闭 OneDrive".to_string())
            }
            "disable_auto_maintenance" => {
                let out = std::process::Command::new("schtasks")
                    .args(["/Change", "/TN", r"\Microsoft\Windows\TaskScheduler\MaintenanceConfig", "/DISABLE"])
                    .output();
                let mut success = out.map(|o| o.status.success()).unwrap_or(false);
                // Also disable via registry
                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                if let Ok((key, _)) = hklm.create_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\Maintenance") {
                    let _ = key.set_value("MaintenanceDisabled", &1u32);
                    success = true;
                }
                (success, "已禁用自动维护".to_string())
            }
            "disable_startup_delay" => {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                match hkcu.create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Serialize") {
                    Ok((key, _)) => {
                        let _ = key.set_value("StartupDelayInMSec", &0u32);
                        (true, "已禁用启动延迟".to_string())
                    }
                    Err(e) => (false, e.to_string()),
                }
            }
            _ => (false, "未知优化任务".to_string()),
        };

        results.push(OptimizeResult { task, success, output });
    }

    Ok(results)
}

// ============== 6. 本机文件快速查询 ==============

#[derive(serde::Serialize)]
pub struct FileSearchResult {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: String,
}

#[command]
pub fn search_files(path: String, query: String, limit: i32) -> Result<Vec<FileSearchResult>, String> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();
    let limit_usize = if limit <= 0 { 100usize } else { limit as usize };

    for entry in walkdir::WalkDir::new(&path)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if results.len() >= limit_usize {
            break;
        }

        if !entry.file_type().is_file() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        if name.to_lowercase().contains(&query_lower) {
            let metadata = entry.metadata().ok();
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            let modified = metadata
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| {
                    let secs = d.as_secs() as i64;
                    let dt = chrono::DateTime::from_timestamp(secs, 0)
                        .unwrap_or_default();
                    dt.format("%Y-%m-%d %H:%M").to_string()
                })
                .unwrap_or_default();

            results.push(FileSearchResult {
                name,
                path: entry.path().to_string_lossy().to_string(),
                size,
                modified,
            });
        }
    }

    Ok(results)
}

#[command]
pub fn open_file_location(path: String) -> Result<(), String> {
    std::process::Command::new("explorer")
        .args(["/select,", &path])
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ============== 7. 批量重命名 ==============

#[derive(serde::Serialize)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
}

#[derive(serde::Deserialize)]
pub struct RenameRule {
    pub rule_type: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub start: Option<i32>,
    pub digits: Option<i32>,
    pub new_ext: Option<String>,
}

#[derive(serde::Serialize)]
pub struct RenamePreview {
    pub old_name: String,
    pub new_name: String,
}

#[derive(serde::Serialize)]
pub struct RenameResult {
    pub old_name: String,
    pub new_name: String,
    pub success: bool,
    pub error: Option<String>,
}

#[command]
pub fn list_files_in_dir(path: String) -> Result<Vec<FileEntry>, String> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        entries.push(FileEntry { name, is_dir });
    }
    Ok(entries)
}

#[command]
pub fn preview_rename(path: String, rule: RenameRule) -> Result<Vec<RenamePreview>, String> {
    let entries = list_files_in_dir(path.clone())?;
    let mut previews = Vec::new();
    let mut counter = rule.start.unwrap_or(1);

    for entry in entries.iter().filter(|e| !e.is_dir) {
        let old_name = entry.name.clone();
        let new_name = apply_rule(&old_name, &rule, &mut counter);
        previews.push(RenamePreview { old_name, new_name });
    }

    Ok(previews)
}

#[command]
pub fn execute_rename(path: String, rule: RenameRule) -> Result<Vec<RenameResult>, String> {
    let entries = list_files_in_dir(path.clone())?;
    let mut results = Vec::new();
    let mut counter = rule.start.unwrap_or(1);

    for entry in entries.iter().filter(|e| !e.is_dir) {
        let old_name = entry.name.clone();
        let new_name = apply_rule(&old_name, &rule, &mut counter);

        let old_path = Path::new(&path).join(&old_name);
        let new_path = Path::new(&path).join(&new_name);

        let (success, error) = if old_name == new_name {
            (true, None)
        } else if new_path.exists() {
            (false, Some("目标文件已存在".to_string()))
        } else {
            match std::fs::rename(&old_path, &new_path) {
                Ok(_) => (true, None),
                Err(e) => (false, Some(e.to_string())),
            }
        };

        results.push(RenameResult { old_name, new_name, success, error });
    }

    Ok(results)
}

fn apply_rule(name: &str, rule: &RenameRule, counter: &mut i32) -> String {
    let (stem, ext) = match name.rfind('.') {
        Some(idx) => (&name[..idx], &name[idx..]),
        None => (name, ""),
    };

    match rule.rule_type.as_str() {
        "add_prefix" => {
            let prefix = rule.prefix.as_deref().unwrap_or("");
            format!("{}{}{}", prefix, stem, ext)
        }
        "add_suffix" => {
            let suffix = rule.suffix.as_deref().unwrap_or("");
            format!("{}{}{}", stem, suffix, ext)
        }
        "replace" => {
            let from = rule.from.as_deref().unwrap_or("");
            let to = rule.to.as_deref().unwrap_or("");
            name.replace(from, to)
        }
        "numbering" => {
            let digits = rule.digits.unwrap_or(3) as usize;
            let prefix = rule.prefix.as_deref().unwrap_or("");
            let num = format!("{:0width$}", *counter, width = digits);
            *counter += 1;
            format!("{}{}{}", prefix, num, ext)
        }
        "change_ext" => {
            let new_ext = rule.new_ext.as_deref().unwrap_or("");
            if new_ext.is_empty() {
                stem.to_string()
            } else {
                format!("{}.{}", stem, new_ext)
            }
        }
        _ => name.to_string(),
    }
}


// ============== 软件依赖检测 ==============

#[derive(serde::Serialize, Clone)]
pub struct SoftwareInfo {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub version: String,
    pub install_path: String,
    pub download_url: String,
    pub installer_args: String,
}

fn check_registry_key(path: &str) -> Option<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for base in [&hklm, &hkcu] {
        if let Ok(key) = base.open_subkey_with_flags(path, KEY_READ) {
            if let Ok(val) = key.get_value::<String, _>("DisplayVersion") {
                return Some(val);
            }
            if let Ok(val) = key.get_value::<String, _>("Version") {
                return Some(val);
            }
        }
    }
    None
}

fn check_file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

fn run_cmd_output(cmd: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            let trimmed = out.trim();
            if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
        })
}

#[command]
pub fn get_installed_software() -> Result<Vec<SoftwareInfo>, String> {
    let mut software = Vec::new();

    // .NET Framework
    let dotnet_ver = check_registry_key(r"SOFTWARE\Microsoft\NET Framework Setup\NDP\v4\Full")
        .or_else(|| check_registry_key(r"SOFTWARE\WOW6432Node\Microsoft\NET Framework Setup\NDP\v4\Full"));
    software.push(SoftwareInfo {
        id: "dotnet".to_string(),
        name: ".NET Framework 4.x".to_string(),
        installed: dotnet_ver.is_some(),
        version: dotnet_ver.unwrap_or_default(),
        install_path: "".to_string(),
        download_url: "https://dotnet.microsoft.com/download/dotnet-framework".to_string(),
        installer_args: "".to_string(),
    });

    // VC++ Redistributable 2015-2022
    let vc_ver = check_registry_key(r"SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64")
        .or_else(|| check_registry_key(r"SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\x64"));
    software.push(SoftwareInfo {
        id: "vcredist".to_string(),
        name: "Visual C++ Redistributable 2015-2022".to_string(),
        installed: vc_ver.is_some(),
        version: vc_ver.unwrap_or_default(),
        install_path: "".to_string(),
        download_url: "https://aka.ms/vs/17/release/vc_redist.x64.exe".to_string(),
        installer_args: "/install /quiet /norestart".to_string(),
    });

    // Java
    let java_ver = run_cmd_output("java", &["-version"])
        .or_else(|| {
            if check_file_exists(r"C:\Program Files\Java\jdk-17\bin\java.exe") { Some("JDK 17".to_string()) }
            else if check_file_exists(r"C:\Program Files\Java\jdk-21\bin\java.exe") { Some("JDK 21".to_string()) }
            else { None }
        });
    software.push(SoftwareInfo {
        id: "java".to_string(),
        name: "Java (JDK/JRE)".to_string(),
        installed: java_ver.is_some(),
        version: java_ver.unwrap_or_default(),
        install_path: "".to_string(),
        download_url: "https://adoptium.net/temurin/releases/".to_string(),
        installer_args: "".to_string(),
    });

    // Python
    let python_ver = run_cmd_output("python", &["--version"])
        .or_else(|| run_cmd_output(r"C:\Users\1\AppData\Local\Programs\Python\Python311\python.exe", &["--version"]))
        .or_else(|| {
            if check_file_exists(r"C:\Python311\python.exe") { Some("Python 3.11".to_string()) }
            else { None }
        });
    software.push(SoftwareInfo {
        id: "python".to_string(),
        name: "Python 3".to_string(),
        installed: python_ver.is_some(),
        version: python_ver.unwrap_or_default(),
        install_path: "".to_string(),
        download_url: "https://www.python.org/ftp/python/3.11.9/python-3.11.9-amd64.exe".to_string(),
        installer_args: "/quiet InstallAllUsers=1 PrependPath=1".to_string(),
    });

    // Node.js
    let node_ver = run_cmd_output("node", &["--version"]);
    software.push(SoftwareInfo {
        id: "nodejs".to_string(),
        name: "Node.js".to_string(),
        installed: node_ver.is_some(),
        version: node_ver.unwrap_or_default(),
        install_path: "".to_string(),
        download_url: "https://nodejs.org/dist/v20.14.0/node-v20.14.0-x64.msi".to_string(),
        installer_args: "/qn".to_string(),
    });

    // Git
    let git_ver = run_cmd_output("git", &["--version"]);
    software.push(SoftwareInfo {
        id: "git".to_string(),
        name: "Git".to_string(),
        installed: git_ver.is_some(),
        version: git_ver.unwrap_or_default(),
        install_path: "".to_string(),
        download_url: "https://github.com/git-for-windows/git/releases/download/v2.45.2.windows.1/Git-2.45.2-64-bit.exe".to_string(),
        installer_args: "/VERYSILENT /NORESTART".to_string(),
    });

    // Chrome
    let chrome_installed = check_file_exists(r"C:\Program Files\Google\Chrome\Application\chrome.exe")
        || check_file_exists(r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe");
    software.push(SoftwareInfo {
        id: "chrome".to_string(),
        name: "Google Chrome".to_string(),
        installed: chrome_installed,
        version: "".to_string(),
        install_path: "".to_string(),
        download_url: "https://dl.google.com/chrome/install/GoogleChromeStandaloneEnterprise64.msi".to_string(),
        installer_args: "/qn".to_string(),
    });

    // 7-Zip
    let seven_zip = check_file_exists(r"C:\Program Files\7-Zip\7z.exe")
        || check_file_exists(r"C:\Program Files (x86)\7-Zip\7z.exe");
    software.push(SoftwareInfo {
        id: "7zip".to_string(),
        name: "7-Zip".to_string(),
        installed: seven_zip,
        version: "".to_string(),
        install_path: "".to_string(),
        download_url: "https://www.7-zip.org/a/7z2407-x64.exe".to_string(),
        installer_args: "/S".to_string(),
    });

    // WinRAR
    let winrar = check_file_exists(r"C:\Program Files\WinRAR\WinRAR.exe")
        || check_file_exists(r"C:\Program Files (x86)\WinRAR\WinRAR.exe");
    software.push(SoftwareInfo {
        id: "winrar".to_string(),
        name: "WinRAR".to_string(),
        installed: winrar,
        version: "".to_string(),
        install_path: "".to_string(),
        download_url: "https://www.win-rar.com/fileadmin/winrar-versions/winrar/winrar-x64-701sc.exe".to_string(),
        installer_args: "/S".to_string(),
    });

    // Office / WPS
    let office = check_registry_key(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{90160000-008C-0000-0000-0000000FF1CE}")
        .is_some()
        || check_file_exists(r"C:\Program Files\Microsoft Office\root\Office16\WINWORD.EXE")
        || check_file_exists(r"C:\Program Files (x86)\Kingsoft\WPS Office\wps.exe");
    software.push(SoftwareInfo {
        id: "office".to_string(),
        name: "Office / WPS".to_string(),
        installed: office,
        version: "".to_string(),
        install_path: "".to_string(),
        download_url: "https://platform.wps.cn/".to_string(),
        installer_args: "".to_string(),
    });

    Ok(software)
}

#[command]
pub fn install_software(download_url: String, installer_args: String) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let file_name = download_url
        .split('?').next().unwrap_or(&download_url)
        .split('/').last().unwrap_or("installer.exe");
    let installer_path = temp_dir.join(file_name);

    let resp = ureq::get(&download_url)
        .timeout(std::time::Duration::from_secs(120))
        .call()
        .map_err(|e| format!("下载失败: {}", e))?;

    let mut file = std::fs::File::create(&installer_path)
        .map_err(|e| format!("创建文件失败: {}", e))?;
    let mut reader = resp.into_reader();
    let bytes = std::io::copy(&mut reader, &mut file)
        .map_err(|e| format!("写入失败: {}", e))?;
    if bytes > 500 * 1024 * 1024 {
        return Err("下载文件超过 500MB，拒绝安装".to_string());
    }

    // Run installer
    let ext = installer_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let status = if ext == "msi" {
        std::process::Command::new("msiexec")
            .args(["/i", &installer_path.to_string_lossy(), &installer_args])
            .spawn()
            .map_err(|e| format!("启动安装失败: {}", e))?
    } else {
        let mut cmd = std::process::Command::new(&installer_path);
        for arg in installer_args.split_whitespace() {
            cmd.arg(arg);
        }
        cmd.spawn().map_err(|e| format!("启动安装失败: {}", e))?
    };

    Ok(format!("安装程序已启动 (PID: {})", status.id()))
}


// ============== 打印机管理 ==============

#[derive(serde::Serialize)]
pub struct PrinterInfo {
    pub name: String,
    pub status: String,
    pub is_default: bool,
    pub port: String,
    pub driver: String,
}

#[derive(serde::Serialize)]
pub struct PrintJob {
    pub id: u32,
    pub document: String,
    pub status: String,
    pub size: String,
}

#[command]
pub fn get_printers() -> Result<Vec<PrinterInfo>, String> {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile", "-ExecutionPolicy", "Bypass",
            "-Command",
            r#"Get-Printer | Select-Object Name, PrinterStatus, IsDefault, PortName, DriverName | ConvertTo-Json -Depth 3"#,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    let text = String::from_utf8_lossy(&output.stdout);
    let text = text.trim();
    if text.is_empty() || text == "null" {
        return Ok(Vec::new());
    }

    let printers: Vec<serde_json::Value> = if text.starts_with('[') {
        serde_json::from_str(text).map_err(|e| e.to_string())?
    } else {
        vec![serde_json::from_str(text).map_err(|e| e.to_string())?]
    };

    let result = printers.into_iter().map(|p| {
        let status_num = p["PrinterStatus"].as_u64().unwrap_or(0);
        let status = match status_num {
            1 => "其他".to_string(),
            2 => "未知".to_string(),
            3 => "空闲".to_string(),
            4 => "打印中".to_string(),
            5 => "预热中".to_string(),
            6 => "停止打印".to_string(),
            7 => "脱机".to_string(),
            _ => "未知".to_string(),
        };

        PrinterInfo {
            name: p["Name"].as_str().unwrap_or("Unknown").to_string(),
            status,
            is_default: p["IsDefault"].as_bool().unwrap_or(false),
            port: p["PortName"].as_str().unwrap_or("").to_string(),
            driver: p["DriverName"].as_str().unwrap_or("").to_string(),
        }
    }).collect();

    Ok(result)
}

#[command]
pub fn get_print_jobs(printer_name: String) -> Result<Vec<PrintJob>, String> {
    let ps_cmd = format!(
        "Get-PrintJob -PrinterName '{}' | Select-Object Id, DocumentName, JobStatus, TotalPages | ConvertTo-Json -Depth 3",
        printer_name.replace("'", "''").replace('\"', "\x60")
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_cmd])
        .output()
        .map_err(|e| e.to_string())?;

    let text = String::from_utf8_lossy(&output.stdout);
    let text = text.trim();
    if text.is_empty() || text == "null" {
        return Ok(Vec::new());
    }

    let jobs: Vec<serde_json::Value> = if text.starts_with('[') {
        serde_json::from_str(text).map_err(|e| e.to_string())?
    } else {
        vec![serde_json::from_str(text).map_err(|e| e.to_string())?]
    };

    let result = jobs.into_iter().map(|j| {
        PrintJob {
            id: j["Id"].as_u64().unwrap_or(0) as u32,
            document: j["DocumentName"].as_str().unwrap_or("").to_string(),
            status: j["JobStatus"].as_str().unwrap_or("未知").to_string(),
            size: format!("{} 页", j["TotalPages"].as_u64().unwrap_or(0)),
        }
    }).collect();

    Ok(result)
}

#[command]
pub fn clear_print_queue(printer_name: String) -> Result<(), String> {
    let ps_cmd = format!(
        "Get-PrintJob -PrinterName '{}' | Remove-PrintJob",
        printer_name.replace("'", "''").replace('\"', "\x60")
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_cmd])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        if !err.trim().is_empty() {
            return Err(err.to_string());
        }
    }

    Ok(())
}

// ============== 本机网络信息 ==============

#[derive(serde::Serialize)]
pub struct NetworkDetail {
    pub adapter_name: String,
    pub mac: String,
    pub ip: String,
    pub subnet_mask: String,
    pub gateway: String,
    pub dns_servers: Vec<String>,
    pub dhcp_enabled: bool,
}

#[derive(serde::Serialize)]
pub struct NetworkStatus {
    pub internet_connected: bool,
    pub lan_connected: bool,
    pub public_ip: String,
}

#[command]
pub fn get_network_details() -> Result<Vec<NetworkDetail>, String> {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile", "-ExecutionPolicy", "Bypass",
            "-Command",
            r#"Get-WmiObject Win32_NetworkAdapterConfiguration | Where-Object {$_.IPEnabled -eq $true} | Select-Object Description, MACAddress, IPAddress, IPSubnet, DefaultIPGateway, DNSServerSearchOrder, DHCPEnabled | ConvertTo-Json -Depth 3"#,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    let text = String::from_utf8_lossy(&output.stdout);
    let text = text.trim();
    if text.is_empty() || text == "null" {
        return Ok(Vec::new());
    }

    let adapters: Vec<serde_json::Value> = if text.starts_with('[') {
        serde_json::from_str(text).map_err(|e| e.to_string())?
    } else {
        vec![serde_json::from_str(text).map_err(|e| e.to_string())?]
    };

    let result = adapters.into_iter().map(|a| {
        let ip = a["IPAddress"].as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let subnet = a["IPSubnet"].as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let gateway = a["DefaultIPGateway"].as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let dns: Vec<String> = a["DNSServerSearchOrder"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        NetworkDetail {
            adapter_name: a["Description"].as_str().unwrap_or("Unknown").to_string(),
            mac: a["MACAddress"].as_str().unwrap_or("").to_string(),
            ip,
            subnet_mask: subnet,
            gateway,
            dns_servers: dns,
            dhcp_enabled: a["DHCPEnabled"].as_bool().unwrap_or(false),
        }
    }).collect();

    Ok(result)
}

fn get_default_gateway() -> Option<String> {
    // 通过 PowerShell 获取默认网关
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command",
            "(Get-NetRoute -DestinationPrefix '0.0.0.0/0' | Select-Object -First 1).NextHop"
        ])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() || text == "null" {
        None
    } else {
        Some(text)
    }
}

#[command]
pub fn get_network_status() -> Result<NetworkStatus, String> {
    // Check internet connectivity by pinging a public DNS
    let internet = std::process::Command::new("ping")
        .args(["-n", "1", "-w", "2000", "223.5.5.5"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check LAN by pinging gateway
    // 获取默认网关作为 LAN 检测目标
    let gateway = get_default_gateway().unwrap_or_else(|| "192.168.1.1".to_string());
    let lan = if internet {
        true
    } else {
        std::process::Command::new("ping")
            .args(["-n", "1", "-w", "1000", &gateway])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    };

    // Try to get public IP
    let public_ip = ureq::get("https://api.ip.sb/ip")
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .ok()
        .and_then(|r| {
            let mut reader = r.into_reader();
            let mut buf = String::new();
            let _ = std::io::Read::read_to_string(&mut reader, &mut buf);
            Some(buf.trim().to_string())
        })
        .unwrap_or_else(|| "获取失败".to_string());

    Ok(NetworkStatus {
        internet_connected: internet,
        lan_connected: lan,
        public_ip,
    })
}
