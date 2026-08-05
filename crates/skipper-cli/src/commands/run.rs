use crate::ipc::IpcClient;
use anyhow::{anyhow, Result};
use owo_colors::OwoColorize;
use skipper_core::Request;

pub async fn run(command: &[String]) -> Result<()> {
    if command.is_empty() {
        return Err(anyhow!(
            "Veuillez spécifier une commande à exécuter (ex: skipper run sudo pacman -Syu)"
        ));
    }

    let client = IpcClient::new()?;
    client.ensure_daemon_running().await?;

    println!(
        "{}",
        format!(
            " Skipper 360 — Exécution surveillée : {}",
            command.join(" ")
        )
        .bold()
        .blue()
    );

    let req = Request::new("run", serde_json::json!({ "command": command }));
    let response = client.send_request(req).await?;

    if let Some(data) = response.data {
        if let Some(code) = data.get("exit_code").and_then(|c| c.as_i64()) {
            if code != 0 {
                println!(
                    "{}",
                    format!("Commande terminée avec le code de sortie {}", code).yellow()
                );
                std::process::exit(code as i32);
            }
        }
    }

    println!("{}", format!(" {}", response.message).green());
    Ok(())
}
