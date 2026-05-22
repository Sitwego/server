use std::future::Future;

use resend_rs::{types::CreateEmailBaseOptions, Resend as ResendSdk};
use thiserror::Error;

use crate::types::EmailMessage;

// ── error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("Resend API error: {0}")]
    Resend(#[from] resend_rs::Error),
    #[error("email send failed: {0}")]
    Send(String),
}

// ── trait ─────────────────────────────────────────────────────────────────────

pub trait EmailSender {
    fn send(
        &self,
        msg: EmailMessage,
    ) -> impl Future<Output = Result<EmailReceipt, EmailError>> + Send;
}

/// Normalised delivery receipt returned after a successful send.
#[derive(Debug, Clone)]
pub struct EmailReceipt {
    /// The Resend message ID (or provider equivalent).
    pub id: String,
}

// ── Resend client ─────────────────────────────────────────────────────────────

/// Client for the [Resend email API](https://resend.com/docs/send-with-rust).
#[derive(Debug, Clone)]
pub struct ResendClient {
    inner: ResendSdk,
}

impl ResendClient {
    /// `api_key` — your Resend API key (starts with `re_`).
    pub fn new(api_key: impl Into<String>) -> Self {
        Self { inner: ResendSdk::new(api_key.into().as_str()) }
    }
}

impl EmailSender for ResendClient {
    async fn send(&self, msg: EmailMessage) -> Result<EmailReceipt, EmailError> {
        let mut req =
            CreateEmailBaseOptions::new(msg.from.as_str(), msg.to.as_slice(), msg.subject.as_str());

        if let Some(html) = &msg.html {
            req = req.with_html(html.as_str());
        }
        if let Some(text) = &msg.text {
            req = req.with_text(text.as_str());
        }
        if let Some(reply_to) = &msg.reply_to {
            req = req.with_reply(reply_to.as_str());
        }

        tracing::debug!(
            to = ?msg.to,
            subject = %msg.subject,
            "sending email via Resend"
        );

        let response = self.inner.emails.send(req).await?;

        Ok(EmailReceipt { id: response.id.to_string() })
    }
}
