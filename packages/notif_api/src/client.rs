use crate::types::{Notification, PushRequest};
use serde::Deserialize;
use thiserror::Error;
use utils::http_reqwest::{Client, Error as HttpError, ReqwClient};

#[derive(Debug, Error)]
pub enum GorushError {
    #[error("HTTP error: {0}")]
    Http(#[from] HttpError),
    #[error("Gorush push failed: {0}")]
    Push(String),
    #[error("Failed to decode Gorush response: {0}")]
    Decode(#[from] serde_json::Error),
}

/// Response body returned by Gorush's `/api/push`.
#[derive(Debug, Deserialize)]
pub struct GorushResponse {
    pub counts: Option<i64>,
    pub success: Option<String>,
    pub logs: Option<Vec<serde_json::Value>>,
}

/// Thin async client that forwards notifications to a Gorush instance.
#[derive(Debug, Clone)]
pub struct GorushClient {
    endpoint: String,
    client: Client,
}

impl GorushClient {
    /// `endpoint` — base URL of your Gorush server, e.g. `"http://localhost:8088"`.
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: ReqwClient::new().into(),
        }
    }

    /// Send a single notification.
    pub async fn send(
        &self,
        notification: Notification,
    ) -> Result<GorushResponse, GorushError> {
        self.send_batch(vec![notification]).await
    }

    /// Send multiple notifications in one request.
    pub async fn send_batch(
        &self,
        notifications: Vec<Notification>,
    ) -> Result<GorushResponse, GorushError> {
        let url = format!("{}/api/push", self.endpoint);
        let count = notifications.len();
        let body = PushRequest { notifications };

        tracing::debug!(count, url, "sending notification batch to Gorush");

        let raw = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .bytes()
            .await
            .map_err(HttpError::from)?;

        if raw.is_empty() {
            return Ok(GorushResponse {
                counts: None,
                success: None,
                logs: None,
            });
        }

        let resp = serde_json::from_slice::<GorushResponse>(&raw).map_err(|e| {
            tracing::warn!(
                body = %String::from_utf8_lossy(&raw),
                error = %e,
                "gorush returned non-JSON body"
            );
            e
        })?;

        if let Some(ref success) = resp.success {
            if success != "ok" {
                return Err(GorushError::Push(success.clone()));
            }
        }

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::NotificationBuilder;
    use crate::types::Platform;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn sample_notification() -> Notification {
        NotificationBuilder::new()
            .token("fcm_tok_abc")
            .platform(Platform::Android)
            .title("Driver arriving")
            .message("2 min away")
            .topic("com.app.rider")
            .data(json!({ "ride_id": "r_123", "eta_minutes": 2 }))
            .android_channel("driver-arrival")
            .android_color("#4CAF50")
            .android_tag("arrival-r_123")
            .click_action("OPEN_RIDE_TRACKING")
            .high_priority()
            .content_available()
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn send_posts_to_api_push_and_returns_ok() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({ "counts": 1, "success": "ok", "logs": [] })),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = GorushClient::new(server.uri());
        let resp = client.send(sample_notification()).await.unwrap();

        assert_eq!(resp.success.as_deref(), Some("ok"));
        assert_eq!(resp.counts, Some(1));
    }

    #[tokio::test]
    async fn send_batch_sends_all_notifications() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({ "counts": 2, "success": "ok", "logs": [] })),
            )
            .expect(1) // one request containing two notifications
            .mount(&server)
            .await;

        let client = GorushClient::new(server.uri());
        let resp = client
            .send_batch(vec![sample_notification(), sample_notification()])
            .await
            .unwrap();

        assert_eq!(resp.counts, Some(2));
    }

    #[tokio::test]
    async fn send_request_body_matches_gorush_schema() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({ "counts": 1, "success": "ok", "logs": [] })),
            )
            .mount(&server)
            .await;

        let client = GorushClient::new(server.uri());
        client.send(sample_notification()).await.unwrap();

        // Inspect the captured request body
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);

        let body: serde_json::Value =
            serde_json::from_slice(&requests[0].body).unwrap();

        let notif = &body["notifications"][0];
        assert_eq!(notif["tokens"][0], "fcm_tok_abc");
        assert_eq!(notif["platform"], 2);
        assert_eq!(notif["title"], "Driver arriving");
        assert_eq!(notif["message"], "2 min away");
        assert_eq!(notif["topic"], "com.app.rider");
        assert_eq!(notif["data"]["ride_id"], "r_123");
        assert_eq!(notif["priority"], "high");
        assert_eq!(notif["content_available"], true);
        assert_eq!(notif["android"]["notification"]["channel_id"], "driver-arrival");
        assert_eq!(notif["android"]["notification"]["color"], "#4CAF50");
        assert_eq!(notif["android"]["notification"]["tag"], "arrival-r_123");
        assert_eq!(notif["android"]["notification"]["click_action"], "OPEN_RIDE_TRACKING");
    }

    #[tokio::test]
    async fn send_returns_err_on_gorush_failure_status() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/push"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({ "counts": 0, "success": "failed", "logs": [] })),
            )
            .mount(&server)
            .await;

        let client = GorushClient::new(server.uri());
        let err = client.send(sample_notification()).await.unwrap_err();

        assert!(matches!(err, GorushError::Push(ref s) if s == "failed"));
    }

    #[tokio::test]
    async fn send_returns_err_on_http_error() {
        // Point at a port nothing is listening on
        let client = GorushClient::new("http://127.0.0.1:1");
        let err = client.send(sample_notification()).await.unwrap_err();
        assert!(matches!(err, GorushError::Http(_)));
    }
}
