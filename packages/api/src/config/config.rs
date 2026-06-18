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
    /// Private admin plane (Option B). The admin router is served on a SECOND
    /// listener bound to `admin_bind_addr:admin_port` — keep that on a private
    /// interface (loopback or a private subnet) so it is unreachable from the
    /// public internet. The plane is opt-in: it only starts when
    /// `admin_internal_token` is non-empty, and every admin request must carry
    /// that token in the `X-Internal-Token` header (set by the admin BFF).
    ///
    /// Left unset, the bind address is app_env-gated — see
    /// [`Config::effective_admin_bind_addr`]. An explicit `ADMIN_BIND_ADDR`
    /// always wins.
    #[serde(default)]
    pub admin_bind_addr: Option<String>,
    #[serde(default = "default_admin_port")]
    pub admin_port: u16,
    #[serde(default)]
    pub admin_internal_token: String,
    /// Referral reward issued to the referrer when their referred driver is
    /// activated. `referral_reward_type` is one of `cash_credit` (default),
    /// `subscription_days`, or `badge`; `referral_reward_value` is the
    /// magnitude in the unit the type implies (KES for cash, days for
    /// subscription_days, unused for badge).
    #[serde(default = "default_referral_reward_type")]
    pub referral_reward_type: String,
    #[serde(default = "default_referral_reward_value")]
    pub referral_reward_value: f64,
}

fn default_admin_port() -> u16 {
    8081
}

fn default_referral_reward_type() -> String {
    "cash_credit".to_string()
}

fn default_referral_reward_value() -> f64 {
    100.0
}

impl Config {
    pub fn is_dev(&self) -> bool {
        matches!(self.app_env.as_str(), "dev" | "development" | "local")
    }

    /// Resolve the interface the admin plane should bind to.
    ///
    /// An explicit `ADMIN_BIND_ADDR` is always honoured. Otherwise the default
    /// is app_env-gated: dev binds `0.0.0.0` so a Windows-side BFF can reach the
    /// WSL listener via localhost forwarding, while every other environment
    /// binds loopback — keeping the admin plane off public interfaces unless
    /// someone opts in deliberately.
    pub fn effective_admin_bind_addr(&self) -> &str {
        match self.admin_bind_addr.as_deref() {
            Some(addr) if !addr.trim().is_empty() => addr,
            _ if self.is_dev() => "0.0.0.0",
            _ => "127.0.0.1",
        }
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
