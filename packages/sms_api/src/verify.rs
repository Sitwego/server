use serde::{Deserialize, Serialize};
use thiserror::Error;
use utils::http_reqwest::{Client, Error as HttpError, ReqwClient};

const VERIFY_BASE: &str = "https://verify.twilio.com";

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("HTTP error: {0}")]
    Http(#[from] HttpError),
    #[error("Failed to decode response: {0}")]
    Decode(#[from] serde_json::Error),
    // ── mapped Twilio Verify error codes ──────────────────────────────────────
    /// 60200
    #[error("Invalid parameter")]
    InvalidParameter,
    /// 60201
    #[error("Invalid phone number")]
    InvalidPhoneNumber,
    /// 60202
    #[error("Max send attempts reached")]
    MaxSendAttemptsReached,
    /// 60203
    #[error("Max check attempts reached")]
    MaxCheckAttemptsReached,
    /// 60204
    #[error("Incorrect OTP code")]
    IncorrectCode,
    /// 60205
    #[error("Landline numbers are not supported")]
    LandlineNotSupported,
    /// 60212
    #[error("Throttle limit exceeded — too many requests")]
    ThrottleLimitExceeded,
    /// 60223
    #[error("Twilio Verify service not found")]
    ServiceNotFound,
    /// Catch-all for any other non-zero error code returned by Twilio.
    #[error("Twilio Verify error {code}: {message}")]
    TwilioError { code: u32, message: String },
}

/// Map a Twilio Verify numeric `error_code` to a typed [`VerifyError`].
fn map_error_code(code: u32, message: String) -> VerifyError {
    match code {
        60200 => VerifyError::InvalidParameter,
        60201 => VerifyError::InvalidPhoneNumber,
        60202 => VerifyError::MaxSendAttemptsReached,
        60203 => VerifyError::MaxCheckAttemptsReached,
        60204 => VerifyError::IncorrectCode,
        60205 => VerifyError::LandlineNotSupported,
        60212 => VerifyError::ThrottleLimitExceeded,
        60223 => VerifyError::ServiceNotFound,
        _ => VerifyError::TwilioError { code, message },
    }
}

/// The channel over which the OTP is delivered.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VerifyChannel {
    Sms,
    Call,
    WhatsApp,
}

/// Client for the [Twilio Verify v2 API](https://www.twilio.com/docs/verify/api).
#[derive(Debug, Clone)]
pub struct TwilioVerifyClient {
    account_sid: String,
    auth_token: String,
    service_sid: String,
    base_url: String,
    client: Client,
}

impl TwilioVerifyClient {
    /// `service_sid` — the SID of your Twilio Verify Service (starts with `VA`).
    pub fn new(
        account_sid: impl Into<String>,
        auth_token: impl Into<String>,
        service_sid: impl Into<String>,
    ) -> Self {
        Self {
            account_sid: account_sid.into(),
            auth_token: auth_token.into(),
            service_sid: service_sid.into(),
            base_url: VERIFY_BASE.to_string(),
            client: ReqwClient::new().into(),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Send an OTP to `to` (E.164) via the given channel.
    pub async fn send_otp(
        &self,
        to: &str,
        channel: VerifyChannel,
    ) -> Result<SendOtpResponse, VerifyError> {
        let url = format!(
            "{}/v2/Services/{}/Verifications",
            self.base_url, self.service_sid
        );

        tracing::debug!(to, ?channel, "sending Twilio Verify OTP");

        let channel_str = match channel {
            VerifyChannel::Sms => "sms",
            VerifyChannel::Call => "call",
            VerifyChannel::WhatsApp => "whatsapp",
        };

        let raw = self
            .client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&[("To", to), ("Channel", channel_str)])
            .send()
            .await?
            .bytes()
            .await
            .map_err(HttpError::from)?;

        let value = serde_json::from_slice::<serde_json::Value>(&raw).map_err(|e| {
            tracing::warn!(
                body = %String::from_utf8_lossy(&raw),
                error = %e,
                "Twilio Verify returned non-JSON body on send"
            );
            e
        })?;

        // Twilio error bodies carry an integer `status` (e.g. 404);
        // success bodies carry a string `status` (e.g. "pending").
        if value["status"].is_number() {
            let code = value["code"].as_u64().unwrap_or(0) as u32;
            let msg = value["message"].as_str().unwrap_or("unknown error").to_string();
            tracing::warn!(to, error_code = code, message = %msg, "Twilio Verify send failed");
            return Err(map_error_code(code, msg));
        }

        let resp = serde_json::from_value::<SendOtpResponse>(value)?;
        Ok(resp)
    }

    /// Check the OTP code submitted by the user.
    pub async fn check_otp(
        &self,
        to: &str,
        code: &str,
    ) -> Result<CheckOtpResponse, VerifyError> {
        let url = format!(
            "{}/v2/Services/{}/VerificationCheck",
            self.base_url, self.service_sid
        );

        tracing::debug!(to, "checking Twilio Verify OTP");

        let raw = self
            .client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&[("To", to), ("Code", code)])
            .send()
            .await?
            .bytes()
            .await
            .map_err(HttpError::from)?;

        let value = serde_json::from_slice::<serde_json::Value>(&raw).map_err(|e| {
            tracing::warn!(
                body = %String::from_utf8_lossy(&raw),
                error = %e,
                "Twilio Verify returned non-JSON body on check"
            );
            e
        })?;

        // Twilio error bodies carry an integer `status` (e.g. 404);
        // success bodies carry a string `status` (e.g. "approved").
        if value["status"].is_number() {
            let code = value["code"].as_u64().unwrap_or(0) as u32;
            let msg = value["message"].as_str().unwrap_or("unknown error").to_string();
            tracing::warn!(to, error_code = code, message = %msg, "Twilio Verify check failed");
            return Err(map_error_code(code, msg));
        }

        let resp = serde_json::from_value::<CheckOtpResponse>(value)?;
        Ok(resp)
    }
}

