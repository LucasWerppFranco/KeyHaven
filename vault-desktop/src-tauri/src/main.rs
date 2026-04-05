mod commands;
mod state;

use crate::state::AppState;
use commands::entries::*;
use commands::generator::*;
use commands::vault::*;
use std::path::PathBuf;

fn main() {
    tauri::Builder::default()
        .manage(create_app_state())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Vault operations
            init_vault,
            unlock_vault,
            lock_vault,
            is_unlocked,
            vault_exists,
            // Entry operations
            list_entries,
            get_entry,
            add_entry,
            update_entry,
            delete_entry,
            copy_password,
            // Generator operations
            generate_password_cmd,
            generate_passphrase_cmd,
            check_password_strength,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn create_app_state() -> AppState {
    // Determine the vault database path
    let db_path = get_vault_path();
    AppState::new(db_path)
}

fn get_vault_path() -> PathBuf {
    // Use standard directories
    let data_dir = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."));

    data_dir.join("keyhaven").join("vault.db")
}
