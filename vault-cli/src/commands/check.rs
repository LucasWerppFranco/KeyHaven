use anyhow::Result;
use colored::Colorize;
use sha1::{Digest, Sha1};

pub async fn run(password: &str) -> Result<()> {
    println!("{}", "Checking password...".bold().cyan());
    println!();

    // Check local strength
    let strength = vault_core::check_strength(password);
    let label_colored = match strength.score {
        0 | 1 => strength.label.red().bold(),
        2 => strength.label.yellow().bold(),
        _ => strength.label.green().bold(),
    };
    println!(
        "{} Password strength: {} ({} bits)",
        match strength.score {
            0 | 1 => "⚠".red(),
            2 => "~".yellow(),
            _ => "✓".green(),
        },
        label_colored,
        strength.entropy_bits
    );
    if let Some(warning) = &strength.warning {
        println!("   {}", warning.yellow());
    }

    println!();

    // Check HIBP (Have I Been Pwned)
    println!(
        "{} Checking for breaches (HIBP)...",
        "🔍".dimmed()
    );

    match check_hibp(password).await {
        Ok(count) => {
            if count == 0 {
                println!(
                    "{} Password not found in known breaches",
                    "✓".green().bold()
                );
            } else {
                println!(
                    "{} Password found in {} breaches!",
                    "⚠".red().bold(),
                    count.to_string().red().bold()
                );
                println!(
                    "   {}",
                    "This password is not secure. Change it immediately!".red()
                );
            }
        }
        Err(e) => {
            println!("{} Unable to check HIBP: {}", "⚠".yellow(), e);
        }
    }

    Ok(())
}

/// Check password against Have I Been Pwned API using k-anonymity
async fn check_hibp(password: &str) -> Result<u64> {
    // Calculate SHA-1 of password
    let mut hasher = Sha1::new();
    hasher.update(password.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = hex::encode_upper(&hash);

    // Split into prefix (first 5 chars) and suffix
    let prefix = &hash_hex[..5];
    let suffix = &hash_hex[5..];

    // Make request to API
    let url = format!("https://api.pwnedpasswords.com/range/{}", prefix);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "KeyHaven-CLI")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("API returned error: {}", response.status()));
    }

    let body = response.text().await?;

    // Look for suffix in response
    let count = body
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 && parts[0] == suffix {
                parts[1].parse::<u64>().ok()
            } else {
                None
            }
        })
        .next()
        .unwrap_or(0);

    Ok(count)
}
