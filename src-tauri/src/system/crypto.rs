use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};

const SALT: &[u8] = b"shimmen-lan-suite-salt-2024";

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
    let nonce = Nonce::from_slice(b"shimmen-12!!");
    let ciphertext = cipher.encrypt(nonce, plain.as_bytes()).map_err(|e| format!("{:?}", e))?;
    Ok(STANDARD.encode(ciphertext))
}

pub fn decrypt_password(cipher_b64: &str, device_id: &str) -> Result<String, String> {
    let key = derive_key(device_id);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("{:?}", e))?;
    let nonce = Nonce::from_slice(b"shimmen-12!!");
    let ciphertext = STANDARD.decode(cipher_b64).map_err(|e| e.to_string())?;
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|e| format!("{:?}", e))?;
    Ok(String::from_utf8(plaintext).map_err(|e| e.to_string())?)
}
