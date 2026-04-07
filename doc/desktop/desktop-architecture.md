# Desktop App Architecture

This document describes the technical architecture of the KeyHaven Desktop Application built with Tauri.

## Core Principles

1. **Security First** — The encryption key never touches disk, only exists in memory while the vault is unlocked
2. **Code Reuse** — All cryptographic and storage operations use the shared `vault-core` crate
3. **Type Safety** — Full TypeScript type coverage with generated types from Rust
4. **Native Performance** — Rust handles all crypto/storage; React handles UI only

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Desktop App                             │
│  ┌─────────────────────┐    ┌─────────────────────────────┐  │
│  │   Frontend (Web)    │    │      Backend (Rust)         │  │
│  │                     │    │                             │  │
│  │  ┌───────────────┐  │    │  ┌─────────────────────┐    │  │
│  │  │   React UI    │  │    │  │   Tauri Runtime     │    │  │
│  │  │               │  │    │  │                     │    │  │
│  │  │  ┌─────────┐  │  │    │  │  ┌───────────────┐  │    │  │
│  │  │  │  Pages  │  │  │◄─┐ │  │  │   Commands    │  │    │  │
│  │  │  │         │  │  │  │ │  │  │               │  │    │  │
│  │  │  │ • Setup │  │  │  │ │  │  │ • vault.rs    │  │    │  │
│  │  │  │ • Unlock│  │  │  │ │  │  │ • entries.rs  │  │    │  │
│  │  │  │ • Vault │  │  │  │ │  │  │ • generator.rs│  │    │  │
│  │  │  └─────────┘  │  │  │ │  │  └───────┬───────┘  │    │  │
│  │  │       │       │  │  │ │  │          │          │    │  │
│  │  │  ┌────┴────┐  │  │  │ │  │  ┌───────┴───────┐  │    │  │
│  │  │  │   API   │  │◄─┼──┘ │  │  │    AppState   │  │    │  │
│  │  │  │         │  │  │    │  │  │               │  │    │  │
│  │  │  │ invoke()│  │  │    │  │  │ ┌───────────┐ │  │    │  │
│  │  │  └─────────┘  │  │    │  │  │ │session:   │ │  │    │  │
│  │  └───────────────┘  │    │  │  │ │Mutex<     │ │  │    │  │
│  │                     │    │  │  │ │ Option<   │ │  │    │  │
│  └─────────────────────┘    │  │  │ │  DerivedKey│ │  │    │  │
│                             │  │  │ │>>         │ │  │    │  │
│                             │  │  │ └───────────┘ │  │    │  │
│                             │  │  └───────────────┘  │    │  │
│                             │  │          │          │    │  │
│                             │  │  ┌───────┴───────┐  │    │  │
│                             │  │  │   vault-core  │  │    │  │
│                             │  │  │               │  │    │  │
│                             │  │  │ • crypto.rs   │  │    │  │
│                             │  │  │ • store.rs    │  │    │  │
│                             │  │  │ • generator.rs│  │    │  │
│                             │  │  └───────────────┘  │    │  │
│                             │  └───────────────────────┘    │  │
│                             └─────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

## IPC Bridge (Frontend ↔ Backend)

Tauri uses an IPC (Inter-Process Communication) bridge to communicate between the web frontend and Rust backend.

### Frontend Side

```typescript
// src/api/vault.ts
import { invoke } from "@tauri-apps/api/core";

export async function unlockVault(masterPassword: string): Promise<void> {
  return invoke("unlock_vault", { masterPassword });
}
```

### Backend Side

```rust
// src/commands/vault.rs
#[tauri::command]
pub async fn unlock_vault(
    masterPassword: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let key = vault_core::unlock(&masterPassword, &state.db_path)
        .await
        .map_err(|e| e.to_string())?;
    
    *state.session.lock().await = Some(key);
    Ok(())
}
```

### Command Registration

All commands must be registered in `main.rs`:

```rust
// src/main.rs
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        unlock_vault,
        lock_vault,
        list_entries,
        // ... other commands
    ])
```

## State Management

### AppState

`AppState` is a Tauri-managed state struct that persists for the lifetime of the application:

