use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ProxyConfig {
    pub proxy: ProxySettings,
    pub tls: TlsConfig,
    pub upstreams: UpstreamsConfig,
    pub routing: RoutingConfig,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProxySettings {
    pub name: String,
    pub worker_threads: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TlsConfig {
    pub listen: String,
    pub hostname: String,
    pub certificate: String,
    pub private_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpstreamsConfig {
    pub web: String,
    pub app: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoutingConfig {
    pub target_header: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub max_requests: u64,
    pub window_seconds: u64,
    pub strike_window_seconds: u64,
    pub strikes_before_block: usize,
    pub block_seconds: u64,
    pub max_tracked_ips: usize,
}
