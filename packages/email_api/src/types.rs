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

/// An admin-authored template as stored in Postgres, in the DB-agnostic shape the
/// renderer consumes. The `api`/admin-plane layer maps its sea-orm entity onto
/// this; `email_api` deliberately never depends on the database.
///
/// `subject_src` and `html_src` are [minijinja] sources. `html_src` is the HTML
/// exported by the Unlayer editor (`editor.exportHtml()`) — a complete email
/// document whose `{{ merge_tag }}`s are jinja variables. The editor's design
/// JSON (for re-editing) is stored alongside but not needed to render.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTemplate {
    /// Stable identifier used by calling code, e.g. `"ride_receipt"`.
    pub slug: String,
    /// minijinja source for the subject line.
    pub subject_src: String,
    /// minijinja source for the HTML body (Unlayer-exported document).
    pub html_src: String,
    /// Optional `"Name <addr>"` sender override; falls back to the caller default.
    pub from: Option<String>,
    /// Optional reply-to override.
    pub reply_to: Option<String>,
}

/// The output of rendering a template against a context: ready to hand to
/// [`crate::EmailBuilder`].
#[derive(Debug, Clone)]
pub struct RenderedEmail {
    pub subject: String,
    pub html: String,
}
