use chrono::Utc;
use db_store::Database;
use redis_store::r_types::AppError;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DbBackend,
    EntityTrait, QueryFilter, QueryOrder, Statement, Value,
    sea_query::OnConflict,
};
use utils::{Result, gen_strings::ulid_string};

use crate::schemas::ride_fare;

pub trait RideFareQueries {
    fn insert_ride_fare(
        &self,
        ride_id: &str,
        components: serde_json::Value,
        total: Decimal,
        status: &str,
        reason: Option<String>,
    ) -> impl std::future::Future<Output = Result<ride_fare::Model, AppError>> + Send;

    fn upsert_ride_fare(
        &self,
        model: ride_fare::Model,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;

    fn get_current_fare(
        &self,
        ride_id: &str,
    ) -> impl std::future::Future<
        Output = Result<Option<ride_fare::Model>, AppError>,
    > + Send;

    fn get_fare_history(
        &self,
        ride_id: &str,
    ) -> impl std::future::Future<
        Output = Result<Vec<ride_fare::Model>, AppError>,
    > + Send;

    /// Update a single component key in place using `jsonb_set`.
    /// Recalculates `total` by summing all numeric values in `components`
    /// after the update — no need to pass a new total manually.
    ///
    /// Example: set_fare_component(id, "waiting_charge", 45.0)
    fn set_fare_component(
        &self,
        fare_id: &str,
        key: &str,
        value: Decimal,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;

    /// Shallow-merge a partial JSONB object into `components` using `||`.
    /// Keys in `patch` overwrite existing keys; absent keys are untouched.
    /// `total` is recomputed from all numeric leaf values after the merge.
    ///
    /// Example: merge_fare_components(id, json!({"tolls": 80, "extra_dx": 20}))
    fn merge_fare_components(
        &self,
        fare_id: &str,
        patch: serde_json::Value,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}

impl RideFareQueries for Database {
    async fn insert_ride_fare(
        &self,
        ride_id: &str,
        components: serde_json::Value,
        total: Decimal,
        status: &str,
        reason: Option<String>,
    ) -> Result<ride_fare::Model, AppError> {
        let id = ulid_string();
        let ride_id = ride_id.to_string();
        let status = status.to_string();

        let model = self
            .transaction(move |tx| {
                let id = id.clone();
                let ride_id = ride_id.clone();
                let components = components.clone();
                let status = status.clone();
                let reason = reason.clone();
                async move {
                    let active = ride_fare::ActiveModel {
                        id: ActiveValue::Set(id),
                        ride_id: ActiveValue::Set(ride_id),
                        components: ActiveValue::Set(components),
                        total: ActiveValue::Set(total),
                        status: ActiveValue::Set(status),
                        reason: ActiveValue::Set(reason),
                        recorded_at: ActiveValue::Set(
                            Utc::now().fixed_offset(),
                        ),
                    };
                    let inserted = active.insert(&*tx).await?;
                    Ok(inserted)
                }
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;

        Ok(model)
    }

    async fn upsert_ride_fare(
        &self,
        model: ride_fare::Model,
    ) -> Result<(), AppError> {
        self.transaction(move |tx| {
            let model = model.clone();
            async move {
                ride_fare::Entity::insert(ride_fare::ActiveModel {
                    id: ActiveValue::Set(model.id),
                    ride_id: ActiveValue::Set(model.ride_id),
                    components: ActiveValue::Set(model.components),
                    total: ActiveValue::Set(model.total),
                    status: ActiveValue::Set(model.status),
                    reason: ActiveValue::Set(model.reason),
                    recorded_at: ActiveValue::Set(model.recorded_at),
                })
                .on_conflict(
                    OnConflict::column(ride_fare::Column::Id)
                        .update_columns([
                            ride_fare::Column::Components,
                            ride_fare::Column::Total,
                            ride_fare::Column::Status,
                            ride_fare::Column::Reason,
                        ])
                        .to_owned(),
                )
                .exec(&*tx)
                .await?;
                Ok(())
            }
        })
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))
    }

    async fn get_current_fare(
        &self,
        ride_id: &str,
    ) -> Result<Option<ride_fare::Model>, AppError> {
        let ride_id = ride_id.to_string();

        self.transaction(move |tx| {
            let ride_id = ride_id.clone();
            async move {
                let fare = ride_fare::Entity::find()
                    .filter(ride_fare::Column::RideId.eq(&ride_id))
                    .order_by_desc(ride_fare::Column::RecordedAt)
                    .one(&*tx)
                    .await?;
                Ok(fare)
            }
        })
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))
    }

    async fn get_fare_history(
        &self,
        ride_id: &str,
    ) -> Result<Vec<ride_fare::Model>, AppError> {
        let ride_id = ride_id.to_string();

        self.transaction(move |tx| {
            let ride_id = ride_id.clone();
            async move {
                let history = ride_fare::Entity::find()
                    .filter(ride_fare::Column::RideId.eq(&ride_id))
                    .order_by_asc(ride_fare::Column::RecordedAt)
                    .all(&*tx)
                    .await?;
                Ok(history)
            }
        })
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))
    }

    async fn set_fare_component(
        &self,
        fare_id: &str,
        key: &str,
        value: Decimal,
    ) -> Result<(), AppError> {
        let fare_id = fare_id.to_string();
        let key = key.to_string();

        self.transaction(move |tx| {
            let fare_id = fare_id.clone();
            let key = key.clone();
            async move {
                // jsonb_set replaces the single key; to_jsonb converts the
                // numeric param so the stored value stays a JSON number.
                // total is recomputed by summing every numeric leaf in the
                // updated components object — avoids any caller-side arithmetic.
                let stmt = Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                    UPDATE ride_fare
                    SET
                        components = jsonb_set(
                            components,
                            ARRAY[$1::text],
                            to_jsonb($2::numeric),
                            true
                        ),
                        total = (
                            SELECT COALESCE(SUM(v::numeric), 0)
                            FROM jsonb_each_text(
                                jsonb_set(components, ARRAY[$1::text], to_jsonb($2::numeric), true)
                            ) AS t(k, v)
                            WHERE v ~ '^-?[0-9]+(\.[0-9]+)?$'
                        )
                    WHERE id = $3
                    "#,
                    vec![
                        Value::String(Some(Box::new(key))),
                        Value::Decimal(Some(Box::new(value))),
                        Value::String(Some(Box::new(fare_id))),
                    ],
                );
                tx.execute(stmt).await?;
                Ok(())
            }
        })
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))
    }

    async fn merge_fare_components(
        &self,
        fare_id: &str,
        patch: serde_json::Value,
    ) -> Result<(), AppError> {
        let fare_id = fare_id.to_string();

        self.transaction(move |tx| {
            let fare_id = fare_id.clone();
            let patch = patch.clone();
            async move {
                // || merges patch into components; keys in patch overwrite,
                // absent keys are preserved. total is recomputed the same way.
                let stmt = Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                    UPDATE ride_fare
                    SET
                        components = components || $1::jsonb,
                        total = (
                            SELECT COALESCE(SUM(v::numeric), 0)
                            FROM jsonb_each_text(components || $1::jsonb) AS t(k, v)
                            WHERE v ~ '^-?[0-9]+(\.[0-9]+)?$'
                        )
                    WHERE id = $2
                    "#,
                    vec![
                        Value::Json(Some(Box::new(patch))),
                        Value::String(Some(Box::new(fare_id))),
                    ],
                );
                tx.execute(stmt).await?;
                Ok(())
            }
        })
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))
    }
}
