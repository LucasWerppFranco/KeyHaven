use std::path::PathBuf;
use std::time::Duration;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Path to the encrypted database
    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,

    /// Path to the Unix socket
    #[serde(default = "default_socket_path")]
    pub socket_path: PathBuf,

    /// Inactivity time before auto-lock (default: 15 min)
    #[serde(default = "default_timeout", with = "duration_secs")]
    pub session_timeout: Duration,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("vault/config.toml");

        if config_path.exists() {
            let text = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&text)?)
        } else {
            Ok(Self::default())
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_path: default_db_path(),
            socket_path: default_socket_path(),
            session_timeout: default_timeout(),
        }
    }
}

fn default_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("vault/vault.db")
}

fn default_socket_path() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("vault.sock")
}

fn default_timeout() -> Duration {
    Duration::from_secs(15 * 60)
}

mod duration_secs {
    use serde::{Deserialize, Deserializer};
    use std::time::Duration;

    pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(d)?;
        Ok(Duration::from_secs(secs))
    }
}
