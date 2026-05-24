use serde::{Deserialize, Serialize};

/// Push platform. Gorush uses 1 = iOS, 2 = Android.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "u8")]
pub enum Platform {
    Ios = 1,
    Android = 2,
}

impl From<Platform> for u8 {
    fn from(p: Platform) -> u8 {
        p as u8
    }
}

/// Android-specific notification display options.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AndroidNotificationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/// Android config wrapper as Gorush expects it.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AndroidConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification: Option<AndroidNotificationConfig>,
}

/// A single Gorush notification entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub tokens: Vec<String>,
    pub platform: u8,
    pub title: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub android: Option<AndroidConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_available: Option<bool>,
}

/// Top-level request body for `POST /api/push`.
#[derive(Debug, Clone, Serialize)]
pub struct PushRequest {
    pub notifications: Vec<Notification>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn platform_ios_serializes_to_1() {
        let v: Value = serde_json::to_value(Platform::Ios).unwrap();
        assert_eq!(v, json!(1));
    }

    #[test]
    fn platform_android_serializes_to_2() {
        let v: Value = serde_json::to_value(Platform::Android).unwrap();
        assert_eq!(v, json!(2));
    }

    #[test]
    fn notification_omits_none_fields() {
        let n = Notification {
            tokens: vec!["tok".into()],
            platform: 2,
            title: "T".into(),
            message: "M".into(),
            topic: None,
            data: None,
            android: None,
            priority: None,
            content_available: None,
        };
        let v: Value = serde_json::to_value(&n).unwrap();
        assert!(!v.as_object().unwrap().contains_key("topic"));
        assert!(!v.as_object().unwrap().contains_key("data"));
        assert!(!v.as_object().unwrap().contains_key("android"));
        assert!(!v.as_object().unwrap().contains_key("priority"));
        assert!(!v.as_object().unwrap().contains_key("content_available"));
    }

    #[test]
    fn notification_includes_set_fields() {
        let n = Notification {
            tokens: vec!["tok".into()],
            platform: 2,
            title: "T".into(),
            message: "M".into(),
            topic: Some("com.app".into()),
            data: Some(json!({ "key": "val" })),
            android: Some(AndroidConfig {
                notification: Some(AndroidNotificationConfig {
                    channel_id: Some("ch".into()),
                    color: Some("#fff".into()),
                    click_action: Some("ACT".into()),
                    tag: Some("tag-1".into()),
                }),
            }),
            priority: Some("high".into()),
            content_available: Some(true),
        };
        let v: Value = serde_json::to_value(&n).unwrap();
        assert_eq!(v["topic"], "com.app");
        assert_eq!(v["data"]["key"], "val");
        assert_eq!(v["priority"], "high");
        assert_eq!(v["content_available"], true);
        assert_eq!(v["android"]["notification"]["channel_id"], "ch");
        assert_eq!(v["android"]["notification"]["color"], "#fff");
        assert_eq!(v["android"]["notification"]["click_action"], "ACT");
        assert_eq!(v["android"]["notification"]["tag"], "tag-1");
    }

    #[test]
    fn push_request_wraps_notifications_under_key() {
        let req = PushRequest {
            notifications: vec![Notification {
                tokens: vec!["t".into()],
                platform: 2,
                title: "T".into(),
                message: "M".into(),
                topic: None,
                data: None,
                android: None,
                priority: None,
                content_available: None,
            }],
        };
        let v: Value = serde_json::to_value(&req).unwrap();
        assert!(v["notifications"].is_array());
        assert_eq!(v["notifications"].as_array().unwrap().len(), 1);
    }
}
