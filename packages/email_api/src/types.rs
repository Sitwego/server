use serde::{Deserialize, Serialize};

/// A normalised email message passed to any [`crate::EmailSender`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    /// `"Name <address@example.com>"` or bare `"address@example.com"`.
    pub from: String,
    /// One or more recipient addresses.
    pub to: Vec<String>,
    pub subject: String,
    /// HTML body (preferred).
    pub html: Option<String>,
    /// Plain-text fallback.
    pub text: Option<String>,
    /// Optional reply-to address.
    pub reply_to: Option<String>,
}
