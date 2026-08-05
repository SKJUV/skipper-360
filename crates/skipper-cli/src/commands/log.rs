use anyhow::Result;
use owo_colors::OwoColorize;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn run() -> Result<()> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
        .join("skipper360");
    let log_file = config_dir.join("skipper.log");

    println!("\n{}", "📜 Skipper 360 — Journal des Événements & Interceptions".bold().cyan());
    println!("{}\n", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    if !log_file.exists() {
        println!("{}", "ℹ️ Aucun journal trouvé pour le moment (le fichier skipper.log sera créé lors des premières interceptions).".yellow());
        return Ok(());
    }

    let file = File::open(&log_file)?;
    let reader = BufReader::new(file);

    let mut count = 0;
    for line in reader.lines() {
        if let Ok(content) = line {
            count += 1;
            if content.contains("INTERCEPTED") || content.contains("MATCH_WHITELIST") {
                println!("{}", content.cyan());
            } else if content.contains("INJECT_START") {
                println!("{}", content.yellow());
            } else if content.contains("INJECT_SUCCESS") {
                println!("{}", content.green().bold());
            } else if content.contains("INJECT_FAIL") || content.contains("ERROR") {
                println!("{}", content.red().bold());
            } else {
                println!("{}", content.dimmed());
            }
        }
    }

    if count == 0 {
        println!("{}", "ℹ️ Le fichier journal est vide.".yellow());
    }

    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", format!("📊 Total d'entrées affichées : {}", count).bold());
    println!();

    Ok(())
}
