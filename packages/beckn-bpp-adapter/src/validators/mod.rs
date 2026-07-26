//! The inbound gate — the single rejection point for every request.
//!
//! Order of checks: signature over the RAW bytes first (Step 3), then context
//! structure, action match, core version, and ttl expiry (Step 1). An
//! unauthenticated request is never parsed as protocol input.

use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use chrono::Utc;

use crate::context::{Action, Context, ContextError};
use crate::crypto::{RegistryClient, SignatureError, SignatureHeader};
use crate::schemas::{AckResponse, BecknError};

/// Map a [`ContextError`] to the Beckn error returned in a NACK.
fn to_beckn_error(err: &ContextError) -> BecknError {
    match err {
        ContextError::MissingField(_)
        | ContextError::ActionMismatch { .. }
        | ContextError::InvalidTtl(_)
        | ContextError::MissingVersion
        | ContextError::UnsupportedVersion { .. } => {
            BecknError::context("30001", err.to_string())
        }
        // 30008 = stale/expired request in the Beckn error code taxonomy.
        ContextError::Expired => BecknError::context("30008", err.to_string()),
    }
}

/// Validate an inbound request's context for the given route action.
///
/// Mirrors nammayatri's `validateContext` ordering: required fields → action →
/// core version → ttl. On success returns `Ok(())` and the handler may ACK; on
/// failure returns the NACK [`AckResponse`] to send back directly.
pub fn validate_inbound(
    ctx: &Context,
    expected: Action,
    supported_version: &str,
) -> Result<(), AckResponse> {
    let run = || -> Result<(), ContextError> {
        ctx.validate_required()?;
        ctx.check_action(expected)?;
        ctx.check_version(supported_version)?;
        ctx.check_not_expired(Utc::now())?;
        Ok(())
    };

    run().map_err(|e| AckResponse::nack(to_beckn_error(&e)))
}

/// Why an inbound request failed signature verification. Details are logged
/// server-side only; the caller just gets `401`.
#[derive(Debug)]
pub enum SignatureRejection {
    MissingHeader,
    Malformed(String),
    UnknownSubscriber {
        subscriber_id: String,
        unique_key_id: String,
    },
    Registry(String),
    Invalid(SignatureError),
}

impl SignatureRejection {
    /// The response mandated by the Beckn signing spec: `401 Unauthorized`
    /// with a `WWW-Authenticate: Signature realm=…` challenge. No body — the
    /// spec defines none, and we don't leak why verification failed.
    pub fn into_response(self, realm: &str) -> Response {
        tracing::warn!(rejection = ?self, "rejecting unverified request");
        let challenge = format!(
            "Signature realm=\"{realm}\",headers=\"{}\"",
            crate::crypto::signature::SIGNED_HEADERS
        );
        (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, challenge)],
        )
            .into_response()
    }
}

/// The subscriber a request's signature proved it came from.
#[derive(Debug, Clone)]
pub struct VerifiedCaller {
    pub subscriber_id: String,
    pub unique_key_id: String,
}

/// Verify the `Authorization` signature over the RAW request bytes
/// (Inviolable Rule #7): parse the header, resolve the signer's public key
/// from the registry, check the validity window, verify Ed25519.
pub async fn verify_signature(
    headers: &HeaderMap,
    raw_body: &[u8],
    registry: &dyn RegistryClient,
) -> Result<VerifiedCaller, SignatureRejection> {
    let header = headers
        .get(header::AUTHORIZATION)
        .ok_or(SignatureRejection::MissingHeader)?
        .to_str()
        .map_err(|_| {
            SignatureRejection::Malformed("non-ASCII Authorization".into())
        })?;

    let sig = SignatureHeader::parse(header)
        .map_err(|e| SignatureRejection::Malformed(e.to_string()))?;

    let public_key = registry
        .signing_public_key(&sig.subscriber_id, &sig.unique_key_id)
        .await
        .map_err(|e| SignatureRejection::Registry(e.to_string()))?
        .ok_or_else(|| SignatureRejection::UnknownSubscriber {
            subscriber_id: sig.subscriber_id.clone(),
            unique_key_id: sig.unique_key_id.clone(),
        })?;

    sig.verify(raw_body, &public_key, Utc::now().timestamp())
        .map_err(SignatureRejection::Invalid)?;

    Ok(VerifiedCaller {
        subscriber_id: sig.subscriber_id,
        unique_key_id: sig.unique_key_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    const VER: &str = "2.0.0";

    fn ctx_with(ttl: Option<&str>, ts_offset_secs: i64) -> Context {
        serde_json::from_value(serde_json::json!({
            "domain": "mobility",
            "action": "search",
            "bap_id": "bap.example.com",
            "bap_uri": "https://bap.example.com/beckn",
            "transaction_id": "txn-1",
            "message_id": "msg-1",
            "timestamp": (Utc::now() + Duration::seconds(ts_offset_secs)).to_rfc3339(),
            "ttl": ttl,
            "version": "2.0.0",
        }))
        .unwrap()
    }

    #[test]
    fn valid_passes() {
        let ctx = ctx_with(Some("PT30S"), 0);
        assert!(validate_inbound(&ctx, Action::Search, VER).is_ok());
    }

    #[test]
    fn expired_ttl_nacks() {
        let ctx = ctx_with(Some("PT1S"), -60); // stamped 60s ago, 1s ttl
        let resp = validate_inbound(&ctx, Action::Search, VER).unwrap_err();
        let v = serde_json::to_value(resp).unwrap();
        assert_eq!(v["message"]["ack"]["status"], "NACK");
        assert_eq!(v["error"]["code"], "30008");
    }

    #[test]
    fn wrong_route_action_nacks() {
        let ctx = ctx_with(Some("PT30S"), 0);
        assert!(validate_inbound(&ctx, Action::Confirm, VER).is_err());
    }

    #[test]
    fn unsupported_version_nacks() {
        let ctx = ctx_with(Some("PT30S"), 0);
        let resp = validate_inbound(&ctx, Action::Search, "1.1.0").unwrap_err();
        let v = serde_json::to_value(resp).unwrap();
        assert_eq!(v["message"]["ack"]["status"], "NACK");
        assert_eq!(v["error"]["code"], "30001");
    }
}
