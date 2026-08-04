use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatingMode {
    Standard,
    Silent,
}

impl fmt::Display for OperatingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperatingMode::Standard => write!(f, "Standard"),
            OperatingMode::Silent => write!(f, "Silent"),
        }
    }
}

impl Default for OperatingMode {
    fn default() -> Self {
        OperatingMode::Standard
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipperStatus {
    Active,
    Inactive,
}

impl fmt::Display for SkipperStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkipperStatus::Active => write!(f, "Actif (🟢)"),
            SkipperStatus::Inactive => write!(f, "Inactif (🔴)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchMode {
    Exact,
    Prefix,
}

impl Default for MatchMode {
    fn default() -> Self {
        MatchMode::Prefix
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WhitelistEntry {
    pub command: String,
    pub keyring_key: String,
    #[serde(default)]
    pub match_mode: MatchMode,
}
