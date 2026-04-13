use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Script file not found: {0}")]
    ScriptNotFound(String),

    #[error("Permission denied: {0}. Try: sudo htbox ...")]
    PermissionDenied(String),

    #[error("Service already running: {0}")]
    ServiceAlreadyRunning(String),

    #[error("systemd unavailable, using cron backend")]
    SystemdUnavailable,

    #[error("Invalid config: {0}")]
    ConfigError(String),

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
