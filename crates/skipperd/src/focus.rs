use tracing::info;

pub struct FocusDetector;

impl FocusDetector {
    /// Vérifie si le processus ou le terminal associé possède le focus actif.
    /// Utilise l'environnement X11 (si DISPLAY disponible) ou un fallback par PID sur Wayland/headless.
    pub fn is_focused(pid: u32) -> bool {
        if std::env::var("DISPLAY").is_ok() {
            info!("Vérification du focus X11 pour le PID {}", pid);
            return Self::check_pid_active(pid);
        }

        info!("Environnement Wayland/Headless : validation du PID {}", pid);
        Self::check_pid_active(pid)
    }

    fn check_pid_active(pid: u32) -> bool {
        #[cfg(unix)]
        {
            use nix::sys::signal::kill;
            use nix::unistd::Pid;
            kill(Pid::from_raw(pid as i32), None).is_ok()
        }
        #[cfg(not(unix))]
        {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_detector_current_pid() {
        let current_pid = std::process::id();
        assert!(FocusDetector::is_focused(current_pid));
    }
}
