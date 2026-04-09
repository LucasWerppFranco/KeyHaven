use anyhow::{bail, Result};
use colored::Colorize;
use std::io::IsTerminal;
use std::io::Write;
use std::process::{Command, Stdio};

pub async fn run(
    query: &str,
    copy: bool,
    show: bool,
    field: &Option<String>,
    key: &[u8],
    db_path: &std::path::Path,
) -> Result<()> {
    // Display command cover art
    crate::ascii::display_command_cover("get");
    println!();

    let entry = vault_core::get_entry(key, db_path, query).await?;

    let entry = match entry {
        Some(e) => e,
        None => {
            if field.is_some() {
                // Empty output for scripts when not found
                return Ok(());
            }
            bail!("Entry not found: {}", query);
        }
    };

    // If --field was specified, clean output
    if let Some(field_name) = field {
        let value = match field_name.as_str() {
            "password" | "senha" => entry.password.as_str(),
            "username" | "usuario" => entry.username.as_ref().map(|s| s.as_str()).unwrap_or(""),
            "title" | "titulo" => entry.title.as_str(),
            "url" => entry.url.as_ref().map(|s| s.as_str()).unwrap_or(""),
            "notes" | "notas" => entry.notes.as_ref().map(|s| s.as_str()).unwrap_or(""),
            _ => bail!("Unknown field: {}", field_name),
        };
        print!("{}", value);
        return Ok(());
    }

    // Check if we are in an interactive terminal
    let is_terminal = std::io::stdout().is_terminal();

    if copy {
        // Copy to clipboard
        copy_to_clipboard(&entry.password).await?;
        if is_terminal {
            println!("{} Password copied to clipboard!", "✓".green().bold());
            println!("   (will be cleared in 30 seconds)");

            // Schedule clipboard clearing
            let _ = tokio::spawn(async {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                clear_clipboard().await.ok();
            });
        }
        return Ok(());
    }

    // Display the entry
    if is_terminal {
        println!("{}", "═".repeat(50).dimmed());
        println!("  {} {}", "Title:".bold(), entry.title);

        if let Some(user) = &entry.username {
            println!("  {} {}", "Username:".bold(), user);
        }

        if let Some(url) = &entry.url {
            println!("  {} {}", "URL:".bold(), url.cyan().underline());
        }

        if show {
            println!("  {} {}", "Password:".bold(), entry.password.green());
        } else {
            println!("  {} {}", "Password:".bold(), "••••••••".dimmed());
            println!(
                "      {}",
                "Use --show to reveal or --copy to copy".dimmed()
            );
        }

        if let Some(notes) = &entry.notes {
            println!();
            println!("  {}", "Notes:".bold());
            for line in notes.lines() {
                println!("    {}", line);
            }
        }

        let created = chrono::DateTime::from_timestamp(entry.created_at, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "?".to_string());

        let updated = chrono::DateTime::from_timestamp(entry.updated_at, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "?".to_string());

        println!();
        println!(
            "  {} Created on {} | Modified on {}",
            "🕐".dimmed(),
            created.dimmed(),
            updated.dimmed()
        );
        println!("{}", "═".repeat(50).dimmed());
    } else {
        // Clean output for pipes
        println!("title:{}", entry.title);
        if let Some(user) = &entry.username {
            println!("username:{}", user);
        }
        if let Some(url) = &entry.url {
            println!("url:{}", url);
        }
        if show {
            println!("password:{}", entry.password);
        }
        if let Some(notes) = &entry.notes {
            println!("notes:{}", notes);
        }
    }

    Ok(())
}

async fn copy_to_clipboard(text: &str) -> Result<()> {
    // Try wl-copy first (Wayland)
    if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
        if let Some(stdin) = child.stdin.take() {
            let mut stdin = stdin;
            stdin.write_all(text.as_bytes())?;
        }
        return Ok(());
    }

    // Fallback to xclip (X11)
    if let Ok(mut child) = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(stdin) = child.stdin.take() {
            let mut stdin = stdin;
            stdin.write_all(text.as_bytes())?;
        }
        return Ok(());
    }

    Err(anyhow::anyhow!(
        "Unable to copy to clipboard. Install wl-copy or xclip."
    ))
}

async fn clear_clipboard() -> Result<()> {
    // Clear Wayland clipboard
    let _ = Command::new("wl-copy").arg("--clear").spawn();
    // Clear X11 clipboard
    let _ = Command::new("xclip")
        .args(["-selection", "clipboard", "/dev/null"])
        .spawn();
    Ok(())
}
