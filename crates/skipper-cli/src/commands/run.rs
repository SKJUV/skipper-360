use crate::ipc::IpcClient;
use anyhow::{anyhow, Result};
use owo_colors::OwoColorize;
use skipper_core::{Request, ResponseStatus, StreamMessage};
use std::io::Write;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn run(command: &[String]) -> Result<()> {
    if command.is_empty() {
        return Err(anyhow!("Veuillez fournir la commande à exécuter sous surveillance (ex: skipper run sudo pacman -Syu)"));
    }

    let client = IpcClient::new()?;
    if let Err(e) = client.ensure_daemon_running().await {
        eprintln!(
            "[ERR] {}",
            format!("Impossible de démarrer le daemon skipperd : {}", e).red()
        );
        return Ok(());
    }

    let req = Request::new("run", serde_json::json!({ "command": command }));

    let mut stream = client.connect().await?;
    let req_json = serde_json::to_string(&req)?;

    use tokio::io::AsyncWriteExt;
    stream
        .write_all(format!("{}\n", req_json).as_bytes())
        .await?;
    stream.flush().await?;

    let (reader, _) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
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
                    return Ok(());
                }
            }
            continue;
        }

        // Otherwise parse as final Response
        if let Ok(resp) = serde_json::from_str::<skipper_core::Response>(trimmed) {
            if resp.status != ResponseStatus::Ok {
                eprintln!("[ERR] {}", resp.message.red());
            }
            break;
        }
    }

    Ok(())
}
