use anyhow::{anyhow, Result};
use skipper_core::{Request, Response, ResponseStatus};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

pub struct IpcClient {
    socket_path: PathBuf,
}

impl IpcClient {
    pub fn new() -> Result<Self> {
        let base_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Impossible de localiser le répertoire config"))?;
        let socket_path = base_dir.join("skipper360").join("skipper.sock");
        Ok(Self { socket_path })
    }

    pub fn is_daemon_running(&self) -> bool {
        self.socket_path.exists()
    }

    pub async fn ensure_daemon_running(&self) -> Result<()> {
        if self.is_daemon_running() {
            return Ok(());
        }

        println!("⚙️  Démarrage automatique du daemon skipperd en arrière-plan...");

        let spawn_res = std::process::Command::new("skipperd")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        match spawn_res {
            Ok(_) => {
                // Wait up to 1.5 seconds for socket file to be created by daemon
                for _ in 0..15 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    if self.is_daemon_running() {
                        return Ok(());
                    }
                }
                Err(anyhow!(
                    "Le daemon skipperd a été démarré mais le socket IPC n'a pas répondu à temps ({})",
                    self.socket_path.display()
                ))
            }
            Err(e) => Err(anyhow!(
                "Impossible d'exécuter l'exécutable 'skipperd' ({}) : vérifiez qu'il est présent dans votre PATH.",
                e
            )),
        }
    }

    pub async fn send_request(&self, req: Request) -> Result<Response> {
        if !self.socket_path.exists() {
            return Err(anyhow!(
                "Le daemon skipperd n'est pas démarré (socket introuvable : {}).\n💡 Conseil : Lancer 'skipper activate' pour démarrer automatiquement le daemon.",
                self.socket_path.display()
            ));
        }

        let stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            anyhow!(
                "Impossible de se connecter au socket du daemon ({}): {}",
                self.socket_path.display(),
                e
            )
        })?;

        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);

        let req_json = serde_json::to_string(&req)?;
        writer
            .write_all(format!("{}\n", req_json).as_bytes())
            .await?;

        let mut line = String::new();
        buf_reader.read_line(&mut line).await?;

        let response: Response = serde_json::from_str(line.trim())
            .map_err(|e| anyhow!("Erreur de parsing de la réponse JSON du daemon: {}", e))?;

        if response.status == ResponseStatus::Error {
            return Err(anyhow!("{}", response.message));
        }

        Ok(response)
    }
}
