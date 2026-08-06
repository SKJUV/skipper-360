use secrecy::SecretString;
use skipper_core::{AlignedSecretBuffer, Config, KeyringManager, MatchMode, Result};

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

    pub fn format_aligned_buffer(password: &SecretString) -> AlignedSecretBuffer {
        AlignedSecretBuffer::from_secret(password)
    }
}
