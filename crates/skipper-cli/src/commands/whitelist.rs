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

pub async fn add(command: &[String], match_mode_flag: Option<String>) -> Result<()> {
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

    let config_manager = ConfigManager::new()?;
    let mut config = config_manager.load()?;

    let keyring_key = config.add_whitelist_entry(&cmd_str, match_mode);

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

    println!(
        "{}",
        format!(
            "✅ Commande '{}' ajoutée avec succès [{:?}].",
            cmd_str, match_mode
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
        ]);

    for (i, entry) in config.whitelist.entries.iter().enumerate() {
        let pwd_display = if show_passwords {
            match KeyringManager::get_whitelist_password(&entry.keyring_key) {
                Ok(secret) => secret.expose_secret().to_string(),
                Err(_) => "(mot de passe par défaut)".italic().to_string(),
            }
        } else {
            "••••••••".to_string()
        };

        table.add_row(vec![
            Cell::new(i + 1),
            Cell::new(&entry.command).fg(Color::Cyan),
            Cell::new(format!("{:?}", entry.match_mode)),
            Cell::new(pwd_display).fg(Color::Yellow),
        ]);
    }

    println!("{table}");
    Ok(())
}
