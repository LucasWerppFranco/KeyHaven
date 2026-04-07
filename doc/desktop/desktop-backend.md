# Desktop Backend Documentation

The KeyHaven Desktop backend is written in Rust using the Tauri framework. It provides secure password management through a native API that the web frontend can invoke.

## Project Structure

```
vault-desktop/src-tauri/
├── src/
│   ├── main.rs           # Application entry point
│   ├── state.rs          # Global application state
│   └── commands/         # Tauri command handlers
│       ├── mod.rs        # Module exports
│       ├── vault.rs      # Vault lifecycle (init, unlock, lock)
│       ├── entries.rs    # Password entry CRUD
│       └── generator.rs  # Password generation
│
├── Cargo.toml            # Rust dependencies
├── tauri.conf.json       # Tauri configuration
└── icons/                # Application icons
```

## Dependencies

```toml
[dependencies]
# Workspace dependencies (shared across all crates)
serde = { workspace = true }
tokio = { workspace = true }
anyhow = { workspace = true }
zeroize = { workspace = true }

# Tauri framework
tauri = { version = "2.0", features = [] }
tauri-plugin-shell = "2.0"

# Vault core (shared business logic)
vault-core = { path = "../../vault-core" }

# System utilities
dirs = "5"
```

## State Management

### AppState

The `AppState` struct holds application-wide state that persists for the lifetime of the application:

```rust
// src/state.rs
pub struct AppState {
    /// The encryption key - None when vault is locked
    pub session: Mutex<Option<DerivedKey>>,
    /// Path to the vault database
    pub db_path: PathBuf,
}

pub type DerivedKey = Vec<u8>;
```

**Key characteristics:**
- `session` is a `tokio::sync::Mutex` for async-safe access
- The key is only `Some` while the vault is unlocked
- Locking sets `session` to `None`, which triggers `Zeroize` to clear memory

### State Initialization

```rust
// src/main.rs
fn main() {
    tauri::Builder::default()
        .manage(create_app_state())  // Register state with Tauri
        // ...
}

fn create_app_state() -> AppState {
    let db_path = get_vault_path();  // ~/.local/share/keyhaven/vault.db
    AppState::new(db_path)
}
```

### Accessing State in Commands

Commands receive state via the `State` extractor:

```rust
#[tauri::command]
pub async fn unlock_vault(
    master_password: String,
    state: State<'_, AppState>,  // Automatically injected by Tauri
) -> Result<(), String> {
    let key = vault_core::unlock(&master_password, &state.db_path).await?;
    *state.session.lock().await = Some(key);
    Ok(())
}
```

## Commands

Commands are Rust functions exposed to the frontend via Tauri's IPC bridge.

### Command Structure

```rust
#[tauri::command]
pub async fn command_name(
    arg: String,                          // Frontend argument
    state: State<'_, AppState>,           // Application state
) -> Result<ReturnType, String> {         // Return value or error
    // Implementation
}
```

### Available Commands

#### Vault Commands (`vault.rs`)

| Command | Arguments | Returns | Description |
|---------|-----------|---------|-------------|
| `init_vault` | `master_password: String` | `Result<(), String>` | Create new vault |
| `unlock_vault` | `master_password: String` | `Result<(), String>` | Unlock existing vault |
| `lock_vault` | - | `Result<(), String>` | Lock vault, clear key |
| `is_unlocked` | - | `Result<bool, String>` | Check unlock status |
| `vault_exists` | - | `Result<bool, String>` | Check if vault file exists |

**Example:**
```rust
#[tauri::command]
pub async fn unlock_vault(
    master_password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let key = vault_core::unlock(&master_password, &state.db_path)
        .await
        .map_err(|e| e.to_string())?;

    let mut session = state.session.lock().await;
    *session = Some(key);
    Ok(())
}
```

#### Entry Commands (`entries.rs`)

| Command | Arguments | Returns | Description |
|---------|-----------|---------|-------------|
| `list_entries` | `search: Option<String>` | `Result<Vec<VaultEntry>, String>` | List/filter entries |
| `get_entry` | `query: String` | `Result<Option<VaultEntry>, String>` | Get single entry |
| `add_entry` | `entry: NewEntry` | `Result<i64, String>` | Create entry |
| `update_entry` | `update: EntryUpdate` | `Result<(), String>` | Update entry |
| `delete_entry` | `id: i64` | `Result<(), String>` | Delete entry |
| `copy_password` | `entry_id: i64` | `Result<(), String>` | Copy to clipboard |

**Security pattern:** All entry commands require the vault to be unlocked:

```rust
#[tauri::command]
pub async fn list_entries(
    search: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<VaultEntry>, String> {
    let session = state.session.lock().await;
    let key = session.as_ref().ok_or("Vault is locked")?;
    
    vault_core::list_entries(key, &state.db_path, &search.unwrap_or_default())
        .await
        .map_err(|e| e.to_string())
}
```

#### Generator Commands (`generator.rs`)

| Command | Arguments | Returns | Description |
|---------|-----------|---------|-------------|
| `generate_password_cmd` | `options: PasswordOptions` | `Result<GeneratedPassword, String>` | Generate password |
| `generate_passphrase_cmd` | `options: PassphraseOptions` | `Result<GeneratedPassphrase, String>` | Generate passphrase |
| `check_password_strength` | `password: String` | `PasswordStrength` | Analyze strength |

**Note:** These commands don't require an unlocked vault (they don't access stored data).

### Command Registration

All commands must be registered in `main.rs`:

```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        // Vault
        init_vault,
        unlock_vault,
        lock_vault,
        is_unlocked,
        vault_exists,
        // Entries
        list_entries,
        get_entry,
        add_entry,
        update_entry,
        delete_entry,
        copy_password,
        // Generator
        generate_password_cmd,
        generate_passphrase_cmd,
        check_password_strength,
    ])
```

