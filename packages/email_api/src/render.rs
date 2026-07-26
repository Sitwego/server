//! Runtime rendering of admin-authored (Unlayer) email templates.
//!
//! Unlayer's `editor.exportHtml()` produces a complete HTML email whose merge
//! tags are written `{{ tag }}` — i.e. jinja syntax — so the exported HTML is a
//! valid [minijinja] template and renders with no conversion step. This module
//! substitutes a caller-supplied context into a [`StoredTemplate`]'s subject and
//! body, yielding a [`RenderedEmail`] ready for [`crate::EmailBuilder`].

use minijinja::{AutoEscape, Environment, UndefinedBehavior};
use serde::Serialize;
use thiserror::Error;

use crate::types::{RenderedEmail, StoredTemplate};

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("failed to render {field} of template '{slug}': {source}")]
    Render {
        slug: String,
        field: &'static str,
        #[source]
        source: minijinja::Error,
    },
}

/// Renders admin-authored templates against a context.
///
/// Cheap to construct but reusable — build one and share it. The environment is
/// configured **strict**: referencing a variable the context does not provide is
/// an error (surfaced to the admin as a failed preview) rather than a silently
/// blank spot in a sent email.
#[derive(Debug, Clone)]
pub struct TemplateRenderer {
    env: Environment<'static>,
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateRenderer {
    pub fn new() -> Self {
        let mut env = Environment::new();
        // Undeclared/missing variables must fail loudly, not render blank.
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        // `render_str` templates are unnamed, so minijinja would not auto-escape
        // by extension. Force HTML escaping for every template: the admin's own
        // markup is the template (passes through), while injected *values* (a
        // rider's name, a note) are escaped and cannot break or inject markup.
        env.set_auto_escape_callback(|_name| AutoEscape::Html);
        Self { env }
    }

    /// Render `tpl` against `ctx` (anything `Serialize`, e.g. a
    /// `serde_json::Value` or a typed struct).
    pub fn render<S: Serialize>(
        &self,
        tpl: &StoredTemplate,
        ctx: &S,
    ) -> Result<RenderedEmail, RenderError> {
        let subject =
            self.env.render_str(&tpl.subject_src, ctx).map_err(|source| {
                RenderError::Render {
                    slug: tpl.slug.clone(),
                    field: "subject",
                    source,
                }
            })?;

        let html =
            self.env.render_str(&tpl.html_src, ctx).map_err(|source| {
                RenderError::Render {
                    slug: tpl.slug.clone(),
                    field: "html",
                    source,
                }
            })?;

        Ok(RenderedEmail { subject, html })
    }
}
