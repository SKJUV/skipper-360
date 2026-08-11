use crate::config::{Config, ConfigManager};
use crate::errors::{Result, SkipperError};
use crate::types::MatchMode;
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::symlink;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};

pub fn get_shims_dir() -> Result<PathBuf> {
    let base_dir = dirs::data_local_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
        .ok_or_else(|| {
            SkipperError::Shim(
                "Impossible de localiser le répertoire de données de l'utilisateur".into(),
            )
        })?;
    Ok(base_dir.join("skipper").join("shims"))
}

pub fn get_skipper_binary_path() -> Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if exe
            .file_name()
            .and_then(|n| n.to_str())
            .map_or(false, |name| name.starts_with("skipper"))
        {
            return Ok(exe);
        }
    }

    if let Some(home) = dirs::home_dir() {
        let cargo_bin = home.join(".cargo").join("bin").join("skipper");
        if cargo_bin.exists() {
            return Ok(cargo_bin);
        }
    }

    Ok(PathBuf::from("skipper"))
}

pub fn extract_binary_name(cmd_str: &str) -> &str {
    cmd_str.trim().split_whitespace().next().unwrap_or("")
}

pub fn get_active_whitelist_binaries(config: &Config) -> Vec<String> {
    let mut set = HashSet::new();
    for entry in &config.whitelist.entries {
        if !entry.is_expired() {
            let bin = extract_binary_name(&entry.command);
            if !bin.is_empty() {
                set.insert(bin.to_string());
            }
        }
    }
    let mut list: Vec<String> = set.into_iter().collect();
    list.sort();
    list
}

pub fn sync_shims(config: &Config) -> Result<Vec<String>> {
    let shims_dir = get_shims_dir()?;
    if !shims_dir.exists() {
        fs::create_dir_all(&shims_dir).map_err(|e| {
            SkipperError::Shim(format!(
                "Échec de création du répertoire de shims {}: {}",
                shims_dir.display(),
                e
            ))
        })?;
    }

    let skipper_bin = get_skipper_binary_path()?;
    let active_bins = get_active_whitelist_binaries(config);
    let active_set: HashSet<&str> = active_bins.iter().map(|s| s.as_str()).collect();

    // Create or update symlinks for all active whitelisted commands
    for bin_name in &active_bins {
        let shim_path = shims_dir.join(bin_name);

        let need_symlink = match fs::symlink_metadata(&shim_path) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    match fs::read_link(&shim_path) {
                        Ok(target) => target != skipper_bin,
                        Err(_) => true,
                    }
                } else {
                    true
                }
            }
            Err(_) => true,
        };

        if need_symlink {
            let _ = fs::remove_file(&shim_path);
            symlink(&skipper_bin, &shim_path).map_err(|e| {
                SkipperError::Shim(format!(
                    "Échec de création du symlink {} -> {}: {}",
                    shim_path.display(),
                    skipper_bin.display(),
                    e
                ))
            })?;
        }
    }

    // Remove obsolete shims
    if let Ok(entries) = fs::read_dir(&shims_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            if !name_str.starts_with('.') && !active_set.contains(name_str.as_ref()) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    Ok(active_bins)
}

pub fn install_shims(config: &Config) -> Result<Vec<String>> {
    let shims_dir = get_shims_dir()?;
    if !shims_dir.exists() {
        fs::create_dir_all(&shims_dir).map_err(|e| {
            SkipperError::Shim(format!(
                "Échec de création de {}: {}",
                shims_dir.display(),
                e
            ))
        })?;
    }
    sync_shims(config)
}

pub fn remove_shims() -> Result<usize> {
    let shims_dir = get_shims_dir()?;
    if !shims_dir.exists() {
        return Ok(0);
    }

    let mut removed = 0;
    if let Ok(entries) = fs::read_dir(&shims_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with('.') {
                if fs::remove_file(entry.path()).is_ok() {
                    removed += 1;
                }
            }
        }
    }
    Ok(removed)
}

pub fn is_shim_invocation(argv0: &str) -> bool {
    let file_name = Path::new(argv0)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    file_name != "skipper" && file_name != "skipperd" && !file_name.is_empty()
}

pub fn is_daemon_active() -> bool {
    let socket_path = match dirs::config_dir() {
        Some(dir) => dir.join("skipper360").join("skipper.sock"),
        None => return false,
    };
    if !socket_path.exists() {
        return false;
    }
    UnixStream::connect(&socket_path).is_ok()
}

