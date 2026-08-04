use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("🛡️ Démarrage du Daemon Skipper 360 (skipperd)...");

    // Prevent process dump & memory inspection for security
    #[cfg(target_os = "linux")]
    {
        use nix::sys::prctl;
        if let Err(e) = prctl::set_dumpable(false) {
            tracing::warn!("Impossible de désactiver PR_SET_DUMPABLE: {}", e);
        }
    }

    info!("Daemon prêt (en attente des phases suivantes).");
    Ok(())
}
