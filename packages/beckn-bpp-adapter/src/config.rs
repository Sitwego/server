//! Adapter configuration, loaded from the environment via `envy` (same pattern
//! as the `api` crate). Signing/verification is native (Step 3 decision), so
//! this process owns its Ed25519 key — provide it via the secrets manager as
//! `BECKN_SIGNING_PRIVATE_KEY`, never commit it.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// TCP port the adapter listens on.
    #[serde(default = "default_port")]
    pub beckn_port: u16,

    /// Our network subscriber id, e.g. `sitwego.mobility.ke`. Becomes `bpp_id`.
    #[serde(default = "default_subscriber_id")]
    pub beckn_subscriber_id: String,

    /// Public base URI other participants call back into. Becomes `bpp_uri`.
    #[serde(default = "default_subscriber_uri")]
    pub beckn_subscriber_uri: String,

    /// Beckn `domain` this BPP serves (mobility).
    #[serde(default = "default_domain")]
    pub beckn_domain: String,

    /// Beckn core version this BPP supports; inbound `context.version` must match.
    #[serde(default = "default_core_version")]
    pub beckn_core_version: String,

    /// Verify inbound `Authorization` signatures (Step 3). ON by default;
    /// only a dev environment may run with this off.
    #[serde(default = "default_true")]
    pub beckn_verify_signatures: bool,

    /// Base URL of the network registry; keys are resolved via
    /// `POST {url}/lookup`. Required whenever verification is on.
    #[serde(default)]
    pub beckn_registry_url: Option<String>,

    /// Gateway to broadcast to / receive `search` from (used from Step 4 on;
    /// carried in config now so the env contract is complete).
    #[serde(default)]
    pub beckn_gateway_url: Option<String>,

    /// The `unique_key_id` our signing key is registered under.
    #[serde(default = "default_unique_key_id")]
    pub beckn_unique_key_id: String,

    /// Base64 Ed25519 private key (32-byte seed or 64-byte keypair) used to
    /// sign outbound `on_*` callbacks. From the secrets manager, never a file
    /// in the repo. Absent → callbacks go out unsigned (dev only).
    #[serde(default)]
    pub beckn_signing_private_key: Option<String>,

    /// Validity window (seconds) stamped on our outbound signatures.
    #[serde(default = "default_signature_validity_secs")]
    pub beckn_signature_validity_secs: u64,

    /// Human-readable provider name shown in the `on_search` catalog.
    #[serde(default = "default_provider_name")]
    pub beckn_provider_name: String,

    /// Currency for quoted prices.
    #[serde(default = "default_currency")]
    pub beckn_currency: String,

    /// Postgres, shared with the main app (correlation store + fare pricing).
    #[serde(default)]
    pub database_url: Option<String>,

    /// OSRM-compatible routing service (same env name the api crate uses).
    #[serde(default)]
    pub routes_api_url: Option<String>,

    /// Redis for the read-only nearby-driver lookup. Mirrors the api crate's
    /// dev default (standalone localhost:6379).
    #[serde(default = "default_redis_host")]
    pub redis_host: String,
    #[serde(default = "default_redis_port")]
    pub redis_port: u16,
    #[serde(default)]
    pub redis_cluster_enabled: bool,
    #[serde(default)]
    pub redis_cluster_urls: String,

    /// Base URL of the main api service's PRIVATE internal plane (the admin
    /// listener), e.g. `http://127.0.0.1:8091`. `confirm` hands bookings to
    /// dispatch through it — dispatch state lives in the api process.
    #[serde(default)]
    pub sitwego_internal_api_url: Option<String>,

    /// Shared secret for the internal plane (sent as `X-Internal-Token`);
    /// must match the api service's `BECKN_INTERNAL_TOKEN`.
    #[serde(default)]
    pub beckn_internal_token: Option<String>,

    /// Profile id (ULID) of the pre-provisioned "network rider" — the FK
    /// anchor every network booking's DB rows attach to. The real customer's
    /// name/phone travel in the dispatch payload, not on this profile.
    #[serde(default)]
    pub beckn_network_rider_id: Option<String>,

    /// Template for the tracking URL `on_track` answers with, e.g.
    /// `https://track.sitwego.com/{ride_id}`. `{ride_id}` is replaced with
    /// the booking's ride id. Rule #5: `track` returns a URL, never raw GPS.
    /// Unset → `track` gets an error callback ("tracking not available").
    #[serde(default)]
    pub beckn_tracking_url_template: Option<String>,

    /// Base64 X25519 public key published in our registry record
    /// (`encr_public_key`). Minted by the `keygen` bin alongside the signing
    /// pair; only the `onboard` bin reads it — our registry's onboarding has
    /// no challenge handshake, so the adapter itself never decrypts with it.
    #[serde(default)]
    pub beckn_encryption_public_key: Option<String>,

    /// ISO-3166 alpha-3 country of our registry record.
    #[serde(default = "default_country")]
    pub beckn_country: String,

    /// Registry admin ApiKey, when `POST /register` requires one (a fresh
    /// beckn-onix registry with `authentication.required=false` does not).
    #[serde(default)]
    pub beckn_registry_api_key: Option<String>,

    /// dev/development/local relax behaviour where appropriate.
    #[serde(default)]
    pub app_env: String,
}

