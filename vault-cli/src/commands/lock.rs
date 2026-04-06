use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    // Clear the session (both file and environment)
    crate::session::clear_session()?;

    println!("{} Vault locked.", "🔒".yellow().bold());
    Ok(())
}
