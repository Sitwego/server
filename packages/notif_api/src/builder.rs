use crate::types::{
    AndroidConfig, AndroidNotificationConfig, Notification, Platform,
};

#[derive(Debug, Default)]
pub struct NotificationBuilder {
    tokens: Vec<String>,
    platform: Option<u8>,
    title: Option<String>,
    message: Option<String>,
    topic: Option<String>,
    data: Option<serde_json::Value>,
    android_channel: Option<String>,
    android_color: Option<String>,
    android_click_action: Option<String>,
    android_tag: Option<String>,
    priority: Option<String>,
    content_available: Option<bool>,
}

impl NotificationBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.tokens.push(token.into());
        self
    }

    pub fn tokens(
        mut self,
        tokens: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.tokens.extend(tokens.into_iter().map(Into::into));
        self
    }

    pub fn platform(mut self, platform: Platform) -> Self {
        self.platform = Some(platform as u8);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn android_channel(mut self, channel: impl Into<String>) -> Self {
        self.android_channel = Some(channel.into());
        self
    }

    pub fn android_color(mut self, color: impl Into<String>) -> Self {
        self.android_color = Some(color.into());
        self
    }

    pub fn android_tag(mut self, tag: impl Into<String>) -> Self {
        self.android_tag = Some(tag.into());
        self
    }

    pub fn click_action(mut self, action: impl Into<String>) -> Self {
        self.android_click_action = Some(action.into());
        self
    }

    pub fn high_priority(mut self) -> Self {
        self.priority = Some("high".to_string());
        self
    }

    pub fn content_available(mut self) -> Self {
        self.content_available = Some(true);
        self
    }

    pub fn build(self) -> Result<Notification, &'static str> {
        if self.tokens.is_empty() {
            return Err("at least one FCM/APNs token is required");
        }
        let platform = self.platform.ok_or("platform is required")?;
        let title = self.title.ok_or("title is required")?;
        let message = self.message.ok_or("message is required")?;

        let has_android = self.android_channel.is_some()
            || self.android_color.is_some()
            || self.android_click_action.is_some()
            || self.android_tag.is_some();

        let android = has_android.then_some(AndroidConfig {
            notification: Some(AndroidNotificationConfig {
                channel_id: self.android_channel,
                color: self.android_color,
                click_action: self.android_click_action,
                tag: self.android_tag,
            }),
        });

        Ok(Notification {
            tokens: self.tokens,
            platform,
            title,
            message,
            topic: self.topic,
            data: self.data,
            android,
            priority: self.priority,
            content_available: self.content_available,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn base() -> NotificationBuilder {
        NotificationBuilder::new()
            .token("tok_abc")
            .platform(Platform::Android)
            .title("Hello")
            .message("World")
    }

    // ── build success ─────────────────────────────────────────────────────────

    #[test]
    fn build_minimal() {
        let n = base().build().unwrap();
        assert_eq!(n.tokens, vec!["tok_abc"]);
        assert_eq!(n.platform, Platform::Android as u8);
        assert_eq!(n.title, "Hello");
        assert_eq!(n.message, "World");
        assert!(n.topic.is_none());
        assert!(n.data.is_none());
        assert!(n.android.is_none());
        assert!(n.priority.is_none());
        assert!(n.content_available.is_none());
    }

    #[test]
    fn build_ios_platform() {
        let n = NotificationBuilder::new()
            .token("tok_ios")
            .platform(Platform::Ios)
            .title("T")
            .message("M")
            .build()
            .unwrap();
        assert_eq!(n.platform, Platform::Ios as u8);
    }

    #[test]
    fn build_multiple_tokens_via_token() {
        let n = base().token("tok_1").token("tok_2").build().unwrap();
        assert_eq!(n.tokens, vec!["tok_abc", "tok_1", "tok_2"]);
    }

    #[test]
    fn build_multiple_tokens_via_tokens() {
        let n = NotificationBuilder::new()
            .tokens(["t1", "t2", "t3"])
            .platform(Platform::Android)
            .title("T")
            .message("M")
            .build()
            .unwrap();
        assert_eq!(n.tokens, vec!["t1", "t2", "t3"]);
    }

    #[test]
    fn token_and_tokens_accumulate() {
        let n = NotificationBuilder::new()
            .token("first")
            .tokens(["second", "third"])
            .platform(Platform::Android)
            .title("T")
            .message("M")
            .build()
            .unwrap();
        assert_eq!(n.tokens, vec!["first", "second", "third"]);
    }

    #[test]
    fn build_with_topic_and_data() {
        let data = json!({ "ride_id": "abc123", "eta_minutes": 2 });
        let n =
            base().topic("com.app.rider").data(data.clone()).build().unwrap();
        assert_eq!(n.topic.as_deref(), Some("com.app.rider"));
        assert_eq!(n.data.as_ref().unwrap(), &data);
    }

    #[test]
    fn build_high_priority_and_content_available() {
        let n = base().high_priority().content_available().build().unwrap();
        assert_eq!(n.priority.as_deref(), Some("high"));
        assert_eq!(n.content_available, Some(true));
    }

    #[test]
    fn build_android_config_populated() {
        let n = base()
            .android_channel("driver-arrival")
            .android_color("#4CAF50")
            .android_tag("arrival-ride_abc")
            .click_action("OPEN_RIDE_TRACKING")
            .build()
            .unwrap();

        let android = n.android.unwrap();
        let notif = android.notification.unwrap();
        assert_eq!(notif.channel_id.as_deref(), Some("driver-arrival"));
        assert_eq!(notif.color.as_deref(), Some("#4CAF50"));
        assert_eq!(notif.tag.as_deref(), Some("arrival-ride_abc"));
        assert_eq!(notif.click_action.as_deref(), Some("OPEN_RIDE_TRACKING"));
    }

    #[test]
    fn android_config_absent_when_no_android_fields_set() {
        let n = base().build().unwrap();
        assert!(n.android.is_none());
    }

    #[test]
    fn android_config_present_with_single_field() {
        let n = base().android_channel("general").build().unwrap();
        assert!(n.android.is_some());
        let notif = n.android.unwrap().notification.unwrap();
        assert_eq!(notif.channel_id.as_deref(), Some("general"));
        assert!(notif.color.is_none());
        assert!(notif.tag.is_none());
        assert!(notif.click_action.is_none());
    }

    // ── build failures ────────────────────────────────────────────────────────

    #[test]
    fn fails_without_token() {
        let err = NotificationBuilder::new()
            .platform(Platform::Android)
            .title("T")
            .message("M")
            .build()
            .unwrap_err();
        assert_eq!(err, "at least one FCM/APNs token is required");
    }

    #[test]
    fn fails_without_platform() {
        let err = NotificationBuilder::new()
            .token("t")
            .title("T")
            .message("M")
            .build()
            .unwrap_err();
        assert_eq!(err, "platform is required");
    }

    #[test]
    fn fails_without_title() {
        let err = NotificationBuilder::new()
            .token("t")
            .platform(Platform::Android)
            .message("M")
            .build()
            .unwrap_err();
        assert_eq!(err, "title is required");
    }

    #[test]
    fn fails_without_message() {
        let err = NotificationBuilder::new()
            .token("t")
            .platform(Platform::Android)
            .title("T")
            .build()
            .unwrap_err();
        assert_eq!(err, "message is required");
    }
}
