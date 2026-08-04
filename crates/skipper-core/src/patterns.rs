pub const DEFAULT_PATTERNS: &[&str] = &[
    // English prompts
    r"(?i)\[sudo\].*password.*:",
    r"(?i)password\s*:",
    r"(?i)password\s+for\s+",
    r"(?i)enter\s+passphrase",
    r"(?i)enter\s+password",
    r"(?i)authentication\s+password",
    r"(?i)\bpasswd\b.*:",
    // French prompts
    r"(?i)mot\s+de\s+passe\s*:",
    r"(?i)entrez\s+(le\s+)?mot\s+de\s+passe",
    // SSH & Key prompts
    r"(?i).*'s\s+password\s*:",
    r"(?i)enter\s+passphrase\s+for\s+key",
    // Pacman / AUR helpers
    r"(?i)\[sudo\].*mot de passe",
    r"(?i)password\s+required",
];

pub fn get_default_patterns() -> Vec<String> {
    DEFAULT_PATTERNS.iter().map(|s| s.to_string()).collect()
}
