mod focus;
mod handler;
mod injector;
mod pid;
mod pty;
mod server;
mod state;

use anyhow::Result;
use pid::PidFile;
use server::IpcServer;
use state::create_shared_state;
use tokio::signal::unix::{signal, SignalKind};
use tracing::info;
use tracing_subscriber::fmt::writer::MakeWriterExt;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup file logging to ~/.config/skipper360/skipper.log
    if let Some(config_dir) = dirs::config_dir() {
        let log_dir = config_dir.join("skipper360");
        let _ = std::fs::create_dir_all(&log_dir);
        let log_file = log_dir.join("skipper.log");

        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
        {
            let file_appender = file.with_max_level(tracing::Level::INFO);
            tracing_subscriber::fmt()
                .with_writer(file_appender)
                .with_env_filter(
                    tracing_subscriber::EnvFilter::from_default_env()
                        .add_directive(tracing::Level::INFO.into()),
                )
                .init();
        } else {
            tracing_subscriber::fmt::init();
        }
    } else {
        tracing_subscriber::fmt::init();
    }

    info!("🛡️ Démarrage du Daemon Skipper 360 (skipperd)...");

    // Prevent process dump & memory inspection for security
    #[cfg(target_os = "linux")]
    {
        use nix::sys::prctl;
        if let Err(e) = prctl::set_dumpable(false) {
            tracing::warn!("Impossible de désactiver PR_SET_DUMPABLE: {}", e);
        }
    }

    // Single instance PID file acquire
    let pid_file = match PidFile::new() {
        Ok(pf) => pf,
        Err(e) => {
            tracing::error!("Erreur d'initialisation PID: {}", e);
            eprintln!("❌ Erreur d'initialisation PID: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = pid_file.acquire() {
        tracing::error!("{}", e);
        eprintln!("❌ {}", e);
        std::process::exit(1);
    }

    let state = create_shared_state();
    let server = IpcServer::new(state.clone())?;

    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);

    // Listen for graceful shutdown signals (SIGINT / SIGTERM)
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to listen for SIGTERM");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("SIGINT (Ctrl+C) reçu.");
            }
            _ = sigterm.recv() => {
                info!("SIGTERM reçu.");
            }
        }
        let _ = shutdown_tx_clone.send(());
    });

    info!("Daemon prêt et en cours d'exécution.");
    println!(
        "🟢 Daemon skipperd démarré avec succès (PID {}).",
        std::process::id()
    );

    // Run IPC server
    server.run(shutdown_rx).await?;

    info!("Daemon arrêté proprement.");
    Ok(())
}
