use anyhow::Result;
use owo_colors::OwoColorize;
use skipper_core::ConfigManager;

pub async fn run(seconds: u32) -> Result<()> {
    let config_manager = ConfigManager::new()?;
    let mut config = config_manager.load()?;

    config.general.timeout_seconds = seconds;
    config_manager.save(&config)?;

    // Notifier le daemon via IPC
    crate::ipc::notify_daemon_reload().await;

    println!(
        "[OK] {}",
        format!("Timeout intelligent mis à jour : {} seconde(s).", seconds)
            .bold()
            .green()
    );

    Ok(())
}
