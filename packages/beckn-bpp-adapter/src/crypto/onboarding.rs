//! Registry onboarding (Step 8) — the subscriber record we register on the
//! network.
//!
//! Our registry (beckn-onix reference, `fidedocker/registry`) onboards via
//! `POST {registry}/register` followed by manual admin approval in the
//! registry UI — there is no encrypted-challenge handshake on this network
//! (see `infra/beckn-network/README.md`). The payload shape is verbatim from
//! the beckn-onix installer (`install/scripts/registry_entry.sh`), which is
//! what this registry demonstrably accepts.

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use serde_json::{Value, json};

use crate::config::Config;

/// How long a registered key stays valid: ~3 years, matching the beckn-onix
/// installer's default. Rotation before then is bumping `unique_key_id` and
/// registering the successor (see the network README).
pub const KEY_VALIDITY_DAYS: i64 = 3 * 365;

/// The `POST {registry}/register` payload for this BPP.
///
/// `pub_key_id` and `unique_key_id` carry the same value, as upstream does;
/// peers resolve our signing key from the signature `keyId`'s
/// `(subscriber_id, unique_key_id)`.
pub fn build_register_payload(
    config: &Config,
    signing_public_key_b64: &str,
    encr_public_key_b64: &str,
    now: DateTime<Utc>,
) -> Value {
    // Backdated a day so clock skew between us and the registry can never
    // make a brand-new key "not yet valid" (upstream does the same).
    //
    // SECOND precision only (no fractional seconds): this registry misparses a
    // fractional-seconds component as MILLISECONDS and adds it to the instant,
    // so `to_rfc3339()`'s 9-digit nanoseconds (e.g. `.361673673`) shove
    // `valid_from` ~4 days into the future — the key then reads "not yet valid"
    // and validity-window lookups (which the gateway uses to pick BPPs) skip it.
    // `SecondsFormat::Secs` sidesteps the bug entirely.
    let valid_from =
        (now - Duration::days(1)).to_rfc3339_opts(SecondsFormat::Secs, true);
    let valid_until = (now + Duration::days(KEY_VALIDITY_DAYS))
        .to_rfc3339_opts(SecondsFormat::Secs, true);
    json!({
        "subscriber_id": config.beckn_subscriber_id,
        "pub_key_id": config.beckn_unique_key_id,
        "unique_key_id": config.beckn_unique_key_id,
        "subscriber_url": config.beckn_subscriber_uri,
        "domain": config.beckn_domain,
        "extended_attributes": { "domains": [] },
        "encr_public_key": encr_public_key_b64,
        "signing_public_key": signing_public_key_b64,
        "valid_from": valid_from,
        "valid_until": valid_until,
        "type": "BPP",
        "country": config.beckn_country,
        "status": "SUBSCRIBED",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_payload_matches_the_onix_contract() {
        let config = Config::default();
        let now = "2026-07-04T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let payload =
            build_register_payload(&config, "SIGNPUB==", "ENCRPUB==", now);

        assert_eq!(payload["subscriber_id"], "sitwego.mobility.ke");
        assert_eq!(payload["type"], "BPP");
        assert_eq!(payload["country"], "KEN");
        assert_eq!(payload["domain"], "mobility");
        // Key ids travel together, like the upstream installer sends them.
        assert_eq!(payload["pub_key_id"], payload["unique_key_id"]);
        assert_eq!(payload["unique_key_id"], "sitwego-key-1");
        assert_eq!(payload["signing_public_key"], "SIGNPUB==");
        assert_eq!(payload["encr_public_key"], "ENCRPUB==");
        // Validity: backdated a day, ~3 years out.
        assert!(
            payload["valid_from"].as_str().unwrap().starts_with("2026-07-03")
        );
        assert!(
            payload["valid_until"].as_str().unwrap().starts_with("2029-07-03")
        );
        // SECOND precision only — a fractional component is misparsed by the
        // registry as milliseconds and pushes validity days into the future.
        assert_eq!(payload["valid_from"], "2026-07-03T12:00:00Z");
        assert!(!payload["valid_until"].as_str().unwrap().contains('.'));
        assert_eq!(payload["status"], "SUBSCRIBED");
        assert!(payload["extended_attributes"]["domains"].is_array());
    }
}
