use crate::handler::handle_request;
use crate::state::SharedState;
use skipper_core::{Request, Response, SkipperError, StreamMessage};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::Mutex;
use tracing::{error, info};

pub struct IpcServer {
    socket_path: PathBuf,
    state: SharedState,
}

impl IpcServer {
    pub fn new(state: SharedState) -> skipper_core::Result<Self> {
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

    pub async fn run(
        &self,
        mut shutdown: tokio::sync::broadcast::Receiver<()>,
    ) -> skipper_core::Result<()> {
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
                                let (reader, writer) = stream.into_split();
                                let mut buf_reader = BufReader::new(reader);
                                let writer_arc = Arc::new(Mutex::new(writer));

                                let mut line = String::new();

                                loop {
                                    line.clear();
                                    match buf_reader.read_line(&mut line).await {
                                        Ok(0) => break, // EOF
                                        Ok(_) => {
                                            if let Ok(req) = serde_json::from_str::<Request>(line.trim()) {
                                                let writer_for_stream = writer_arc.clone();
                                                let on_stream = move |stream_msg: StreamMessage| {
                                                    if let Ok(msg_json) = serde_json::to_string(&stream_msg) {
                                                        let w = writer_for_stream.clone();
                                                        tokio::spawn(async move {
                                                            let mut lock = w.lock().await;
                                                            let _ = lock.write_all(format!("{}\n", msg_json).as_bytes()).await;
                                                            let _ = lock.flush().await;
                                                        });
                                                    }
                                                };

                                                let response = handle_request(req, state.clone(), on_stream).await;
                                                if let Ok(resp_json) = serde_json::to_string(&response) {
                                                    let mut lock = writer_arc.lock().await;
                                                    let _ = lock.write_all(format!("{}\n", resp_json).as_bytes()).await;
                                                    let _ = lock.flush().await;
                                                }
                                            } else {
                                                let err_resp = Response::error("Requête JSON invalide", "0");
                                                if let Ok(resp_json) = serde_json::to_string(&err_resp) {
                                                    let mut lock = writer_arc.lock().await;
                                                    let _ = lock.write_all(format!("{}\n", resp_json).as_bytes()).await;
                                                    let _ = lock.flush().await;
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
