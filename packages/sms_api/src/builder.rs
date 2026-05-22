use crate::types::SmsMessage;

#[derive(Debug, Default)]
pub struct SmsBuilder {
    to: Vec<String>,
    message: Option<String>,
    from: Option<String>,
}

impl SmsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to(mut self, number: impl Into<String>) -> Self {
        self.to.push(number.into());
        self
    }

    pub fn recipients(
        mut self,
        numbers: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.to.extend(numbers.into_iter().map(Into::into));
        self
    }

    pub fn message(mut self, text: impl Into<String>) -> Self {
        self.message = Some(text.into());
        self
    }

    pub fn from(mut self, sender: impl Into<String>) -> Self {
        self.from = Some(sender.into());
        self
    }

    pub fn build(self) -> Result<SmsMessage, &'static str> {
        if self.to.is_empty() {
            return Err("at least one recipient is required");
        }
        let message = self.message.ok_or("message text is required")?;
        Ok(SmsMessage {
            to: self.to,
            message,
            from: self.from,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> SmsBuilder {
        SmsBuilder::new()
            .to("+254711000111")
            .message("Your ride is arriving in 2 minutes.")
    }

    #[test]
    fn build_minimal() {
        let msg = base().build().unwrap();
        assert_eq!(msg.to, vec!["+254711000111"]);
        assert_eq!(msg.message, "Your ride is arriving in 2 minutes.");
        assert!(msg.from.is_none());
    }

    #[test]
    fn build_with_sender_and_multiple_recipients() {
        let msg = SmsBuilder::new()
            .to("+254711000111")
            .to("+254711000222")
            .message("Hello")
            .from("SITWEGO")
            .build()
            .unwrap();
        assert_eq!(msg.to.len(), 2);
        assert_eq!(msg.from.as_deref(), Some("SITWEGO"));
    }

    #[test]
    fn recipients_accumulates() {
        let msg = SmsBuilder::new()
            .recipients(["+1111", "+2222", "+3333"])
            .message("Hi")
            .build()
            .unwrap();
        assert_eq!(msg.to, vec!["+1111", "+2222", "+3333"]);
    }

    #[test]
    fn fails_without_recipient() {
        let err = SmsBuilder::new().message("Hi").build().unwrap_err();
        assert_eq!(err, "at least one recipient is required");
    }

    #[test]
    fn fails_without_message() {
        let err = SmsBuilder::new().to("+254711000111").build().unwrap_err();
        assert_eq!(err, "message text is required");
    }
}
