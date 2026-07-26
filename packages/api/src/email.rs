//! Application email service: the send call site that ties template rendering
//! (`email_api`) to delivery (`ResendClient`).
//!
//! Two entry points mirror the two template paths:
//! * [`EmailService::send_stored`] — an admin-authored (Unlayer) template loaded
//!   from Postgres, rendered at runtime with a caller context.
//! * [`EmailService::send_rendered`] — a pre-rendered system email (askama), e.g.
//!   [`email_api::OtpEmail`].
//!
//! Held on [`crate::APIContext`] as `Arc<EmailService>` and constructed once from
//! config, like the other external clients.

use db_store::Database;
use email_api::{
    EmailBuilder, EmailError, EmailReceipt, EmailSender, RenderError,
    RenderedEmail, ResendClient, StoredTemplate, TemplateRenderer,
};
use serde::Serialize;
use thiserror::Error;

use crate::queries::email_templates::EmailTemplateQueries;

#[derive(Debug, Error)]
pub enum EmailSendError {
    /// No Resend key configured — outbound email is disabled.
    #[error("email sending is disabled (no RESEND_API_KEY configured)")]
    Disabled,
    #[error("template lookup failed: {0}")]
    Lookup(String),
    #[error(transparent)]
    Render(#[from] RenderError),
    #[error(transparent)]
    Build(#[from] email_api::BuildError),
    #[error(transparent)]
    Send(#[from] EmailError),
}

pub struct EmailService {
    client: ResendClient,
    renderer: TemplateRenderer,
    /// Default `"Name <addr>"` sender used when a template/caller supplies none.
    default_from: String,
    /// Mirrors `Config::email_enabled()`; when false, sends short-circuit.
    enabled: bool,
}

impl EmailService {
    pub fn new(
        api_key: &str,
        default_from: impl Into<String>,
        enabled: bool,
    ) -> Self {
        Self {
            client: ResendClient::new(api_key),
            renderer: TemplateRenderer::new(),
            default_from: default_from.into(),
            enabled,
        }
    }

    /// Render a template against `ctx` **without sending** — the admin
    /// preview/validate path. Deliberately not gated on `enabled`, so an admin
    /// can preview and catch bad merge tags even where delivery is unconfigured.
    /// Uses the exact strict/auto-escape config the send path uses.
    pub fn render<S: Serialize>(
        &self,
        tpl: &StoredTemplate,
        ctx: &S,
    ) -> Result<RenderedEmail, RenderError> {
        self.renderer.render(tpl, ctx)
    }

    /// Load a **published** template by slug and send it, rendering against
    /// `ctx`. Returns `Ok(None)` when no published template with that slug exists
    /// — a not-yet-authored template is not an error, so callers on optional
    /// notifications (welcome mail, etc.) can simply skip.
    pub async fn send_template_by_slug<S: Serialize>(
        &self,
        db: &Database,
        slug: &str,
        to: Vec<String>,
        ctx: &S,
    ) -> Result<Option<EmailReceipt>, EmailSendError> {
        if !self.enabled {
            return Err(EmailSendError::Disabled);
        }

        let Some(model) = db
            .get_published_email_template(slug)
            .await
            .map_err(|e| EmailSendError::Lookup(e.to_string()))?
        else {
            return Ok(None);
        };

        let receipt =
            self.send_stored(&model.to_stored_template(), to, ctx).await?;
        Ok(Some(receipt))
    }

    /// Render an admin-authored template against `ctx` and send it.
    ///
    /// The template's own `from`/`reply_to` win; otherwise the service default
    /// sender is used. `ctx` is any `Serialize` (typically a `serde_json::Value`
    /// or a typed struct describing the merge variables).
    pub async fn send_stored<S: Serialize>(
        &self,
        tpl: &StoredTemplate,
        to: Vec<String>,
        ctx: &S,
    ) -> Result<EmailReceipt, EmailSendError> {
        if !self.enabled {
            return Err(EmailSendError::Disabled);
        }

        let rendered = self.renderer.render(tpl, ctx)?;
        let from =
            tpl.from.clone().unwrap_or_else(|| self.default_from.clone());

        let mut builder =
            EmailBuilder::new().from(from).to_many(to).rendered(rendered);
        if let Some(reply_to) = &tpl.reply_to {
            builder = builder.reply_to(reply_to.clone());
        }

        let msg = builder.build()?;
        Ok(self.client.send(msg).await?)
    }

    /// Send a pre-rendered ([`RenderedEmail`]) email — the system/askama path.
    pub async fn send_rendered(
        &self,
        rendered: RenderedEmail,
        to: Vec<String>,
        from: Option<String>,
        reply_to: Option<String>,
    ) -> Result<EmailReceipt, EmailSendError> {
        if !self.enabled {
            return Err(EmailSendError::Disabled);
        }

        let from = from.unwrap_or_else(|| self.default_from.clone());
        let mut builder =
            EmailBuilder::new().from(from).to_many(to).rendered(rendered);
        if let Some(reply_to) = reply_to {
            builder = builder.reply_to(reply_to);
        }

        let msg = builder.build()?;
        Ok(self.client.send(msg).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::email_template;
    use db_store::Database;
    use sea_orm::{DatabaseBackend, MockDatabase};
    use serde_json::json;
    use utils::executor::Executor;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// A `Database` backed by `MockDatabase`, optionally returning `row` for the
    /// single `get_published_email_template` query.
    fn mock_db(row: Option<email_template::Model>) -> Database {
        let results = match row {
            Some(m) => vec![vec![m]],
            None => vec![Vec::<email_template::Model>::new()],
        };
        let conn = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(results)
            .into_connection();
        Database::from_connection(conn, Executor)
    }

    fn published_template() -> email_template::Model {
        let now = chrono::Utc::now().fixed_offset();
        email_template::Model {
            id: "01J000000000000000000WELCM".into(),
            slug: "welcome_customer".into(),
            name: "Welcome".into(),
            subject_src: "Hi {{ first_name }}".into(),
            html_src: "<p>Welcome {{ first_name }}!</p>".into(),
            design_json: json!({}),
            variables: json!([]),
            is_published: true,
            created_at: now,
            updated_at: now,
        }
    }

    // ── disabled gate ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn disabled_service_never_touches_db_or_network() {
        let svc = EmailService::new("re_test", "from@sitwego.com", false);
        let db = mock_db(None);

        let err = svc
            .send_template_by_slug(
                &db,
                "welcome_customer",
                vec!["u@example.com".into()],
                &json!({ "first_name": "Alex" }),
            )
            .await
            .unwrap_err();

        assert!(matches!(err, EmailSendError::Disabled));
    }

    // ── missing template ───────────────────────────────────────────────────

    #[tokio::test]
    async fn missing_published_template_returns_none() {
        let svc = EmailService::new("re_test", "from@sitwego.com", true);
        let db = mock_db(None);

        let out = svc
            .send_template_by_slug(
                &db,
                "welcome_customer",
                vec!["u@example.com".into()],
                &json!({ "first_name": "Alex" }),
            )
            .await
            .expect("a missing template is Ok(None), not an error");

        assert!(out.is_none());
    }

    // ── happy path: load → render → send ───────────────────────────────────

    #[tokio::test]
    async fn published_template_renders_and_sends() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/emails"))
            // Proves the variable was substituted before the send, not left raw.
            .and(body_string_contains("Welcome Alex"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "test-email-id-123"
            })))
            .expect(1)
            .mount(&server)
            .await;

        // resend-rs reads RESEND_BASE_URL at client construction, so set it
        // before building the service.
        unsafe {
            std::env::set_var("RESEND_BASE_URL", server.uri());
        }
        let svc =
            EmailService::new("re_test", "Sit We Go <from@sitwego.com>", true);
        let db = mock_db(Some(published_template()));

        let receipt = svc
            .send_template_by_slug(
                &db,
                "welcome_customer",
                vec!["u@example.com".into()],
                &json!({ "first_name": "Alex" }),
            )
            .await
            .expect("send should succeed")
            .expect("a published template was found");

        assert_eq!(receipt.id, "test-email-id-123");
    }
}
