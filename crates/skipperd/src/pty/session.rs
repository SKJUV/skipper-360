use crate::injector::PasswordInjector;
use crate::pty::detector::PromptDetector;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use skipper_core::{
    apply_kernel_hardened_prctl, flush_cache_line, speculation_barrier, AuditAction, AuditLogger,
    Config, Result, SkipperError,
};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct PtySession;

impl PtySession {
    pub fn run_and_stream<F>(command: &[String], config: &Config, mut on_output: F) -> Result<i32>
    where
        F: FnMut(&str) + Send + 'static,
    {
        if command.is_empty() {
            return Err(SkipperError::Pty("Commande vide fournie".into()));
        }

        apply_kernel_hardened_prctl();
        let audit_logger = AuditLogger::new().ok();

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| SkipperError::Pty(format!("Échec de création du PTY: {}", e)))?;

        let mut cmd = CommandBuilder::new(&command[0]);
        if command.len() > 1 {
            cmd.args(&command[1..]);
        }

        let mut child = pair.slave.spawn_command(cmd).map_err(|e| {
            SkipperError::Pty(format!(
                "Échec de lancement du processus enfant dans le PTY: {}",
                e
            ))
        })?;

        // Drop slave in parent process so EOF is signaled properly when child terminates
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| SkipperError::Pty(format!("Échec d'obtention du reader master: {}", e)))?;

        let mut writer = pair
            .master
            .take_writer()
            .map_err(|e| SkipperError::Pty(format!("Échec d'obtention du writer master: {}", e)))?;

        let detector = PromptDetector::new(&config.patterns.custom_patterns);
        let injected = Arc::new(AtomicBool::new(false));

        let full_command_str = command.join(" ");
        let config_clone = config.clone();

        let mut buffer = [0u8; 1024];
        let mut text_acc = String::new();

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]);
                    on_output(&chunk);
                    text_acc.push_str(&chunk);

                    // Keep text_acc to last 512 chars for prompt window detection
                    if text_acc.len() > 512 {
                        let drain_offset = text_acc.len() - 512;
                        text_acc.drain(..drain_offset);
                    }

                    // Prompt detection & injection check
                    if !injected.load(Ordering::SeqCst) {
                        if let Some(pattern) = detector.detect(&text_acc) {
                            tracing::info!(
                                "Prompt de mot de passe détecté via le pattern : '{}'",
                                pattern
                            );

                            // Check focus detection if enabled in config
                            let child_pid = child.process_id().unwrap_or(0);
                            if config_clone.general.focus_detection
                                && !crate::focus::FocusDetector::is_focused(child_pid)
                            {
                                tracing::warn!(
                                    "Focus non détecté pour le PID {}. Injection annulée par sécurité.",
                                    child_pid
                                );
                                continue;
                            }

                            if let Some(ref al) = audit_logger {
                                let _ = al.log(
                                    AuditAction::PromptDetected,
                                    &full_command_str,
                                    format!("Pattern : {}", pattern),
                                );
                            }

                            // Optional intelligent timeout delay before injection
                            if config_clone.general.timeout_seconds > 0 {
                                tracing::info!(
                                    "Attente du timeout intelligent de {}s avant injection...",
                                    config_clone.general.timeout_seconds
                                );
                                std::thread::sleep(std::time::Duration::from_millis(100));
                            }

                            if let Ok(password) =
                                PasswordInjector::resolve_password(&full_command_str, &config_clone)
                            {
                                let aligned_buf =
                                    PasswordInjector::format_aligned_buffer(&password);
                                speculation_barrier();

                                if writer
                                    .write_all(&aligned_buf.data[..aligned_buf.len])
                                    .is_ok()
                                {
                                    let _ = writer.flush();
                                    injected.store(true, Ordering::SeqCst);
                                    tracing::info!("Mot de passe injecté avec succès dans le PTY.");
                                    if let Some(ref al) = audit_logger {
                                        let _ = al.log(
                                            AuditAction::PasswordInjected,
                                            &full_command_str,
                                            "Mot de passe injecté dans le PTY",
                                        );
                                    }
                                }

                                flush_cache_line(aligned_buf.data.as_ptr(), aligned_buf.len);
                                speculation_barrier();
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Erreur de lecture du master PTY: {}", e);
                    break;
                }
            }
        }

        let exit_status = child.wait().map_err(|e| {
            SkipperError::Pty(format!("Échec de l'attente du processus enfant: {}", e))
        })?;

        Ok(exit_status.exit_code() as i32)
    }
}
