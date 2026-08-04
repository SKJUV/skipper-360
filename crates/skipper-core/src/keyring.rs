use crate::errors::{Result, SkipperError};
use keyring::Entry;
use secrecy::SecretString;

const SERVICE_NAME: &str = "skipper360";
const DEFAULT_KEY: &str = "default_password";

pub struct KeyringManager;

impl KeyringManager {
    pub fn store_default_password(password: &SecretString) -> Result<()> {
        Self::store(DEFAULT_KEY, password)
    }

    pub fn get_default_password() -> Result<SecretString> {
        Self::get(DEFAULT_KEY)
    }

    pub fn store_whitelist_password(key: &str, password: &SecretString) -> Result<()> {
        Self::store(key, password)
    }

    pub fn get_whitelist_password(key: &str) -> Result<SecretString> {
        Self::get(key)
    }

    pub fn delete_whitelist_password(key: &str) -> Result<()> {
        Self::delete(key)
    }

    fn store(key: &str, password: &SecretString) -> Result<()> {
        use secrecy::ExposeSecret;
        let entry = Entry::new(SERVICE_NAME, key).map_err(|e| {
            SkipperError::Keyring(format!(
                "Erreur d'initialisation keyring pour {}: {}",
                key, e
            ))
        })?;

        entry.set_password(password.expose_secret()).map_err(|e| {
            SkipperError::Keyring(format!(
                "Échec de stockage du mot de passe pour {}: {}",
                key, e
            ))
        })?;

        Ok(())
    }

    fn get(key: &str) -> Result<SecretString> {
        let entry = Entry::new(SERVICE_NAME, key).map_err(|e| {
            SkipperError::Keyring(format!("Erreur d'accès keyring pour {}: {}", key, e))
        })?;

        let secret = entry.get_password().map_err(|e| {
            SkipperError::Keyring(format!(
                "Échec de récupération du mot de passe pour {}: {}",
                key, e
            ))
        })?;

        Ok(SecretString::from(secret))
    }

    fn delete(key: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, key).map_err(|e| {
            SkipperError::Keyring(format!("Erreur d'accès keyring pour {}: {}", key, e))
        })?;

        entry.delete_credential().map_err(|e| {
            SkipperError::Keyring(format!(
                "Échec de suppression du mot de passe pour {}: {}",
                key, e
            ))
        })?;

        Ok(())
    }
}
