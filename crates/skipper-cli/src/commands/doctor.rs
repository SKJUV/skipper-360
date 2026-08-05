use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use skipper_core::DiagnosticReport;
use std::time::Duration;

pub fn run() -> Result<()> {
    println!(
        "{}",
        "🩺 Skipper 360 — Diagnostic Système (Doctor)".bold().blue()
    );
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} {msg}")
            .expect("Invalid spinner template"),
    );

    pb.set_message("Analyse des composants du système en cours...");
    pb.enable_steady_tick(Duration::from_millis(80));

    std::thread::sleep(Duration::from_millis(400));
    let report = DiagnosticReport::run();
    pb.finish_and_clear();

    let mut total_passed = 0;

    for item in &report.items {
        if item.status {
            total_passed += 1;
            println!("  ✅ {:<30} : {}", item.name.bold(), item.details.green());
        } else {
            println!("  ❌ {:<30} : {}", item.name.bold(), item.details.red());
        }
    }

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    if total_passed == report.items.len() {
        println!(
            "{}",
            "🎉 Tous les composants de Skipper 360 sont sains et opérationnels !"
                .green()
                .bold()
        );
    } else {
        println!(
            "{}",
            format!(
                "⚠️ {}/{} vérifications validées.",
                total_passed,
                report.items.len()
            )
            .yellow()
            .bold()
        );
    }

    Ok(())
}
