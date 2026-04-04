use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    // Remove session environment variable
    std::env::remove_var("KEYHAVEN_SESSION_KEY");

    println!("{} Vault locked.", "🔒".yellow().bold());
    Ok(())
}
