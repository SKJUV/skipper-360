use anyhow::Result;
use clap_complete::{generate, Shell};
use std::io;

pub fn generate_completion(shell_name: &str, cmd: &mut clap::Command) -> Result<()> {
    let shell = match shell_name.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        unknown => {
            return Err(anyhow::anyhow!(
                "Shell inconnu '{}'. Valeurs acceptées : bash, zsh, fish, powershell, elvish",
                unknown
            ));
        }
    };

    generate(shell, cmd, "skipper", &mut io::stdout());
    Ok(())
}
