use secrecy::{ExposeSecret, SecretString};
use skipper_core::{Config, KeyringManager, MatchMode, Result};

pub struct PasswordInjector;

impl PasswordInjector {
    pub fn resolve_password(command: &str, config: &Config) -> Result<SecretString> {
        // 1. Check whitelist entries for matching command
        for entry in &config.whitelist.entries {
            let matched = match entry.match_mode {
                MatchMode::Exact => command == entry.command,
                MatchMode::Prefix => command.starts_with(&entry.command),
            };

            if matched {
                tracing::info!(
                    "Match Whitelist pour la commande '{}' [{:?}]",
                    command,
                    entry.match_mode
                );
                return KeyringManager::get_whitelist_password(&entry.keyring_key);
            }
        }

        // 2. Fallback to default password in Keyring
        tracing::info!(
            "Aucune entrée whitelist spécifique, utilisation du mot de passe par défaut."
        );
        KeyringManager::get_default_password()
    }

    pub fn format_injection_bytes(password: &SecretString) -> Vec<u8> {
        let mut bytes = password.expose_secret().as_bytes().to_vec();
        bytes.push(b'\n');
        bytes
    }
}
