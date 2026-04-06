//! Storage module for vault entries.
//!
//! Handles SQLite database operations with encrypted fields.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use std::path::Path;

use crate::crypto::{decrypt_str, encrypt_str};
use sqlx::{Row, sqlite::SqliteConnectOptions};

/// Metadata stored in the vault database
#[derive(Debug)]
pub struct VaultMeta {
    pub argon2_salt: Vec<u8>,
    pub argon2_m_cost: u32,
    pub argon2_t_cost: u32,
    pub argon2_p_cost: u32,
    pub verification_hmac: Vec<u8>,
}

/// A vault entry (password record)
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

/// New entry to be added
#[derive(Debug, Deserialize, Clone)]
pub struct NewEntry {
    pub title: String,
    pub username: Option<String>,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Update to an existing entry
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

/// Initialize the database with metadata
pub async fn init_db(
    db_path: &Path,
    salt: &[u8],
    m_cost: u32,
    t_cost: u32,
    p_cost: u32,
    hmac: &[u8],
) -> Result<()> {
    // Create parent directory if needed
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Create database connection pool with create_if_missing
    let connect_options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .connect_with(connect_options)
        .await
        .with_context(|| format!("Failed to connect to database: {}", db_path.display()))?;

    // Create vault_meta table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS vault_meta (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            argon2_salt BLOB NOT NULL,
            argon2_m_cost INTEGER NOT NULL,
            argon2_t_cost INTEGER NOT NULL,
            argon2_p_cost INTEGER NOT NULL,
            verification_hmac BLOB NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .context("Failed to create vault_meta table")?;

    // Create entries table
    sqlx::query(
        r#"
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
        )
        "#,
    )
    .execute(&pool)
    .await
    .context("Failed to create entries table")?;

    // Insert metadata (single row, id = 1)
    sqlx::query(
        r#"
        INSERT INTO vault_meta (id, argon2_salt, argon2_m_cost, argon2_t_cost, argon2_p_cost, verification_hmac)
        VALUES (1, ?1, ?2, ?3, ?4, ?5)
        "#,
    )
    .bind(salt)
    .bind(m_cost as i64)
    .bind(t_cost as i64)
    .bind(p_cost as i64)
    .bind(hmac)
    .execute(&pool)
    .await
    .context("Failed to insert vault metadata")?;

    pool.close().await;

    Ok(())
}

/// Read vault metadata
pub async fn read_meta(db_path: &Path) -> Result<VaultMeta> {
    let connect_options = SqliteConnectOptions::new()
        .filename(db_path);

    let pool = SqlitePoolOptions::new()
        .connect_with(connect_options)
        .await
        .with_context(|| format!("Failed to connect to database: {}", db_path.display()))?;

    let row = sqlx::query(
        r#"
        SELECT argon2_salt, argon2_m_cost, argon2_t_cost, argon2_p_cost, verification_hmac
        FROM vault_meta
        WHERE id = 1
        "#,
    )
    .fetch_one(&pool)
    .await
    .context("Failed to read vault metadata - vault may not be initialized")?;

    let meta = VaultMeta {
        argon2_salt: row.get(0),
        argon2_m_cost: row.get::<i64, _>(1) as u32,
        argon2_t_cost: row.get::<i64, _>(2) as u32,
        argon2_p_cost: row.get::<i64, _>(3) as u32,
        verification_hmac: row.get(4),
    };

    pool.close().await;

    Ok(meta)
}

