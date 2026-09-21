use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ProxyConfig {
    pub proxy: ProxySettings,
    pub listeners: Vec<ListenerConfig>,
    pub rate_limit: RateLimitConfig,
    pub proxy_protocol: ProxyProtocolConfig,
    pub routes: Vec<RouteConfig>,
    pub backends: Vec<BackendConfig>,
    pub admin: AdminConfig,
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProxySettings {
    pub name: String,
    pub worker_threads: usize,
    pub graceful_shutdown: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListenerConfig {
    pub name: String,
    pub address: String,
    pub protocol: TransportProtocol,
    pub sni_inspect_timeout: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportProtocol {
    Tcp,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub key: RateLimitKey,
    pub window: String,
    pub max_connections: u64,
    pub action: RateLimitAction,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitKey {
    SourceIp,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitAction {
    Reject,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProxyProtocolConfig {
    pub enabled: bool,
    pub version: ProxyProtocolVersion,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyProtocolVersion {
    V2,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RouteConfig {
    pub hostname: String,
    pub backend: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackendConfig {
    pub name: String,
    pub balance: BalanceStrategy,
    pub health_check: HealthCheckConfig,
    pub servers: Vec<ServerConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BalanceStrategy {
    RoundRobin,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthCheckConfig {
    pub protocol: TransportProtocol,
    pub interval: String,
    pub timeout: String,
    pub rise: u32,
    pub fall: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub weight: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminConfig {
    pub enabled: bool,
    pub address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObservabilityConfig {
    pub log_level: String,
    pub access_log: bool,
    pub metrics: bool,
}
