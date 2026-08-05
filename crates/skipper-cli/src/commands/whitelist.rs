use anyhow::{anyhow, Result};
use dialoguer::Password;
use owo_colors::OwoColorize;
use secrecy::SecretString;
use skipper_core::types::MatchMode;
use skipper_core::{ConfigManager, KeyringManager, WhitelistEntry};

pub fn add(command: &[String]) -> Result<()> {
    if command.is_empty() {
        return Err(anyhow!("Veuillez spécifier la commande à ajouter à la whitelist"));
    }

    let cmd_str = command.join(" ");

    // Saisie sécurisée du mot de passe
    let raw_password = Password::new()
        .with_prompt(format!("🔒 Entrez le mot de passe pour '{}'", cmd_str))
        .interact()?;

    let password = SecretString::new(raw_password.into());

    let cm = ConfigManager::new()?;
    let mut config = cm.load()?;

    // Génération d'une clé d'alias unique pour le trousseau Keyring
    let keyring_key = format!("whitelist_{}", cmd_str.replace([' ', '/', '@', ':'], "_"));

    // Stockage dans l'OS Keyring
    KeyringManager::store_whitelist_password(&keyring_key, &password)?;

    // Mise à jour de config.toml
    config.whitelist.entries.retain(|e| e.command != cmd_str);
    config.whitelist.entries.push(WhitelistEntry {
        command: cmd_str.clone(),
        keyring_key,
        match_mode: MatchMode::Prefix,
    });

    cm.save(&config)?;

    println!(
        "{}",
        format!("✅ Commande '{}' ajoutée à la whitelist avec succès !", cmd_str)
            .green()
            .bold()
    );

    Ok(())
}

pub fn delete(command: &[String]) -> Result<()> {
    if command.is_empty() {
        return Err(anyhow!("Veuillez spécifier la commande à supprimer de la whitelist"));
    }

    let cmd_str = command.join(" ");
    let cm = ConfigManager::new()?;
    let mut config = cm.load()?;

    let original_len = config.whitelist.entries.len();
    
    // Trouver les clés à supprimer du Keyring
    for entry in config.whitelist.entries.iter().filter(|e| e.command == cmd_str) {
        let _ = KeyringManager::delete_whitelist_password(&entry.keyring_key);
    }

    config.whitelist.entries.retain(|e| e.command != cmd_str);

    if config.whitelist.entries.len() == original_len {
        println!("{}", format!("⚠️ Commande '{}' non trouvée dans la whitelist", cmd_str).yellow());
    } else {
        cm.save(&config)?;
        println!("{}", format!("🗑️ Commande '{}' supprimée de la whitelist !", cmd_str).green());
    }

    Ok(())
}
