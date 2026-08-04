use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkipperError {
    #[error("Erreur de configuration: {0}")]
    Config(String),

    #[error("Erreur du trousseau de clés (Keyring): {0}")]
    Keyring(String),

    #[error("Erreur de communication IPC: {0}")]
    Ipc(String),

    #[error("Erreur de gestion PTY: {0}")]
    Pty(String),

    #[error("Permission refusée: {0}")]
    PermissionDenied(String),

    #[error("Erreur système d'exploitation: {0}")]
    Io(#[from] std::io::Error),

    #[error("Erreur de sérialisation JSON/TOML: {0}")]
    Serialization(String),

    #[error("Erreur indéterminée: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, SkipperError>;
