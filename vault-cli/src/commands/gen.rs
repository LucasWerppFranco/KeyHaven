use anyhow::Result;
use colored::Colorize;
use std::io::IsTerminal;
use std::io::Write;
use std::process::{Command, Stdio};

pub async fn run(
    length: usize,
    words: Option<usize>,
    symbols: bool,
    copy: bool,
) -> Result<()> {
    // Display command cover art
    crate::ascii::display_command_cover("gen");

    let password = if let Some(word_count) = words {
        // Generate Diceware-style passphrase
        vault_core::generate_passphrase(word_count)
    } else {
        // Generate random password
        vault_core::generate_password(length, symbols)
    };

    if copy {
        // Copy to clipboard
        copy_to_clipboard(&password).await?;
        if std::io::stdout().is_terminal() {
            println!("{} Password copied to clipboard!", "✓".green().bold());
        } else {
            println!("copied");
        }
        return Ok(());
    }

    if std::io::stdout().is_terminal() {
        println!();
        println!("{}", "Generated password:".bold());
        println!("{}", password.green().bold());
        println!();

        // Show password strength
        let strength = vault_core::check_strength(&password);
        match strength.score {
            0 | 1 => {
                println!("{} Strength: {} ({} bits)", "⚠".red(), strength.label.red(), strength.entropy_bits);
            }
            2 => {
                println!("{} Strength: {} ({} bits)", "~".yellow(), strength.label.yellow(), strength.entropy_bits);
            }
            _ => {
                println!("{} Strength: {} ({} bits)", "✓".green(), strength.label.green(), strength.entropy_bits);
            }
        }

        println!();
        println!(
            "{}",
            "Tip: Add --copy to copy directly to clipboard".dimmed()
        );
    } else {
        // Clean output for pipes
        println!("{}", password);
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
