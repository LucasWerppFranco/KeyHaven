use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::io::Write;
use std::process::{Command, Stdio};

pub async fn run(
    type_mode: bool,
    key: &[u8],
    db_path: &std::path::Path,
) -> Result<()> {
    // List all entries
    let entries = vault_core::list_entries(key, db_path, "").await?;

    if entries.is_empty() {
        bail!("No entries in vault");
    }

    // Format for rofi stdin
    let lines: Vec<String> = entries
        .iter()
        .map(|e| {
            if let Some(user) = &e.username {
                format!("{}: {}", e.title, user)
            } else {
                e.title.clone()
            }
        })
        .collect();

    // Try rofi first, fallback to wofi
    let rofi_cmd = if Command::new("rofi").arg("--version").output().is_ok() {
        "rofi"
    } else if Command::new("wofi").arg("--version").output().is_ok() {
        "wofi"
    } else {
        bail!("No launcher found. Install rofi or wofi.");
    };

    let output = if rofi_cmd == "rofi" {
        Command::new("rofi")
            .args([
                "-dmenu",
                "-i",
                "-p",
                "vault",
                "-format",
                "i",
                "-matching",
                "fuzzy",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
    } else {
        Command::new("wofi")
            .args(["--dmenu", "-i", "-p", "vault", "--matching", "fuzzy"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
    };

    let mut child = output.context("Failed to spawn launcher")?;

    // Write entries to stdin
    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        stdin.write_all(lines.join("\n").as_bytes())?;
    }

    let result = child.wait_with_output()?;

    if !result.status.success() {
        // User cancelled (ESC or clicked outside)
        return Ok(());
    }

    let selected = String::from_utf8(result.stdout)?.trim().to_string();

    if selected.is_empty() {
        return Ok(());
    }

    let idx: usize = selected.parse().context("Invalid index")?;
    let entry = entries.get(idx).context("Entry not found")?;

    if type_mode {
        // Type via ydotool (works on Wayland without clipboard)
        let status = Command::new("ydotool")
            .args(["type", &entry.password])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("Password typed via ydotool");
            }
            _ => {
                println!(
                    "{} ydotool failed. Copying to clipboard...",
                    "⚠".yellow()
                );
                copy_to_clipboard(&entry.password).await?;
                println!("{} Password copied!", "✓".green());

                // Schedule clearing
                tokio::spawn(async {
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                    let _ = Command::new("wl-copy").arg("--clear").spawn();
                    let _ = Command::new("xclip")
                        .args(["-selection", "clipboard", "/dev/null"])
                        .spawn();
                });
            }
        }
    } else {
        // Copy to clipboard
        copy_to_clipboard(&entry.password).await?;
        println!("{} Password copied to clipboard!", "✓".green().bold());
        println!("   (will be cleared in 30 seconds)");

        // Schedule clearing in 30s
        let _ = tokio::spawn(async {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            let _ = Command::new("wl-copy").arg("--clear").spawn();
            let _ = Command::new("xclip")
                .args(["-selection", "clipboard", "/dev/null"])
                .spawn();
        });
    }

    Ok(())
}

async fn copy_to_clipboard(text: &str) -> Result<()> {
    // Try wl-copy first (Wayland)
    if let Ok(mut child) = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
    {
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
