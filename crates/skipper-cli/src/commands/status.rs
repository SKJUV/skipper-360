use anyhow::Result;
use owo_colors::OwoColorize;
use skipper_core::{ConfigManager, SkipperStatus};

pub fn run() -> Result<()> {
    println!("{}", "🛡️  Skipper 360 — État du Système".bold().blue());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let manager = ConfigManager::new()?;
    let config = manager.load()?;

    let status = SkipperStatus::Inactive; // Sera interrogé via IPC Daemon en Phase 2

    println!("   État     : {}", status);
    println!("   Mode     : 📢 {}", config.general.mode.cyan());
    println!(
        "   Timeout  : ⏱  {} secondes",
        config.general.timeout_seconds.to_string().yellow()
    );
    println!(
        "   Config   : 📁 {}",
        manager.config_path().display().to_string().dimmed()
    );

    println!();
    println!(
        "{}",
        format!(
            "📋 Whitelist ({} entrées) :",
            config.whitelist.entries.len()
        )
        .bold()
    );
    if config.whitelist.entries.is_empty() {
        println!(
            "{}",
            "   Aucune commande dans la whitelist pour le moment."
                .italic()
                .dimmed()
        );
    } else {
        for (i, entry) in config.whitelist.entries.iter().enumerate() {
            println!(
                "   {:2}. {} [{:?}]",
                i + 1,
                entry.command.bold(),
                entry.match_mode
            );
        }
    }

    Ok(())
}
