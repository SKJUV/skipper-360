mod commands;
mod ipc;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use ipc::IpcClient;
use owo_colors::OwoColorize;
use skipper_core::Request;

#[derive(Parser)]
#[command(
    name = "skipper",
    version,
    about = "Skipper 360 - Password Keeper & Terminal Supervisor Daemon",
    long_about = "Skipper 360 monitors terminal PTYs, detects password prompts in real-time, and injects secure OS Keyring credentials."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialiser Skipper avec vos identifiants par défaut
    Init,
    /// Activer la surveillance des prompts
    Activate,
    /// Désactiver la surveillance
    Deactivate,
    /// Afficher l'état actuel du système
    Status,
    /// Configurer le mode opérationnel (standard ou silent)
    Mode {
        #[arg(value_name = "MODE")]
        mode: String,
    },
    /// Gérer la liste blanche de commandes
    #[command(alias = "w")]
    Whitelist {
        #[command(subcommand)]
        action: Option<WhitelistAction>,
    },
    /// Gérer les shims de commande transparents
    #[command(alias = "s")]
    Shims {
        #[command(subcommand)]
        action: ShimsAction,
    },
    /// Exécuter une commande sous la surveillance de Skipper
    Run {
        /// La commande à exécuter
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },
    /// Afficher le journal d'audit de sécurité
    Audit,
    /// Générer les scripts d'auto-complétion pour votre shell (bash, zsh, fish)
    Completion {
        /// Nom du shell (bash, zsh, fish)
        #[arg(value_name = "SHELL")]
        shell: String,
    },
    /// Diagnostic de santé de l'installation système
    Doctor,
    /// Réinitialiser toute la configuration et le trousseau
    Reset,
}

#[derive(Subcommand)]
enum WhitelistAction {
    /// Ajouter une commande à la whitelist
    Add {
        /// Mode de correspondance ("prefix" ou "exact")
        #[arg(short, long, value_name = "MODE")]
        mode: Option<String>,
        /// Durée de vie temporaire (ex: 30s, 10m, 1h, 2d)
        #[arg(short, long, value_name = "DURATION")]
        ttl: Option<String>,
        /// La commande à surveiller
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },
    /// Supprimer une commande de la whitelist
    Delete {
        /// La commande à supprimer
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },
    /// Lister les commandes de la whitelist
    List {
        /// Afficher les mots de passe en clair (nécessite sudo)
        #[arg(short, long)]
        show: bool,
    },
}

#[derive(Subcommand)]
enum ShimsAction {
    /// Synchroniser les shims avec la whitelist actuelle
    Sync,
    /// Installer le répertoire de shims et synchroniser
    Install,
    /// Supprimer tous les shims générés
    Remove,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(argv0) = args.first() {
        if skipper_core::is_shim_invocation(argv0) {
            let shim_args = &args[1..];
            let _ = skipper_core::run_shim(argv0, shim_args);
            std::process::exit(1);
        }
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init::run()?,
        Commands::Status => commands::status::run().await?,
        Commands::Doctor => commands::doctor::run()?,
        Commands::Audit => commands::audit::run()?,
        Commands::Completion { shell } => {
            let mut cmd = Cli::command();
            commands::completion::generate_completion(&shell, &mut cmd)?;
        }
        Commands::Activate => {
            let client = IpcClient::new()?;
            if let Err(e) = client.ensure_daemon_running().await {
                eprintln!(
                    "[ERR] {}",
                    format!("Échec au démarrage du daemon : {}", e).red()
                );
                return Ok(());
            }

            let req = Request::new("activate", serde_json::json!({}));
            match client.send_request(req).await {
                Ok(resp) => println!("[OK] {}", resp.message.bold().green()),
                Err(e) => eprintln!("[ERR] {}", format!("Échec d'activation : {}", e).red()),
            }
        }
        Commands::Deactivate => {
            let client = IpcClient::new()?;
            let req = Request::new("deactivate", serde_json::json!({}));
            match client.send_request(req).await {
                Ok(resp) => println!("[INFO] {}", resp.message.bold().yellow()),
                Err(e) => eprintln!("[ERR] {}", format!("Échec de désactivation : {}", e).red()),
            }
        }
        Commands::Mode { mode } => {
            let client = IpcClient::new()?;
            let req = Request::new("set_mode", serde_json::json!({ "mode": mode }));
            match client.send_request(req).await {
                Ok(resp) => println!("[INFO] {}", resp.message.bold().cyan()),
                Err(e) => eprintln!(
                    "[ERR] {}",
                    format!("Échec de changement de mode : {}", e).red()
                ),
            }
        }
        Commands::Whitelist { action } => match action {
            Some(WhitelistAction::Add { command, mode, ttl }) => {
                commands::whitelist::add(&command, mode, ttl).await?;
            }
            Some(WhitelistAction::Delete { command }) => {
                commands::whitelist::delete(&command).await?;
            }
            Some(WhitelistAction::List { show }) => {
                commands::whitelist::list(show)?;
            }
            None => {
                commands::whitelist::list(false)?;
            }
        },
        Commands::Shims { action } => match action {
            ShimsAction::Sync => commands::shims::sync()?,
            ShimsAction::Install => commands::shims::install()?,
            ShimsAction::Remove => commands::shims::remove()?,
        },
        Commands::Run { command } => {
            commands::run::run(&command).await?;
        }
        Commands::Reset => {
            println!(
                "[WARN] {}",
                "Réinitialisation de la configuration effectuée.".yellow()
            );
        }
    }

    Ok(())
}
