//! Outbound `on_*` callback client.
//!
//! Beckn's second half: after ACK-ing an action, the BPP POSTs the real payload
//! to the BAP's `bap_uri` as `on_<action>` (e.g. `POST {bap_uri}/on_search`).
//!
//! [`HttpCallbackClient`] signs each callback natively (Step 3): the payload is
//! serialized ONCE, the Ed25519 signature is computed over those exact bytes,
//! and those same bytes are sent — sign-then-reserialize would break
//! verification on the BAP side (Inviolable Rule #7, outbound edition). The
//! per-action payload builders arrive from Step 4 on.

use async_trait::async_trait;
use axum::http::header;
use serde_json::Value;

use crate::context::CallbackContext;
use crate::crypto::Signer;
use crate::schemas::BecknError;

/// A fully-formed callback ready to send: where to, which action, and the body.
#[derive(Debug, Clone)]
pub struct Callback {
    /// The BAP base URI from the originating request `context`.
    pub bap_uri: String,
    /// The `on_*` action name (e.g. `on_search`).
    pub action: &'static str,
    /// `{ context, message | error }` for the callback.
    pub payload: Value,
}

impl Callback {
    /// Assemble a callback envelope from a [`CallbackContext`] and a `message`.
    pub fn new(context: CallbackContext, message: Value) -> Self {
        let action = context.action;
        let bap_uri = context.bap_uri.clone();
        let payload = serde_json::json!({
            "context": context,
            "message": message,
        });
        Self {
            bap_uri,
            action,
            payload,
        }
    }

