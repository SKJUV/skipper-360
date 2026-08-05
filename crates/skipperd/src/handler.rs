use crate::pty::manager::PtyManager;
use crate::pty::session::PtySession;
use crate::state::SharedState;
use skipper_core::{
    ConfigManager, MatchMode, OperatingMode, Request, Response, SkipperStatus, StreamMessage,
};
use std::sync::Arc;

pub async fn handle_request<F>(req: Request, state: SharedState, on_stream: F) -> Response
where
    F: Fn(StreamMessage) + Send + Sync + 'static,
{
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
        "reload_config" => {
            if let Ok(manager) = ConfigManager::new() {
                if let Ok(new_config) = manager.load() {
                    let mut state_guard = state.write().await;
                    state_guard.config = new_config;
                    tracing::info!("Configuration du daemon rechargée avec succès.");
                    return Response::ok("Configuration rechargée", None, req.request_id);
                }
            }
            Response::error("Échec du rechargement de la configuration", req.request_id)
        }
        "whitelist_add" => {
            let cmd = req
                .args
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let mode_str = req
                .args
                .get("match_mode")
                .and_then(|v| v.as_str())
                .unwrap_or("prefix");
            let match_mode = match mode_str {
                "exact" => MatchMode::Exact,
                _ => MatchMode::Prefix,
            };

            if cmd.is_empty() {
                return Response::error("Commande whitelist vide", req.request_id);
            }

            let mut state_guard = state.write().await;
            state_guard.config.add_whitelist_entry(cmd, match_mode);

            if let Ok(manager) = ConfigManager::new() {
                let _ = manager.save(&state_guard.config);
            }

            Response::ok(
                format!("Commande '{}' ajoutée à la whitelist", cmd),
                None,
                req.request_id,
            )
        }
        "whitelist_delete" => {
            let cmd = req
                .args
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if cmd.is_empty() {
                return Response::error("Commande whitelist vide", req.request_id);
            }

            let mut state_guard = state.write().await;
            let removed = state_guard.config.remove_whitelist_entry(cmd);

            if removed {
                if let Ok(manager) = ConfigManager::new() {
                    let _ = manager.save(&state_guard.config);
                }
                Response::ok(
                    format!("Commande '{}' supprimée de la whitelist", cmd),
                    None,
                    req.request_id,
                )
            } else {
                Response::error(
                    format!("Commande '{}' introuvable dans la whitelist", cmd),
                    req.request_id,
                )
            }
        }
        "run" => {
            let command_args = req
                .args
                .get("command")
                .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
                .unwrap_or_default();

            if command_args.is_empty() {
                return Response::error(
                    "Aucune commande spécifiée dans la requête run",
                    req.request_id,
                );
            }

            let config = {
                let state_guard = state.read().await;
                state_guard.config.clone()
            };

            PtyManager::increment_active_sessions(&state).await;

            let on_stream = Arc::new(on_stream);
            let on_stream_clone = on_stream.clone();

            let result = tokio::task::spawn_blocking(move || {
                PtySession::run_and_stream(&command_args, &config, move |chunk| {
                    on_stream_clone(StreamMessage::Stdout(chunk.to_string()));
                })
            })
            .await;

            PtyManager::decrement_active_sessions(&state).await;

            match result {
                Ok(Ok(exit_code)) => Response::ok(
                    format!("Exécution terminée avec le code {}", exit_code),
                    Some(serde_json::json!({ "exit_code": exit_code })),
                    req.request_id,
                ),
                Ok(Err(e)) => {
                    Response::error(format!("Erreur d'exécution PTY: {}", e), req.request_id)
                }
                Err(e) => Response::error(
                    format!("Échec de la tâche d'exécution: {}", e),
                    req.request_id,
                ),
            }
        }
        unknown => Response::error(format!("Commande inconnue: {}", unknown), req.request_id),
    }
}
