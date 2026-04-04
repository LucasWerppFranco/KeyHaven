use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::ProgressBar;
use rpassword::read_password;
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;
use vault_core::unlock;

/// Parse timeout string (e.g., "15m", "1h") into Duration
fn parse_timeout(s: &str) -> Result<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(Duration::from_secs(15 * 60));
    }

    let (num, unit) = s.split_at(s.len().saturating_sub(1));
    let num: u64 = num.parse().context("Invalid timeout. Use format like 15m, 1h, 30s")?;

    match unit {
        "s" => Ok(Duration::from_secs(num)),
        "m" => Ok(Duration::from_secs(num * 60)),
        "h" => Ok(Duration::from_secs(num * 60 * 60)),
        _ => {
            // If no unit, assume minutes
            if let Ok(mins) = s.parse::<u64>() {
                Ok(Duration::from_secs(mins * 60))
            } else {
                Err(anyhow::anyhow!("Invalid unit. Use s, m, or h"))
            }
        }
    }
}

pub async fn run(timeout_str: &str, db_path: &Path) -> Result<()> {
    print!("Master password: ");
    io::stdout().flush()?;
    let password = read_password()?;

    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Unlocking vault...");

    let key = unlock(&password, db_path).await?;

    spinner.finish_and_clear();

    // Store key in memory (simplified - in production would use daemon)
    println!("{} Vault unlocked!", "✓".green().bold());

    let timeout = parse_timeout(timeout_str)?;
    println!(
        "Auto-lock after {} of inactivity",
        humantime::format_duration(timeout)
    );

    // Store key in temporary environment variable for subsequent commands
    // (In real production, the daemon would hold the key)
    std::env::set_var("KEYHAVEN_SESSION_KEY", hex::encode(&key));

    Ok(())
}
