use serde::{Deserialize, Serialize};

/// A single SMS message to be sent.
#[derive(Debug, Clone)]
pub struct SmsMessage {
    /// E.164 phone numbers, e.g. `"+254711XXXYYY"`.
    pub to: Vec<String>,
    pub message: String,
    /// Optional sender ID / shortcode registered with the provider.
    pub from: Option<String>,
}

// ── Africa's Talking request ──────────────────────────────────────────────────

/// Form body for `POST /version1/messaging`.
#[derive(Debug, Serialize)]
pub(crate) struct AtSmsRequest<'a> {
    pub username: &'a str,
    pub to: String,
    pub message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<&'a str>,
}

// ── Africa's Talking response ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AtSmsResponse {
    #[serde(rename = "SMSMessageData")]
    pub sms_message_data: AtSmsMessageData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AtSmsMessageData {
    pub message: String,
    pub recipients: Vec<AtRecipient>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtRecipient {
    pub number: String,
    pub status: String,
    pub status_code: i32,
    pub message_id: Option<String>,
    pub cost: Option<String>,
}

impl AtRecipient {
    pub fn is_success(&self) -> bool {
        // Africa's Talking uses statusCode 101 for success
        self.status_code == 101
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn at_response_deserializes() {
        let raw = json!({
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
        });

        let resp: AtSmsResponse = serde_json::from_value(raw).unwrap();
        assert_eq!(resp.sms_message_data.recipients.len(), 1);
        let r = &resp.sms_message_data.recipients[0];
        assert_eq!(r.number, "+254711000111");
        assert!(r.is_success());
        assert_eq!(r.message_id.as_deref(), Some("ATXid_abc123"));
    }

    #[test]
    fn recipient_not_success_on_non_101() {
        let r = AtRecipient {
            number: "+254711000111".into(),
            status: "Failed".into(),
            status_code: 403,
            message_id: None,
            cost: None,
        };
        assert!(!r.is_success());
    }
}
