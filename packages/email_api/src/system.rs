//! Code-owned **system** emails: transactional messages we never want editable in
//! the admin drag-and-drop editor (verification codes, password resets). These
//! are [askama] templates — compile-time checked, in version control, and unit
//! testable without a database or network.
//!
//! Admin-editable marketing/notification emails take the other path: authored in
//! Unlayer and rendered at runtime by [`crate::TemplateRenderer`].

use askama::Template;

use crate::types::RenderedEmail;

/// A one-time verification code email.
#[derive(Debug, Template)]
#[template(path = "system/otp.html")]
pub struct OtpEmail {
    /// The verification code shown to the user.
    pub code: String,
    /// Minutes until the code expires (rendered in the copy).
    pub expiry_minutes: u32,
}

impl OtpEmail {
    pub const SUBJECT: &'static str = "Your Sit We Go verification code";

    /// Render to a [`RenderedEmail`] ready for [`crate::EmailBuilder`].
    pub fn render_email(&self) -> Result<RenderedEmail, askama::Error> {
        Ok(RenderedEmail {
            subject: Self::SUBJECT.to_string(),
            html: self.render()?,
        })
    }
}
