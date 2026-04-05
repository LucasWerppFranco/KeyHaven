use crate::state::AppState;
use std::path::Path;
use tauri::State;

/// Initialize a new vault with the given master password
#[tauri::command]
pub async fn init_vault(
    master_password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    vault_core::init_vault(&master_password, Path::new(&state.db_path))
        .await
        .map_err(|e| e.to_string())
}

/// Unlock the vault with the master password
#[tauri::command]
pub async fn unlock_vault(
    master_password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let key = vault_core::unlock(&master_password, Path::new(&state.db_path))
        .await
        .map_err(|e| e.to_string())?;

    let mut session = state.session.lock().await;
    *session = Some(key);
    Ok(())
}

/// Lock the vault - clears the encryption key from memory
#[tauri::command]
pub async fn lock_vault(state: State<'_, AppState>) -> Result<(), String> {
    state.lock().await;
    Ok(())
}

/// Check if the vault is currently unlocked
#[tauri::command]
pub async fn is_unlocked(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.is_unlocked().await)
}

/// Check if a vault exists at the configured path
#[tauri::command]
pub async fn vault_exists(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(Path::new(&state.db_path).exists())
}
