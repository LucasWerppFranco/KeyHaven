use anyhow::Result;
use colored::Colorize;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, ContentArrangement, Table};
use std::io::IsTerminal;

pub async fn run(
    search: Option<String>,
    json_output: bool,
    key: &[u8],
    db_path: &std::path::Path,
) -> Result<()> {
    // Display command cover art
    crate::ascii::display_command_cover("list");
    println!();

    let entries = vault_core::list_entries(key, db_path, &search.unwrap_or_default()).await?;

    if entries.is_empty() {
        println!("No entries found.");
        return Ok(());
    }

    if json_output {
        // JSON output for scripts
        let json = serde_json::to_string_pretty(&entries)?;
        println!("{}", json);
        return Ok(());
    }

    let is_terminal = std::io::stdout().is_terminal();

    if is_terminal {
        // Formatted table
        let mut table = Table::new();
        table
            .set_content_arrangement(ContentArrangement::Dynamic)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec![
                "ID".dimmed().to_string(),
                "Title".bold().to_string(),
                "Username".bold().to_string(),
                "URL".dimmed().to_string(),
                "Modified".dimmed().to_string(),
            ]);

        for entry in &entries {
            let url = entry.url.as_ref().map(|u| u.as_str()).unwrap_or("-");
            let modified = chrono::DateTime::from_timestamp(entry.updated_at, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "-".to_string());

            table.add_row(vec![
                entry.id.to_string().dimmed().to_string(),
                entry.title.clone(),
                entry.username.clone().unwrap_or_else(|| "-".to_string()),
                url.to_string().dimmed().to_string(),
                modified.dimmed().to_string(),
            ]);
        }

        println!("{}", table);
        println!();
        println!("{} entries found", entries.len().to_string().cyan());
    } else {
        // Clean output for pipes
        for entry in &entries {
            let user = entry.username.as_ref().map(|s| s.as_str()).unwrap_or("");
            let url = entry.url.as_ref().map(|s| s.as_str()).unwrap_or("");
            println!("{}\t{}\t{}", entry.title, user, url);
        }
    }

    Ok(())
}
