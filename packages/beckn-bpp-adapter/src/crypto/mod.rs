//! Signing & verification (Step 3) — implemented NATIVELY in this process.
//!
//! Decision (2026-07-02, John): no beckn-onix sidecar; the adapter itself
//! verifies inbound signatures and signs outbound `on_*` callbacks.
//!
//! - [`signature`] — the Beckn HTTP-signature profile: Ed25519 over a
//!   BLAKE2b-512 digest of the RAW request bytes (Inviolable Rule #7; the
//!   controllers hand us the unmodified [`axum::body::Bytes`]).
//! - [`registry`]  — resolves a subscriber's signing public key via the network
//!   registry `/lookup`, with an in-process cache.
//! - [`Signer`]    — holds our Ed25519 key and stamps `Authorization` headers
//!   onto outbound callbacks.

pub mod onboarding;
pub mod registry;
pub mod signature;

pub use registry::{HttpRegistryClient, RegistryClient, StaticRegistry};
pub use signature::{SignatureError, SignatureHeader};

use chrono::Utc;
use ed25519_dalek::SigningKey;

use crate::config::Config;

/// Signs outbound `on_*` callback bodies with our subscriber key.
pub struct Signer {
    subscriber_id: String,
    unique_key_id: String,
    key: SigningKey,
    validity_secs: i64,
}

impl Signer {
    /// Build from config. `None` when no private key is configured (dev boots
    /// without one; callbacks then go out unsigned and peers will reject them).
    pub fn from_config(config: &Config) -> anyhow::Result<Option<Self>> {
        let Some(key_b64) = &config.beckn_signing_private_key else {
            return Ok(None);
        };
        Ok(Some(Self {
            subscriber_id: config.beckn_subscriber_id.clone(),
            unique_key_id: config.beckn_unique_key_id.clone(),
            key: signature::signing_key_from_b64(key_b64)?,
            validity_secs: config.beckn_signature_validity_secs as i64,
        }))
    }

    /// `Authorization` header value for `raw_body` — the exact bytes that will
    /// be sent must be the bytes signed here.
    pub fn authorization_header(&self, raw_body: &[u8]) -> String {
        let created = Utc::now().timestamp();
        signature::sign(
            raw_body,
            &self.subscriber_id,
            &self.unique_key_id,
            &self.key,
            created,
            created + self.validity_secs,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD as B64;

    #[test]
    fn signer_output_verifies_against_our_public_key() {
        let seed = [3u8; 32];
        let config = Config {
            beckn_signing_private_key: Some(B64.encode(seed)),
            ..Config::default()
        };
        let signer = Signer::from_config(&config).unwrap().unwrap();

        let body = br#"{"context":{"action":"on_search"}}"#;
        let header = signer.authorization_header(body);
        let parsed = SignatureHeader::parse(&header).unwrap();
        assert_eq!(parsed.subscriber_id, config.beckn_subscriber_id);
        assert_eq!(parsed.unique_key_id, config.beckn_unique_key_id);

        let public = B64
            .encode(SigningKey::from_bytes(&seed).verifying_key().to_bytes());
        parsed.verify(body, &public, Utc::now().timestamp()).unwrap();
    }

    #[test]
    fn no_key_means_no_signer() {
        let config = Config {
            beckn_signing_private_key: None,
            ..Config::default()
        };
        assert!(Signer::from_config(&config).unwrap().is_none());
    }
}
