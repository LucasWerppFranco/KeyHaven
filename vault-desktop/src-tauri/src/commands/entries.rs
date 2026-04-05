use crate::state::AppState;
use std::path::Path;
use tauri::State;
use vault_core::{EntryUpdate, NewEntry, VaultEntry};

/// List all entries in the vault, optionally filtered by search query
#[tauri::command]
pub async fn list_entries(
    search: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<VaultEntry>, String> {
    let session = state.session.lock().await;
    let key = session.as_ref().ok_or("Vault is locked")?;

    let search = search.unwrap_or_default();
    vault_core::list_entries(key, Path::new(&state.db_path), &search)
        .await
        .map_err(|e| e.to_string())
}

/// Get a single entry by ID or name
#[tauri::command]
pub async fn get_entry(
    query: String,
    state: State<'_, AppState>,
) -> Result<Option<VaultEntry>, String> {
    let session = state.session.lock().await;
    let key = session.as_ref().ok_or("Vault is locked")?;

    vault_core::get_entry(key, Path::new(&state.db_path), &query)
        .await
        .map_err(|e| e.to_string())
}

/// Add a new entry to the vault
#[tauri::command]
pub async fn add_entry(
    entry: NewEntry,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let session = state.session.lock().await;
    let key = session.as_ref().ok_or("Vault is locked")?;

    vault_core::add_entry(key, Path::new(&state.db_path), entry)
        .await
        .map_err(|e| e.to_string())
}

/// Update an existing entry
#[tauri::command]
pub async fn update_entry(
    update: EntryUpdate,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let session = state.session.lock().await;
    let key = session.as_ref().ok_or("Vault is locked")?;

    vault_core::update_entry(key, Path::new(&state.db_path), update)
        .await
        .map_err(|e| e.to_string())
}

/// Delete an entry by ID
#[tauri::command]
pub async fn delete_entry(
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let session = state.session.lock().await;
    let key = session.as_ref().ok_or("Vault is locked")?;

    vault_core::delete_entry(key, Path::new(&state.db_path), id)
        .await
        .map_err(|e| e.to_string())
}

/// Copy a password to the clipboard (with auto-clear after 30 seconds)
#[tauri::command]
pub async fn copy_password(
    entry_id: i64,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let session = state.session.lock().await;
    let key = session.as_ref().ok_or("Vault is locked")?;

    // Get the entry
    let entry = vault_core::get_entry(
        key,
        Path::new(&state.db_path),
        &entry_id.to_string(),
    )
    .await
    .map_err(|e| e.to_string())?
    .ok_or("Entry not found")?;

    // Copy to clipboard using Tauri's clipboard API
    #[cfg(desktop)]
    {
        use tauri::Manager;
        if let Some(clipboard) = app.clipboard() {
            clipboard
                .write_text(entry.password)
                .map_err(|e| e.to_string())?;
        }
    }

    // TODO: Schedule clipboard clear after 30 seconds
    // This would require spawning a task that waits and then clears

    Ok(())
}
