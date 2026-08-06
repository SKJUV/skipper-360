use anyhow::Result;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};
use owo_colors::OwoColorize;
use skipper_core::AuditLogger;

pub fn run() -> Result<()> {
    println!("{}", "Skipper 360 — Journal d'Audit de Securite".bold());
    println!(
        "{}",
        "──────────────────────────────────────────────────".dimmed()
    );

    let logger = AuditLogger::new()?;
    let entries = logger.read_entries()?;

    if entries.is_empty() {
        println!("  Aucun événement d'audit enregistré.");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Timestamp").add_attribute(Attribute::Bold),
            Cell::new("Utilisateur").add_attribute(Attribute::Bold),
            Cell::new("Action").add_attribute(Attribute::Bold),
            Cell::new("Commande").add_attribute(Attribute::Bold),
            Cell::new("Détails").add_attribute(Attribute::Bold),
        ]);

    for entry in entries.iter().rev().take(50) {
        let action_cell = match entry.action {
            skipper_core::AuditAction::CommandExecuted => {
                Cell::new("CommandExecuted").fg(Color::Blue)
            }
            skipper_core::AuditAction::WhitelistMatched => {
                Cell::new("WhitelistMatched").fg(Color::Magenta)
            }
            skipper_core::AuditAction::TimeoutElapsed => {
                Cell::new("TimeoutElapsed").fg(Color::Yellow)
            }
            skipper_core::AuditAction::PasswordInjected => {
                Cell::new("PasswordInjected").fg(Color::Green)
            }
            skipper_core::AuditAction::PasswordInjectionFailed => {
                Cell::new("InjectionFailed").fg(Color::Red)
            }
            skipper_core::AuditAction::PromptDetected => {
                Cell::new("PromptDetected").fg(Color::Yellow)
            }
            skipper_core::AuditAction::WhitelistAdded => {
                Cell::new("WhitelistAdded").fg(Color::Cyan)
            }
            skipper_core::AuditAction::WhitelistDeleted => {
                Cell::new("WhitelistDeleted").fg(Color::Red)
            }
            _ => Cell::new(format!("{:?}", entry.action)),
        };

        table.add_row(vec![
            Cell::new(&entry.timestamp).fg(Color::DarkGrey),
            Cell::new(&entry.user),
            action_cell,
            Cell::new(&entry.command).fg(Color::Cyan),
            Cell::new(&entry.details),
        ]);
    }

    println!("{table}");
    Ok(())
}
