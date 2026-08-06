use crate::ipc::IpcClient;
use anyhow::Result;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};
use owo_colors::OwoColorize;
use skipper_core::{ConfigManager, Request, ResponseStatus};

pub async fn run() -> Result<()> {
    println!("{}", "Skipper 360 — Etat du Systeme".bold());
    println!(
        "{}",
        "──────────────────────────────────────────────────".dimmed()
    );

    let client = IpcClient::new()?;
    let is_running = client.is_daemon_running().await;

    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?;

    if is_running {
        let req = Request::new("status", serde_json::json!({}));
        match client.send_request(req).await {
            Ok(resp) => {
                if resp.status == ResponseStatus::Ok {
                    if let Some(val) = resp.data {
                        let active = val.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
                        let mode = val
                            .get("mode")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Standard");
                        let pid = val.get("pid").and_then(|v| v.as_u64()).unwrap_or(0);
                        let active_sessions = val
                            .get("active_sessions")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);

                        let state_str = if active {
                            "Actif".bold().green().to_string()
                        } else {
                            "Inactif".bold().yellow().to_string()
                        };

                        println!("  Statut        : {}", state_str);
                        println!("  Mode          : {}", mode.bold());
                        println!("  Daemon        : PID {}", pid);
                        println!("  Sessions      : {} actives", active_sessions);
                        println!(
                            "  Timeout       : {} secondes",
                            config.general.timeout_seconds
                        );
                        println!(
                            "  Config        : {}",
                            config_manager.config_path().display()
                        );
                    }
                }
            }
            Err(_) => {
                println!("  Statut        : Daemon non joignable");
            }
        }
    } else {
        println!("  Statut        : Arrêté");
        println!("  Mode          : {}", config.general.mode);
        println!(
            "  Config        : {}",
            config_manager.config_path().display()
        );
    }

    println!();
    println!("{}", "Whitelist des Commandes".bold());
    println!(
        "{}",
        "──────────────────────────────────────────────────".dimmed()
    );

    if config.whitelist.entries.is_empty() {
        println!("  Aucune commande dans la whitelist.");
    } else {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec![
                Cell::new("#").add_attribute(Attribute::Bold),
                Cell::new("Commande").add_attribute(Attribute::Bold),
                Cell::new("Mode").add_attribute(Attribute::Bold),
                Cell::new("TTL").add_attribute(Attribute::Bold),
            ]);

        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for (i, entry) in config.whitelist.entries.iter().enumerate() {
            let ttl_str = match entry.expires_at {
                Some(exp) => {
                    if exp > now_secs {
                        format!("{}s", exp - now_secs)
                    } else {
                        "Expiré".to_string()
                    }
                }
                None => "Permanent".to_string(),
            };

            table.add_row(vec![
                Cell::new(i + 1),
                Cell::new(&entry.command).fg(Color::Cyan),
                Cell::new(format!("{:?}", entry.match_mode)),
                Cell::new(ttl_str).fg(Color::Magenta),
            ]);
        }
        println!("{table}");
    }

    Ok(())
}
