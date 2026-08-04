mod commands;
mod ipc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use ipc::IpcClient;
use owo_colors::OwoColorize;
use skipper_core::Request;

#[derive(Parser)]
#[command(
    name = "skipper",
    version,
    about = "🛡️ Skipper 360 — Gardien automatique de mots de passe",
    long_about = "Skipper 360 est un outil CLI et daemon qui surveille les PTYs et injecte automatiquement les mots de passe de manière sécurisée."
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
    /// Exécuter une commande sous la surveillance de Skipper
    Run {
        /// La commande à exécuter
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },
    /// Réinitialiser toute la configuration et le trousseau
    Reset,
}

#[derive(Subcommand)]
enum WhitelistAction {
    /// Ajouter une commande à la whitelist
    Add {
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },
    /// Supprimer une commande de la whitelist
    Delete {
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },
    /// Lister les commandes de la whitelist
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init::run()?,
        Commands::Status => commands::status::run().await?,
        Commands::Activate => {
            let client = IpcClient::new()?;
            if let Err(e) = client.ensure_daemon_running().await {
                eprintln!(
                    "{}",
                    format!("❌ Échec au démarrage du daemon : {}", e).red()
                );
                return Ok(());
            }

            let req = Request::new("activate", serde_json::json!({}));
            match client.send_request(req).await {
                Ok(resp) => println!("{}", format!("🟢 {}", resp.message).green().bold()),
                Err(e) => eprintln!("{}", format!("❌ Échec d'activation : {}", e).red()),
            }
        }
        Commands::Deactivate => {
            let client = IpcClient::new()?;
            let req = Request::new("deactivate", serde_json::json!({}));
            match client.send_request(req).await {
                Ok(resp) => println!("{}", format!("🔴 {}", resp.message).yellow().bold()),
                Err(e) => eprintln!("{}", format!("❌ Échec de désactivation : {}", e).red()),
            }
        }
        Commands::Mode { mode } => {
            let client = IpcClient::new()?;
            let req = Request::new("set_mode", serde_json::json!({ "mode": mode }));
            match client.send_request(req).await {
                Ok(resp) => println!("{}", format!("⚙️  {}", resp.message).cyan().bold()),
                Err(e) => eprintln!(
                    "{}",
                    format!("❌ Échec de changement de mode : {}", e).red()
                ),
            }
        }
        Commands::Whitelist { action } => match action {
            Some(WhitelistAction::Add { command }) => {
                println!(
                    "{}",
                    format!("➕ Ajout à la whitelist: {}", command.join(" ")).green()
                );
            }
            Some(WhitelistAction::Delete { command }) => {
                println!(
                    "{}",
                    format!("➖ Suppression de la whitelist: {}", command.join(" ")).yellow()
                );
            }
            Some(WhitelistAction::List) | None => {
                commands::status::run().await?;
            }
        },
        Commands::Run { command } => {
            if command.is_empty() {
                println!("{}", "❌ Veuillez préciser une commande à exécuter (ex: skipper run ssh user@server)".red());
            } else {
                println!(
                    "{}",
                    format!("🚀 Exécution surveillée de: {}", command.join(" ")).bold()
                );
            }
        }
        Commands::Reset => {
            println!("{}", "⚠️ Réinitialisation effectuée.".yellow());
        }
    }

    Ok(())
}
