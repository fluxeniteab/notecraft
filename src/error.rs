// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NoteCraftError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Invalid vault: {0}")]
    InvalidVault(String),
    #[error("Path is outside the vault: {0}")]
    PathTraversal(String),
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("Conflict: file changed externally")]
    Conflict,
    #[error("Cancelled")]
    Cancelled,
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, NoteCraftError>;
