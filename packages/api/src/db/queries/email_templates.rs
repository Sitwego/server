use chrono::Utc;
use db_store::Database;
use redis_store::r_types::AppError;
use sea_orm::{
    ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    sea_query::OnConflict,
};
use utils::{Result, gen_strings::ulid_string};

use crate::schemas::email_template;

pub trait EmailTemplateQueries {
    /// Fetch a template by its stable slug (any state, incl. drafts). Used by the
    /// admin editor.
    fn get_email_template(
        &self,
        slug: &str,
    ) -> impl std::future::Future<
        Output = Result<Option<email_template::Model>, AppError>,
    > + Send;

    /// Fetch a template only if published. Used by the send path so a draft can
    /// never be sent.
    fn get_published_email_template(
        &self,
        slug: &str,
    ) -> impl std::future::Future<
        Output = Result<Option<email_template::Model>, AppError>,
    > + Send;

    /// List every template, newest first. Used by the admin list view.
    fn list_email_templates(
        &self,
    ) -> impl std::future::Future<
        Output = Result<Vec<email_template::Model>, AppError>,
    > + Send;

    /// Create or replace a template by slug (the admin "save" action). Returns
    /// the stored row. `id`/`created_at` are preserved on update; everything else
    /// is overwritten from the editor.
    #[allow(clippy::too_many_arguments)]
    fn upsert_email_template(
        &self,
        slug: &str,
        name: &str,
        subject_src: &str,
        html_src: &str,
        design_json: serde_json::Value,
        variables: serde_json::Value,
        is_published: bool,
    ) -> impl std::future::Future<
        Output = Result<email_template::Model, AppError>,
    > + Send;

    /// Toggle the published flag for a slug.
    fn set_email_template_published(
        &self,
        slug: &str,
        is_published: bool,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;

    /// Delete a template by slug.
    fn delete_email_template(
        &self,
        slug: &str,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}

impl EmailTemplateQueries for Database {
    async fn get_email_template(
        &self,
        slug: &str,
    ) -> Result<Option<email_template::Model>, AppError> {
        let slug = slug.to_string();
        self.transaction(move |tx| {
            let slug = slug.clone();
            async move {
                let tpl = email_template::Entity::find()
                    .filter(email_template::Column::Slug.eq(&slug))
                    .one(&*tx)
                    .await?;
                Ok(tpl)
            }
        })
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))
    }

    async fn get_published_email_template(
        &self,
        slug: &str,
    ) -> Result<Option<email_template::Model>, AppError> {
        let slug = slug.to_string();
        self.transaction(move |tx| {
            let slug = slug.clone();
            async move {
                let tpl = email_template::Entity::find()
                    .filter(email_template::Column::Slug.eq(&slug))
                    .filter(email_template::Column::IsPublished.eq(true))
                    .one(&*tx)
                    .await?;
                Ok(tpl)
            }
        })
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))
    }

    async fn list_email_templates(
        &self,
    ) -> Result<Vec<email_template::Model>, AppError> {
        self.transaction(move |tx| async move {
            let all = email_template::Entity::find()
                .order_by_desc(email_template::Column::UpdatedAt)
                .all(&*tx)
                .await?;
            Ok(all)
        })
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))
    }

    async fn upsert_email_template(
        &self,
        slug: &str,
        name: &str,
        subject_src: &str,
        html_src: &str,
        design_json: serde_json::Value,
        variables: serde_json::Value,
        is_published: bool,
    ) -> Result<email_template::Model, AppError> {
        let slug = slug.to_string();
        let name = name.to_string();
        let subject_src = subject_src.to_string();
        let html_src = html_src.to_string();

        self.transaction(move |tx| {
            let slug = slug.clone();
            let name = name.clone();
            let subject_src = subject_src.clone();
            let html_src = html_src.clone();
            let design_json = design_json.clone();
            let variables = variables.clone();
            async move {
                let now = Utc::now().fixed_offset();
                let active = email_template::ActiveModel {
                    id: ActiveValue::Set(ulid_string()),
                    slug: ActiveValue::Set(slug.clone()),
                    name: ActiveValue::Set(name),
                    subject_src: ActiveValue::Set(subject_src),
                    html_src: ActiveValue::Set(html_src),
                    design_json: ActiveValue::Set(design_json),
                    variables: ActiveValue::Set(variables),
                    is_published: ActiveValue::Set(is_published),
                    created_at: ActiveValue::Set(now),
                    updated_at: ActiveValue::Set(now),
                };

                // Conflict on the unique slug: keep the existing id/created_at,
                // overwrite the editable columns (id/created_at are simply not in
                // the update set).
                email_template::Entity::insert(active)
                    .on_conflict(
                        OnConflict::column(email_template::Column::Slug)
                            .update_columns([
                                email_template::Column::Name,
                                email_template::Column::SubjectSrc,
                                email_template::Column::HtmlSrc,
                                email_template::Column::DesignJson,
                                email_template::Column::Variables,
                                email_template::Column::IsPublished,
                                email_template::Column::UpdatedAt,
                            ])
                            .to_owned(),
                    )
                    .exec(&*tx)
                    .await?;

                let stored = email_template::Entity::find()
                    .filter(email_template::Column::Slug.eq(&slug))
                    .one(&*tx)
                    .await?
                    .ok_or_else(|| {
                        sea_orm::DbErr::Custom(
                            "template vanished after upsert".into(),
                        )
                    })?;
                Ok(stored)
            }
        })
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))
    }

    async fn set_email_template_published(
        &self,
        slug: &str,
        is_published: bool,
    ) -> Result<(), AppError> {
        let slug = slug.to_string();
        self.transaction(move |tx| {
            let slug = slug.clone();
            async move {
                let Some(existing) = email_template::Entity::find()
                    .filter(email_template::Column::Slug.eq(&slug))
                    .one(&*tx)
                    .await?
                else {
                    return Ok(());
                };
                let mut active: email_template::ActiveModel = existing.into();
                active.is_published = ActiveValue::Set(is_published);
                active.updated_at = ActiveValue::Set(Utc::now().fixed_offset());
                email_template::Entity::update(active).exec(&*tx).await?;
                Ok(())
            }
        })
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))
    }

    async fn delete_email_template(&self, slug: &str) -> Result<(), AppError> {
        let slug = slug.to_string();
        self.transaction(move |tx| {
            let slug = slug.clone();
            async move {
                email_template::Entity::delete_many()
                    .filter(email_template::Column::Slug.eq(&slug))
                    .exec(&*tx)
                    .await?;
                Ok(())
            }
        })
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))
    }
}
