//! Session management for CLI authentication

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Get the path to the session key file
fn session_key_path() -> PathBuf {
    dirs::runtime_dir()
        .or_else(|| dirs::cache_dir())
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("keyhaven")
        .join("session.key")
}

/// Ensure the session directory exists
fn ensure_session_dir() -> Result<()> {
    let path = session_key_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create session directory: {}", parent.display()))?;
    }
    Ok(())
}

/// Save the session key to a file
pub fn save_session_key(key: &[u8]) -> Result<()> {
    ensure_session_dir()?;
    let path = session_key_path();

    // Write key with restricted permissions (0o600)
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)
            .with_context(|| format!("Failed to create session file: {}", path.display()))?;

        std::io::Write::write_all(&mut file, key)
            .with_context(|| format!("Failed to write session key: {}", path.display()))?;
    }

    #[cfg(not(unix))]
    {
        std::fs::write(&path, key)
            .with_context(|| format!("Failed to write session key: {}", path.display()))?;
    }

    // Also set environment variable for current process
    std::env::set_var("KEYHAVEN_SESSION_KEY", hex::encode(key));

    Ok(())
}

/// Load the session key from file or environment
pub fn load_session_key() -> Result<Vec<u8>> {
    // First try environment variable (for current process)
    if let Ok(key_hex) = std::env::var("KEYHAVEN_SESSION_KEY") {
        return hex::decode(&key_hex)
            .context("Invalid session key in environment variable");
    }

    // Fall back to file
    let path = session_key_path();
    if path.exists() {
        let key = std::fs::read(&path)
            .with_context(|| format!("Failed to read session file: {}", path.display()))?;

        // Set it in environment for this process too
        std::env::set_var("KEYHAVEN_SESSION_KEY", hex::encode(&key));

        return Ok(key);
    }

    Err(anyhow::anyhow!("Vault is locked. Run 'keyhaven unlock' first."))
}

/// Clear the session (lock the vault)
pub fn clear_session() -> Result<()> {
    // Clear environment variable
    std::env::remove_var("KEYHAVEN_SESSION_KEY");

    // Remove session file
    let path = session_key_path();
    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("Failed to remove session file: {}", path.display()))?;
    }

    Ok(())
}

/// Check if there's an active session
#[allow(dead_code)]
pub fn is_unlocked() -> bool {
    std::env::var("KEYHAVEN_SESSION_KEY").is_ok() ||
        session_key_path().exists()
}