// ── response types ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SendOtpResponse {
    pub sid: Option<String>,
    pub status: Option<String>,
    pub to: Option<String>,
    pub channel: Option<String>,
    pub error_code: Option<u32>,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckOtpResponse {
    pub sid: Option<String>,
    pub status: Option<String>,
    pub to: Option<String>,
    pub valid: Option<bool>,
    pub error_code: Option<u32>,
    pub message: Option<String>,
}

impl CheckOtpResponse {
    pub fn is_approved(&self) -> bool {
        self.valid.unwrap_or(false)
            && self.status.as_deref() == Some("approved")
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client(base: &str) -> TwilioVerifyClient {
        TwilioVerifyClient::new("AC123", "auth_tok", "VA456")
            .with_base_url(base)
    }

    #[tokio::test]
    async fn send_otp_returns_pending() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v2/Services/VA456/Verifications"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "sid": "VEabc",
                "status": "pending",
                "to": "+254711000111",
                "channel": "sms"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let resp = client(&server.uri())
            .send_otp("+254711000111", VerifyChannel::Sms)
            .await
            .unwrap();

        assert_eq!(resp.status.as_deref(), Some("pending"));
        assert_eq!(resp.sid.as_deref(), Some("VEabc"));
    }

    #[tokio::test]
    async fn send_otp_landline_maps_to_typed_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v2/Services/VA456/Verifications"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({
                "status": 400,
                "error_code": 60205,
                "message": "Landline phone numbers are not supported"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let err = client(&server.uri())
            .send_otp("+254711000111", VerifyChannel::Sms)
            .await
            .unwrap_err();

        assert!(matches!(err, VerifyError::LandlineNotSupported));
    }

    #[tokio::test]
    async fn send_otp_max_attempts_maps_to_typed_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v2/Services/VA456/Verifications"))
            .respond_with(ResponseTemplate::new(429).set_body_json(json!({
                "status": 429,
                "error_code": 60202,
                "message": "Max send attempts reached"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let err = client(&server.uri())
            .send_otp("+254711000111", VerifyChannel::Sms)
            .await
            .unwrap_err();

        assert!(matches!(err, VerifyError::MaxSendAttemptsReached));
    }

    #[tokio::test]
    async fn check_otp_approved() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v2/Services/VA456/VerificationCheck"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "sid": "VEabc",
                "status": "approved",
                "to": "+254711000111",
                "valid": true
            })))
            .expect(1)
            .mount(&server)
            .await;

        let resp = client(&server.uri())
            .check_otp("+254711000111", "123456")
            .await
            .unwrap();

        assert!(resp.is_approved());
    }

    #[tokio::test]
    async fn check_otp_wrong_code_returns_false() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v2/Services/VA456/VerificationCheck"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "sid": "VEabc",
                "status": "pending",
                "to": "+254711000111",
                "valid": false
            })))
            .mount(&server)
            .await;

        let resp = client(&server.uri())
            .check_otp("+254711000111", "000000")
            .await
            .unwrap();

        assert!(!resp.is_approved());
    }

    #[tokio::test]
    async fn check_otp_max_attempts_maps_to_typed_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v2/Services/VA456/VerificationCheck"))
            .respond_with(ResponseTemplate::new(429).set_body_json(json!({
                "status": 429,
                "error_code": 60203,
                "message": "Max check attempts reached"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let err = client(&server.uri())
            .check_otp("+254711000111", "123456")
            .await
            .unwrap_err();

        assert!(matches!(err, VerifyError::MaxCheckAttemptsReached));
    }

    #[tokio::test]
    async fn send_otp_http_error() {
        let client = TwilioVerifyClient::new("AC123", "tok", "VA456")
            .with_base_url("http://127.0.0.1:1");
        let err = client
            .send_otp("+254711000111", VerifyChannel::Sms)
            .await
            .unwrap_err();
        assert!(matches!(err, VerifyError::Http(_)));
    }
}
