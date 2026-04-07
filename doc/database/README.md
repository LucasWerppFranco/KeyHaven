# Database Documentation

This directory contains documentation for KeyHaven's SQLite database layer.

## Overview

KeyHaven uses SQLite with **field-by-field encryption** — sensitive data (passwords, usernames, notes) is individually encrypted with AES-256-GCM before storage, while searchable fields (title, URL, tags) remain in plaintext.

## Available Documentation

- **[database.md](./database.md)** — Complete guide to the database architecture, schema, and encryption model
- **[schema-reference.md](./schema-reference.md)** — Detailed reference of tables, columns, and data types

## Quick Reference

### Database Location
- Default: `~/.local/share/vault/vault.db`
- Configurable via `VAULT_DB_PATH` environment variable

### Schema Overview

```sql
-- Vault metadata (Argon2id parameters)
CREATE TABLE vault_meta (
    id                INTEGER PRIMARY KEY CHECK (id = 1),
    argon2_salt       BLOB    NOT NULL,
    argon2_m_cost     INTEGER NOT NULL,
    argon2_t_cost     INTEGER INTEGER NOT NULL,
    argon2_p_cost     INTEGER NOT NULL,
    verification_hmac BLOB    NOT NULL
);

-- Password entries
CREATE TABLE entries (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    title       TEXT    NOT NULL,          -- plaintext (searchable)
    username    BLOB,                      -- encrypted
    password    BLOB    NOT NULL,          -- encrypted
    url         TEXT,                      -- plaintext (searchable)
    notes       BLOB,                      -- encrypted
    tags        TEXT,                      -- plaintext (searchable)
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);
```

### Encryption Format

Encrypted fields stored as: `[nonce: 12 bytes || ciphertext: N bytes]`

Encrypted with AES-256-GCM using the key derived from the master password via Argon2id.

## Security Model

| Aspect | Implementation |
|--------|----------------|
| Key Derivation | Argon2id (OWASP 2024 recommended: 64MB, 3 iterations, 4 parallelism) |
| Encryption | AES-256-GCM per-field with unique random nonces |
| Verification | HMAC-SHA256 of "vault-v1-ok" for password verification |
| Key Storage | Never stored; held in daemon memory only while unlocked |

## Related Code

- Schema & CRUD: `vault-core/src/store.rs`
- Encryption: `vault-core/src/crypto.rs`
- Public API: `vault-core/src/lib.rs`
