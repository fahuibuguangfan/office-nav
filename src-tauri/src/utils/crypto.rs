//! SSH 密码加密存储工具
//!
//! 使用 AES-256-GCM 加密，密钥由「机器指纹 + 应用标识」派生，
//! 密文与随机 nonce 一起 base64 存入 store.json。
//! 不是高安全方案（本机用户可逆），但避免明文密码落盘。

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// 加密标识前缀（用于区分明文与密文，兼容旧明文配置）
const ENC_PREFIX: &str = "enc::";

/// 从机器特征派生 32 字节密钥
fn derive_key() -> [u8; 32] {
    let mut seed = String::new();

    // 用户名 + 机器名（best effort，取不到就跳过）
    if let Ok(user) = std::env::var("USERNAME") {
        seed.push_str(&user);
    }
    seed.push('|');
    if let Ok(computer) = std::env::var("COMPUTERNAME") {
        seed.push_str(&computer);
    }
    seed.push('|');
    seed.push_str(env!("CARGO_PKG_NAME"));

    let mut key = [0u8; 32];
    key.copy_from_slice(&Sha256::digest(seed.as_bytes()));
    key
}

/// 加密密码：返回 "enc::base64(nonce+密文)"
pub fn encrypt_password(plain: &str) -> Result<String, String> {
    if plain.is_empty() || plain.starts_with(ENC_PREFIX) {
        return Ok(plain.to_string());
    }

    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    // 96-bit nonce，随机生成
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plain.as_bytes())
        .map_err(|e| e.to_string())?;

    // nonce(12) + ciphertext 拼接后 base64
    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(format!("{}{}", ENC_PREFIX, B64.encode(combined)))
}

/// 解密密码：输入 "enc::base64(...)" 返回明文；普通明文原样返回（兼容旧配置）
pub fn decrypt_password(stored: &str) -> Result<String, String> {
    if !stored.starts_with(ENC_PREFIX) {
        // 旧版明文密码，原样返回
        return Ok(stored.to_string());
    }

    let combined = B64
        .decode(stored.trim_start_matches(ENC_PREFIX))
        .map_err(|e| format!("密码密文格式错误: {}", e))?;

    if combined.len() <= 12 {
        return Err("密码密文长度异常".to_string());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    let plain = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("密码解密失败（可能是在其他机器上保存的）: {}", e))?;

    String::from_utf8(plain).map_err(|e| format!("密码解码失败: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let plain = "my-secret-pwd!@#中文";
        let enc = encrypt_password(plain).unwrap();
        assert!(enc.starts_with(ENC_PREFIX));
        assert_ne!(enc, plain);
        let dec = decrypt_password(&enc).unwrap();
        assert_eq!(dec, plain);
    }

    #[test]
    fn test_plaintext_passthrough() {
        // 旧版明文原样返回
        assert_eq!(decrypt_password("plain-old").unwrap(), "plain-old");
        // 空串往返
        let enc = encrypt_password("").unwrap();
        assert_eq!(enc, "");
    }
}
