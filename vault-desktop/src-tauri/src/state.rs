use std::path::PathBuf;
use tokio::sync::Mutex;

/// The derived key from Argon2id - stored in memory only while unlocked
pub type DerivedKey = Vec<u8>;

/// Application state shared across all Tauri commands
pub struct AppState {
    /// The encryption key - None when vault is locked
    pub session: Mutex<Option<DerivedKey>>,
    /// Path to the vault database
    pub db_path: PathBuf,
}

impl AppState {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            session: Mutex::new(None),
            db_path,
        }
    }

    /// Lock the vault by clearing the session key
    pub async fn lock(&self) {
        let mut session = self.session.lock().await;
        if let Some(ref mut key) = *session {
            key.zeroize();
        }
        *session = None;
    }

    /// Check if the vault is unlocked
    pub async fn is_unlocked(&self) -> bool {
        self.session.lock().await.is_some()
    }
}

use zeroize::Zeroize;
