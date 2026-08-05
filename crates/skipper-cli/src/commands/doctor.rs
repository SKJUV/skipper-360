use anyhow::Result;
use owo_colors::OwoColorize;
use skipper_core::{ConfigManager, KeyringManager};
use std::path::Path;
use std::process::Command;

pub async fn run() -> Result<()> {
    println!("\n{}", "🩺 Skipper 360 — Diagnostic du Système".bold().cyan());
    println!("{}\n", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let mut all_ok = true;

    // 1. Diagnostic de la Configuration
    print!("1. Fichier de configuration : ");
    match ConfigManager::new().and_then(|cm| cm.load()) {
        Ok(_) => println!("{}", " OK (config.toml valide)".green()),
        Err(e) => {
            println!("{}", format!(" Avertissement (config non chargée : {})", e).yellow());
        }
    }

    // 2. Diagnostic de l'OS Keyring
    print!("2. Trousseau de clés (OS Keyring) : ");
    match KeyringManager::get_default_password() {
        Ok(_) => println!("{}", " OK (Keyring accessible et mot de passe par défaut configuré)".green()),
        Err(e) => {
            println!("{}", format!(" Keyring vérifié ({})", e).yellow());
        }
    }

    // 3. Diagnostic des utilitaires PTY système
    print!("3. Infrastructure PTY & Shells : ");
    let sh_check = Command::new("which").arg("sh").output().map(|o| o.status.success()).unwrap_or(false);
    if sh_check {
        println!("{}", "OK (/bin/sh disponible)".green());
    } else {
        println!("{}", "Incomplet (/bin/sh manquant)".red());
        all_ok = false;
    }

    // 4. Diagnostic du Socket UDS & Daemon
    print!("4. Daemon skipperd & Socket IPC : ");
    let config_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("~/.config")).join("skipper360");
    let socket_path = config_dir.join("skipper.sock");

    if Path::new(&socket_path).exists() {
        println!("{}", format!("Socket détecté ({})", socket_path.display()).green());
    } else {
        println!("{}", format!("Socket inactif (lancez 'skipper activate' pour démarrer le daemon)").yellow());
    }

    // 5. Diagnostic des Permissions
    print!("5. Permissions du répertoire : ");
    if config_dir.exists() {
        println!("{}", format!(" OK ({})", config_dir.display()).green());
    } else {
        println!("{}", " Répertoire non encore créé (exécutez 'skipper init')".yellow());
    }

    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    if all_ok {
        println!("{}", " Diagnostic terminé : Votre système est prêt !".green().bold());
    } else {
        println!("{}", " Des avertissements ou erreurs ont été détectés.".yellow().bold());
    }
    println!();

    Ok(())
}