/// List entries matching a search query
pub async fn list_entries(key: &[u8], db_path: &Path, search: &str) -> Result<Vec<VaultEntry>> {
    let connect_options = SqliteConnectOptions::new()
        .filename(db_path);

    let pool = SqlitePoolOptions::new()
        .connect_with(connect_options)
        .await
        .with_context(|| format!("Failed to connect to database: {}", db_path.display()))?;

    let search_pattern = format!("%{}%", search);

    let rows = sqlx::query(
        r#"
        SELECT id, title, username, password, url, notes, tags, created_at, updated_at
        FROM entries
        WHERE title LIKE ?1 OR url LIKE ?1 OR tags LIKE ?1
        ORDER BY title
        "#,
    )
    .bind(&search_pattern)
    .fetch_all(&pool)
    .await
    .context("Failed to list entries")?;

    let mut entries = Vec::new();

    for row in rows {
        let username_blob: Option<Vec<u8>> = row.get::<Option<Vec<u8>>, _>(2);
        let password_blob: Vec<u8> = row.get::<Vec<u8>, _>(3);
        let notes_blob: Option<Vec<u8>> = row.get::<Option<Vec<u8>>, _>(5);

        let username = match username_blob {
            Some(blob) => Some(decrypt_str(key, &blob)?),
            None => None,
        };

        let password = decrypt_str(key, &password_blob)?;

        let notes = match notes_blob {
            Some(blob) => Some(decrypt_str(key, &blob)?),
            None => None,
        };

        entries.push(VaultEntry {
            id: row.get::<i64, _>(0),
            title: row.get::<String, _>(1),
            username,
            password,
            url: row.get::<Option<String>, _>(4),
            notes,
            tags: row.get::<Option<String>, _>(6),
            created_at: row.get::<i64, _>(7),
            updated_at: row.get::<i64, _>(8),
        });
    }

    pool.close().await;

    Ok(entries)
}

/// Get a single entry by query (ID or title match)
pub async fn get_entry(key: &[u8], db_path: &Path, query: &str) -> Result<Option<VaultEntry>> {
    let connect_options = SqliteConnectOptions::new()
        .filename(db_path);

    let pool = SqlitePoolOptions::new()
        .connect_with(connect_options)
        .await
        .with_context(|| format!("Failed to connect to database: {}", db_path.display()))?;

    // Try to parse query as ID first
    let entry = if let Ok(id) = query.parse::<i64>() {
        sqlx::query(
            r#"
            SELECT id, title, username, password, url, notes, tags, created_at, updated_at
            FROM entries
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&pool)
        .await
        .context("Failed to get entry by ID")?
    } else {
        // Search by title (exact match preferred, fallback to partial)
        sqlx::query(
            r#"
            SELECT id, title, username, password, url, notes, tags, created_at, updated_at
            FROM entries
            WHERE title = ?1
            "#,
        )
        .bind(query)
        .fetch_optional(&pool)
        .await
        .context("Failed to get entry by title")?
        .or(
            // Fallback to partial match
            sqlx::query(
                r#"
                SELECT id, title, username, password, url, notes, tags, created_at, updated_at
                FROM entries
                WHERE title LIKE ?1
                LIMIT 1
                "#,
            )
            .bind(format!("%{}%", query))
            .fetch_optional(&pool)
            .await
            .context("Failed to get entry by partial match")?
        )
    };

    let result = match entry {
        Some(row) => {
            let username_blob: Option<Vec<u8>> = row.get::<Option<Vec<u8>>, _>(2);
            let password_blob: Vec<u8> = row.get::<Vec<u8>, _>(3);
            let notes_blob: Option<Vec<u8>> = row.get::<Option<Vec<u8>>, _>(5);

            let username = match username_blob {
                Some(blob) => Some(decrypt_str(key, &blob)?),
                None => None,
            };

            let password = decrypt_str(key, &password_blob)?;

            let notes = match notes_blob {
                Some(blob) => Some(decrypt_str(key, &blob)?),
                None => None,
            };

            Some(VaultEntry {
                id: row.get::<i64, _>(0),
                title: row.get::<String, _>(1),
                username,
                password,
                url: row.get::<Option<String>, _>(4),
                notes,
                tags: row.get::<Option<String>, _>(6),
                created_at: row.get::<i64, _>(7),
                updated_at: row.get::<i64, _>(8),
            })
        }
        None => None,
    };

    pool.close().await;

    Ok(result)
}