pub fn is_command_whitelisted(command_str: &str, config: &Config) -> bool {
    config.whitelist.entries.iter().any(|entry| {
        if entry.is_expired() {
            return false;
        }
        match entry.match_mode {
            MatchMode::Exact => command_str == entry.command,
            MatchMode::Prefix => {
                command_str == entry.command
                    || command_str.starts_with(&format!("{} ", entry.command))
                    || entry.command == extract_binary_name(command_str)
                    || command_str.starts_with(&entry.command)
            }
        }
    })
}

pub fn find_real_binary(cmd_name: &str) -> Option<PathBuf> {
    let shims_dir = get_shims_dir().ok()?;
    let path_env = std::env::var_os("PATH")?;

    for path in std::env::split_paths(&path_env) {
        if path == shims_dir || path.starts_with(&shims_dir) {
            continue;
        }
        let candidate = path.join(cmd_name);
        if candidate.is_file() {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = candidate.metadata() {
                if metadata.permissions().mode() & 0o111 != 0 {
                    return Some(candidate);
                }
            }
        }
    }

    let usr_bin = PathBuf::from("/usr/bin").join(cmd_name);
    if usr_bin.is_file() {
        return Some(usr_bin);
    }

    None
}

pub fn run_shim(argv0: &str, args: &[String]) -> Result<()> {
    let cmd_name = Path::new(argv0)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| SkipperError::Shim("Nom d'exécutable invalide pour le shim".into()))?;

    // Check if bypass flag is set to prevent recursive loops
    if std::env::var("SKIPPER_SHIM_BYPASS").is_ok() {
        let real_bin =
            find_real_binary(cmd_name).unwrap_or_else(|| PathBuf::from("/usr/bin").join(cmd_name));
        let mut cmd = std::process::Command::new(&real_bin);
        cmd.args(args);
        let err = cmd.exec();
        return Err(SkipperError::Shim(format!(
            "Échec de l'exécution de {}: {}",
            real_bin.display(),
            err
        )));
    }

    let command_str = if args.is_empty() {
        cmd_name.to_string()
    } else {
        format!("{} {}", cmd_name, args.join(" "))
    };

    let daemon_active = is_daemon_active();
    let config = ConfigManager::new()
        .and_then(|m| m.load())
        .unwrap_or_default();
    let whitelisted = is_command_whitelisted(&command_str, &config);

    if daemon_active && whitelisted {
        let skipper_bin = get_skipper_binary_path()?;
        let mut cmd = std::process::Command::new(&skipper_bin);
        cmd.arg("run").arg("--").arg(cmd_name).args(args);
        cmd.env("SKIPPER_SHIM_BYPASS", "1");
        let err = cmd.exec();
        Err(SkipperError::Shim(format!(
            "Échec d'exécution de 'skipper run': {}",
            err
        )))
    } else {
        let real_bin =
            find_real_binary(cmd_name).unwrap_or_else(|| PathBuf::from("/usr/bin").join(cmd_name));
        let mut cmd = std::process::Command::new(&real_bin);
        cmd.args(args);
        let err = cmd.exec();
        Err(SkipperError::Shim(format!(
            "Échec d'exécution de {}: {}",
            real_bin.display(),
            err
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MatchMode;

    #[test]
    fn test_extract_binary_name() {
        assert_eq!(extract_binary_name("ssh user@host"), "ssh");
        assert_eq!(extract_binary_name("pacman -Syu"), "pacman");
        assert_eq!(extract_binary_name("  sudo   su "), "sudo");
        assert_eq!(extract_binary_name("git"), "git");
    }

    #[test]
    fn test_is_shim_invocation() {
        assert!(!is_shim_invocation("skipper"));
        assert!(!is_shim_invocation("/usr/bin/skipper"));
        assert!(!is_shim_invocation("skipperd"));
        assert!(!is_shim_invocation("/usr/bin/skipperd"));
        assert!(is_shim_invocation("ssh"));
        assert!(is_shim_invocation(
            "/home/user/.local/share/skipper/shims/ssh"
        ));
        assert!(is_shim_invocation("sudo"));
    }

    #[test]
    fn test_is_command_whitelisted() {
        let mut config = Config::default();
        config.add_whitelist_entry("ssh user@prod", MatchMode::Prefix);
        config.add_whitelist_entry("pacman -Syu", MatchMode::Exact);

        assert!(is_command_whitelisted("ssh user@prod -p 22", &config));
        assert!(is_command_whitelisted("pacman -Syu", &config));
        assert!(!is_command_whitelisted("pacman -R foo", &config));
        assert!(!is_command_whitelisted("git status", &config));
    }
}
