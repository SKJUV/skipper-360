use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use skipper_core::DiagnosticReport;
use std::thread;
use std::time::Duration;

pub fn run() -> Result<()> {
    println!("{}", "Skipper 360 — Diagnostic de Sante Systeme".bold());
    println!(
        "{}",
        "──────────────────────────────────────────────────".dimmed()
    );

    let pb = ProgressBar::new(4);
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.blue} {msg}")
            .expect("Style valide"),
    );

    pb.set_message("Vérification des composants système...");
    thread::sleep(Duration::from_millis(300));

    let report = DiagnosticReport::run();
    pb.finish_and_clear();

    let mut all_passed = true;
    for item in &report.items {
        if !item.status {
            all_passed = false;
        }

        let status_str = if item.status {
            "[OK]".bold().green().to_string()
        } else {
            "[FAIL]".bold().red().to_string()
        };

        println!(
            "  {:6} {:30} : {}",
            status_str,
            item.name.bold(),
            item.details
        );
    }

    println!(
        "{}",
        "──────────────────────────────────────────────────".dimmed()
    );
    if all_passed {
        println!(
            "{}",
            "[OK] Tous les composants sont sains et operationnels."
                .bold()
                .green()
        );
    } else {
        println!(
            "{}",
            "[WARN] Des anomalies ont ete detectees.".bold().yellow()
        );
    }

    Ok(())
}
