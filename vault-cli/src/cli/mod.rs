use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "keyhaven", about = "Local password manager")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to the database (default: ~/.config/keyhaven/vault.db)
    #[arg(short, long, global = true)]
    pub db_path: Option<std::path::PathBuf>,

    /// Path to the daemon socket
    #[arg(short, long, global = true)]
    pub socket_path: Option<std::path::PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new vault
    Init,

    /// Unlock the vault with the master password
    Unlock {
        #[arg(long, default_value = "15m")]
        timeout: String,
    },

    /// Lock the vault immediately
    Lock,

    /// List all entries
    List {
        #[arg(short, long)]
        search: Option<String>,
        #[arg(long)]
        json: bool,
    },

    /// Search and display an entry
    Get {
        query: String,
        #[arg(long)]
        copy: bool,
        #[arg(long)]
        show: bool,
        #[arg(long)]
        field: Option<String>,
    },

    /// Add a new entry interactively
    Add {
        /// Entry title (optional, prompts if not provided)
        title: Option<String>,
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        gen: bool,
    },

    /// Generate a secure password
    Gen {
        #[arg(short, long, default_value = "20")]
        length: usize,
        #[arg(long)]
        words: Option<usize>,
        #[arg(long)]
        symbols: bool,
        #[arg(long)]
        copy: bool,
    },

    /// Check password strength and breaches
    Check { password: String },

    /// Open rofi/wofi selector for Hyprland
    Rofi {
        #[arg(long)]
        type_: bool,
    },
}
