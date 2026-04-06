# KeyHaven Database Documentation

## Architecture Overview

KeyHaven uses a **field-by-field encryption** model with SQLite. Instead of encrypting the entire database file, individual sensitive fields are encrypted before being stored. This provides a balance between security and usability:

- **Plaintext fields**: Allow for efficient searching and indexing
- **Encrypted fields**: Protect sensitive data at rest

## Why This Approach?

### Alternative: Full Database Encryption
Tools like SQLCipher encrypt the entire database file. This is simpler but has limitations:
- Cannot search without decrypting the entire database
- Harder to integrate with async Rust ecosystem
- Requires custom SQLite builds

### KeyHaven's Approach: Field-by-Field
- **Searchable**: Title, URL, and tags are plaintext for fast queries
- **Secure**: Passwords, usernames, and notes are encrypted
- **Flexible**: Native async SQLite via sqlx
- **Transparent**: Database file is readable schema, opaque data

## Database Schema

### vault_meta Table

Stores the vault's cryptographic parameters. Only one row exists (id = 1).

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Fixed value 1 (CHECK constraint) |
| `argon2_salt` | BLOB | 32-byte random salt for key derivation |
| `argon2_m_cost` | INTEGER | Memory cost in KB (default: 65536 = 64MB) |
| `argon2_t_cost` | INTEGER | Time iterations (default: 3) |
| `argon2_p_cost` | INTEGER | Parallelism (default: 4) |
| `verification_hmac` | BLOB | HMAC-SHA256("vault-v1-ok") for password verification |

### entries Table

Stores password entries with mixed plaintext/encrypted fields.

| Column | Type | Storage | Description |
|--------|------|---------|-------------|
| `id` | INTEGER | — | Auto-increment primary key |
| `title` | TEXT | Plaintext | Entry name (searchable) |
| `username` | BLOB | Encrypted | User identifier |
| `password` | BLOB | Encrypted | The secret password |
| `url` | TEXT | Plaintext | Website URL (searchable) |
| `notes` | BLOB | Encrypted | Additional notes |
| `tags` | TEXT | Plaintext | Comma-separated tags (searchable) |
| `created_at` | INTEGER | — | Unix timestamp |
| `updated_at` | INTEGER | — | Unix timestamp |

## Encryption Details

### Key Derivation

The encryption key is never stored. It is derived from the master password using Argon2id:

```rust
key = Argon2id(password, salt, m_cost=65536, t_cost=3, p_cost=4)
```

Parameters follow OWASP 2024 recommendations:
- **Memory**: 64 MB (resists GPU/ASIC attacks)
- **Time**: 3 iterations
- **Parallelism**: 4 lanes

### Per-Field Encryption Format

Each encrypted field is stored as a binary blob:

```
[nonce: 12 bytes][ciphertext + auth_tag: N bytes]
```

- **Nonce**: 96-bit random IV, unique per encryption operation
- **Ciphertext**: AES-256-GCM encrypted data
- **Auth Tag**: 128-bit authentication tag (GCM mode)

Total overhead per field: 28 bytes (12 nonce + 16 tag)

### Encryption/Decryption Flow

**On Write (Add/Update):**
```
plaintext → AES-256-GCM → [nonce || ciphertext] → SQLite BLOB
```

**On Read (Get/List):**
```
SQLite BLOB → [nonce || ciphertext] → AES-256-GCM → plaintext
```

### Security Properties

1. **Unique Nonces**: Each encryption generates a fresh random nonce
2. **No Key Storage**: Master password only used for derivation; key held in daemon memory
3. **Constant-Time HMAC**: Password verification uses constant-time comparison
4. **Memory Zeroing**: Keys are wiped from memory using `zeroize`

## SQL Queries

### Initialize Database

```sql
CREATE TABLE IF NOT EXISTS vault_meta (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    argon2_salt BLOB NOT NULL,
    argon2_m_cost INTEGER NOT NULL,
    argon2_t_cost INTEGER NOT NULL,
    argon2_p_cost INTEGER NOT NULL,
    verification_hmac BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    username BLOB,
    password BLOB NOT NULL,
    url TEXT,
    notes BLOB,
    tags TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### Search Entries

```sql
SELECT id, title, username, password, url, notes, tags, created_at, updated_at
FROM entries
WHERE title LIKE ? OR url LIKE ? OR tags LIKE ?
ORDER BY title
```

### Update Entry (Partial)

Uses `COALESCE` to only update provided fields:

```sql
UPDATE entries SET
    title = COALESCE(?1, title),
    username = COALESCE(?2, username),
    password = COALESCE(?3, password),
    url = COALESCE(?4, url),
    notes = COALESCE(?5, notes),
    tags = COALESCE(?6, tags),
    updated_at = ?7
WHERE id = ?8
```

## Rust Implementation

### Key Dependencies

```toml
[dependencies]
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio", "migrate"] }
aes-gcm = "0.10"
argon2 = "0.5"
rand = "0.8"
anyhow = "1"
tokio = { version = "1", features = ["full"] }
chrono = "0.4"
```

### Encrypting a Field

```rust
use vault_core::crypto::encrypt_str;

async fn add_password(key: &[u8], db: &Path, title: &str, password: &str) -> Result<()> {
    let encrypted_password = encrypt_str(key, password)?;
    // Store encrypted_password as BLOB in SQLite
    Ok(())
}
```

### Decrypting a Field

```rust
use vault_core::crypto::decrypt_str;

fn get_password(key: &[u8], encrypted_blob: &[u8]) -> Result<String> {
    let plaintext = decrypt_str(key, encrypted_blob)?;
    Ok(plaintext)
}
```

### Database Operations

The `store` module in `vault-core` provides async functions:

```rust
pub async fn init_db(db_path: &Path, salt: &[u8], ...) -> Result<()>;
pub async fn read_meta(db_path: &Path) -> Result<VaultMeta>;
pub async fn add_entry(key: &[u8], db_path: &Path, entry: NewEntry) -> Result<i64>;
pub async fn get_entry(key: &[u8], db_path: &Path, query: &str) -> Result<Option<VaultEntry>>;
pub async fn list_entries(key: &[u8], db_path: &Path, search: &str) -> Result<Vec<VaultEntry>>;
pub async fn update_entry(key: &[u8], db_path: &Path, update: EntryUpdate) -> Result<()>;
pub async fn delete_entry(_key: &[u8], db_path: &Path, id: i64) -> Result<()>;
```

## Security Considerations

### Threat Model

**Protected against:**
- Database file theft (passwords encrypted at rest)
- Memory dumps while locked (key not in memory)
- Brute-force attacks (Argon2id with high memory cost)
- Tampering (GCM authentication tags)

**Not protected against:**
- Compromised master password
- Active malware on running system (while unlocked)
- Side-channel attacks on the daemon process

### Best Practices

1. **Key Never Stored**: The derived key is never written to disk
2. **Auto-Lock**: Daemon wipes key from memory after inactivity
3. **Secure Memory**: Uses `zeroize` to clear sensitive buffers
4. **Constant-Time**: HMAC verification uses constant-time comparison
5. **Unique Nonces**: Each encryption uses a fresh random nonce

## Testing

The database layer includes comprehensive tests:

```bash
cargo test -p vault-core
```

Tests cover:
- Database initialization and schema creation
- Meta data read/write
- Entry add/get/update/delete operations
- Encryption/decryption roundtrips
- Search functionality

## Migration Notes

Future schema migrations can use sqlx's built-in migration system:

```rust
sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;
```

Migrations should:
1. Preserve existing encrypted data
2. Only modify plaintext schema or add new columns
3. Never re-encrypt existing data without user action
