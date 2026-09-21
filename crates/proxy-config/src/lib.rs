pub mod error;
pub mod loader;
pub mod model;
pub mod validator;

use std::path::Path;

use error::ConfigResult;
use model::ProxyConfig;

pub fn load_and_validate<P: AsRef<Path>>(path: P) -> ConfigResult<ProxyConfig> {
    let config = loader::load(path)?;
    validator::validate(&config)?;

    Ok(config)
}
