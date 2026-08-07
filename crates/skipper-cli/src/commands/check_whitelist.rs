use anyhow::Result;
use skipper_core::{ConfigManager, MatchMode};

pub fn check(command: &[String]) -> Result<bool> {
    let cmd_str = command.join(" ");
    let trimmed = cmd_str.trim();

    if trimmed.is_empty() {
        return Ok(false);
    }

    // Ne pas boucler récursivement sur les commandes skipper elles-mêmes
    if trimmed.starts_with("skipper") {
        return Ok(false);
    }

    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?;

    for entry in &config.whitelist.entries {
        let matched = match entry.match_mode {
            MatchMode::Exact => trimmed == entry.command,
            MatchMode::Prefix => trimmed.starts_with(&entry.command),
        };

        if matched {
            return Ok(true);
        }
    }

    Ok(false)
}