/// Add a new entry
pub async fn add_entry(key: &[u8], db_path: &Path, entry: NewEntry) -> Result<i64> {
    let connect_options = SqliteConnectOptions::new()
        .filename(db_path);

    let pool = SqlitePoolOptions::new()
        .connect_with(connect_options)
        .await
        .with_context(|| format!("Failed to connect to database: {}", db_path.display()))?;

    // Encrypt sensitive fields
    let username_blob = match entry.username {
        Some(u) => Some(encrypt_str(key, &u)?),
        None => None,
    };
    let password_blob = encrypt_str(key, &entry.password)?;
    let notes_blob = match entry.notes {
        Some(n) => Some(encrypt_str(key, &n)?),
        None => None,
    };

    let tags_str = entry.tags.map(|t| t.join(","));

    let now = chrono::Utc::now().timestamp();

    let result = sqlx::query(
        r#"
        INSERT INTO entries (title, username, password, url, notes, tags, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
    )
    .bind(&entry.title)
    .bind(&username_blob)
    .bind(&password_blob)
    .bind(&entry.url)
    .bind(&notes_blob)
    .bind(&tags_str)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .context("Failed to insert entry")?;

    let id = result.last_insert_rowid();

    pool.close().await;

    Ok(id)
}

/// Update an existing entry
pub async fn update_entry(key: &[u8], db_path: &Path, update: EntryUpdate) -> Result<()> {
    let connect_options = SqliteConnectOptions::new()
        .filename(db_path);

    let pool = SqlitePoolOptions::new()
        .connect_with(connect_options)
        .await
        .with_context(|| format!("Failed to connect to database: {}", db_path.display()))?;

    let now = chrono::Utc::now().timestamp();

    // Build dynamic query based on which fields are provided
    let mut updates = Vec::new();
    let mut has_update = false;

    if update.title.is_some() {
        updates.push("title = ?1");
        has_update = true;
    }

    if update.username.is_some() {
        updates.push("username = ?2");
        has_update = true;
    }

    if update.password.is_some() {
        updates.push("password = ?3");
        has_update = true;
    }

    if update.url.is_some() {
        updates.push("url = ?4");
        has_update = true;
    }

    if update.notes.is_some() {
        updates.push("notes = ?5");
        has_update = true;
    }

    if update.tags.is_some() {
        updates.push("tags = ?6");
        has_update = true;
    }

    if !has_update {
        return Ok(());
    }

    updates.push("updated_at = ?7");

    let query_str = format!(
        "UPDATE entries SET {} WHERE id = ?8",
        updates.join(", ")
    );

    // Encrypt sensitive fields
    let username_blob = update
        .username
        .as_ref()
        .map(|u| encrypt_str(key, u))
        .transpose()?;
    let password_blob = update
        .password
        .as_ref()
        .map(|p| encrypt_str(key, p))
        .transpose()?;
    let notes_blob = update
        .notes
        .as_ref()
        .map(|n| encrypt_str(key, n))
        .transpose()?;

    let tags_str = update.tags.map(|t| t.join(","));

    let mut query = sqlx::query(&query_str);

    // Bind values in order
    if update.title.is_some() {
        query = query.bind(&update.title);
    }
    query = query.bind(&username_blob);
    query = query.bind(&password_blob);
    query = query.bind(&update.url);
    query = query.bind(&notes_blob);
    query = query.bind(&tags_str);
    query = query.bind(now);
    query = query.bind(update.id);

    query.execute(&pool).await.context("Failed to update entry")?;

    pool.close().await;

    Ok(())
}

/// Delete an entry by ID
pub async fn delete_entry(_key: &[u8], db_path: &Path, id: i64) -> Result<()> {
    let connect_options = SqliteConnectOptions::new()
        .filename(db_path);

    let pool = SqlitePoolOptions::new()
        .connect_with(connect_options)
        .await
        .with_context(|| format!("Failed to connect to database: {}", db_path.display()))?;

    sqlx::query("DELETE FROM entries WHERE id = ?1")
        .bind(id)
        .execute(&pool)
        .await
        .context("Failed to delete entry")?;

    pool.close().await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{compute_hmac, derive_key};
    use std::path::PathBuf;
    use tempfile::TempDir;

    async fn setup_test_db() -> (TempDir, PathBuf, Vec<u8>) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_vault.db");
        let password = "test-password-123";
        let salt = b"test-salt-32-bytes-exact!!!xxxx";

        let key = derive_key(password, salt, 65536, 3, 4).unwrap();
        let hmac = compute_hmac(&key).unwrap();

        init_db(&db_path, salt, 65536, 3, 4, &hmac).await.unwrap();

        (temp_dir, db_path, key)
    }

    #[tokio::test]
    async fn test_init_and_read_meta() {
        let (_temp_dir, db_path, _key) = setup_test_db().await;

        let meta = read_meta(&db_path).await.unwrap();

        assert_eq!(meta.argon2_m_cost, 65536);
        assert_eq!(meta.argon2_t_cost, 3);
        assert_eq!(meta.argon2_p_cost, 4);
        assert!(!meta.argon2_salt.is_empty());
        assert!(!meta.verification_hmac.is_empty());
    }

    #[tokio::test]
    async fn test_add_and_get_entry() {
        let (_temp_dir, db_path, key) = setup_test_db().await;

        let entry = NewEntry {
            title: "GitHub".to_string(),
            username: Some("user@example.com".to_string()),
            password: "my-secret-password".to_string(),
            url: Some("https://github.com".to_string()),
            notes: Some("Personal account".to_string()),
            tags: Some(vec!["dev".to_string(), "important".to_string()]),
        };

        let id = add_entry(&key, &db_path, entry).await.unwrap();
        assert_eq!(id, 1);

        let retrieved = get_entry(&key, &db_path, "GitHub").await.unwrap();
        assert!(retrieved.is_some());

        let entry = retrieved.unwrap();
        assert_eq!(entry.id, 1);
        assert_eq!(entry.title, "GitHub");
        assert_eq!(entry.username, Some("user@example.com".to_string()));
        assert_eq!(entry.password, "my-secret-password");
        assert_eq!(entry.url, Some("https://github.com".to_string()));
        assert_eq!(entry.notes, Some("Personal account".to_string()));
        assert_eq!(entry.tags, Some("dev,important".to_string()));
    }

    #[tokio::test]
    async fn test_list_entries() {
        let (_temp_dir, db_path, key) = setup_test_db().await;

        // Add two entries
        let entry1 = NewEntry {
            title: "GitHub".to_string(),
            username: Some("user@example.com".to_string()),
            password: "password1".to_string(),
            url: Some("https://github.com".to_string()),
            notes: None,
            tags: Some(vec!["dev".to_string()]),
        };

        let entry2 = NewEntry {
            title: "Gmail".to_string(),
            username: Some("user@gmail.com".to_string()),
            password: "password2".to_string(),
            url: Some("https://gmail.com".to_string()),
            notes: None,
            tags: Some(vec!["email".to_string()]),
        };

        add_entry(&key, &db_path, entry1).await.unwrap();
        add_entry(&key, &db_path, entry2).await.unwrap();

        // List all entries
        let entries = list_entries(&key, &db_path, "").await.unwrap();
        assert_eq!(entries.len(), 2);

        // Search by title
        let entries = list_entries(&key, &db_path, "Git").await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "GitHub");
    }

    #[tokio::test]
    async fn test_update_entry() {
        let (_temp_dir, db_path, key) = setup_test_db().await;

        let entry = NewEntry {
            title: "GitHub".to_string(),
            username: Some("old@example.com".to_string()),
            password: "old-password".to_string(),
            url: None,
            notes: None,
            tags: None,
        };

        add_entry(&key, &db_path, entry).await.unwrap();

        let update = EntryUpdate {
            id: 1,
            title: Some("GitHub Updated".to_string()),
            username: Some("new@example.com".to_string()),
            password: Some("new-password".to_string()),
            url: None,
            notes: None,
            tags: None,
        };

        update_entry(&key, &db_path, update).await.unwrap();

        let retrieved = get_entry(&key, &db_path, "GitHub Updated").await.unwrap();
        assert!(retrieved.is_some());
        let entry = retrieved.unwrap();
        assert_eq!(entry.username, Some("new@example.com".to_string()));
        assert_eq!(entry.password, "new-password");
    }

    #[tokio::test]
    async fn test_delete_entry() {
        let (_temp_dir, db_path, key) = setup_test_db().await;

        let entry = NewEntry {
            title: "GitHub".to_string(),
            username: None,
            password: "password".to_string(),
            url: None,
            notes: None,
            tags: None,
        };

        add_entry(&key, &db_path, entry).await.unwrap();
        assert!(get_entry(&key, &db_path, "GitHub").await.unwrap().is_some());

        delete_entry(&key, &db_path, 1).await.unwrap();
        assert!(get_entry(&key, &db_path, "GitHub").await.unwrap().is_none());
    }
}
