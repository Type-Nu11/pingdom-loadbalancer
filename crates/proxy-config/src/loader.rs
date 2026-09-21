use std::{fs, path::Path};

use crate::{
    error::{ConfigError, ConfigResult},
    model::ProxyConfig,
};

pub fn load<P: AsRef<Path>>(path: P) -> ConfigResult<ProxyConfig> {
    let path = path.as_ref();

    let content = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    parse(&content)
}

pub fn parse(content: &str) -> ConfigResult<ProxyConfig> {
    Ok(toml::from_str::<ProxyConfig>(content)?)
}
