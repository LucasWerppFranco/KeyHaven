//! KeyHaven CLI - Command-line interface for the password manager

use anyhow::{Context, Result};
use clap::Parser;
use std::io::IsTerminal;

mod ascii;
mod cli;
mod commands;
mod session;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize colors only in interactive terminal
    if std::io::stdout().is_terminal() {
        colored::control::set_override(true);
    } else {
        colored::control::set_override(false);
    }

    // Display cover art at startup in interactive terminals
    if std::io::stdout().is_terminal() {
        ascii::display_cover();
    }

    let cli = Cli::parse();

    // Resolve paths
    let db_path = cli.db_path.unwrap_or_else(commands::default_db_path);
    let _socket_path = cli
        .socket_path
        .unwrap_or_else(commands::default_socket_path);

    // Dispatch commands
    match cli.command {
        Commands::Init => {
            commands::init::run(&db_path).await?;
        }

        Commands::Unlock { timeout } => {
            commands::unlock::run(&timeout, &db_path).await?;
        }

        Commands::Lock => {
            commands::lock::run()?;
        }

        Commands::List { search, json } => {
            let key = load_key()?;
            commands::list::run(search, json, &key, &db_path).await?;
        }

        Commands::Get {
            query,
            copy,
            show,
            field,
        } => {
            let key = load_key()?;
            commands::get::run(&query, copy, show, &field, &key, &db_path).await?;
        }

        Commands::Add { title, url, gen } => {
            let key = load_key()?;
            commands::add::run(title, url, gen, &key, &db_path).await?;
        }

        Commands::Gen {
            length,
            words,
            symbols,
            copy,
        } => {
            commands::gen::run(length, words, symbols, copy).await?;
        }

        Commands::Check { password } => {
            commands::check::run(&password).await?;
        }

        Commands::Rofi { type_ } => {
            let key = load_key()?;
            commands::rofi::run(type_, &key, &db_path).await?;
        }
    }

    Ok(())
}

/// Load session key from file or environment
fn load_key() -> Result<Vec<u8>> {
    session::load_session_key()
        .context("Vault is locked. Run 'keyhaven unlock' first.")
}
