//! Beckn registry lookup — resolves a subscriber's Ed25519 signing public key.
//!
//! Every network participant registers its keys with the network registry. To
//! verify an inbound signature we resolve the `keyId`'s
//! `(subscriber_id, unique_key_id)` to a `signing_public_key` via
//! `POST {registry}/lookup`, the beckn core registry contract.
//!
//! Keys rotate rarely, so successful lookups are cached in-process.

use std::time::{Duration, Instant};

use async_trait::async_trait;
use dashmap::DashMap;
use serde::Deserialize;

/// How long a resolved key is trusted before re-consulting the registry.
const CACHE_TTL: Duration = Duration::from_secs(3600);

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("registry transport error: {0}")]
    Transport(String),
    #[error("registry returned status {0}")]
    Status(u16),
    #[error("registry response was not valid JSON: {0}")]
    BadResponse(String),
}

/// Resolves subscriber signing keys. Trait so tests and dev setups can use a
/// static in-memory registry instead of the network one.
#[async_trait]
pub trait RegistryClient: Send + Sync {
    /// The base64 Ed25519 signing public key for `(subscriber_id,
    /// unique_key_id)`, or `None` when the registry doesn't know the pair.
    async fn signing_public_key(
        &self,
        subscriber_id: &str,
        unique_key_id: &str,
    ) -> Result<Option<String>, RegistryError>;
}

/// One entry of a registry `/lookup` response. Only the fields we consume are
/// modeled; the registry may return more.
#[derive(Debug, Deserialize)]
struct LookupEntry {
    signing_public_key: Option<String>,
    #[serde(default)]
    status: Option<String>,
}

struct CachedKey {
    key: String,
    fetched_at: Instant,
}

/// HTTP registry client with an in-process key cache.
pub struct HttpRegistryClient {
    base_url: String,
    http: reqwest::Client,
    cache: DashMap<(String, String), CachedKey>,
}

impl HttpRegistryClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http: reqwest::Client::new(),
            cache: DashMap::new(),
        }
    }
}

#[async_trait]
impl RegistryClient for HttpRegistryClient {
    async fn signing_public_key(
        &self,
        subscriber_id: &str,
        unique_key_id: &str,
    ) -> Result<Option<String>, RegistryError> {
        let cache_key = (subscriber_id.to_string(), unique_key_id.to_string());
        if let Some(hit) = self.cache.get(&cache_key)
            && hit.fetched_at.elapsed() < CACHE_TTL
        {
            return Ok(Some(hit.key.clone()));
        }

        let url = format!("{}/lookup", self.base_url.trim_end_matches('/'));
        let resp = self
            .http
            .post(&url)
            .json(&serde_json::json!({
                "subscriber_id": subscriber_id,
                "unique_key_id": unique_key_id,
            }))
            .send()
            .await
            .map_err(|e| RegistryError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(RegistryError::Status(resp.status().as_u16()));
        }
        let entries: Vec<LookupEntry> = resp
            .json()
            .await
            .map_err(|e| RegistryError::BadResponse(e.to_string()))?;

        // Prefer an actively subscribed record; fall back to any entry that
        // carries a key (registries differ on whether `status` is present).
        let key = entries
            .iter()
            .find(|e| {
                e.status.as_deref() == Some("SUBSCRIBED")
                    && e.signing_public_key.is_some()
            })
            .or_else(|| entries.iter().find(|e| e.signing_public_key.is_some()))
            .and_then(|e| e.signing_public_key.clone());

        if let Some(key) = &key {
            self.cache.insert(
                cache_key,
                CachedKey {
                    key: key.clone(),
                    fetched_at: Instant::now(),
                },
            );
        }
        Ok(key)
    }
}

/// Fixed in-memory registry for tests and single-partner dev setups.
#[derive(Default)]
pub struct StaticRegistry {
    keys: DashMap<(String, String), String>,
}

impl StaticRegistry {
    pub fn with_key(
        self,
        subscriber_id: &str,
        unique_key_id: &str,
        public_key_b64: &str,
    ) -> Self {
        self.keys.insert(
            (subscriber_id.to_string(), unique_key_id.to_string()),
            public_key_b64.to_string(),
        );
        self
    }
}

#[async_trait]
impl RegistryClient for StaticRegistry {
    async fn signing_public_key(
        &self,
        subscriber_id: &str,
        unique_key_id: &str,
    ) -> Result<Option<String>, RegistryError> {
        Ok(self
            .keys
            .get(&(subscriber_id.to_string(), unique_key_id.to_string()))
            .map(|k| k.clone()))
    }
}
