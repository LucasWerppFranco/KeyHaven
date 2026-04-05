use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, Params, Version};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use anyhow::{bail, Result};

type HmacSha256 = Hmac<Sha256>;

/// Derives an AES-256 key from the master password using Argon2id.
/// Parameters (m_cost, t_cost, p_cost) are read from vault_meta
/// so we can increase them in future versions without invalidating old vaults.
pub fn derive_key(
    password: &str,
    salt: &[u8],
    m_cost: u32, // KB de memória
    t_cost: u32, // iterações
    p_cost: u32, // paralelismo
) -> Result<Vec<u8>> {
    let params = Params::new(m_cost, t_cost, p_cost, Some(32))
        .map_err(|e| anyhow::anyhow!("Invalid Argon2 params: {e}"))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

    let mut key = vec![0u8; 32]; // AES-256 → 32 byte key
    argon2.hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow::anyhow!("Argon2 failed: {e}"))?;

    Ok(key)
}

/// Computes HMAC-SHA256 of "vault-v1-ok" with the derived key.
/// Stored in vault_meta for password verification without decrypting entries.
pub fn compute_hmac(key: &[u8]) -> Result<Vec<u8>> {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key)
        .map_err(|e| anyhow::anyhow!("HMAC initialization failed: {e}"))?;
    mac.update(b"vault-v1-ok");
    Ok(mac.finalize().into_bytes().to_vec())
}

/// Verifies if the HMAC matches the derived key.
/// Uses constant-time comparison to prevent timing attacks.
pub fn verify_hmac(key: &[u8], stored_hmac: &[u8]) -> Result<()> {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key)
        .map_err(|e| anyhow::anyhow!("HMAC initialization failed: {e}"))?;
    mac.update(b"vault-v1-ok");
    mac.verify_slice(stored_hmac)
        .map_err(|_| anyhow::anyhow!("Incorrect master password"))?;
    Ok(())
}

/// Encrypts a field with AES-256-GCM.
/// Blob format: nonce(12 bytes) || ciphertext || tag(16 bytes)
/// Each call generates a different random nonce.
pub fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    let aes_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(aes_key);

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96 bits aleatórios

    let ciphertext = cipher.encrypt(&nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {e}"))?;

    // nonce || ciphertext+tag
    let mut blob = nonce.to_vec();
    blob.extend_from_slice(&ciphertext);
    Ok(blob)
}

/// Decrypts a blob produced by `encrypt`.
pub fn decrypt(key: &[u8], blob: &[u8]) -> Result<Vec<u8>> {
    if blob.len() < 12 + 16 {
        bail!("Encrypted blob too short");
    }

    let aes_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(aes_key);

    let nonce = Nonce::from_slice(&blob[..12]);
    let ciphertext = &blob[12..];

    cipher.decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("Decryption failed — corrupted data or wrong key"))
}

/// Convenience: encrypts a UTF-8 string
pub fn encrypt_str(key: &[u8], s: &str) -> Result<Vec<u8>> {
    encrypt(key, s.as_bytes())
}

/// Convenience: decrypts and converts to UTF-8 String
pub fn decrypt_str(key: &[u8], blob: &[u8]) -> Result<String> {
    let bytes = decrypt(key, blob)?;
    String::from_utf8(bytes).map_err(|e| anyhow::anyhow!("Invalid UTF-8: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = vec![0x42u8; 32]; // test key
        let plaintext = "super-secret-password-123!@#";

        let blob = encrypt_str(&key, plaintext).unwrap();
        let recovered = decrypt_str(&key, &blob).unwrap();

        assert_eq!(plaintext, recovered);
    }

    #[test]
    fn test_nonces_are_unique() {
        let key = vec![0x42u8; 32];
        let blob1 = encrypt_str(&key, "same message").unwrap();
        let blob2 = encrypt_str(&key, "same message").unwrap();

        // Different blobs because nonces are random
        assert_ne!(blob1, blob2);
        // But both decrypt correctly
        assert_eq!(decrypt_str(&key, &blob1).unwrap(), "same message");
        assert_eq!(decrypt_str(&key, &blob2).unwrap(), "same message");
    }

    #[test]
    fn test_hmac_verifies_correct_password() {
        let key = derive_key("password123", b"test-salt-32-bytes-exact!!!xxxxx", 8, 1, 1).unwrap();
        let hmac = compute_hmac(&key).unwrap();
        assert!(verify_hmac(&key, &hmac).is_ok());
    }

    #[test]
    fn test_hmac_rejects_wrong_password() {
        let correct_key = derive_key("correct-password",  b"test-salt-32-bytes-exact!!!xxxxx", 8, 1, 1).unwrap();
        let wrong_key   = derive_key("wrong-password",   b"test-salt-32-bytes-exact!!!xxxxx", 8, 1, 1).unwrap();
        let hmac = compute_hmac(&correct_key).unwrap();
        assert!(verify_hmac(&wrong_key, &hmac).is_err());
    }
}