## Type Serialization

Tauri automatically serializes Rust types to JSON for IPC.

### Supported Types

| Rust | TypeScript |
|------|------------|
| `String` | `string` |
| `i64`, `i32`, etc. | `number` |
| `bool` | `boolean` |
| `Option<T>` | `T \| null` |
| `Vec<T>` | `T[]` |
| `struct { ... }` | `interface { ... }` |
| `enum` | `string` or discriminated union |

### Struct Example

```rust
// Rust
#[derive(Serialize)]
pub struct GeneratedPassword {
    pub password: String,
    pub strength: PasswordStrength,
}

#[derive(Serialize)]
pub enum PasswordStrength {
    Weak,
    Medium,
    Strong,
}
```

```typescript
// Generated TypeScript
generatedPassword: {
  password: string;
  strength: "Weak" | "Medium" | "Strong";
}
```

### Input Types

Tauri deserializes JSON arguments to Rust types:

```rust
#[derive(Deserialize)]
pub struct PasswordOptions {
    pub length: usize,
    pub include_symbols: bool,
}

#[tauri::command]
pub fn generate_password_cmd(options: PasswordOptions) -> ...
```

```typescript
// Frontend call
generatePassword({ length: 20, include_symbols: true });
```

## Error Handling

Commands return `Result<T, String>` where the error is passed to the frontend as a rejected promise.

### Pattern

```rust
#[tauri::command]
pub async fn command_name(...) -> Result<T, String> {
    // Use anyhow for internal errors
    let result = some_operation().await.map_err(|e| e.to_string())?;
    
    // Return specific errors for user-facing issues
    let key = session.as_ref().ok_or("Vault is locked")?;
    
    Ok(result)
}
```

### Frontend Error Handling

```typescript
try {
  await unlockVault(password);
} catch (error) {
  // error is the String from Rust Result::Err
  showNotification(error);
}
```

## Security Implementation

### Key Storage

The encryption key is stored in memory only:

```rust
pub struct AppState {
    pub session: Mutex<Option<DerivedKey>>,
}
```

- `Mutex` ensures thread-safe access
- `Option` allows `None` when locked
- `Vec<u8>` is automatically zeroed by the `zeroize` crate when dropped

### Auto-Lock

When the vault locks, the key is zeroed:

```rust
pub async fn lock(&self) {
    let mut session = self.session.lock().await;
    if let Some(ref mut key) = *session {
        key.zeroize();  // Securely wipe memory
    }
    *session = None;
}
```

### Clipboard Security

The `copy_password` command:
1. Decrypts the password using the session key
2. Writes to system clipboard
3. Spawns a task to clear after 30 seconds

```rust
#[tauri::command]
pub async fn copy_password(
    entry_id: i64,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // Decrypt password
    let entry = get_decrypted_entry(entry_id, state).await?;
    
    // Copy to clipboard
    #[cfg(desktop)]
    if let Some(clipboard) = app.clipboard() {
        clipboard.write_text(entry.password)?;
    }
    
    // Schedule clear
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(30)).await;
        if let Some(clipboard) = app.clipboard() {
            let _ = clipboard.clear();
        }
    });
    
    Ok(())
}
```

## Configuration

### tauri.conf.json

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "KeyHaven",
  "identifier": "com.keyhaven.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [{
      "title": "KeyHaven",
      "width": 1000,
      "height": 700,
      "minWidth": 800,
      "minHeight": 600
    }]
  }
}
```

### Cargo.toml

```toml
[package]
name = "vault-desktop"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2.0", features = [] }
vault-core = { path = "../../vault-core" }
# ... other deps

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

## Workspace Integration

The desktop crate is part of the KeyHaven workspace:

```toml
# /Cargo.toml
[workspace]
members = [
    "vault-core",
    "vault-daemon",
    "vault-cli",
    "vault-desktop/src-tauri",  # ← Desktop app
]
```

This allows it to depend on `vault-core`:

```toml
[dependencies]
vault-core = { path = "../../vault-core" }
```

## Platform-Specific Code

Use conditional compilation for platform-specific features:

```rust
#[cfg(desktop)]
{
    // Desktop-only code
    use tauri::Manager;
    let clipboard = app.clipboard();
}

#[cfg(target_os = "macos")]
{
    // macOS-specific code
}

#[cfg(target_os = "linux")]
{
    // Linux-specific code
}

#[cfg(target_os = "windows")]
{
    // Windows-specific code
}
```

## Building

### Development

```bash
cd vault-desktop
npm run tauri dev
```

### Production Build

```bash
cd vault-desktop
npm run tauri build
```

Output locations:
- Linux: `src-tauri/target/release/vault-desktop`
- macOS: `src-tauri/target/release/bundle/`
- Windows: `src-tauri/target/release/vault-desktop.exe`

## Testing

Commands can be tested by invoking them directly:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unlock_vault() {
        let state = AppState::new(test_db_path());
        
        // Initialize vault first
        init_vault("test_password".to_string(), State(&state)).await.unwrap();
        
        // Unlock should succeed
        let result = unlock_vault("test_password".to_string(), State(&state)).await;
        assert!(result.is_ok());
        
        // Verify unlocked
        assert!(state.is_unlocked().await);
    }
}
```

## Debugging

### Logging

Use the `log` crate for structured logging:

```rust
log::info!("Vault unlocked successfully");
log::error!("Failed to decrypt entry: {}", error);
log::debug!("Session state: {:?}", session);
```

### Panic Handling

Tauri catches panics in commands and converts them to error responses. For debugging:

```rust
std::panic::set_hook(Box::new(|info| {
    log::error!("Panic occurred: {:?}", info);
}));
```
