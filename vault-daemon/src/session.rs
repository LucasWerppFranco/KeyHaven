use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use zeroize::Zeroizing;

use crate::config::Config;

/// Derived key held in memory while the vault is unlocked.
/// Zeroizing ensures bytes are zeroed when dropped.
pub struct Session {
    pub key: Zeroizing<Vec<u8>>,
    pub unlocked_at: Instant,
    pub last_activity: Instant,
}

pub struct AppState {
    pub session: Mutex<Option<Session>>,
    pub config: Config,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            session: Mutex::new(None),
            config,
        }
    }

    /// Executes an operation that needs the key.
    /// Returns VaultLocked if the vault is locked.
    /// Automatically updates last_activity.
    pub async fn with_key<F, Fut, R>(&self, f: F) -> Result<R, VaultError>
    where
        F: FnOnce(&Zeroizing<Vec<u8>>) -> Fut,
        Fut: std::future::Future<Output = Result<R, VaultError>>,
    {
        let mut guard = self.session.lock().await;
        match guard.as_mut() {
            None => Err(VaultError::Locked),
            Some(session) => {
                session.last_activity = Instant::now();
                f(&session.key).await
            }
        }
    }

    pub async fn is_unlocked(&self) -> bool {
        self.session.lock().await.is_some()
    }

    pub async fn unlock(&self, key: Vec<u8>) {
        let mut guard = self.session.lock().await;
        *guard = Some(Session {
            key: Zeroizing::new(key),
            unlocked_at: Instant::now(),
            last_activity: Instant::now(),
        });
        log::info!("Vault unlocked.");
    }

    pub async fn lock(&self) {
        let mut guard = self.session.lock().await;
        // Drop the Zeroizing<Vec<u8>> → memory automatically zeroed
        *guard = None;
        log::info!("Vault locked.");
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("Vault locked. Run: vault unlock")]
    Locked,
    #[error("Incorrect master password")]
    WrongPassword,
    #[error("Entry not found")]
    NotFound,
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Background task: checks for inactivity and locks the vault.
/// Runs every 30 seconds to avoid hitting the Mutex too often.
pub async fn auto_lock_task(state: Arc<AppState>) {
    let check_interval = Duration::from_secs(30);

    loop {
        tokio::time::sleep(check_interval).await;

        let timeout = state.config.session_timeout;
        let mut guard = state.session.lock().await;

        if let Some(session) = guard.as_ref() {
            let idle = session.last_activity.elapsed();
            if idle >= timeout {
                log::info!(
                    "Auto-lock due to inactivity ({:.0?} idle, timeout={:.0?})",
                    idle, timeout
                );
                // Drop the Zeroizing → bytes zeroed
                *guard = None;
            }
        }
    }
}
