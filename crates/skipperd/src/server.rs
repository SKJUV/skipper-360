use crate::handler::handle_request;
use crate::state::SharedState;
use skipper_core::{Request, Result, SkipperError};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tracing::{error, info};

pub struct IpcServer {
    socket_path: PathBuf,
    state: SharedState,
}

impl IpcServer {
    pub fn new(state: SharedState) -> Result<Self> {
        let base_dir = dirs::config_dir().ok_or_else(|| {
            SkipperError::Config("Impossible de localiser le répertoire config".into())
        })?;
        let socket_dir = base_dir.join("skipper360");
        if !socket_dir.exists() {
            fs::create_dir_all(&socket_dir)?;
        }
        let socket_path = socket_dir.join("skipper.sock");

        Ok(Self { socket_path, state })
    }

    pub async fn run(&self, mut shutdown: tokio::sync::broadcast::Receiver<()>) -> Result<()> {
        if self.socket_path.exists() {
            let _ = fs::remove_file(&self.socket_path);
        }

        let listener = UnixListener::bind(&self.socket_path).map_err(|e| {
            SkipperError::Ipc(format!(
                "Échec de liaison du socket Unix {}: {}",
                self.socket_path.display(),
                e
            ))
        })?;

        // Restrict socket permissions to owner only (0o600)
        let _ = fs::set_permissions(&self.socket_path, fs::Permissions::from_mode(0o600));

        info!("Serveur IPC à l'écoute sur {}", self.socket_path.display());

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, _)) => {
                            let state = self.state.clone();
                            tokio::spawn(async move {
                                let (reader, mut writer) = stream.into_split();
                                let mut buf_reader = BufReader::new(reader);
                                let mut line = String::new();

                                loop {
                                    line.clear();
                                    match buf_reader.read_line(&mut line).await {
                                        Ok(0) => break, // EOF
                                        Ok(_) => {
                                            if let Ok(req) = serde_json::from_str::<Request>(line.trim()) {
                                                let response = handle_request(req, state.clone()).await;
                                                if let Ok(resp_json) = serde_json::to_string(&response) {
                                                    let _ = writer.write_all(format!("{}\n", resp_json).as_bytes()).await;
                                                }
                                            } else {
                                                let err_resp = skipper_core::Response::error("Requête JSON invalide", "0");
                                                if let Ok(resp_json) = serde_json::to_string(&err_resp) {
                                                    let _ = writer.write_all(format!("{}\n", resp_json).as_bytes()).await;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("Erreur de lecture du stream IPC client: {}", e);
                                            break;
                                        }
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            error!("Échec d'acceptation de connexion IPC: {}", e);
                        }
                    }
                }
                _ = shutdown.recv() => {
                    info!("Signal d'arrêt reçu par le serveur IPC.");
                    break;
                }
            }
        }

        if self.socket_path.exists() {
            let _ = fs::remove_file(&self.socket_path);
        }

        Ok(())
    }
}
