use anyhow::Result;
use owo_colors::OwoColorize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn get_log_dir() -> Result<PathBuf> {
    let base_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Impossible de localiser le répertoire config"))?;
    Ok(base_dir.join("skipper360").join("logs"))
}

pub fn show(lines: usize) -> Result<()> {
    println!("{}", "Skipper 360 — Journaux d'Execution Daemon".bold());
    println!(
        "{}",
        "──────────────────────────────────────────────────".dimmed()
    );

    let log_dir = get_log_dir()?;
    if !log_dir.exists() {
        println!("  Aucun fichier de journalisation trouvé.");
        return Ok(());
    }

    let mut log_files: Vec<_> = fs::read_dir(&log_dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("")
                    .starts_with("skipper.log")
        })
        .collect();

    log_files.sort();

    if log_files.is_empty() {
        println!(
            "  Aucun journal d'exécution actif dans {}.",
            log_dir.display()
        );
        return Ok(());
    }

    let latest_log = log_files.last().unwrap();
    let content = fs::read_to_string(latest_log)?;
    let all_lines: Vec<&str> = content.lines().collect();

    let total = all_lines.len();
    let start = total.saturating_sub(lines);

    for line in &all_lines[start..] {
        println!("  {}", line);
    }

    println!(
        "{}",
        "──────────────────────────────────────────────────".dimmed()
    );
    println!(
        "  Fichier : {} | Total lignes : {}",
        latest_log.display().to_string().cyan(),
        total
    );

    Ok(())
}

pub fn clear() -> Result<()> {
    let log_dir = get_log_dir()?;
    if log_dir.exists() {
        for entry in fs::read_dir(&log_dir)?.flatten() {
            let path = entry.path();
            if path.is_file() {
                let _ = fs::remove_file(&path);
            }
        }
        let _ = fs::set_permissions(&log_dir, fs::Permissions::from_mode(0o700));
    }

    println!(
        "{}",
        "[OK] Fichiers de journaux nettoyés avec succès."
            .green()
            .bold()
    );
    Ok(())
}
