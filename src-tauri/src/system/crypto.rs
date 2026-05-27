use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use aes_gcm::aead::rand_core::RngCore;
use base64::{engine::general_purpose::STANDARD, Engine as _};

const SALT: &[u8] = b"shimmen-lan-suite-salt-2024";
const NONCE_SIZE: usize = 12;

fn derive_key(device_id: &str) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(device_id.as_bytes());
    hasher.update(SALT);
    let hash = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(hash.as_bytes());
    key
}

pub fn encrypt_password(plain: &str, device_id: &str) -> Result<String, String> {
    let key = derive_key(device_id);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("{:?}", e))?;
    // 安全：每次加密生成随机 nonce，避免固定 nonce 导致密钥流重用
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plain.as_bytes()).map_err(|e| format!("{:?}", e))?;
    // 格式：nonce (12 bytes) || ciphertext
    let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(combined))
}

pub fn decrypt_password(cipher_b64: &str, device_id: &str) -> Result<String, String> {
    let key = derive_key(device_id);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("{:?}", e))?;
    let combined = STANDARD.decode(cipher_b64).map_err(|e| e.to_string())?;
    
    // 尝试新格式：nonce (12 bytes) || ciphertext
    if combined.len() >= NONCE_SIZE {
        let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        if let Ok(plaintext) = cipher.decrypt(nonce, ciphertext) {
            return Ok(String::from_utf8(plaintext).map_err(|e| e.to_string())?);
        }
    }
    
    // 向后兼容：尝试旧格式（固定 nonce + 纯 ciphertext）
    let old_nonce = Nonce::from_slice(b"shimmen-12!!");
    let plaintext = cipher.decrypt(old_nonce, combined.as_ref())
        .map_err(|e| format!("{:?}", e))?;
    Ok(String::from_utf8(plaintext).map_err(|e| e.to_string())?)
}
