use anyhow::Result;
use dialoguer::{Input, Password};
use owo_colors::OwoColorize;
use secrecy::SecretString;
use skipper_core::{ConfigManager, KeyringManager};

pub fn run() -> Result<()> {
    println!("{}", "🛡️  Skipper 360 — Initialisation".bold().blue());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let username: String = Input::new()
        .with_prompt("👤 Nom d'utilisateur")
        .interact_text()?;

    let password = Password::new()
        .with_prompt("🔒 Mot de passe par défaut")
        .with_confirmation("🔒 Confirmez le mot de passe", "Les mots de passe ne correspondent pas")
        .interact()?;

    let secret_password = SecretString::from(password);

    // Stockage sécurisé dans le trousseau système OS
    match KeyringManager::store_default_password(&secret_password) {
        Ok(_) => println!("{}", "✅ Identifiants stockés dans le trousseau système.".green().bold()),
        Err(e) => {
            println!("{}", format!("⚠️  Avertissement trousseau : {}", e).yellow());
            println!("{}", "Création du fichier de configuration quand même...".dimmed());
        }
    }

    let manager = ConfigManager::new()?;
    let mut config = manager.load().unwrap_or_default();
    config.default_credentials.username = Some(username);

    manager.save(&config)?;

    println!("{}", format!("📁 Configuration créée : {}", manager.config_path().display()).dimmed());
    println!("{}", format!("🔧 Mode : {} | ⏱  Timeout : {}s", config.general.mode.cyan(), config.general.timeout_seconds.to_string().yellow()).bold());
    println!();
    println!("{}", "💡 Conseil : Tapez 'skipper activate' pour démarrer la surveillance.".green());

    Ok(())
}
