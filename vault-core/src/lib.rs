// vault-core: pure library without session I/O.
// Receives the key as a parameter — never manages unlock/lock.

pub mod crypto;
pub mod store;
pub mod generator;
pub mod matcher;

pub use store::{NewEntry, EntryUpdate, VaultEntry};
pub use generator::{generate_password, generate_passphrase, check_strength, PasswordStrength};

use std::path::Path;
use anyhow::Result;

// ── Public API called by the daemon ─────────────────────────────────────────

/// Attempts to unlock the vault with the master password.
/// Returns the derived key if the password is correct.
pub async fn unlock(master_password: &str, db_path: &Path) -> Result<Vec<u8>> {
    // 1. Read salt and parameters from vault_meta
    let meta = store::read_meta(db_path).await?;

    // 2. Derive key with Argon2id
    let key = crypto::derive_key(
        master_password,
        &meta.argon2_salt,
        meta.argon2_m_cost,
        meta.argon2_t_cost,
        meta.argon2_p_cost,
    )?;

    // 3. Verify HMAC — if wrong, password is incorrect
    crypto::verify_hmac(&key, &meta.verification_hmac)?;

    Ok(key)
}

/// Initializes a new vault: creates the database, generates salt, computes initial HMAC.
pub async fn init_vault(master_password: &str, db_path: &Path) -> Result<()> {
    use rand::RngCore;

    // Generate random salt (32 bytes)
    let mut salt = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);

    // Argon2id parameters (OWASP minimum recommendation for 2024)
    let m_cost = 65536; // 64 MB
    let t_cost = 3;
    let p_cost = 4;

    let key = crypto::derive_key(master_password, &salt, m_cost, t_cost, p_cost)?;
    let hmac = crypto::compute_hmac(&key)?;

    store::init_db(db_path, &salt, m_cost, t_cost, p_cost, &hmac).await
}

pub async fn list_entries(key: &[u8], db_path: &Path, search: &str) -> Result<Vec<VaultEntry>> {
    store::list_entries(key, db_path, search).await
}

pub async fn get_entry(key: &[u8], db_path: &Path, query: &str) -> Result<Option<VaultEntry>> {
    store::get_entry(key, db_path, query).await
}

pub async fn add_entry(key: &[u8], db_path: &Path, entry: NewEntry) -> Result<i64> {
    store::add_entry(key, db_path, entry).await
}

pub async fn update_entry(key: &[u8], db_path: &Path, update: EntryUpdate) -> Result<()> {
    store::update_entry(key, db_path, update).await
}

pub async fn delete_entry(key: &[u8], db_path: &Path, id: i64) -> Result<()> {
    store::delete_entry(key, db_path, id).await
}
