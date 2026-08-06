use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OperatingMode {
    #[default]
    Standard,
    Silent,
}

impl std::fmt::Display for OperatingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatingMode::Standard => write!(f, "Standard"),
            OperatingMode::Silent => write!(f, "Silent"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SkipperStatus {
    #[default]
    Inactive,
    Active,
}

impl std::fmt::Display for SkipperStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkipperStatus::Inactive => write!(f, "Inactive"),
            SkipperStatus::Active => write!(f, "Active"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MatchMode {
    Exact,
    #[default]
    Prefix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub command: String,
    pub keyring_key: String,
    #[serde(default)]
    pub match_mode: MatchMode,
    #[serde(default)]
    pub expires_at: Option<u64>,
}

impl WhitelistEntry {
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            if let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                return now.as_secs() > expires;
            }
        }
        false
    }
}
