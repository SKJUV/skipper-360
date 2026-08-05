use crate::config::ConfigManager;
use crate::keyring::KeyringManager;
use secrecy::SecretString;

#[derive(Debug, Clone)]
pub struct DiagnosticItem {
    pub name: &'static str,
    pub status: bool,
    pub details: String,
}

pub struct DiagnosticReport {
    pub items: Vec<DiagnosticItem>,
}

impl DiagnosticReport {
    pub fn run() -> Self {
        let mut items = Vec::new();

        // 1. Config Check
        match ConfigManager::new() {
            Ok(manager) => match manager.load() {
                Ok(_) => items.push(DiagnosticItem {
                    name: "Fichier de configuration",
                    status: true,
                    details: format!("Config OK à {}", manager.config_path().display()),
                }),
                Err(e) => items.push(DiagnosticItem {
                    name: "Fichier de configuration",
                    status: false,
                    details: format!("Erreur de lecture : {}", e),
                }),
            },
            Err(e) => items.push(DiagnosticItem {
                name: "Fichier de configuration",
                status: false,
                details: format!("Répertoire inatteignable : {}", e),
            }),
        }

        // 2. Keyring Check
        let dummy_key = "doctor_health_check";
        let test_secret = SecretString::from("doctor_test");

        match KeyringManager::store_whitelist_password(dummy_key, &test_secret) {
            Ok(_) => {
                let _ = KeyringManager::delete_whitelist_password(dummy_key);
                items.push(DiagnosticItem {
                    name: "Trousseau de clés (Keyring)",
                    status: true,
                    details: "Backend OS Keyring fonctionnel (Secret Service / KWallet)".into(),
                });
            }
            Err(e) => items.push(DiagnosticItem {
                name: "Trousseau de clés (Keyring)",
                status: false,
                details: format!("Backend Keyring indisponible ou erreur : {}", e),
            }),
        }

        // 3. Socket UDS Check
        if let Ok(base_dir) = dirs::config_dir().ok_or(()) {
            let socket_path = base_dir.join("skipper360").join("skipper.sock");
            if socket_path.exists() {
                items.push(DiagnosticItem {
                    name: "Socket IPC Daemon",
                    status: true,
                    details: format!("Socket actif sur {}", socket_path.display()),
                });
            } else {
                items.push(DiagnosticItem {
                    name: "Socket IPC Daemon",
                    status: false,
                    details: "Socket inactif (le daemon skipperd est éteint)".into(),
                });
            }
        }

        // 4. System & PTY Check
        #[cfg(target_os = "linux")]
        items.push(DiagnosticItem {
            name: "Système d'exploitation",
            status: true,
            details: "Plateforme Linux POSIX compatible PTY".into(),
        });

        Self { items }
    }
}
