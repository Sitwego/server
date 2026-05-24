use crate::types::{AtRecipient, AtSmsRequest, AtSmsResponse, SmsMessage};
use openapi::apis::api20100401_message_api::{
    CreateMessageError, CreateMessageParams,
};
use openapi::apis::configuration::Configuration as TwilioConfig;
use thiserror::Error;
use utils::http_reqwest::{Client, Error as HttpError, ReqwClient};

// ── shared error ──────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum SmsError {
    #[error("HTTP error: {0}")]
    Http(#[from] HttpError),
    #[error("Twilio error: {0}")]
    Twilio(#[from] openapi::apis::Error<CreateMessageError>),
    #[error("SMS send failed: {0}")]
    Send(String),
    #[error("Failed to decode response: {0}")]
    Decode(#[from] serde_json::Error),
}

// ── trait ─────────────────────────────────────────────────────────────────────

pub trait SmsSender {
    fn send(
        &self,
        msg: SmsMessage,
    ) -> impl Future<Output = Result<Vec<SmsReceipt>, SmsError>> + Send;
}

/// Normalised delivery receipt returned by every provider.
#[derive(Debug, Clone)]
pub struct SmsReceipt {
    pub to: String,
    pub message_id: Option<String>,
    pub status: String,
    pub cost: Option<String>,
}

use std::future::Future;

// ── Africa's Talking ──────────────────────────────────────────────────────────

const AT_API_URL: &str = "https://api.africastalking.com/version1/messaging";

/// Client for the [Africa's Talking SMS API](https://developers.africastalking.com/docs/sms/sending).
#[derive(Debug, Clone)]
pub struct AfricasTalkingClient {
    username: String,
    api_key: String,
    /// Override the endpoint — useful in tests.
    endpoint: String,
    client: Client,
}

impl AfricasTalkingClient {
    pub fn new(
        username: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            username: username.into(),
            api_key: api_key.into(),
            endpoint: AT_API_URL.to_string(),
            client: ReqwClient::new().into(),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }
}

impl SmsSender for AfricasTalkingClient {
    async fn send(&self, msg: SmsMessage) -> Result<Vec<SmsReceipt>, SmsError> {
        let to = msg.to.join(",");
        let body = AtSmsRequest {
            username: &self.username,
            to,
            message: &msg.message,
            from: msg.from.as_deref(),
        };

        tracing::debug!(
            recipients = msg.to.len(),
            endpoint = %self.endpoint,
            "sending SMS via Africa's Talking"
        );

        let raw = self
            .client
            .post(&self.endpoint)
            .header("apiKey", &self.api_key)
            .header("Accept", "application/json")
            .form(&body)
            .send()
            .await?
            .bytes()
            .await
            .map_err(HttpError::from)?;

        let resp =
            serde_json::from_slice::<AtSmsResponse>(&raw).map_err(|e| {
                tracing::warn!(
                    body = %String::from_utf8_lossy(&raw),
                    error = %e,
                    "Africa's Talking returned non-JSON body"
                );
                e
            })?;

        let receipts = resp
            .sms_message_data
            .recipients
            .into_iter()
            .map(|r: AtRecipient| {
                if !r.is_success() {
                    tracing::warn!(number = %r.number, status = %r.status, "AT delivery failed");
                }
                SmsReceipt {
                    to: r.number,
                    message_id: r.message_id,
                    status: r.status,
                    cost: r.cost,
                }
            })
            .collect();

        Ok(receipts)
    }
}

// ── Twilio ────────────────────────────────────────────────────────────────────

/// Client for the [Twilio SMS REST API](https://www.twilio.com/docs/sms/api),
/// backed by the generated `openapi` crate at `/home/john/twilio`.
#[derive(Debug, Clone)]
pub struct TwilioClient {
    account_sid: String,
    /// The Twilio phone number (E.164) or messaging service SID to send from.
    from: String,
    config: TwilioConfig,
}

impl TwilioClient {
    /// `from` — your Twilio number (E.164) or messaging service SID.
    pub fn new(
        account_sid: impl Into<String>,
        auth_token: impl Into<String>,
        from: impl Into<String>,
    ) -> Self {
        let account_sid = account_sid.into();
        let auth_token = auth_token.into();
        let config = TwilioConfig {
            basic_auth: Some((account_sid.clone(), Some(auth_token))),
            ..TwilioConfig::default()
        };
        Self {
            account_sid,
            from: from.into(),
            config,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_base_path(
        mut self,
        base_path: impl Into<String>,
    ) -> Self {
        self.config.base_path = base_path.into();
        self
    }
}

impl SmsSender for TwilioClient {
    async fn send(&self, msg: SmsMessage) -> Result<Vec<SmsReceipt>, SmsError> {
        let from = msg.from.as_deref().unwrap_or(&self.from).to_owned();
        let mut receipts = Vec::with_capacity(msg.to.len());

        for number in &msg.to {
            tracing::debug!(
                to = %number,
                base_path = %self.config.base_path,
                "sending SMS via Twilio"
            );

            let params = CreateMessageParams {
                account_sid: self.account_sid.clone(),
                to: number.clone(),
                from: Some(from.clone()),
                body: Some(msg.message.clone()),
                // remaining optional fields
                status_callback: None,
                application_sid: None,
                max_price: None,
                provide_feedback: None,
                attempt: None,
                validity_period: None,
                force_delivery: None,
                content_retention: None,
                address_retention: None,
                smart_encoded: None,
                persistent_action: None,
                traffic_type: None,
                shorten_urls: None,
                schedule_type: None,
                send_at: None,
                send_as_mms: None,
                content_variables: None,
                risk_check: None,
                messaging_service_sid: None,
                media_url: None,
                content_sid: None,
            };

            let result =
                openapi::apis::api20100401_message_api::create_message(
                    &self.config,
                    params,
                )
                .await?;

            if let Some(Some(ref err_msg)) = result.error_message {
                tracing::warn!(
                    to = %number,
                    error = %err_msg,
                    "Twilio delivery failed"
                );
                return Err(SmsError::Send(err_msg.clone()));
            }

            let sid = result.sid.and_then(|s| s);
            let status = result
                .status
                .map(|s| format!("{s:?}"))
                .unwrap_or_else(|| "unknown".into());
            let to =
                result.to.and_then(|t| t).unwrap_or_else(|| number.clone());

            receipts.push(SmsReceipt {
                to,
                message_id: sid,
                status,
                cost: None,
            });
        }

        Ok(receipts)
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::SmsBuilder;
    use serde_json::json;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn at_ok_body() -> serde_json::Value {
        json!({
            "SMSMessageData": {
                "Message": "Sent to 1/1 Total Cost: KES 0.8000",
                "Recipients": [{
                    "number": "+254711000111",
                    "status": "Success",
                    "statusCode": 101,
                    "messageId": "ATXid_abc123",
                    "cost": "KES 0.8000"
                }]
            }
        })
    }

    fn sample_msg() -> SmsMessage {
        SmsBuilder::new()
            .to("+254711000111")
            .message("Your ride arrives in 2 min")
            .build()
            .unwrap()
    }

    // ── Africa's Talking ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn at_send_posts_and_returns_receipt() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/version1/messaging"))
            .and(header("apiKey", "test-key"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(at_ok_body()),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = AfricasTalkingClient::new("sandbox", "test-key")
            .with_endpoint(format!("{}/version1/messaging", server.uri()));

        let receipts = client.send(sample_msg()).await.unwrap();
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].to, "+254711000111");
        assert_eq!(receipts[0].message_id.as_deref(), Some("ATXid_abc123"));
        assert_eq!(receipts[0].status, "Success");
    }

    #[tokio::test]
    async fn at_send_returns_http_err_on_connection_failure() {
        let client = AfricasTalkingClient::new("sandbox", "test-key")
            .with_endpoint("http://127.0.0.1:1/version1/messaging");
        let err = client.send(sample_msg()).await.unwrap_err();
        assert!(matches!(err, SmsError::Http(_)));
    }

    // ── Twilio ────────────────────────────────────────────────────────────────

    fn twilio_ok_body(to: &str) -> serde_json::Value {
        json!({
            "sid": "SMxxx",
            "status": "queued",
            "to": to,
            "error_code": null,
            "error_message": null
        })
    }

    #[tokio::test]
    async fn twilio_send_posts_and_returns_receipt() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/2010-04-01/Accounts/AC123/Messages.json"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(twilio_ok_body("+254711000111")),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = TwilioClient::new("AC123", "auth_tok", "+15005550006")
            .with_base_path(server.uri());

        let receipts = client.send(sample_msg()).await.unwrap();
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].message_id.as_deref(), Some("SMxxx"));
    }

    #[tokio::test]
    async fn twilio_send_errs_on_error_message_in_body() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({
                "sid": null,
                "status": "failed",
                "to": "+254711000111",
                "error_code": 21211,
                "error_message": "The 'To' number is not a valid phone number"
            })))
            .mount(&server)
            .await;

        let client = TwilioClient::new("AC123", "auth_tok", "+15005550006")
            .with_base_path(server.uri());

        let err = client.send(sample_msg()).await.unwrap_err();
        assert!(
            matches!(err, SmsError::Send(ref s) if s.contains("valid phone number"))
        );
    }
}
