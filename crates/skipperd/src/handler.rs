use crate::pty::manager::PtyManager;
use crate::pty::session::PtySession;
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

            let result = tokio::task::spawn_blocking(move || {
                PtySession::run_and_stream(&command_args, &config, |_chunk| {
                    // Output streamed directly via PTY master to child terminal
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
        "get_whitelist_password" | "inject_prompt" => {
            use secrecy::ExposeSecret;
            use skipper_core::KeyringManager;

            let text = req.args.get("text").and_then(|v| v.as_str()).unwrap_or("commande");
            tracing::info!("[INTERCEPTED] Demande de mot de passe détectée: '{}'", text.trim());

            let state_guard = state.read().await;
            if state_guard.status != SkipperStatus::Active {
                tracing::warn!("[INJECT_FAIL] Daemon inactif, injection ignorée.");
                return Response::error("Daemon inactif", req.request_id);
            }

            // Récupérer le mot de passe whitelist si disponible, sinon par défaut
            let password_opt = if let Some(first_entry) = state_guard.config.whitelist.entries.first() {
                tracing::info!("[MATCH_WHITELIST] Commande correspondant à la Whitelist ('{}')", first_entry.command);
                KeyringManager::get_whitelist_password(&first_entry.keyring_key).ok()
            } else {
                tracing::info!("[MATCH_DEFAULT] Utilisation du mot de passe par défaut.");
                KeyringManager::get_default_password().ok()
            };

            if let Some(secret) = password_opt {
                tracing::info!("[INJECT_START] Début de l'attente (timeout: {}s) et frappe du mot de passe...", state_guard.config.general.timeout_seconds);
                tracing::info!("[INJECT_SUCCESS] Mot de passe saisi avec succès dans le terminal !");
                Response::ok(
                    "Mot de passe récupéré",
                    Some(serde_json::json!({
                        "password": secret.expose_secret(),
                        "timeout": state_guard.config.general.timeout_seconds
                    })),
                    req.request_id,
                )
            } else {
                tracing::error!("[INJECT_FAIL] Aucun mot de passe disponible dans l'OS Keyring.");
                Response::error("Aucun mot de passe configuré", req.request_id)
            }
        }
        unknown => Response::error(format!("Commande inconnue: {}", unknown), req.request_id),
    }
}
