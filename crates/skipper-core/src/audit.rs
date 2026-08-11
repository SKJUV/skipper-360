use crate::errors::{Result, SkipperError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    CommandExecuted,
    WhitelistMatched,
    TimeoutElapsed,
    PasswordInjected,
    PasswordInjectionFailed,
    PromptDetected,
    WhitelistAdded,
    WhitelistDeleted,
    ModeChanged,
    StatusChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub user: String,
    pub pid: u32,
    pub action: AuditAction,
    pub command: String,
    pub details: String,
}

use regex::Regex;
use std::sync::OnceLock;

/// Sanitize and redact potential secret strings, passwords, or tokens from audit log messages.
pub fn redact_secrets(input: &str) -> String {
    static REGEXES: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    let rules = REGEXES.get_or_init(|| {
        vec![
            // 1. Long flags like --password=xxx, --token xxx, --secret xxx
            (
                Regex::new(r#"(?i)(--(?:password|passwd|passphrase|secret|token|api[_-]?key|auth[_-]?token)(?:\s+|=))['"]?[^\s'"]+['"]?"#).unwrap(),
                "$1[REDACTED]",
            ),
            // 2. Key-Value pairs like password=xxx, secret: xxx, mot de passe = xxx
            (
                Regex::new(r#"(?i)\b((?:password|passwd|passphrase|secret|token|api[_-]?key|auth[_-]?token|mot\s+de\s+passe)\s*[:=]\s*)['"]?[^\s'"]+['"]?"#).unwrap(),
                "$1[REDACTED]",
            ),
            // 3. Authorization headers (Bearer / Basic)
            (
                Regex::new(r"(?i)\b((?:bearer|basic)\s+)[a-zA-Z0-9._~\+\/-]+=*").unwrap(),
                "$1[REDACTED]",
            ),
            // 4. URL inline password: http://user:password@host
            (
                Regex::new(r"(?i)(https?://[^:\s]+:)[^@\s]+(@)").unwrap(),
                "$1[REDACTED]$2",
            ),
            // 5. mysql/mariadb/pg_dump -pPassword inline password
            (
                Regex::new(r"(?i)\b(mysql|mariadb|pg_dump)\b(.*?\s+-p)\S+").unwrap(),
                "$1$2[REDACTED]",
            ),
        ]
    });

    let mut result = input.to_string();
    for (re, replacement) in rules {
        result = re.replace_all(&result, *replacement).to_string();
    }
    result
}

pub struct AuditLogger {
    log_path: PathBuf,
}

impl AuditLogger {
    pub fn new() -> Result<Self> {
        let base_dir = dirs::config_dir().ok_or_else(|| {
            SkipperError::Config("Impossible de localiser le répertoire config".into())
        })?;
        let log_dir = base_dir.join("skipper360");
        if !log_dir.exists() {
            let _ = fs::create_dir_all(&log_dir);
        }
        let log_path = log_dir.join("audit.log");
        Ok(Self { log_path })
    }

    pub fn log(
        &self,
        action: AuditAction,
        command: impl Into<String>,
        details: impl Into<String>,
    ) -> Result<()> {
        let entry = AuditEntry {
            timestamp: chrono_now_iso(),
            user: std::env::var("USER").unwrap_or_else(|_| "unknown".into()),
            pid: std::process::id(),
            action,
            command: redact_secrets(&command.into()),
            details: redact_secrets(&details.into()),
        };

        let json_line = serde_json::to_string(&entry).map_err(|e| {
            SkipperError::Other(format!("Erreur de sérialisation de l'audit : {}", e))
        })?;

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(SkipperError::Io)?;

        use std::io::Write;
        writeln!(file, "{}", json_line).map_err(SkipperError::Io)?;

        let _ = fs::set_permissions(&self.log_path, fs::Permissions::from_mode(0o600));
        Ok(())
    }

    pub fn read_entries(&self) -> Result<Vec<AuditEntry>> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.log_path).map_err(SkipperError::Io)?;
        let mut entries = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<AuditEntry>(line.trim()) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }
}

fn chrono_now_iso() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_logging_and_reading() {
        let temp_dir =
            std::env::temp_dir().join(format!("skipper_audit_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let log_path = temp_dir.join("audit.log");

        let logger = AuditLogger { log_path };
        logger
            .log(
                AuditAction::PasswordInjected,
                "sudo pacman",
                "Mot de passe injecté",
            )
            .unwrap();

        let entries = logger.read_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].command, "sudo pacman");

        let _ = fs::remove_dir_all(temp_dir);
    }
}
