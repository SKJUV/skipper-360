mod handler;
mod injector;
mod pid;
mod pty;
mod server;
mod state;

use anyhow::Result;
use notify::{Event, RecursiveMode, Watcher};
use pid::PidFile;
use server::IpcServer;
use skipper_core::apply_kernel_hardened_prctl;
use state::create_shared_state;
use std::os::unix::fs::PermissionsExt;
use tokio::signal::unix::{signal, SignalKind};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Setup rolling log appender to ~/.config/skipper360/logs/skipper.log
    let _log_guard = if let Some(config_dir) = dirs::config_dir() {
        let log_dir = config_dir.join("skipper360").join("logs");
        let _ = std::fs::create_dir_all(&log_dir);

        // Restrict directory permissions to owner only (0o700)
        let _ = std::fs::set_permissions(&log_dir, std::fs::Permissions::from_mode(0o700));

        let file_appender = tracing_appender::rolling::daily(&log_dir, "skipper.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::fmt()
            .with_writer(non_blocking)
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing::Level::INFO.into()),
            )
            .init();
        Some(guard)
    } else {
        tracing_subscriber::fmt::init();
        None
    };

    info!("Démarrage du Daemon Skipper 360 (skipperd)...");

    // 2. Prevent process dump & memory inspection for security
    apply_kernel_hardened_prctl();
    #[cfg(target_os = "linux")]
    {
        use nix::sys::prctl;
        if let Err(e) = prctl::set_dumpable(false) {
            tracing::warn!("Impossible de désactiver PR_SET_DUMPABLE: {}", e);
        }
    }

    // 3. Single instance PID file acquire
    let pid_file = match PidFile::new() {
        Ok(pf) => pf,
        Err(e) => {
            tracing::error!("Erreur d'initialisation PID: {}", e);
            eprintln!("[ERR] Erreur d'initialisation PID: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = pid_file.acquire() {
        tracing::error!("{}", e);
        eprintln!("[ERR] {}", e);
        std::process::exit(1);
    }

    let state = create_shared_state();
    let server = IpcServer::new(state.clone())?;

    // 4. Dynamic Config Hot-Reloading using notify file watcher
    let state_for_watcher = state.clone();
    tokio::spawn(async move {
        if let Ok(cm) = skipper_core::ConfigManager::new() {
            let config_path = cm.config_path().clone();
            let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(10);

            let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if event.kind.is_modify() || event.kind.is_create() {
                        let _ = tx.blocking_send(());
                    }
                }
            });

            if let Ok(ref mut w) = watcher {
                if let Some(parent) = config_path.parent() {
                    let _ = w.watch(parent, RecursiveMode::NonRecursive);
                }
            }

            while rx.recv().await.is_some() {
                if let Ok(updated_config) = cm.load() {
                    let mut guard = state_for_watcher.write().await;
                    guard.config = updated_config;
                    tracing::info!("Configuration rechargée dynamiquement à chaud !");
                }
            }
        }
    });

    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);

    // 5. Listen for graceful shutdown signals (SIGINT / SIGTERM)
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
        "[OK] Daemon skipperd démarré avec succès (PID {}).",
        std::process::id()
    );

    // Run IPC server
    server.run(shutdown_rx).await?;

    info!("Daemon arrêté proprement.");
    Ok(())
}
