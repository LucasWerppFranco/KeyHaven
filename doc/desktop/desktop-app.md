# KeyHaven Desktop Application

The KeyHaven Desktop Application is a cross-platform password manager built with [Tauri](https://tauri.app/) — combining a Rust backend with a modern web frontend.

## Overview

The desktop app provides a graphical interface for managing your KeyHaven vault, featuring:

- **Secure vault initialization** with Argon2id key derivation
- **Master password unlock** with memory-only key storage
- **Password entry management** — add, edit, delete, and search
- **Built-in password generator** with strength analysis
- **Auto-lock** capability (key wiped from memory after inactivity)
- **Clipboard integration** with auto-clear after 30 seconds

## Architecture

```
vault-desktop/
├── src-tauri/           # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs      # Tauri setup, command registration
│   │   ├── state.rs     # AppState - session management
│   │   └── commands/    # Tauri commands (IPC handlers)
│   │       ├── vault.rs     # Vault lifecycle commands
│   │       ├── entries.rs   # Password CRUD commands
│   │       └── generator.rs # Password generation commands
│   ├── Cargo.toml       # Rust dependencies
│   └── tauri.conf.json  # Tauri configuration
│
├── src/                 # Frontend (React + TypeScript)
│   ├── api/             # Typed IPC wrappers
│   ├── components/      # React components
│   ├── pages/           # Route-level views
│   ├── App.tsx          # Main application router
│   ├── main.tsx         # React entry point
│   └── styles.css       # Application styles
│
└── package.json         # Node.js dependencies
```

## Technology Stack

### Backend (Rust)
- **Tauri v2** — Desktop framework, WebView integration
- **vault-core** — Shared business logic (crypto, storage)
- **tokio** — Async runtime
- **serde** — Serialization for IPC

### Frontend
- **React 18** — UI framework
- **TypeScript** — Type safety
- **React Router** — Navigation
- **Vite** — Build tooling

## Getting Started

### Prerequisites

Install Tauri system dependencies:

**Arch Linux:**
```bash
sudo pacman -S webkit2gtk-4.1 javascriptcoregtk-4.1
```

**Ubuntu/Debian:**
```bash
sudo apt install libwebkit2gtk-4.1-dev javascriptcoregtk-4.1
```

**macOS:**
No additional dependencies required.

**Windows:**
No additional dependencies required.

### Installation

```bash
cd vault-desktop
npm install
```

### Development

Run the app in development mode with hot reload:

```bash
npm run tauri dev
```

This starts:
- Vite dev server on `http://localhost:1420`
- Tauri app with WebView pointing to the dev server

### Building

Create a production build:

```bash
npm run tauri build
```

Output binaries are placed in `src-tauri/target/release/`.

## Security Model

The desktop app inherits KeyHaven's security architecture:

1. **Master Password** — Never stored; used to derive encryption key via Argon2id
2. **Memory-Only Keys** — Derived key lives only in `AppState.session` while unlocked
3. **Auto-Lock** — Key zeroed from memory when vault locks
4. **Unix Sockets** — IPC between frontend and backend is local-only
5. **Encrypted Database** — SQLite database with AES-256-GCM encrypted fields

See [how-it-works.md](../how-it-works.md) for the full security model.

## Integration with Existing Components

The desktop app reuses the existing `vault-core` crate:

```rust
// From vault-desktop/src-tauri/src/commands/vault.rs
use vault_core::{init_vault, unlock};

#[tauri::command]
pub async fn unlock_vault(
    master_password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let key = vault_core::unlock(&master_password, &state.db_path)
        .await
        .map_err(|e| e.to_string())?;
    
    // Store key in AppState
    *state.session.lock().await = Some(key);
    Ok(())
}
```

This ensures the desktop app, daemon, and CLI all share the same crypto and storage logic.

## Next Steps

- Read [desktop-architecture.md](./desktop-architecture.md) for detailed technical design
- Read [desktop-frontend.md](./desktop-frontend.md) for frontend development guide
- Read [desktop-backend.md](./desktop-backend.md) for Rust/Tauri implementation details
