use anyhow::Result;
use dialoguer::{Input, Password};
use owo_colors::OwoColorize;
use secrecy::SecretString;
use skipper_core::{Config, ConfigManager, KeyringManager};

pub async fn run() -> Result<()> {
    println!("{}", "Skipper 360 — Initialisation Système".bold());
    println!(
        "{}",
        "──────────────────────────────────────────────────".dimmed()
    );

    let default_user = std::env::var("USER").unwrap_or_else(|_| "".into());

    let username: String = Input::new()
        .with_prompt("Nom d'utilisateur")
        .default(default_user)
        .interact_text()?;

    let password = Password::new()
        .with_prompt("Mot de passe par défaut (ex: sudo)")
        .interact()?;

    let secret = SecretString::from(password);
    KeyringManager::store_default_password(&secret)?;
    println!("[OK] Identifiants enregistrés dans le trousseau système.");

    let config_manager = ConfigManager::new()?;
    let mut config = Config::default();
    config.default_credentials.username = Some(username);

    config_manager.save(&config)?;
    crate::ipc::notify_daemon_reload().await;
    println!(
        "[OK] Fichier de configuration créé : {}",
        config_manager.config_path().display().to_string().cyan()
    );

    println!("[INFO] Configuration initiale : Mode Standard | Timeout 30s");
    println!(
        "{}",
        "──────────────────────────────────────────────────".dimmed()
    );
    println!(
        "Conseil : Tapez '{}' pour activer le service de surveillance.",
        "skipper activate".bold()
    );

    Ok(())
}
