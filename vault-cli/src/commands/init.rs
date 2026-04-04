use anyhow::{Context, Result};
use colored::Colorize;
use rpassword::read_password;
use std::io::{self, Write};
use std::path::Path;
use vault_core::init_vault;

pub async fn run(db_path: &Path) -> Result<()> {
    if db_path.exists() {
        return Err(anyhow::anyhow!(
            "Vault already exists at {}. Use a different path or delete the file.",
            db_path.display()
        ));
    }

    // Create directory if needed
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    println!("{}", "Initializing new vault...".bold().cyan());
    println!();

    // Read master password
    print!("Master password: ");
    io::stdout().flush()?;
    let password = read_password()?;

    if password.len() < 12 {
        return Err(anyhow::anyhow!(
            "Password too short. Use at least 12 characters."
        ));
    }

    print!("Confirm password: ");
    io::stdout().flush()?;
    let confirm = read_password()?;

    if password != confirm {
        return Err(anyhow::anyhow!("Passwords do not match"));
    }

    // Initialize vault
    init_vault(&password, db_path).await?;

    println!();
    println!(
        "{} Vault initialized successfully at {}",
        "✓".green().bold(),
        db_path.display().to_string().cyan()
    );
    println!();
    println!(
        "{}",
        "IMPORTANT: Store your master password in a secure location. It cannot be recovered!"
            .yellow()
            .bold()
    );

    Ok(())
}
