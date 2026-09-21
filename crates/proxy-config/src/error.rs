use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file `{path}`: {source}")]
    Io {
        path: PathBuf,

        #[source]
        source: io::Error,
    },

    #[error("failed to parse TOML: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("configuration validation failed:\n{0}")]
    Validation(String),
}

pub type ConfigResult<T> = Result<T, ConfigError>;
