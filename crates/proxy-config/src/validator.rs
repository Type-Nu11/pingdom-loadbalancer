use std::net::SocketAddr;

use crate::{
    error::{ConfigError, ConfigResult},
    model::ProxyConfig,
};

pub fn validate(config: &ProxyConfig) -> ConfigResult<()> {
    let mut errors = Vec::new();

    if config.proxy.name.trim().is_empty() {
        errors.push("proxy.name must not be empty".to_owned());
    }
    if config.tls.listen.parse::<SocketAddr>().is_err() {
        errors.push("tls.listen must be a socket address".to_owned());
    }
    if config.tls.hostname.trim().is_empty() {
        errors.push("tls.hostname must not be empty".to_owned());
    }
    if config.tls.certificate.trim().is_empty() || config.tls.private_key.trim().is_empty() {
        errors.push("tls.certificate and tls.private_key must be set".to_owned());
    }
    for (name, address) in [
        ("web", &config.upstreams.web),
        ("app", &config.upstreams.app),
    ] {
        if address.parse::<SocketAddr>().is_err() {
            errors.push(format!("upstreams.{name} must be a socket address"));
        }
    }
    if config
        .routing
        .target_header
        .parse::<http::header::HeaderName>()
        .is_err()
    {
        errors.push("routing.target_header must be a valid HTTP header name".to_owned());
    }
    let limit = &config.rate_limit;
    if limit.enabled
        && (limit.max_requests == 0
            || limit.window_seconds == 0
            || limit.strike_window_seconds < limit.window_seconds
            || limit.strikes_before_block == 0
            || limit.block_seconds == 0
            || limit.max_tracked_ips < 64)
    {
        errors.push("rate_limit values must be positive, strike_window_seconds must be at least window_seconds, and max_tracked_ips must be at least 64".to_owned());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ConfigError::Validation(errors.join("\n")))
    }
}
