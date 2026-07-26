//! Admin-plane endpoints for the email template editor.
//!
//! These back the `react-email-editor` (Unlayer) page in the admin app: list /
//! load / save / publish / delete templates, plus a **preview** that renders
//! unsaved editor content so a bad merge tag surfaces before publish. Same rules
//! as the rest of the admin plane — served on the private listener, gated by the
//! internal token, thin delegations to `EmailTemplateQueries` and `EmailService`.

use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use email_api::StoredTemplate;
use redis_store::r_types::AppError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::APIContext;
use crate::queries::email_templates::EmailTemplateQueries;

/// Routes for the template editor. Merged into `admin_handlers` before the
/// internal-token layer, so they inherit the same gating.
pub fn routes() -> Router {
    Router::new()
        .route(
            "/admin/email-templates",
            get(list_templates).put(upsert_template),
        )
        .route(
            "/admin/email-templates/{slug}",
            get(get_template).delete(delete_template),
        )
        .route(
            "/admin/email-templates/{slug}/publish",
            post(publish_template),
        )
        .route("/admin/email-templates/preview", post(preview_template))
}

/// List every template (newest first) for the editor's template picker.
async fn list_templates(
    Extension(ctx): Extension<Arc<APIContext>>,
) -> Result<Response, AppError> {
    let templates = ctx
        .db
        .list_email_templates()
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(Json(templates).into_response())
}

/// Load one template by slug (any state, incl. drafts) so the editor can
/// re-hydrate its Unlayer design and current sources.
async fn get_template(
    Path(slug): Path<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
) -> Result<Response, AppError> {
    let tpl = ctx
        .db
        .get_email_template(&slug)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("template not found".into()))?;
    Ok(Json(tpl).into_response())
}

#[derive(Debug, Deserialize)]
struct UpsertReq {
    /// Stable identifier; the editor supplies it (e.g. `"ride_receipt"`).
    slug: String,
    /// Human label shown in the admin list.
    name: String,
    /// minijinja source for the subject line.
    subject_src: String,
    /// Unlayer-exported HTML (a complete email) with `{{ merge_tag }}`s.
    html_src: String,
    /// Unlayer design document, retained so the editor can re-open the design.
    #[serde(default)]
    design_json: Option<Value>,
    /// Declared variable contract feeding the editor's `mergeTags` option.
    #[serde(default)]
    variables: Option<Value>,
    /// Publish on save; drafts (`false`) can never be sent.
    #[serde(default)]
    is_published: bool,
}

/// Create or replace a template (the editor "save"). Keyed by slug.
async fn upsert_template(
    Extension(ctx): Extension<Arc<APIContext>>,
    Json(body): Json<UpsertReq>,
) -> Result<Response, AppError> {
    if body.slug.trim().is_empty() {
        return Err(AppError::ValidationError("slug is required".into()));
    }

    let stored = ctx
        .db
        .upsert_email_template(
            &body.slug,
            &body.name,
            &body.subject_src,
            &body.html_src,
            body.design_json.unwrap_or_else(|| serde_json::json!({})),
            body.variables.unwrap_or_else(|| serde_json::json!([])),
            body.is_published,
        )
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(Json(stored).into_response())
}

#[derive(Debug, Deserialize)]
struct PublishReq {
    is_published: bool,
}

/// Publish or unpublish a template without touching its content.
async fn publish_template(
    Path(slug): Path<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
    Json(body): Json<PublishReq>,
) -> Result<StatusCode, AppError> {
    ctx.db
        .set_email_template_published(&slug, body.is_published)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(StatusCode::OK)
}

/// Delete a template by slug.
async fn delete_template(
    Path(slug): Path<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
) -> Result<StatusCode, AppError> {
    ctx.db
        .delete_email_template(&slug)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
struct PreviewReq {
    /// The subject source currently in the editor (may be unsaved).
    subject_src: String,
    /// The body HTML currently in the editor (may be unsaved).
    html_src: String,
    /// Sample merge-variable values to render against.
    #[serde(default)]
    context: Value,
}

#[derive(Debug, Serialize)]
struct PreviewResp {
    subject: String,
    html: String,
}

/// Render the editor's current (possibly unsaved) content against sample values.
/// A bad or undeclared merge tag comes back as a 422 with the renderer's message,
/// so the editor can show it inline before anything is saved or sent.
async fn preview_template(
    Extension(ctx): Extension<Arc<APIContext>>,
    Json(body): Json<PreviewReq>,
) -> Result<Response, AppError> {
    let tpl = StoredTemplate {
        slug: "preview".into(),
        subject_src: body.subject_src,
        html_src: body.html_src,
        from: None,
        reply_to: None,
    };

    let rendered = ctx
        .email
        .render(&tpl, &body.context)
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    Ok(Json(PreviewResp {
        subject: rendered.subject,
        html: rendered.html,
    })
    .into_response())
}
