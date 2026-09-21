use std::collections::HashSet;

use crate::{
    error::{ConfigError, ConfigResult},
    model::ProxyConfig,
};

pub fn validate(config: &ProxyConfig) -> ConfigResult<()> {
    let mut errors = Vec::new();

    validate_listeners(config, &mut errors);
    validate_rate_limit(config, &mut errors);
    validate_routes(config, &mut errors);
    validate_backends(config, &mut errors);
    validate_admin(config, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ConfigError::Validation(errors.join("\n")))
    }
}

fn validate_listeners(config: &ProxyConfig, errors: &mut Vec<String>) {
    let mut names = HashSet::new();

    if config.listeners.is_empty() {
        errors.push("listeners must not be empty".to_owned());
    }

    for (index, listener) in config.listeners.iter().enumerate() {
        if listener.name.trim().is_empty() {
            errors.push(format!("listeners[{index}].name must not be empty"));
        }

        if !names.insert(listener.name.to_ascii_lowercase()) {
            errors.push(format!(
                "listeners[{index}].name `{}` is duplicated",
                listener.name
            ));
        }

        if listener.address.trim().is_empty() {
            errors.push(format!("listeners[{index}].address must not be empty"));
        }

        if listener.sni_inspect_timeout.trim().is_empty() {
            errors.push(format!(
                "listeners[{index}].sni_inspect_timeout must not be empty"
            ));
        }
    }
}

fn validate_rate_limit(config: &ProxyConfig, errors: &mut Vec<String>) {
    if !config.rate_limit.enabled {
        return;
    }

    if config.rate_limit.window.trim().is_empty() {
        errors.push("rate_limit.window must not be empty".to_owned());
    }

    if config.rate_limit.max_connections == 0 {
        errors.push("rate_limit.max_connections must be greater than zero".to_owned());
    }
}

fn validate_routes(config: &ProxyConfig, errors: &mut Vec<String>) {
    let backend_names: HashSet<&str> = config
        .backends
        .iter()
        .map(|backend| backend.name.as_str())
        .collect();

    let mut hostnames = HashSet::new();

    if config.routes.is_empty() {
        errors.push("routes must not be empty".to_owned());
    }

    for (index, route) in config.routes.iter().enumerate() {
        let hostname = route.hostname.to_ascii_lowercase();

        if route.hostname.trim().is_empty() {
            errors.push(format!("routes[{index}].hostname must not be empty"));
        }

        if !hostnames.insert(hostname) {
            errors.push(format!(
                "routes[{index}].hostname `{}` is duplicated",
                route.hostname
            ));
        }

        if !backend_names.contains(route.backend.as_str()) {
            errors.push(format!(
                "routes[{index}].backend `{}` does not exist",
                route.backend
            ));
        }
    }
}

fn validate_backends(config: &ProxyConfig, errors: &mut Vec<String>) {
    let mut backend_names = HashSet::new();

    if config.backends.is_empty() {
        errors.push("backends must not be empty".to_owned());
    }

    for (backend_index, backend) in config.backends.iter().enumerate() {
        if backend.name.trim().is_empty() {
            errors.push(format!("backends[{backend_index}].name must not be empty"));
        }

        if !backend_names.insert(backend.name.to_ascii_lowercase()) {
            errors.push(format!(
                "backends[{backend_index}].name `{}` is duplicated",
                backend.name
            ));
        }

        if backend.servers.is_empty() {
            errors.push(format!(
                "backends[{backend_index}].servers must not be empty"
            ));
        }

        if backend.health_check.interval.trim().is_empty() {
            errors.push(format!(
                "backends[{backend_index}].health_check.interval must not be empty"
            ));
        }

        if backend.health_check.timeout.trim().is_empty() {
            errors.push(format!(
                "backends[{backend_index}].health_check.timeout must not be empty"
            ));
        }

        if backend.health_check.rise == 0 {
            errors.push(format!(
                "backends[{backend_index}].health_check.rise must be greater than zero"
            ));
        }

        if backend.health_check.fall == 0 {
            errors.push(format!(
                "backends[{backend_index}].health_check.fall must be greater than zero"
            ));
        }

        let mut server_names = HashSet::new();

        for (server_index, server) in backend.servers.iter().enumerate() {
            if server.name.trim().is_empty() {
                errors.push(format!(
                    "backends[{backend_index}].servers[{server_index}].name must not be empty"
                ));
            }

            if !server_names.insert(server.name.to_ascii_lowercase()) {
                errors.push(format!(
                    "backends[{backend_index}].servers[{server_index}].name `{}` is duplicated",
                    server.name
                ));
            }

            if server.address.trim().is_empty() {
                errors.push(format!(
                    "backends[{backend_index}].servers[{server_index}].address must not be empty"
                ));
            }

            if server.port == 0 {
                errors.push(format!(
                    "backends[{backend_index}].servers[{server_index}].port must be greater than zero"
                ));
            }

            if server.weight == 0 {
                errors.push(format!(
                    "backends[{backend_index}].servers[{server_index}].weight must be greater than zero"
                ));
            }
        }
    }
}

fn validate_admin(config: &ProxyConfig, errors: &mut Vec<String>) {
    if config.admin.enabled && config.admin.address.trim().is_empty() {
        errors.push("admin.address must not be empty when admin is enabled".to_owned());
    }
}
