use anyhow::Result;
use owo_colors::OwoColorize;
use skipper_core::{get_shims_dir, install_shims, remove_shims, sync_shims, ConfigManager};

pub fn sync() -> Result<()> {
    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?;
    let shims_dir = get_shims_dir()?;
    let synced = sync_shims(&config)?;

    if synced.is_empty() {
        println!(
            "{}",
            format!(
                "[INFO] Aucun shim à synchroniser (la whitelist est vide). Répertoire : {}",
                shims_dir.display()
            )
            .yellow()
        );
    } else {
        println!(
            "{}",
            format!(
                "[OK] Synchronisation réussie de {} shim(s) dans {} : {}",
                synced.len(),
                shims_dir.display(),
                synced.join(", ")
            )
            .green()
            .bold()
        );
    }
    Ok(())
}

pub fn install() -> Result<()> {
    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?;
    let shims_dir = get_shims_dir()?;
    let installed = install_shims(&config)?;

    println!(
        "{}",
        format!(
            "[OK] Répertoire de shims configuré avec succès : {}",
            shims_dir.display()
        )
        .green()
        .bold()
    );

    if installed.is_empty() {
        println!("[INFO] Aucune commande whitelistée à synchroniser actuellement.");
    } else {
        println!(
            "[OK] Shims générés pour les commandes suivantes : {}",
            installed.join(", ").cyan()
        );
    }

    println!(
        "\n{}",
        format!(
            "  Assurez-vous que {} est présent au début de votre variable PATH.\n  Exemple : export PATH=\"{}:$PATH\"",
            shims_dir.display(),
            shims_dir.display()
        )
        .dimmed()
    );

    Ok(())
}

pub fn remove() -> Result<()> {
    let shims_dir = get_shims_dir()?;
    let count = remove_shims()?;

    println!(
        "{}",
        format!(
            "[OK] {} shim(s) supprimé(s) du répertoire {}.",
            count,
            shims_dir.display()
        )
        .yellow()
        .bold()
    );

    Ok(())
}
