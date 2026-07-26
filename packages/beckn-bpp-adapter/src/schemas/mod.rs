//! Wire types shared across actions.
//!
//! The synchronous reply to every inbound Beckn action is an ACK or a NACK —
//! never business data (that arrives later on the `on_*` callback). Shapes here
//! follow the Beckn `ack`/`error` schema.

pub mod on_search;
pub mod order;
pub mod search;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

/// `ACK`/`NACK` are the literal Beckn status strings, so the variants are named
/// to match the wire form exactly.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AckStatus {
    ACK,
    NACK,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ack {
    pub status: AckStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckMessage {
    pub ack: Ack,
}

/// Beckn `error` object. `type` is a coarse category; `code`/`message` describe
/// the specific failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BecknError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: String,
    pub message: String,
}

impl BecknError {
    pub fn context(code: &str, message: impl Into<String>) -> Self {
        Self {
            error_type: "CONTEXT-ERROR".to_string(),
            code: code.to_string(),
            message: message.into(),
        }
    }

    pub fn protocol(code: &str, message: impl Into<String>) -> Self {
        Self {
            error_type: "CORE-ERROR".to_string(),
            code: code.to_string(),
            message: message.into(),
        }
    }
}

/// The synchronous response envelope: `{ message: { ack }, error? }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckResponse {
    pub message: AckMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BecknError>,
}

impl AckResponse {
    pub fn ack() -> Self {
        Self {
            message: AckMessage {
                ack: Ack {
                    status: AckStatus::ACK,
                },
            },
            error: None,
        }
    }

    pub fn nack(error: BecknError) -> Self {
        Self {
            message: AckMessage {
                ack: Ack {
                    status: AckStatus::NACK,
                },
            },
            error: Some(error),
        }
    }
}

impl IntoResponse for AckResponse {
    fn into_response(self) -> Response {
        // Beckn replies are HTTP 200 with the ACK/NACK conveyed in the body.
        // A NACK is still a well-formed protocol response, not a transport error.
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ack_shape() {
        let v = serde_json::to_value(AckResponse::ack()).unwrap();
        assert_eq!(v["message"]["ack"]["status"], "ACK");
        assert!(v.get("error").is_none());
    }

    #[test]
    fn nack_shape_carries_error() {
        let v = serde_json::to_value(AckResponse::nack(BecknError::context(
            "30001",
            "stale request",
        )))
        .unwrap();
        assert_eq!(v["message"]["ack"]["status"], "NACK");
        assert_eq!(v["error"]["code"], "30001");
        assert_eq!(v["error"]["type"], "CONTEXT-ERROR");
    }
}
