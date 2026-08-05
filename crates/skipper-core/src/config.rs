use crate::errors::{Result, SkipperError};
use crate::types::{MatchMode, OperatingMode, WhitelistEntry};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default)]
    pub mode: OperatingMode,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
    #[serde(default)]
    pub auto_activate: bool,
}

fn default_timeout() -> u32 {
    30
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            mode: OperatingMode::Standard,
            timeout_seconds: default_timeout(),
            auto_activate: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefaultCredentials {
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhitelistConfig {
    #[serde(default)]
    pub entries: Vec<WhitelistEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatternsConfig {
    #[serde(default)]
    pub custom_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub default_credentials: DefaultCredentials,
    #[serde(default)]
    pub whitelist: WhitelistConfig,
    #[serde(default)]
    pub patterns: PatternsConfig,
}

impl Config {
    pub fn generate_keyring_key(command: &str) -> String {
        let slug: String = command
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        format!("whitelist:{}", slug)
    }

    pub fn add_whitelist_entry(&mut self, command: &str, match_mode: MatchMode) -> String {
        let keyring_key = Self::generate_keyring_key(command);
        self.remove_whitelist_entry(command); // Overwrite if existing

        self.whitelist.entries.push(WhitelistEntry {
            command: command.to_string(),
            keyring_key: keyring_key.clone(),
            match_mode,
        });

        keyring_key
    }

    pub fn remove_whitelist_entry(&mut self, command: &str) -> bool {
        let original_len = self.whitelist.entries.len();
        self.whitelist.entries.retain(|e| e.command != command);
        self.whitelist.entries.len() < original_len
    }
}

pub struct ConfigManager {
    config_dir: PathBuf,
    config_file: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let base_dir = dirs::config_dir().ok_or_else(|| {
            SkipperError::Config("Impossible de localiser le répertoire config".into())
        })?;
        let config_dir = base_dir.join("skipper360");
        let config_file = config_dir.join("config.toml");

        Ok(Self {
            config_dir,
            config_file,
        })
    }

    pub fn with_custom_path(config_file: PathBuf) -> Self {
        let config_dir = config_file.parent().unwrap_or(&config_file).to_path_buf();
        Self {
            config_dir,
            config_file,
        }
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_file
    }

    pub fn ensure_config_dir(&self) -> Result<()> {
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir).map_err(|e| {
                SkipperError::Config(format!(
                    "Échec de création de {}: {}",
                    self.config_dir.display(),
                    e
                ))
            })?;
        }
        Ok(())
    }

    pub fn load(&self) -> Result<Config> {
        if !self.config_file.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&self.config_file).map_err(|e| {
            SkipperError::Config(format!(
                "Impossible de lire {}: {}",
                self.config_file.display(),
                e
            ))
        })?;

        toml::from_str(&content)
            .map_err(|e| SkipperError::Config(format!("Erreur de parsing TOML: {}", e)))
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        self.ensure_config_dir()?;

        let content = toml::to_string_pretty(config)
            .map_err(|e| SkipperError::Config(format!("Erreur de sérialisation TOML: {}", e)))?;

        fs::write(&self.config_file, content).map_err(|e| {
            SkipperError::Config(format!(
                "Échec d'écriture dans {}: {}",
                self.config_file.display(),
                e
            ))
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            let _ = fs::set_permissions(&self.config_file, permissions);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_save_load() {
        let temp_dir = std::env::temp_dir().join(format!("skipper_test_{}", std::process::id()));
        let config_path = temp_dir.join("config.toml");
        let manager = ConfigManager::with_custom_path(config_path);

        let mut cfg = Config::default();
        cfg.general.mode = OperatingMode::Silent;
        cfg.default_credentials.username = Some("testuser".into());

        manager.save(&cfg).expect("Failed to save config");

        let loaded = manager.load().expect("Failed to load config");
        assert_eq!(loaded.general.mode, OperatingMode::Silent);
        assert_eq!(
            loaded.default_credentials.username.as_deref(),
            Some("testuser")
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_whitelist_helpers() {
        let mut cfg = Config::default();
        let key = cfg.add_whitelist_entry("ssh user@srv", MatchMode::Prefix);
        assert!(key.contains("whitelist:ssh_user_srv"));
        assert_eq!(cfg.whitelist.entries.len(), 1);

        let removed = cfg.remove_whitelist_entry("ssh user@srv");
        assert!(removed);
        assert_eq!(cfg.whitelist.entries.len(), 0);
    }
}
