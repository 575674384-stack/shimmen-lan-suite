#[cfg(windows)]
pub fn setup_autostart() -> Result<(), Box<dyn std::error::Error>> {
    use winreg::enums::*;
    use winreg::RegKey;
    
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let (key, _) = hkcu.create_subkey(path)?;
    
    let app_path = std::env::current_exe()?.to_string_lossy().to_string();
    key.set_value("ShimmenLanSuite", &format!("\"{}\" --minimized", app_path))?;
    Ok(())
}

#[cfg(windows)]
pub fn remove_autostart() -> Result<(), Box<dyn std::error::Error>> {
    use winreg::enums::*;
    use winreg::RegKey;
    
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let key = hkcu.open_subkey_with_flags(path, KEY_WRITE)?;
    key.delete_value("ShimmenLanSuite")?;
    Ok(())
}

#[cfg(not(windows))]
pub fn setup_autostart() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[cfg(not(windows))]
pub fn remove_autostart() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
