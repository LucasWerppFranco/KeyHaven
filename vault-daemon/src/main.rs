use std::sync::Arc;
use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
mod protocol;
mod router;
mod session;
mod config;

use session::AppState;
use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    let config = Config::load()?;
    let socket_path = config.socket_path.clone();

    // Remove old socket if it exists (from a previous crash)
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    // Ensure the socket directory exists
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let state = Arc::new(AppState::new(config));
    let listener = UnixListener::bind(&socket_path)?;

    // Permissions 600: only the owner can connect
    std::fs::set_permissions(
        &socket_path,
        std::os::unix::fs::PermissionsExt::from_mode(0o600),
    )?;

    log::info!("vault-daemon listening on {:?}", socket_path);

    // Auto-lock background task
    let state_lock = Arc::clone(&state);
    tokio::spawn(async move {
        session::auto_lock_task(state_lock).await;
    });

    // Main loop: accept connections
    loop {
        let (stream, _) = listener.accept().await?;
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, state).await {
                log::warn!("Connection error: {e}");
            }
        });
    }
}

async fn handle_connection(
    mut stream: tokio::net::UnixStream,
    state: Arc<AppState>,
) -> anyhow::Result<()> {
    // Read message size (4 bytes big-endian) + JSON payload
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let msg_len = u32::from_be_bytes(len_buf) as usize;

    if msg_len > 1024 * 1024 {
        anyhow::bail!("Message too large: {msg_len} bytes");
    }

    let mut msg_buf = vec![0u8; msg_len];
    stream.read_exact(&mut msg_buf).await?;

    // Process the request
    let request: protocol::Request = serde_json::from_slice(&msg_buf)?;
    let id = request.id.clone();

    let response = router::dispatch(request, &state).await;

    // Serializa e envia a resposta
    let response_bytes = serde_json::to_vec(&protocol::Response {
        id,
        ..response
    })?;

    let resp_len = (response_bytes.len() as u32).to_be_bytes();
    stream.write_all(&resp_len).await?;
    stream.write_all(&response_bytes).await?;

    Ok(())
}
