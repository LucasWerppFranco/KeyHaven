//! Password and passphrase generation module.

use rand::{distributions::Alphanumeric, Rng};

/// Strength analysis result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordStrength {
    Weak,
    Medium,
    Strong,
}

/// Generate a random password of given length
pub fn generate_password(length: usize, use_symbols: bool) -> String {
    let mut rng = rand::thread_rng();

    if use_symbols {
        // Include symbols: alphanumeric + common special chars
        let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789\
                             !@#$%^&*()_+-=[]{}|;:,.<>?";
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect()
    } else {
        // Alphanumeric only
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }
}

/// Generate a passphrase with given number of words
pub fn generate_passphrase(word_count: usize) -> String {
    // Word list for passphrases (EFF long wordlist sample)
    const WORDS: &[&str] = &[
        "abandon", "ability", "able", "about", "above", "absent", "absorb",
        "abstract", "absurd", "abuse", "access", "accident", "account",
        "accuse", "achieve", "acid", "acoustic", "acquire", "across",
        "act", "action", "actor", "actual", "adapt", "add", "addict",
        "address", "adjust", "admit", "adult", "advance", "advice",
        "aerobic", "affair", "afford", "afraid", "again", "age", "agent",
        "agree", "ahead", "aim", "air", "airport", "aisle", "alarm",
        "album", "alcohol", "alert", "alien", "all", "alley", "allow",
        // TODO: Add full wordlist
    ];

    let mut rng = rand::thread_rng();
    let words: Vec<_> = (0..word_count)
        .map(|_| {
            let idx = rng.gen_range(0..WORDS.len());
            WORDS[idx]
        })
        .collect();

    words.join("-")
}

/// Check the strength of a password
pub fn check_strength(password: &str) -> PasswordStrength {
    let length = password.len();

    // Calculate entropy (simplified)
    // Assuming 95 printable ASCII chars for mixed passwords
    let pool_size: f64 = 95.0;
    let entropy_bits = (length as f64) * pool_size.log2();

    // Score based on entropy
    if entropy_bits < 50.0 {
        PasswordStrength::Weak
    } else if entropy_bits < 75.0 {
        PasswordStrength::Medium
    } else {
        PasswordStrength::Strong
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_password_length() {
        let pwd = generate_password(20, false);
        assert_eq!(pwd.len(), 20);
    }

    #[test]
    fn test_generate_passphrase_words() {
        let phrase = generate_passphrase(6);
        let words: Vec<_> = phrase.split('-').collect();
        assert_eq!(words.len(), 6);
    }

    #[test]
    fn test_check_strength() {
        let strong = check_strength("a very long and complex passphrase!");
        assert_eq!(strong, PasswordStrength::Strong);

        let weak = check_strength("123");
        assert_eq!(weak, PasswordStrength::Weak);
    }
}
