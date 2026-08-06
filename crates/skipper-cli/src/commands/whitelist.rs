use crate::ipc::IpcClient;
use anyhow::{anyhow, Result};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};
use dialoguer::Password;
use nix::unistd::Uid;
use owo_colors::OwoColorize;
use secrecy::{ExposeSecret, SecretString};
use skipper_core::{ConfigManager, KeyringManager, MatchMode, Request};

pub async fn add(
    command: &[String],
    match_mode_flag: Option<String>,
    ttl_flag: Option<String>,
) -> Result<()> {
    let cmd_str = command.join(" ");
    if cmd_str.trim().is_empty() {
        return Err(anyhow!("Veuillez préciser la commande à ajouter à la whitelist (ex: skipper whitelist add ssh user@srv)"));
    }

    println!(
        "{}",
        format!(
            "📋 Ajout de la commande à la whitelist : {}",
            cmd_str.bold()
        )
        .blue()
    );

    let password_prompt = Password::new()
        .with_prompt("🔒 Mot de passe pour cette commande (laissez vide pour utiliser le mot de passe par défaut)")
        .allow_empty_password(true)
        .interact()?;

    let match_mode = match match_mode_flag.as_deref() {
        Some("exact") => MatchMode::Exact,
        _ => MatchMode::Prefix,
    };

    let ttl_seconds = match ttl_flag.as_deref() {
        Some(s) => Some(parse_duration_to_seconds(s)?),
        None => None,
    };

    let config_manager = ConfigManager::new()?;
    let mut config = config_manager.load()?;

    let keyring_key = config.add_whitelist_entry_with_ttl(&cmd_str, match_mode, ttl_seconds);

    if !password_prompt.is_empty() {
        let secret = SecretString::from(password_prompt);
        KeyringManager::store_whitelist_password(&keyring_key, &secret)?;
        println!(
            "{}",
            "✅ Mot de passe spécifique enregistré dans le trousseau système.".green()
        );
    } else {
        println!(
            "{}",
            "ℹ️  Aucun mot de passe saisi, l'entrée utilisera le mot de passe par défaut.".cyan()
        );
    }

    config_manager.save(&config)?;

    // Notify daemon via IPC if running
    let client = IpcClient::new()?;
    let req = Request::new("reload_config", serde_json::json!({}));
    let _ = client.send_request(req).await;

    let ttl_desc = match ttl_seconds {
        Some(sec) => format!(" (expire dans {}s)", sec),
        None => "".to_string(),
    };

    println!(
        "{}",
        format!(
            "✅ Commande '{}' ajoutée avec succès [{:?}]{}.",
            cmd_str, match_mode, ttl_desc
        )
        .green()
        .bold()
    );
    Ok(())
}

pub async fn delete(command: &[String]) -> Result<()> {
    let cmd_str = command.join(" ");
    if cmd_str.trim().is_empty() {
        return Err(anyhow!("Veuillez préciser la commande à supprimer de la whitelist (ex: skipper whitelist delete ssh user@srv)"));
    }

    let config_manager = ConfigManager::new()?;
    let mut config = config_manager.load()?;

    let keyring_key = skipper_core::Config::generate_keyring_key(&cmd_str);
    let removed = config.remove_whitelist_entry(&cmd_str);

    if removed {
        let _ = KeyringManager::delete_whitelist_password(&keyring_key);
        config_manager.save(&config)?;

        // Notify daemon via IPC
        let client = IpcClient::new()?;
        let req = Request::new("reload_config", serde_json::json!({}));
        let _ = client.send_request(req).await;

        println!(
            "{}",
            format!("✅ Commande '{}' supprimée de la whitelist.", cmd_str)
                .yellow()
                .bold()
        );
    } else {
        println!(
            "{}",
            format!(
                "⚠️  La commande '{}' n'a pas été trouvée dans la whitelist.",
                cmd_str
            )
            .red()
        );
    }

    Ok(())
}

pub fn list(show_passwords: bool) -> Result<()> {
    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?;

    if show_passwords && !Uid::effective().is_root() {
        return Err(anyhow!("🔒 L'affichage des mots de passe en clair exige des privilèges de super-utilisateur (sudo skipper whitelist list --show)"));
    }

    println!(
        "{}",
        "🛡️  Skipper 360 — Whitelist des Commandes".bold().blue()
    );
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    if config.whitelist.entries.is_empty() {
        println!(
            "{}",
            "   Aucune commande enregistrée dans la whitelist."
                .italic()
                .dimmed()
        );
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("#").add_attribute(Attribute::Bold),
            Cell::new("Commande").add_attribute(Attribute::Bold),
            Cell::new("Mode Match").add_attribute(Attribute::Bold),
            Cell::new("Mot de passe").add_attribute(Attribute::Bold),
            Cell::new("Expiration (TTL)").add_attribute(Attribute::Bold),
        ]);

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    for (i, entry) in config.whitelist.entries.iter().enumerate() {
        let pwd_display = if show_passwords {
            match KeyringManager::get_whitelist_password(&entry.keyring_key) {
                Ok(secret) => secret.expose_secret().to_string(),
                Err(_) => "(mot de passe par défaut)".italic().to_string(),
            }
        } else {
            "••••••••".to_string()
        };

        let ttl_display = match entry.expires_at {
            Some(exp) => {
                if exp > now_secs {
                    format!("{}s restantes", exp - now_secs)
                } else {
                    "Expiré".to_string()
                }
            }
            None => "Permanente".to_string(),
        };

        table.add_row(vec![
            Cell::new(i + 1),
            Cell::new(&entry.command).fg(Color::Cyan),
            Cell::new(format!("{:?}", entry.match_mode)),
            Cell::new(pwd_display).fg(Color::Yellow),
            Cell::new(ttl_display).fg(Color::Magenta),
        ]);
    }

    println!("{table}");
    Ok(())
}

fn parse_duration_to_seconds(input: &str) -> Result<u64> {
    let s = input.trim();
    if let Ok(num) = s.parse::<u64>() {
        return Ok(num);
    }

    let len = s.len();
    if len < 2 {
        return Err(anyhow!(
            "Format de durée TTL invalide (ex: 30s, 10m, 2h, 1d)"
        ));
    }

    let unit = &s[len - 1..];
    let num_str = &s[..len - 1];
    let num: u64 = num_str
        .parse()
        .map_err(|_| anyhow!("Format numérique TTL invalide dans '{}'", s))?;

    match unit {
        "s" => Ok(num),
        "m" => Ok(num * 60),
        "h" => Ok(num * 3600),
        "d" => Ok(num * 86400),
        _ => Err(anyhow!(
            "Unité de durée inconnue '{}' (utilisez s, m, h, d)",
            unit
        )),
    }
}
