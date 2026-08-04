use skipper_core::{Config, ConfigManager, OperatingMode, SkipperStatus};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct DaemonState {
    pub status: SkipperStatus,
    pub mode: OperatingMode,
    pub config: Config,
    pub active_sessions_count: usize,
    pub pid: u32,
}

impl DaemonState {
    pub fn new(config: Config) -> Self {
        let mode = config.general.mode;
        Self {
            status: SkipperStatus::Inactive,
            mode,
            config,
            active_sessions_count: 0,
            pid: std::process::id(),
        }
    }
}

pub type SharedState = Arc<RwLock<DaemonState>>;

pub fn create_shared_state() -> SharedState {
    let config_manager = ConfigManager::new()
        .unwrap_or_else(|_| ConfigManager::with_custom_path("config.toml".into()));
    let config = config_manager.load().unwrap_or_default();
    Arc::new(RwLock::new(DaemonState::new(config)))
}
