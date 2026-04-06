use anyhow::Result;
use colored::Colorize;
use rpassword::prompt_password;
use std::io::{self, Write};

pub async fn run(
    url: Option<String>,
    gen: bool,
    key: &[u8],
    db_path: &std::path::Path,
) -> Result<()> {
    println!();
    println!("{}", "  adding new entry".dimmed());

    // Title
    crate::ascii::print_minimal_prompt("title: ");
    io::stdout().flush()?;
    let mut title = String::new();
    io::stdin().read_line(&mut title)?;
    let title = title.trim();

    if title.is_empty() {
        return Err(anyhow::anyhow!("Title is required"));
    }

    // Username (optional)
    crate::ascii::print_optional_prompt("username");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim();
    let username = if username.is_empty() {
        None
    } else {
        Some(username.to_string())
    };

    // Password
    let password = if gen {
        let pwd = vault_core::generate_password(20, true);
        println!("  {} generated password: {}", "→".dimmed(), pwd.green());
        println!("  press enter to continue...");
        io::stdin().read_line(&mut String::new())?;
        pwd
    } else {
        crate::ascii::print_minimal_prompt("password: ");
        let pwd1 = prompt_password("")?;
        if pwd1.is_empty() {
            return Err(anyhow::anyhow!("Password cannot be empty"));
        }
        crate::ascii::print_minimal_prompt("confirm password: ");
        let pwd2 = prompt_password("")?;
        if pwd1 != pwd2 {
            return Err(anyhow::anyhow!("Passwords do not match"));
        }
        pwd1
    };

    // URL
    let url = url.or_else(|| {
        crate::ascii::print_optional_prompt("url");
        io::stdout().flush().ok()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok()?;
        let trimmed = input.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    // Notes
    crate::ascii::print_optional_prompt("notes");
    io::stdout().flush()?;
    let mut notes = String::new();
    io::stdin().read_line(&mut notes)?;
    let notes = notes.trim();
    let notes = if notes.is_empty() {
        None
    } else {
        Some(notes.to_string())
    };

    // Create the entry
    let entry = vault_core::NewEntry {
        title: title.to_string(),
        username,
        password,
        url,
        notes,
        tags: None,
    };

    let id = vault_core::add_entry(key, db_path, entry).await?;

    println!();
    crate::ascii::print_separator();
    println!();
    println!("{} entry added", "✓".green().bold());
    println!("  {} id: {}", "→".dimmed(), id.to_string().cyan());

    Ok(())
}
