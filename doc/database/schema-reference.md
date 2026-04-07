# Database Schema Reference

Complete reference for KeyHaven's SQLite database schema.

## Tables

- [`vault_meta`](#vault_meta) — Vault cryptographic parameters
- [`entries`](#entries) — Password entries

---

## vault_meta

Stores the vault's master cryptographic parameters. This table contains exactly one row (id = 1).

### Columns

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `id` | `INTEGER` | No | Fixed value: `1` (enforced by `CHECK (id = 1)`) |
| `argon2_salt` | `BLOB` | No | 32-byte random salt for Argon2id key derivation |
| `argon2_m_cost` | `INTEGER` | No | Memory cost in KB (default: `65536` = 64 MB) |
| `argon2_t_cost` | `INTEGER` | No | Time iterations (default: `3`) |
| `argon2_p_cost` | `INTEGER` | No | Parallelism lanes (default: `4`) |
| `verification_hmac` | `BLOB` | No | HMAC-SHA256 of `"vault-v1-ok"` for password verification |

### Constraints

| Constraint | Definition | Purpose |
|------------|------------|---------|
| `PRIMARY KEY` | `id` | Single row enforcement |
| `CHECK` | `id = 1` | Prevents multiple meta rows |

### Example Data

```
id | argon2_salt | argon2_m_cost | argon2_t_cost | argon2_p_cost | verification_hmac
---|-------------|---------------|---------------|---------------|------------------
1  | <32 bytes>  | 65536         | 3             | 4             | <32 bytes>
```

### Rust Type

```rust
pub struct VaultMeta {
    pub argon2_salt: Vec<u8>,        // 32 bytes
    pub argon2_m_cost: u32,          // e.g., 65536
    pub argon2_t_cost: u32,          // e.g., 3
    pub argon2_p_cost: u32,          // e.g., 4
    pub verification_hmac: Vec<u8>,  // 32 bytes
}
```

---

## entries

Stores password entries with individually encrypted sensitive fields.

### Columns

| Column | Type | Nullable | Storage | Description |
|--------|------|----------|---------|-------------|
| `id` | `INTEGER` | No | — | Auto-increment primary key |
| `title` | `TEXT` | No | Plaintext | Entry name/title (searchable) |
| `username` | `BLOB` | Yes | **Encrypted** | User identifier |
| `password` | `BLOB` | No | **Encrypted** | The secret password |
| `url` | `TEXT` | Yes | Plaintext | Website URL (searchable) |
| `notes` | `BLOB` | Yes | **Encrypted** | Additional notes |
| `tags` | `TEXT` | Yes | Plaintext | Comma-separated tags (searchable) |
| `created_at` | `INTEGER` | No | — | Unix timestamp (seconds since epoch) |
| `updated_at` | `INTEGER` | No | — | Unix timestamp (seconds since epoch) |

### Constraints

| Constraint | Definition |
|------------|------------|
| `PRIMARY KEY` | `id` (auto-increment) |

### Indexes

Currently, the database relies on SQLite's default indexes:
- `PRIMARY KEY` on `id` creates an implicit index
- `TEXT` fields support `LIKE` queries for searching

### Example Data (Logical View)

| id | title | username | password | url | notes | tags | created_at | updated_at |
|----|-------|----------|----------|-----|-------|------|------------|------------|
| 1 | GitHub | user@example.com | my-secret-pass | https://github.com | Personal account | dev,important | 1704067200 | 1704067200 |
| 2 | Gmail | user@gmail.com | another-password | https://gmail.com | — | email | 1704153600 | 1704153600 |

### Physical Storage

In the actual database, encrypted fields store binary blobs:

```
title:    "GitHub" (TEXT)
username: <nonce: 12 bytes><ciphertext: 24 bytes><tag: 16 bytes> (BLOB)
password: <nonce: 12 bytes><ciphertext: 20 bytes><tag: 16 bytes> (BLOB)
url:      "https://github.com" (TEXT)
notes:    <nonce: 12 bytes><ciphertext: 18 bytes><tag: 16 bytes> (BLOB)
tags:     "dev,important" (TEXT)
```

### Rust Types

```rust
/// Full entry as returned from database
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultEntry {
    pub id: i64,
    pub title: String,
    pub username: Option<String>,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// New entry for insertion
#[derive(Debug, Deserialize, Clone)]
pub struct NewEntry {
    pub title: String,
    pub username: Option<String>,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Update to an existing entry (all fields optional)
#[derive(Debug, Deserialize, Clone)]
pub struct EntryUpdate {
    pub id: i64,
    pub title: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
}
```

---

## Field Storage Reference

### Plaintext Fields

| Field | SQLite Type | Search | Index | Notes |
|-------|-------------|--------|-------|-------|
| `title` | `TEXT NOT NULL` | Full-text, `LIKE` | Implicit | Primary identifier |
| `url` | `TEXT` | Full-text, `LIKE` | No | Web address |
| `tags` | `TEXT` | Full-text, `LIKE` | No | Comma-separated values |

### Encrypted Fields

| Field | SQLite Type | Encrypted At Rest | Decrypted In |
|-------|-------------|-------------------|--------------|
| `username` | `BLOB` | Yes | Daemon memory (transient) |
| `password` | `BLOB NOT NULL` | Yes | Daemon memory (transient) |
| `notes` | `BLOB` | Yes | Daemon memory (transient) |

### Timestamp Fields

| Field | SQLite Type | Format | Source |
|-------|-------------|--------|--------|
| `created_at` | `INTEGER NOT NULL` | Unix timestamp (seconds) | `chrono::Utc::now().timestamp()` |
| `updated_at` | `INTEGER NOT NULL` | Unix timestamp (seconds) | `chrono::Utc::now().timestamp()` |

---

## SQL DDL

Complete schema definition:

```sql
-- Vault metadata table
CREATE TABLE IF NOT EXISTS vault_meta (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    argon2_salt BLOB NOT NULL,
    argon2_m_cost INTEGER NOT NULL,
    argon2_t_cost INTEGER NOT NULL,
    argon2_p_cost INTEGER NOT NULL,
    verification_hmac BLOB NOT NULL
);

-- Entries table
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

---

## Common Queries

### Search Entries

```sql
SELECT id, title, username, password, url, notes, tags, created_at, updated_at
FROM entries
WHERE title LIKE '%' || ? || '%'
   OR url LIKE '%' || ? || '%'
   OR tags LIKE '%' || ? || '%'
ORDER BY title;
```

### Get Entry by ID

```sql
SELECT id, title, username, password, url, notes, tags, created_at, updated_at
FROM entries
WHERE id = ?;
```

### Get Entry by Exact Title

```sql
SELECT id, title, username, password, url, notes, tags, created_at, updated_at
FROM entries
WHERE title = ?;
```

### Insert New Entry

```sql
INSERT INTO entries (title, username, password, url, notes, tags, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?, ?, ?);
```

### Update Entry (Partial)

```sql
UPDATE entries SET
    title = COALESCE(?, title),
    username = COALESCE(?, username),
    password = COALESCE(?, password),
    url = COALESCE(?, url),
    notes = COALESCE(?, notes),
    tags = COALESCE(?, tags),
    updated_at = ?
WHERE id = ?;
```

### Delete Entry

```sql
DELETE FROM entries WHERE id = ?;
```

---

## Binary Format

### Encrypted Field Structure

Each encrypted BLOB field follows this layout:

```
+--------+---------------------------------------------------+
| Nonce  | Ciphertext + Auth Tag                               |
| 12 bytes | N + 16 bytes                                      |
+--------+---------------------------------------------------+
```

- **Nonce**: 96 bits (12 bytes), randomly generated per encryption
- **Ciphertext**: Variable length (original plaintext length)
- **Auth Tag**: 128 bits (16 bytes), GCM authentication tag

Total overhead: **28 bytes** per encrypted field

### Example Encryption

Plaintext: `"user@example.com"` (18 bytes)

Encrypted blob:
```
Offset 0-11:   Nonce (12 bytes)              → "a3f9e2..."
Offset 12-29:  Ciphertext (18 bytes)         → (encrypted data)
Offset 30-45:  Auth Tag (16 bytes)           → (integrity check)
```

Total: 46 bytes stored in `username` BLOB column

---

## See Also

- [database.md](./database.md) — Architecture and design overview
- `vault-core/src/store.rs` — Implementation source
- `vault-core/src/crypto.rs` — Encryption primitives