    /// Assemble an error callback: `{ context, error }`. The asynchronous
    /// counterpart of a NACK, for failures discovered only after the ACK went
    /// out (e.g. the selected tier vanished between search and select) —
    /// mirrors the optional `error` on nammayatri's `On*Req` envelopes.
    pub fn error(context: CallbackContext, error: &BecknError) -> Self {
        let action = context.action;
        let bap_uri = context.bap_uri.clone();
        let payload = serde_json::json!({
            "context": context,
            "error": error,
        });
        Self {
            bap_uri,
            action,
            payload,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CallbackError {
    #[error("callback transport error: {0}")]
    Transport(String),
    #[error("BAP returned non-success status {0}")]
    Status(u16),
}

/// Sends `on_*` callbacks to BAPs. Trait so we can swap a real HTTP sender for a
/// recording fake in tests.
#[async_trait]
pub trait CallbackClient: Send + Sync {
    async fn send(&self, callback: Callback) -> Result<(), CallbackError>;
}

/// Step-1 stub: logs the callback instead of sending it. Lets the service boot
/// and the inbound half be exercised before the network is wired up.
pub struct LoggingCallbackClient;

#[async_trait]
impl CallbackClient for LoggingCallbackClient {
    async fn send(&self, callback: Callback) -> Result<(), CallbackError> {
        tracing::info!(
            action = callback.action,
            bap_uri = %callback.bap_uri,
            "STUB on_* callback (not sent in Step 1)"
        );
        Ok(())
    }
}

/// A response status worth retrying: the BAP (or a proxy in front of it) had a
/// transient problem. Anything else in 4xx means the request itself was
/// rejected — retrying the same bytes cannot succeed.
fn retryable_status(status: u16) -> bool {
    matches!(status, 408 | 429 | 500..=599)
}

/// Real HTTP sender, wired into the handlers from Step 4. Signs each request
/// when a [`Signer`] is configured; without one, callbacks go out unsigned and
/// network peers will 401 them (acceptable only against a dev BAP).
///
/// Delivery is at-least-once with bounded retries: transport errors and
/// transient statuses are retried with exponential backoff, all inside the
/// signature's validity window so the same signed bytes stay verifiable.
pub struct HttpCallbackClient {
    client: reqwest::Client,
    signer: Option<Signer>,
    max_attempts: u32,
    base_backoff: std::time::Duration,
}

impl HttpCallbackClient {
    pub fn new(signer: Option<Signer>) -> Self {
        Self::with_retry(signer, 3, std::time::Duration::from_millis(500))
    }

    /// `max_attempts` total tries (min 1); backoff doubles from
    /// `base_backoff` between them.
    pub fn with_retry(
        signer: Option<Signer>,
        max_attempts: u32,
        base_backoff: std::time::Duration,
    ) -> Self {
        Self {
            // A hung BAP must not pin a callback task forever.
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
            signer,
            max_attempts: max_attempts.max(1),
            base_backoff,
        }
    }
}

#[async_trait]
impl CallbackClient for HttpCallbackClient {
    async fn send(&self, callback: Callback) -> Result<(), CallbackError> {
        let url = format!(
            "{}/{}",
            callback.bap_uri.trim_end_matches('/'),
            callback.action
        );

        // Serialize once; the signature covers exactly the bytes we send.
        // Signed once too — every retry re-sends the identical bytes and
        // header, valid for the whole signature window (Rule #7).
        let raw_body = serde_json::to_vec(&callback.payload)
            .map_err(|e| CallbackError::Transport(e.to_string()))?;
        let auth = match &self.signer {
            Some(signer) => Some(signer.authorization_header(&raw_body)),
            None => {
                tracing::warn!(
                    action = callback.action,
                    "sending UNSIGNED callback (no BECKN_SIGNING_PRIVATE_KEY)"
                );
                None
            }
        };

        let mut last_err = CallbackError::Transport("not attempted".into());
        for attempt in 1..=self.max_attempts {
            let mut request = self
                .client
                .post(&url)
                .header(header::CONTENT_TYPE, "application/json");
            if let Some(auth) = &auth {
                request = request.header(header::AUTHORIZATION, auth.clone());
            }
            last_err = match request.body(raw_body.clone()).send().await {
                Ok(resp) if resp.status().is_success() => return Ok(()),
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    if !retryable_status(status) {
                        return Err(CallbackError::Status(status));
                    }
                    CallbackError::Status(status)
                }
                Err(e) => CallbackError::Transport(e.to_string()),
            };
            if attempt < self.max_attempts {
                let backoff = self.base_backoff * 2u32.pow(attempt - 1);
                tracing::warn!(
                    action = callback.action,
                    bap_uri = %callback.bap_uri,
                    attempt,
                    "callback attempt failed ({last_err}); retrying in {backoff:?}"
                );
                tokio::time::sleep(backoff).await;
            }
        }
        Err(last_err)
    }
}

/// Test double: forwards every callback into a channel so tests can await and
/// inspect exactly what would have been POSTed to the BAP.
#[cfg(test)]
pub struct RecordingCallbackClient {
    pub tx: tokio::sync::mpsc::UnboundedSender<Callback>,
}

#[cfg(test)]
#[async_trait]
impl CallbackClient for RecordingCallbackClient {
    async fn send(&self, callback: Callback) -> Result<(), CallbackError> {
        self.tx
            .send(callback)
            .map_err(|e| CallbackError::Transport(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    use axum::Router;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::routing::post;

    use super::*;

    /// A fake BAP whose `/on_status` fails the first `fail_first` requests
    /// with `fail_status`, then returns 200. Returns its base URI and the
    /// request counter.
    async fn spawn_bap(
        fail_first: u32,
        fail_status: u16,
    ) -> (String, Arc<AtomicU32>) {
        let hits = Arc::new(AtomicU32::new(0));
        let app = Router::new()
            .route(
                "/on_status",
                post(
                    |State((hits, fail_first, fail_status)): State<(
                        Arc<AtomicU32>,
                        u32,
                        u16,
                    )>| async move {
                        let n = hits.fetch_add(1, Ordering::SeqCst) + 1;
                        if n <= fail_first {
                            StatusCode::from_u16(fail_status).unwrap()
                        } else {
                            StatusCode::OK
                        }
                    },
                ),
            )
            .with_state((hits.clone(), fail_first, fail_status));
        let listener =
            tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (format!("http://{addr}"), hits)
    }

    fn on_status(bap_uri: String) -> Callback {
        Callback {
            bap_uri,
            action: "on_status",
            payload: serde_json::json!({ "test": true }),
        }
    }

    #[tokio::test]
    async fn retries_transient_failures_then_succeeds() {
        let (uri, hits) = spawn_bap(2, 503).await;
        let client =
            HttpCallbackClient::with_retry(None, 3, Duration::from_millis(1));
        client.send(on_status(uri)).await.unwrap();
        assert_eq!(hits.load(Ordering::SeqCst), 3, "two failures + success");
    }

    #[tokio::test]
    async fn does_not_retry_a_rejected_request() {
        let (uri, hits) = spawn_bap(u32::MAX, 400).await;
        let client =
            HttpCallbackClient::with_retry(None, 3, Duration::from_millis(1));
        let err = client.send(on_status(uri)).await.unwrap_err();
        assert!(matches!(err, CallbackError::Status(400)), "{err}");
        assert_eq!(
            hits.load(Ordering::SeqCst),
            1,
            "a 4xx rejection must not be retried"
        );
    }

    #[tokio::test]
    async fn gives_up_after_max_attempts() {
        let (uri, hits) = spawn_bap(u32::MAX, 503).await;
        let client =
            HttpCallbackClient::with_retry(None, 3, Duration::from_millis(1));
        let err = client.send(on_status(uri)).await.unwrap_err();
        assert!(matches!(err, CallbackError::Status(503)), "{err}");
        assert_eq!(hits.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn transport_errors_are_retried() {
        // Nothing listens here — every attempt is a connection error.
        let client =
            HttpCallbackClient::with_retry(None, 2, Duration::from_millis(1));
        let err = client
            .send(on_status("http://127.0.0.1:1".to_string()))
            .await
            .unwrap_err();
        assert!(matches!(err, CallbackError::Transport(_)), "{err}");
    }

    #[test]
    fn retryable_status_matrix() {
        for status in [408, 429, 500, 502, 503, 599] {
            assert!(retryable_status(status), "{status} should be retryable");
        }
        for status in [400, 401, 403, 404, 422] {
            assert!(!retryable_status(status), "{status} must not be retried");
        }
    }
}
