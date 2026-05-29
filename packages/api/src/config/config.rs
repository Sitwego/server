use std::path::PathBuf;

use aws::AwsCredentials;
use serde::Deserialize;
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub migrations_path: Option<PathBuf>,
    pub seed_path: Option<PathBuf>,
    pub database_max_connections: u32,
    pub origin: String,
    /// When set to "dev"/"development"/"local", forces Redis into standalone
    /// mode against localhost:6379 regardless of the REDIS_CLUSTER_* vars.
    #[serde(default)]
    pub app_env: String,
    //Redis
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_pool_size: usize,
    pub redis_partition: usize,
    pub reconnect_max_attempts: u32,
    pub reconnect_delay: u32,
    pub exp_ttl: i64,
    pub default_ttl: i64,
    pub default_hash_ttl: i64,
    pub stream_read_count: i64,
    #[serde(default)]
    pub redis_cluster_enabled: bool,
    #[serde(default)]
    pub redis_cluster_urls: String,
    pub port: u16,
    pub kms_key_id: String,
    pub bucket: String,
    pub jwt_secrete_key: String,
    pub aws_region: String,
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub gorush_url: String,
    pub twilio_account_sid: String,
    pub twilio_auth_token: String,
    pub twilio_verify_service_sid: String,
    /// Shared secret that clients must send in `X-Api-Key` to use the 2FA endpoints.
    pub otp_api_key: String,
    /// Comma-separated list of trusted reverse-proxy IPs (e.g. HAProxy, LB).
    /// The rate-limit middleware walks `X-Forwarded-For` right-to-left,
    /// skipping these addresses to find the real client IP. Leave empty in
    /// dev; in prod always set to the known ingress IPs or untrusted XFF
    /// entries can spoof per-IP buckets.
    #[serde(default)]
    pub trusted_proxies: String,
}

impl Config {
    pub fn is_dev(&self) -> bool {
        matches!(self.app_env.as_str(), "dev" | "development" | "local")
    }

    /// Parse `trusted_proxies` from its CSV form into a list of `IpAddr`.
    /// Malformed entries are skipped with a warning rather than panicking —
    /// a typo in env config shouldn't take down the API.
    pub fn parsed_trusted_proxies(&self) -> Vec<std::net::IpAddr> {
        self.trusted_proxies
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter_map(|s| match s.parse() {
                Ok(ip) => Some(ip),
                Err(e) => {
                    tracing::warn!(
                        raw = s,
                        error = %e,
                        "trusted_proxies: skipping malformed entry"
                    );
                    None
                }
            })
            .collect()
    }

    pub fn aws_credentials(&self) -> AwsCredentials {
        AwsCredentials::new(
            None,
            Some(false),
            self.aws_region.clone(),
            self.aws_access_key_id.clone(),
            self.aws_secret_access_key.clone(),
            None,
        )
    }
}
