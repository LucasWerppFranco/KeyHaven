use tauri::command;
use vault_core::{check_strength, generate_password, generate_passphrase, PasswordStrength};

/// Generate a random password with the given length and character sets
#[derive(serde::Deserialize)]
pub struct PasswordOptions {
    pub length: usize,
    pub include_symbols: bool,
}

#[derive(serde::Serialize)]
pub struct GeneratedPassword {
    pub password: String,
    pub strength: PasswordStrength,
}

#[command]
pub fn generate_password_cmd(options: PasswordOptions) -> Result<GeneratedPassword, String> {
    let password = generate_password(options.length, options.include_symbols);
    let strength = check_strength(&password);
    Ok(GeneratedPassword { password, strength })
}

/// Generate a passphrase (Diceware-style)
#[derive(serde::Deserialize)]
pub struct PassphraseOptions {
    pub word_count: usize,
}

#[derive(serde::Serialize)]
pub struct GeneratedPassphrase {
    pub passphrase: String,
    pub strength: PasswordStrength,
}

#[command]
pub fn generate_passphrase_cmd(options: PassphraseOptions) -> Result<GeneratedPassphrase, String> {
    let passphrase = generate_passphrase(options.word_count);
    let strength = check_strength(&passphrase);

    Ok(GeneratedPassphrase {
        passphrase,
        strength,
    })
}

/// Check the strength of a given password
#[command]
pub fn check_password_strength(password: String) -> PasswordStrength {
    check_strength(&password)
}
