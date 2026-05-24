use crate::types::EmailMessage;

#[derive(Debug, Default)]
pub struct EmailBuilder {
    from: Option<String>,
    to: Vec<String>,
    subject: Option<String>,
    html: Option<String>,
    text: Option<String>,
    reply_to: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("at least one recipient is required")]
    NoRecipients,
    #[error("either html or text body must be provided")]
    NoBody,
}

impl EmailBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    pub fn to(mut self, address: impl Into<String>) -> Self {
        self.to.push(address.into());
        self
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_many(
        mut self,
        addresses: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.to.extend(addresses.into_iter().map(Into::into));
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn reply_to(mut self, reply_to: impl Into<String>) -> Self {
        self.reply_to = Some(reply_to.into());
        self
    }

    pub fn build(self) -> Result<EmailMessage, BuildError> {
        if self.to.is_empty() {
            return Err(BuildError::NoRecipients);
        }
        if self.html.is_none() && self.text.is_none() {
            return Err(BuildError::NoBody);
        }
        Ok(EmailMessage {
            from: self.from.ok_or(BuildError::MissingField("from"))?,
            to: self.to,
            subject: self.subject.ok_or(BuildError::MissingField("subject"))?,
            html: self.html,
            text: self.text,
            reply_to: self.reply_to,
        })
    }
}
