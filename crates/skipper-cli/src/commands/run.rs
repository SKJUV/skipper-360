use crate::ipc::IpcClient;
use anyhow::{anyhow, Result};
use owo_colors::OwoColorize;
use skipper_core::{Request, Response, ResponseStatus, StreamMessage};
use std::io::Write;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

pub async fn run(command: &[String]) -> Result<()> {
    if command.is_empty() {
        return Err(anyhow!(
            "Veuillez spécifier une commande à exécuter (ex: skipper run sudo pacman -Syu)"
        ));
    }

    let client = IpcClient::new()?;
    client.ensure_daemon_running().await?;

    let req = Request::new("run", serde_json::json!({ "command": command }));

    let base_dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("Impossible de localiser le répertoire config"))?;
    let socket_path = base_dir.join("skipper360").join("skipper.sock");

    let stream = UnixStream::connect(&socket_path).await.map_err(|e| {
        anyhow!(
            "Impossible de se connecter au socket du daemon ({}): {}",
            socket_path.display(),
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

    loop {
        line.clear();
        let bytes_read = buf_reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break; // EOF
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Try parsing as StreamMessage first (real-time PTY output chunks)
        if let Ok(stream_msg) = serde_json::from_str::<StreamMessage>(trimmed) {
            match stream_msg {
                StreamMessage::Stdout(text) => {
                    print!("{}", text);
                    let _ = std::io::stdout().flush();
                }
                StreamMessage::Stderr(text) => {
                    eprint!("{}", text);
                    let _ = std::io::stderr().flush();
                }
                StreamMessage::PromptDetected { .. } => {}
                StreamMessage::PasswordInjected { .. } => {}
                StreamMessage::ProcessExited { code } => {
                    if code != 0 {
                        std::process::exit(code);
                    }
                    break;
                }
            }
            continue;
        }

        // Try parsing as final Response
        if let Ok(resp) = serde_json::from_str::<Response>(trimmed) {
            if resp.status == ResponseStatus::Error {
                eprintln!("{}", format!("❌ Error: {}", resp.message).red());
                std::process::exit(1);
            }

            if let Some(data) = resp.data {
                if let Some(code) = data.get("exit_code").and_then(|c| c.as_i64()) {
                    if code != 0 {
                        std::process::exit(code as i32);
                    }
                }
            }
            break;
        }
    }

    Ok(())
}
