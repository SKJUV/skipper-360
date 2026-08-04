use nix::sys::signal;
use nix::unistd::Pid;
use skipper_core::{Result, SkipperError};
use std::fs;
use std::path::PathBuf;

pub struct PidFile {
    path: PathBuf,
}

impl PidFile {
    pub fn new() -> Result<Self> {
        let base_dir = dirs::config_dir().ok_or_else(|| {
            SkipperError::Config("Impossible de localiser le répertoire config".into())
        })?;
        let pid_dir = base_dir.join("skipper360");
        if !pid_dir.exists() {
            fs::create_dir_all(&pid_dir)?;
        }
        let path = pid_dir.join("skipper.pid");

        Ok(Self { path })
    }

    pub fn acquire(&self) -> Result<()> {
        if self.path.exists() {
            if let Ok(content) = fs::read_to_string(&self.path) {
                if let Ok(pid_num) = content.trim().parse::<i32>() {
                    let pid = Pid::from_raw(pid_num);
                    // Check if process with this PID is still running
                    if signal::kill(pid, None).is_ok() {
                        return Err(SkipperError::Other(format!(
                            "Un autre instance de skipperd est déjà en cours d'exécution (PID {})",
                            pid_num
                        )));
                    }
                }
            }
            // Stale PID file, clean it up
            let _ = fs::remove_file(&self.path);
        }

        let current_pid = std::process::id();
        fs::write(&self.path, current_pid.to_string()).map_err(SkipperError::Io)?;

        Ok(())
    }
}

impl Drop for PidFile {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = fs::remove_file(&self.path);
        }
    }
}