```rust
// src/state.rs
pub struct AppState {
    /// The encryption key - None when vault is locked
    pub session: Mutex<Option<DerivedKey>>,
    /// Path to the vault database
    pub db_path: PathBuf,
}
```

- `session` holds the Argon2id-derived key **only while unlocked**
- Commands requiring the key check `session.is_some()` and return an error if locked
- Locking the vault sets `session = None`, immediately zeroing the key from memory

### Accessing State in Commands

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
}
```

## Security Flow

### Unlock Flow

```
1. User enters master password in frontend
2. Frontend invokes unlock_vault(password)
3. Backend calls vault_core::unlock():
   a. Read salt from database
   b. Derive key with Argon2id (~100ms)
   c. Verify HMAC
4. If valid, store key in AppState.session
5. Return success to frontend
6. Frontend navigates to vault view
```

### Auto-Lock Flow

```
1. Timer detects inactivity (e.g., 15 minutes)
2. Backend sets AppState.session = None
   (automatically zeroizes memory via zeroize crate)
3. Backend emits "vault-locked" event to frontend
4. Frontend navigates to unlock screen
5. Any subsequent commands return "Vault is locked" error
```

## Frontend Architecture

### Routing

The frontend uses React Router for navigation:

```
/           → Redirect based on vault state
/setup      → Create new vault (if none exists)
/unlock     → Unlock existing vault
/vault      → Main vault interface
/vault/new  → Add new entry form
/vault/edit/:id → Edit entry form
```

### State Flow

```
App.tsx (checks vault status on mount)
    ↓
Routes mounted based on state:
    - No vault → SetupPage
    - Locked  → UnlockPage
    - Unlocked → VaultPage
```

### Component Hierarchy

```
App
├── SetupPage
│   └── Form (init vault)
├── UnlockPage
│   └── Form (unlock vault)
└── VaultPage
    ├── Header (lock button, navigation)
    ├── EntryList (search results)
    │   └── EntryCard (individual entries)
    ├── EntryForm (add/edit entries)
    └── PasswordGenerator
```

## Data Flow Example: Adding an Entry

```
User fills EntryForm → clicks Save
    ↓
Frontend calls addEntry(entry) from api/entries.ts
    ↓
api/entries.ts calls invoke("add_entry", { entry })
    ↓
Tauri serializes entry to JSON, sends to Rust
    ↓
Backend command receives Entry struct (deserialized)
    ↓
Command extracts key from AppState
    ↓
Command calls vault_core::add_entry(key, db_path, entry)
    ↓
vault_core encrypts fields and inserts into SQLite
    ↓
Result propagated back through the chain
    ↓
Frontend refreshes entry list on success
```

## Workspace Integration

The desktop app is part of the KeyHaven workspace:

```toml
# /Cargo.toml (root)
[workspace]
members = [
    "vault-core",        # ← Shared library
    "vault-daemon",      # ← Background service
    "vault-cli",         # ← Command-line interface
    "vault-desktop/src-tauri",  # ← Desktop app
]
```

This allows `vault-desktop` to depend on `vault-core`:

```toml
# vault-desktop/src-tauri/Cargo.toml
[dependencies]
vault-core = { path = "../../vault-core" }
```

## Build Process

### Development Build

```
npm run tauri dev
    ↓
Vite starts dev server (localhost:1420)
    ↓
Tauri builds Rust code
    ↓
Tauri launches WebView pointing to dev server
    ↓
Hot reload on code changes
```

### Production Build

```
npm run tauri build
    ↓
Vite builds frontend to dist/
    ↓
Tauri bundles frontend assets into binary
    ↓
Rust compiles to native code
    ↓
Native installer created (.deb, .dmg, .msi, etc.)
```

## Platform-Specific Notes

### Linux
- Uses WebKit2GTK for WebView
- Requires `webkit2gtk-4.1-dev` at build time
- Uses native notifications via `notify-rust`

### macOS
- Uses WebKit (system framework)
- Code signing recommended for distribution
- Notarization required for Gatekeeper

### Windows
- Uses WebView2 (Edge/Chromium)
- WebView2 runtime must be present (or bundled)
- Supports Windows 10/11
