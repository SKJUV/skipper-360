use crate::ipc::IpcClient;
use anyhow::Result;
use owo_colors::OwoColorize;
use skipper_core::{ConfigManager, Request, SkipperStatus};

pub async fn run() -> Result<()> {
    println!("{}", "🛡️  Skipper 360 — État du Système".bold().blue());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let manager = ConfigManager::new()?;
    let config = manager.load()?;

    let client = IpcClient::new()?;
    let req = Request::new("status", serde_json::json!({}));

    match client.send_request(req).await {
        Ok(response) => {
            if let Some(data) = response.data {
                let status_str = data
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("active");
                let mode_str = data
                    .get("mode")
                    .and_then(|m| m.as_str())
                    .unwrap_or("standard");
                let pid = data.get("pid").and_then(|p| p.as_u64()).unwrap_or(0);
                let active_sessions = data
                    .get("active_sessions")
                    .and_then(|a| a.as_u64())
                    .unwrap_or(0);

                let status = if status_str == "active" {
                    SkipperStatus::Active
                } else {
                    SkipperStatus::Inactive
                };

                println!("   État     : {}", status);
                println!("   Mode     : 📢 {}", mode_str.cyan());
                println!("   Daemon   : ✅ PID {}", pid.to_string().yellow());
                println!(
                    "   Sessions : 🖥️  {} actives",
                    active_sessions.to_string().cyan()
                );
                println!(
                    "   Timeout  : ⏱  {} secondes",
                    config.general.timeout_seconds.to_string().yellow()
                );
                println!(
                    "   Config   : 📁 {}",
                    manager.config_path().display().to_string().dimmed()
                );
            }
        }
        Err(_) => {
            println!("   État     : {}", SkipperStatus::Inactive);
            println!("   Daemon   : 🔴 Non démarré");
            println!("   Mode     : 📢 {}", config.general.mode.cyan());
            println!(
                "   Timeout  : ⏱  {} secondes",
                config.general.timeout_seconds.to_string().yellow()
            );
            println!(
                "   Config   : 📁 {}",
                manager.config_path().display().to_string().dimmed()
            );
        }
    }

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
