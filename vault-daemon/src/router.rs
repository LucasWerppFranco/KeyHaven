use serde_json::json;

use crate::protocol::{Action, Request, Response};
use crate::session::AppState;

/// Central dispatch point: reads the action and calls the correct handler.
pub async fn dispatch(req: Request, state: &AppState) -> Response {
    match req.action {
        // ── Session ─────────────────────────────────────────────
        Action::Unlock => handle_unlock(req.params, state).await,
        Action::Lock   => handle_lock(state).await,
        Action::Status => handle_status(state).await,

        // ── Entries ─────────────────────────────────────────────
        Action::ListEntries   => handle_list(req.params, state).await,
        Action::GetEntry      => handle_get(req.params, state).await,
        Action::AddEntry      => handle_add(req.params, state).await,
        Action::UpdateEntry   => handle_update(req.params, state).await,
        Action::DeleteEntry   => handle_delete(req.params, state).await,

        // ── Generator ────────────────────────────────────────────
        Action::GeneratePassword => handle_generate(req.params),
        Action::CheckPassword    => handle_check(req.params),
    }
}

// ── Session handlers ──────────────────────────────────────────────

async fn handle_unlock(params: serde_json::Value, state: &AppState) -> Response {
    let master_password = match params["password"].as_str() {
        Some(p) => p.to_string(),
        None => return Response::err("Field 'password' is required"),
    };

    // Delegate to vault-core: read salt from vault.db, derive key, verify HMAC
    match vault_core::unlock(&master_password, &state.config.db_path).await {
        Ok(derived_key) => {
            state.unlock(derived_key).await;
            let timeout = state.config.session_timeout;
            Response::ok(json!({
                "message": "Vault unlocked",
                "timeout_secs": timeout.as_secs()
            }))
        }
        Err(e) => Response::err(e.to_string()),
    }
}

async fn handle_lock(state: &AppState) -> Response {
    state.lock().await;
    Response::ok(json!({ "message": "Vault locked" }))
}

async fn handle_status(state: &AppState) -> Response {
    let unlocked = state.is_unlocked().await;
    Response::ok(json!({ "unlocked": unlocked }))
}

// ── Entry handlers ────────────────────────────────────────────────

async fn handle_list(params: serde_json::Value, state: &AppState) -> Response {
    let search = params["search"].as_str().unwrap_or("").to_string();
    let db_path = state.config.db_path.clone();

    state.with_key(|key| {
        let key = key.clone();
        async move {
            let entries = vault_core::list_entries(&key, &db_path, &search).await?;
            Ok::<_, crate::session::VaultError>(Response::ok(entries))
        }
    }).await.unwrap_or_else(|e| Response::err(e.to_string()))
}

async fn handle_get(params: serde_json::Value, state: &AppState) -> Response {
    let query = match params["query"].as_str() {
        Some(q) => q.to_string(),
        None => return Response::err("Field 'query' is required"),
    };
    let db_path = state.config.db_path.clone();

    state.with_key(|key| {
        let key = key.clone();
        async move {
            match vault_core::get_entry(&key, &db_path, &query).await? {
                Some(entry) => Ok::<_, crate::session::VaultError>(Response::ok(entry)),
                None => Err(crate::session::VaultError::NotFound),
            }
        }
    }).await.unwrap_or_else(|e| Response::err(e.to_string()))
}

async fn handle_add(params: serde_json::Value, state: &AppState) -> Response {
    let db_path = state.config.db_path.clone();

    state.with_key(|key| {
        let key = key.clone();
        async move {
            let entry: vault_core::NewEntry = serde_json::from_value(params.clone())
                .map_err(|e| anyhow::anyhow!("Invalid params: {e}"))?;
            let id = vault_core::add_entry(&key, &db_path, entry).await?;
            Ok::<_, crate::session::VaultError>(Response::ok(json!({ "id": id })))
        }
    }).await.unwrap_or_else(|e| Response::err(e.to_string()))
}

async fn handle_update(params: serde_json::Value, state: &AppState) -> Response {
    let db_path = state.config.db_path.clone();

    state.with_key(|key| {
        let key = key.clone();
        async move {
            let update: vault_core::EntryUpdate = serde_json::from_value(params.clone())
                .map_err(|e| anyhow::anyhow!("Invalid params: {e}"))?;
            vault_core::update_entry(&key, &db_path, update).await?;
            Ok::<_, crate::session::VaultError>(Response::ok(json!({ "updated": true })))
        }
    }).await.unwrap_or_else(|e| Response::err(e.to_string()))
}

async fn handle_delete(params: serde_json::Value, state: &AppState) -> Response {
    let id = match params["id"].as_i64() {
        Some(i) => i,
        None => return Response::err("Field 'id' is required"),
    };
    let db_path = state.config.db_path.clone();

    state.with_key(|key| {
        let key = key.clone();
        async move {
            vault_core::delete_entry(&key, &db_path, id).await?;
            Ok::<_, crate::session::VaultError>(Response::ok(json!({ "deleted": true })))
        }
    }).await.unwrap_or_else(|e| Response::err(e.to_string()))
}

// ── Generator handlers (no session required) ─────────────────────

fn handle_generate(params: serde_json::Value) -> Response {
    let length = params["length"].as_u64().unwrap_or(20) as usize;
    let use_symbols = params["symbols"].as_bool().unwrap_or(true);
    let words = params["words"].as_u64().map(|w| w as usize);

    let password = if let Some(word_count) = words {
        vault_core::generate_passphrase(word_count)
    } else {
        vault_core::generate_password(length, use_symbols)
    };

    let strength = vault_core::check_strength(&password);
    Response::ok(json!({
        "password": password,
        "entropy_bits": strength.entropy_bits,
        "score": strength.score,          // 0-4
        "label": strength.label,          // "weak", "fair", "strong"...
    }))
}

fn handle_check(params: serde_json::Value) -> Response {
    let password = match params["password"].as_str() {
        Some(p) => p,
        None => return Response::err("Field 'password' is required"),
    };

    let strength = vault_core::check_strength(password);
    Response::ok(json!({
        "entropy_bits": strength.entropy_bits,
        "score": strength.score,
        "label": strength.label,
        "warning": strength.warning,
    }))
}
