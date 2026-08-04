use crate::state::SharedState;
use skipper_core::{ConfigManager, OperatingMode, Request, Response, SkipperStatus};

pub async fn handle_request(req: Request, state: SharedState) -> Response {
    match req.command.as_str() {
        "ping" => Response::ok("pong", None, req.request_id),
        "activate" => {
            let mut state_guard = state.write().await;
            state_guard.status = SkipperStatus::Active;
            tracing::info!("Status changed to Active");
            Response::ok("Daemon activé", None, req.request_id)
        }
        "deactivate" => {
            let mut state_guard = state.write().await;
            state_guard.status = SkipperStatus::Inactive;
            tracing::info!("Status changed to Inactive");
            Response::ok("Daemon désactivé", None, req.request_id)
        }
        "status" => {
            let state_guard = state.read().await;
            let data = serde_json::json!({
                "status": state_guard.status,
                "mode": state_guard.mode,
                "pid": state_guard.pid,
                "active_sessions": state_guard.active_sessions_count,
                "whitelist_count": state_guard.config.whitelist.entries.len(),
            });
            Response::ok("Statut du daemon", Some(data), req.request_id)
        }
        "set_mode" => {
            let mode_str = req
                .args
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("standard");
            let mode = match mode_str.to_lowercase().as_str() {
                "silent" => OperatingMode::Silent,
                _ => OperatingMode::Standard,
            };

            let mut state_guard = state.write().await;
            state_guard.mode = mode;
            state_guard.config.general.mode = mode;

            if let Ok(manager) = ConfigManager::new() {
                let _ = manager.save(&state_guard.config);
            }

            Response::ok(format!("Mode changé vers {}", mode), None, req.request_id)
        }
        unknown => Response::error(format!("Commande inconnue: {}", unknown), req.request_id),
    }
}
