use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};

/// Admin-authored email template (see `202607011200000_email_templates.sql`).
///
/// Map [`Model`] onto [`email_api::StoredTemplate`] to render it; `email_api`
/// stays database-agnostic and never depends on sea-orm.
#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "email_templates")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(StringLen::N(26))")]
    pub id: String,
    #[sea_orm(unique)]
    pub slug: String,
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub subject_src: String,
    #[sea_orm(column_type = "Text")]
    pub html_src: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub design_json: serde_json::Value,
    #[sea_orm(column_type = "JsonBinary")]
    pub variables: serde_json::Value,
    pub is_published: bool,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTimeWithTimeZone,
}

impl Model {
    /// Project this stored row onto the DB-agnostic shape the `email_api`
    /// renderer consumes. `design_json`/`variables` are editor/validation
    /// concerns and are intentionally not carried into rendering.
    pub fn to_stored_template(&self) -> email_api::StoredTemplate {
        email_api::StoredTemplate {
            slug: self.slug.clone(),
            subject_src: self.subject_src.clone(),
            html_src: self.html_src.clone(),
            from: None,
            reply_to: None,
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now().fixed_offset();
        Self {
            design_json: Set(serde_json::json!({})),
            variables: Set(serde_json::json!([])),
            is_published: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
            ..ActiveModelTrait::default()
        }
    }
}
