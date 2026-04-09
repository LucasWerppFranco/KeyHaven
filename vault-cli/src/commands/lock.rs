use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    // Display command cover art
    crate::ascii::display_command_cover("lock");
    println!();

    // Clear the session (both file and environment)
    crate::session::clear_session()?;

    println!("{} Vault locked.", "🔒".yellow().bold());
    Ok(())
}
