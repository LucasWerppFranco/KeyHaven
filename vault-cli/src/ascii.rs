//! ASCII art utilities for the CLI

use colored::Colorize;
use serde::Deserialize;
use std::collections::HashMap;
use terminal_size::{terminal_size, Width};

const COVER_ART: &str = r#"┌──────────────────────────────────────────────────────────────────────┬──────────────────────────────┐
│                                                                      │                              │
│ ██╗  ██╗███████╗██╗   ██╗██╗  ██╗ █████╗ ██╗   ██╗███████╗███╗   ██╗ │                    █████     │
│ ██║ ██╔╝██╔════╝╚██╗ ██╔╝██║  ██║██╔══██╗██║   ██║██╔════╝████╗  ██║ │                    █   █     │
│ █████╔╝ █████╗   ╚████╔╝ ███████║███████║██║   ██║█████╗  ██╔██╗ ██║ │                    █         │
│ ██╔═██╗ ██╔══╝    ╚██╔╝  ██╔══██║██╔══██║╚██╗ ██╔╝██╔══╝  ██║╚██╗██║ │   ████           █████████   │
│ ██║  ██╗███████╗   ██║   ██║  ██║██║  ██║ ╚████╔╝ ███████╗██║ ╚████║ │   █  █████████   ████ ████   │
│ ╚═╝  ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝  ╚═══╝  ╚══════╝╚═╝  ╚═══╝ │   ████     █ █   ████ ████   │
│                                                                      │                  █████████   │
│https://github.com/LucasWerppFranco/KeyHaven                   0.1.0v │                              │
└──────────────────────────────────────────────────────────────────────┴──────────────────────────────┘"#;

const DEFAULT_SEPARATOR_LEN: usize = 70;
const SEPARATOR_CHAR: char = '─';
const MIN_SEPARATOR_LEN: usize = 40;
const MAX_SEPARATOR_LEN: usize = 120;

/// Cover art data structure from JSON
#[derive(Deserialize)]
struct CoverData {
    #[serde(flatten)]
    arts: HashMap<String, Vec<String>>,
}

/// Load cover arts from JSON file
fn load_cover_arts() -> HashMap<String, Vec<String>> {
    let json_content = include_str!("../../ascii/cover.json");
    serde_json::from_str::<CoverData>(json_content)
        .map(|data| data.arts)
        .unwrap_or_default()
}

/// Display the cover art
pub fn display_cover() {
    println!("{}", COVER_ART.cyan());
}

/// Display the cover art for a specific command
pub fn display_command_cover(command: &str) {
    let arts = load_cover_arts();

    if let Some(art_lines) = arts.get(command) {
        // Join lines with newlines and print
        let art = art_lines.join("\n");
        println!("{}", art.cyan());
    } else {
        // Fallback to default cover if command not found
        display_cover();
    }
}

/// Get the terminal width, clamped between MIN and MAX
fn get_terminal_width() -> usize {
    terminal_size()
        .map(|(Width(w), _)| {
            let width = w as usize;
            // Account for some padding (e.g., prompt symbols)
            let adjusted = width.saturating_sub(4);
            adjusted.clamp(MIN_SEPARATOR_LEN, MAX_SEPARATOR_LEN)
        })
        .unwrap_or(DEFAULT_SEPARATOR_LEN)
}

/// Get a dynamic separator line that fits the terminal width
pub fn get_separator() -> String {
    let width = get_terminal_width();
    SEPARATOR_CHAR.to_string().repeat(width)
}

/// Print the separator line (dimmed)
pub fn print_separator() {
    println!("{}", get_separator().dimmed());
}

/// Print a minimal prompt with dynamic separator
pub fn print_minimal_prompt(label: &str) {
    println!();
    print_separator();
    print!("{} {}", "❯".cyan().bold(), label);
}

/// Print an optional field prompt with dynamic separator
pub fn print_optional_prompt(label: &str) {
    println!();
    print_separator();
    print!("{} {} {}: ", "❯".cyan().bold(), label, "[optional]".dimmed());
}