fn default_port() -> u16 {
    8090
}
fn default_subscriber_id() -> String {
    "sitwego.mobility.ke".to_string()
}
fn default_subscriber_uri() -> String {
    "http://localhost:8090".to_string()
}
fn default_domain() -> String {
    // Beckn mobility domain code. Confirmed against the spec in Step 4 when we
    // build the catalog; kept configurable so we never hard-code a guess.
    "mobility".to_string()
}
fn default_core_version() -> String {
    // Beckn v2 core version, matching nammayatri's beckn-spec (BecknV2).
    "2.0.0".to_string()
}
fn default_true() -> bool {
    true
}
fn default_unique_key_id() -> String {
    "sitwego-key-1".to_string()
}
fn default_provider_name() -> String {
    "Sitwego".to_string()
}
fn default_currency() -> String {
    "KES".to_string()
}
fn default_redis_host() -> String {
    "localhost".to_string()
}
fn default_redis_port() -> u16 {
    6379
}
fn default_signature_validity_secs() -> u64 {
    // Matches the common network default; a signature is replayable inside
    // this window, so keep it short.
    600
}
fn default_country() -> String {
    "KEN".to_string()
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(envy::from_env::<Config>()?)
    }

    pub fn is_dev(&self) -> bool {
        matches!(self.app_env.as_str(), "dev" | "development" | "local")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            beckn_port: default_port(),
            beckn_subscriber_id: default_subscriber_id(),
            beckn_subscriber_uri: default_subscriber_uri(),
            beckn_domain: default_domain(),
            beckn_core_version: default_core_version(),
            beckn_verify_signatures: true,
            beckn_registry_url: None,
            beckn_gateway_url: None,
            beckn_unique_key_id: default_unique_key_id(),
            beckn_signing_private_key: None,
            beckn_signature_validity_secs: default_signature_validity_secs(),
            beckn_provider_name: default_provider_name(),
            beckn_currency: default_currency(),
            database_url: None,
            routes_api_url: None,
            redis_host: default_redis_host(),
            redis_port: default_redis_port(),
            redis_cluster_enabled: false,
            redis_cluster_urls: String::new(),
            sitwego_internal_api_url: None,
            beckn_internal_token: None,
            beckn_network_rider_id: None,
            beckn_tracking_url_template: None,
            beckn_encryption_public_key: None,
            beckn_country: default_country(),
            beckn_registry_api_key: None,
            app_env: "dev".to_string(),
        }
    }
}
