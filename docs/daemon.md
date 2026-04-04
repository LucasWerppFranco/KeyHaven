# KeyHaven Daemon

The **vault-daemon** is the core service that manages the encrypted vault and serves as the single point of access for all client operations. It runs as a background process and communicates via Unix domain sockets.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Security Model](#security-model)
- [Communication Protocol](#communication-protocol)
- [Session Management](#session-management)
- [Configuration](#configuration)
- [API Reference](#api-reference)
- [Running the Daemon](#running-the-daemon)

---

## Architecture Overview

```
┌─────────────────────────────────────────┐
│                 Clients                 │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  │
│  │   CLI   │  │   GUI   │  │ Browser │  │
│  └────┬────┘  └────┬────┘  └────┬────┘  │
└───────┼────────────┼────────────┼───────┘
        │            │            │
        └────────────┴────────────┘
                     │
              Unix Domain Socket
                     │
        ┌────────────▼──────────┐
        │     vault-daemon      │
        │  ┌─────────────────┐  │
        │  │  Session State  │  │
        │  │  (Derived Key)  │  │
        │  └─────────────────┘  │
        │  ┌─────────────────┐  │
        │  │  Request Router │  │
        │  └─────────────────┘  │
        └────────────┬──────────┘
                     │
              SQLite + AES-256-GCM
                     │
              ┌──────▼──────┐
              │  vault.db   │
              └─────────────┘
```

The daemon follows a **single-process, multi-client** architecture:

- One daemon instance manages one vault database
- Multiple clients can connect concurrently via Unix socket
- Session state is shared across all connections (global lock/unlock)

---

## Security Model

### Unix Socket Permissions

The daemon creates a Unix socket with **0600 permissions** (owner read/write only). This ensures:

- Only the owning user can connect to the daemon
- Other users on the system cannot access the vault

### Session State

When the vault is unlocked:

1. The derived encryption key is held in memory wrapped in a `Zeroizing<Vec<u8>>`
2. `Zeroizing` guarantees the memory is securely wiped when the session ends
3. The key never leaves the daemon process

### Auto-Lock

The daemon automatically locks the vault after a period of inactivity:

- Default timeout: **15 minutes**
- Check interval: **30 seconds**
- Configurable via `config.toml`

When auto-locked, the key is dropped and memory is zeroed.

---

## Communication Protocol

The daemon uses a simple message-based protocol over Unix sockets.

### Message Format

Each message is length-prefixed:

```
┌───────────┬─────────────────────────────────────┐
│  Length   │              Payload                │
│ (4 bytes) │          (JSON, variable)           │
│  big-u32  │                                     │
└───────────┴─────────────────────────────────────┘
```

### Request Structure

```json
{
  "id": "unique-request-id",
  "action": "Unlock",
  "params": {}
}
```

**Actions:**

| Category  | Actions                                                     |
|-----------|-------------------------------------------------------------|
| Session   | `Unlock`, `Lock`, `Status`                                        |
| Entries   | `ListEntries`, `GetEntry`, `AddEntry`, `UpdateEntry`, `DeleteEntry`   |
| Generator | `GeneratePassword`, `CheckPassword`                             |

### Response Structure

```json
{
  "id": "same-request-id",
  "ok": true,
  "data": { ... },
  "error": null
}
```

Or on error:

```json
{
  "id": "same-request-id",
  "ok": false,
  "data": null,
  "error": "Vault locked. Run: vault unlock"
}
```

---

## Session Management

### Unlock Flow

```
Client                    Daemon                    vault-core
  │                         │                            │
  │────── unlock ───────>   │                            │
  │    {password}           │                            │
  │                         │ ─── derive_key() ────────> │
  │                         │   {salt, Argon2id params}  │
  │                         │ <──── derived key ──────── │
  │                         │                            │
  │                         │ Store key in Session       │
  │                         │ (Zeroizing<Vec<u8>>)       │
  │                         │                            │
  │  <──── success ─────────│                            │
  │     {timeout_secs}      │                            │
```

### Key Derivation

When unlocking, the daemon:

1. Reads the Argon2id parameters and salt from `vault_meta` table
2. Derives the AES-256 key using Argon2id with stored parameters
3. Verifies the HMAC-SHA256 of "vault-v1-ok" against stored verification tag
4. If verification passes, stores the key in session state

This design allows future parameter upgrades without invalidating old vaults.

### Session State Machine

```
┌─────────────┐
│   Locked    │<─────────────────────────┐
│  (no key)   │                          │
└──────┬──────┘                          │
       │ unlock                          │
       │ (verify HMAC)                   │
       ▼                                 │
┌─────────────┐      activity timeout    │
│  Unlocked   │──────────────────────────┤
│ (key in mem)│    or explicit lock      │
└─────────────┘──────────────────────────┘
```

---

## Configuration

The daemon loads configuration from `~/.config/vault/config.toml`.

### Default Configuration

```toml
# Path to the encrypted SQLite database
db_path = "~/.local/share/vault/vault.db"

# Path to the Unix socket
socket_path = "/tmp/vault.sock"  # or $XDG_RUNTIME_DIR/vault.sock

# Auto-lock timeout in seconds
session_timeout = 900  # 15 minutes
```

### Configuration Precedence

1. User config file (`~/.config/vault/config.toml`)
2. Built-in defaults

---

## API Reference

### Session Actions

#### `Unlock`

Unlocks the vault with the master password.

**Request:**
```json
{
  "id": "req-1",
  "action": "Unlock",
  "params": {
    "password": "my-master-password"
  }
}
```

**Response (success):**
```json
{
  "id": "req-1",
  "ok": true,
  "data": {
    "message": "Vault unlocked",
    "timeout_secs": 900
  }
}
```

**Response (error):**
```json
{
  "id": "req-1",
  "ok": false,
  "error": "Incorrect master password"
}
```

---

#### `Lock`

Immediately locks the vault and wipes the key from memory.

**Request:**
```json
{
  "id": "req-2",
  "action": "Lock",
  "params": {}
}
```

**Response:**
```json
{
  "id": "req-2",
  "ok": true,
  "data": {
    "message": "Vault locked"
  }
}
```

---

#### `Status`

Returns whether the vault is currently unlocked.

**Request:**
```json
{
  "id": "req-3",
  "action": "Status",
  "params": {}
}
```

**Response:**
```json
{
  "id": "req-3",
  "ok": true,
  "data": {
    "unlocked": true
  }
}
```

---

### Entry Actions

All entry actions require the vault to be unlocked. Returns error if locked.

#### `ListEntries`

Lists vault entries with optional search filter.

**Request:**
```json
{
  "id": "req-4",
  "action": "ListEntries",
  "params": {
    "search": "github"
  }
}
```

**Response:**
```json
{
  "id": "req-4",
  "ok": true,
  "data": [
    {
      "id": 1,
      "title": "GitHub",
      "username": "myuser",
      "password": "encrypted...",
      "url": "https://github.com",
      "notes": "",
      "tags": ["dev"],
      "created_at": "2024-01-15T10:30:00Z",
      "modified_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

---

#### `GetEntry`

Retrieves a single entry by ID or title match.

**Request:**
```json
{
  "id": "req-5",
  "action": "GetEntry",
  "params": {
    "query": "GitHub"
  }
}
```

**Response:**
```json
{
  "id": "req-5",
  "ok": true,
  "data": {
    "id": 1,
    "title": "GitHub",
    "username": "myuser",
    "password": "decrypted-password",
    "url": "https://github.com",
    "notes": "",
    "tags": ["dev"],
    "created_at": "2024-01-15T10:30:00Z",
    "modified_at": "2024-01-15T10:30:00Z"
  }
}
```

---

#### `AddEntry`

Creates a new vault entry.

**Request:**
```json
{
  "id": "req-6",
  "action": "AddEntry",
  "params": {
    "title": "Twitter",
    "username": "@myhandle",
    "password": "my-secure-password",
    "url": "https://twitter.com",
    "notes": "Personal account",
    "tags": ["social"]
  }
}
```

**Response:**
```json
{
  "id": "req-6",
  "ok": true,
  "data": {
    "id": 2
  }
}
```

---

#### `UpdateEntry`

Updates an existing entry.

**Request:**
```json
{
  "id": "req-7",
  "action": "UpdateEntry",
  "params": {
    "id": 2,
    "title": "X (Twitter)",
    "password": "new-password"
  }
}
```

**Response:**
```json
{
  "id": "req-7",
  "ok": true,
  "data": {
    "updated": true
  }
}
```

---

#### `DeleteEntry`

Deletes an entry by ID.

**Request:**
```json
{
  "id": "req-8",
  "action": "DeleteEntry",
  "params": {
    "id": 2
  }
}
```

**Response:**
```json
{
  "id": "req-8",
  "ok": true,
  "data": {
    "deleted": true
  }
}
```

---

### Generator Actions

These actions do not require the vault to be unlocked.

#### `GeneratePassword`

Generates a random password or passphrase.

**Request (password):**
```json
{
  "id": "req-9",
  "action": "GeneratePassword",
  "params": {
    "length": 20,
    "symbols": true
  }
}
```

**Request (passphrase):**
```json
{
  "id": "req-10",
  "action": "GeneratePassword",
  "params": {
    "words": 6
  }
}
```

**Response:**
```json
{
  "id": "req-9",
  "ok": true,
  "data": {
    "password": "aB3$k9!mP2@qR7&xL4",
    "entropy_bits": 128,
    "score": 4,
    "label": "strong"
  }
}
```

---

#### `CheckPassword`

Analyzes password strength without storing it.

**Request:**
```json
{
  "id": "req-11",
  "action": "CheckPassword",
  "params": {
    "password": "my-password-to-check"
  }
}
```

**Response:**
```json
{
  "id": "req-11",
  "ok": true,
  "data": {
    "entropy_bits": 52,
    "score": 2,
    "label": "fair",
    "warning": "This is a top-100 common password"
  }
}
```

---

## Running the Daemon

### Development

```bash
# Build the daemon
cargo build -p vault-daemon

# Run with default settings
cargo run -p vault-daemon

# The daemon will:
# 1. Load/create config at ~/.config/vault/config.toml
# 2. Create the socket at ~/.local/share/keyhaven/daemon.sock
# 3. Start listening for connections
```

### Production

```bash
# Build release binary
cargo build --release -p vault-daemon

# Run as systemd service (example service file)
systemctl --user enable vault-daemon
systemctl --user start vault-daemon
```

### Socket Location

By default, the daemon creates its socket at:
- `$XDG_RUNTIME_DIR/vault.sock` (if available)
- `/tmp/vault.sock` (fallback)

Clients should check these locations or read from the config file.

---

## Error Handling

Common error responses:

| Error | Cause | Resolution |
|-------|-------|------------|
| `Vault locked. Run: vault unlock` | Session expired or never unlocked | Call `Unlock` with master password |
| `Incorrect master password` | Wrong password during unlock | Retry with correct password |
| `Entry not found` | Query didn't match any entry | Check the query/id |
| `Field 'X' is required` | Missing parameter in request | Add the required field |
| `Invalid params: ...` | JSON parsing error | Check request format |

---

## Implementation Details

### Thread Safety

The daemon uses `tokio::sync::Mutex` for session state:

- Multiple connections are handled concurrently
- Session state is shared across all connections
- Lock contention is minimized (background task checks every 30s)

### Message Size Limits

The daemon rejects messages larger than **1 MB** to prevent DoS attacks.

### Graceful Shutdown

The daemon removes its socket file on startup if it exists (handles previous crashes). There's no explicit shutdown signal handling currently.

---

## See Also

- [Architecture Overview](./architecture.md) - Overall system design
- [Crypto Details](./crypto.md) - Encryption and key derivation
- [CLI Guide](./cli.md) - Command-line client usage
