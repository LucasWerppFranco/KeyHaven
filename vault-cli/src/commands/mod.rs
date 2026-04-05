pub mod add;
pub mod check;
pub mod gen;
pub mod get;
pub mod init;
pub mod list;
pub mod lock;
pub mod rofi;
pub mod unlock;

use std::path::PathBuf;

/// Resolve the default database path
pub fn default_db_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .expect("Unable to determine configuration directory");
    config_dir.join("keyhaven").join("vault.db")
}

/// Resolve the default socket path
pub fn default_socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"));
    runtime_dir.join("keyhaven-daemon.sock")
}
