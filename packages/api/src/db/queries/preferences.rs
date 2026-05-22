use db_store::Database;
use sea_orm::{
    ActiveModelTrait, ActiveValue, DatabaseConnection, DbBackend, EntityTrait,
    Statement,
};

use crate::schemas::{
    profile,
    travel_preferences::{TravelPreferences, TravelPreferencesUpdate},
};

/// Queries for driver travel preferences stored as JSONB on the profile table.
pub trait PreferencesQueries {
    /// Fetches the current travel preferences for a driver.
    ///
    /// Returns `None` if the driver profile does not exist.
    fn get_driver_preferences(
        &self,
        driver_id: &str,
    ) -> impl std::future::Future<
        Output = utils::Result<Option<TravelPreferences>>,
    > + Send;

    /// Merges `updates` into the driver's existing preferences and persists the result.
    ///
    /// Only `Some` fields in `updates` overwrite the stored values.
    /// Returns the full merged preferences after the update.
    fn update_driver_preferences(
        &self,
        driver_id: &str,
        updates: TravelPreferencesUpdate,
    ) -> impl std::future::Future<Output = utils::Result<TravelPreferences>> + Send;

    /// Returns all profiles whose `travel_preferences` contains the given
    /// `category`/`value` pair, using PostgreSQL's indexed JSONB `@>` operator.
    ///
    /// # Arguments
    /// * `category` - One of `"chattiness"`, `"music"`, `"smoking"`, `"pets"`.
    /// * `value`    - The serialized enum variant string, e.g. `"chatty"`.
    fn find_drivers_by_preference(
        &self,
        category: &str,
        value: &str,
    ) -> impl std::future::Future<Output = utils::Result<Vec<profile::Model>>> + Send;

    /// Saves the bio for the given profile.
    fn save_bio(
        &self,
        profile_id: &str,
        bio: String,
    ) -> impl std::future::Future<Output = utils::Result<()>> + Send;
}

impl PreferencesQueries for Database {
    async fn get_driver_preferences(
        &self,
        driver_id: &str,
    ) -> utils::Result<Option<TravelPreferences>> {
        get_driver_preferences(self.conn(), driver_id).await
    }

    async fn update_driver_preferences(
        &self,
        driver_id: &str,
        updates: TravelPreferencesUpdate,
    ) -> utils::Result<TravelPreferences> {
        let id = driver_id.to_owned();
        self.transaction(move |tx| {
            let id = id.clone();
            let updates = updates.clone();
            async move {
                let profile = profile::Entity::find_by_id(&id)
                    .one(&*tx)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("Profile not found for driver_id: {id}")
                    })?;

                let mut current = TravelPreferences::from_json_value(
                    &profile.travel_preferences,
                )
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to deserialize travel_preferences: {e}"
                    )
                })?;

                current.merge(updates);

                let new_json = current.to_json_value().map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to serialize travel_preferences: {e}"
                    )
                })?;

                let mut active: profile::ActiveModel = profile.into();
                active.travel_preferences = ActiveValue::Set(new_json);
                active.update(&*tx).await?;

                Ok(current)
            }
        })
        .await
    }

    async fn find_drivers_by_preference(
        &self,
        category: &str,
        value: &str,
    ) -> utils::Result<Vec<profile::Model>> {
        find_drivers_by_preference(self.conn(), category, value).await
    }

    async fn save_bio(
        &self,
        profile_id: &str,
        bio: String,
    ) -> utils::Result<()> {
        let id = profile_id.to_owned();
        self.transaction(move |tx| {
            let id = id.clone();
            let bio = bio.clone();
            async move {
                let profile = profile::Entity::find_by_id(&id)
                    .one(&*tx)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("Profile not found: {id}")
                    })?;

                let mut active: profile::ActiveModel = profile.into();
                active.bio = ActiveValue::Set(Some(bio));
                active.update(&*tx).await?;
                Ok(())
            }
        })
        .await
    }
}

/// Standalone helper — fetches preferences directly from a `DatabaseConnection`.
///
/// Prefer the `PreferencesQueries` trait when working through `APIContext::db`.
pub async fn get_driver_preferences(
    db: &DatabaseConnection,
    driver_id: &str,
) -> utils::Result<Option<TravelPreferences>> {
    let profile = profile::Entity::find_by_id(driver_id).one(db).await?;
    match profile {
        None => Ok(None),
        Some(p) => {
            let prefs =
                TravelPreferences::from_json_value(&p.travel_preferences)
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "Failed to deserialize travel_preferences: {e}"
                        )
                    })?;
            Ok(Some(prefs))
        }
    }
}

/// Standalone helper — updates preferences directly from a `DatabaseConnection`.
///
/// Prefer the `PreferencesQueries` trait when working through `APIContext::db`.
pub async fn update_driver_preferences(
    db: &DatabaseConnection,
    driver_id: &str,
    updates: TravelPreferencesUpdate,
) -> utils::Result<TravelPreferences> {
    let profile =
        profile::Entity::find_by_id(driver_id).one(db).await?.ok_or_else(
            || anyhow::anyhow!("Profile not found for driver_id: {driver_id}"),
        )?;

    let mut current =
        TravelPreferences::from_json_value(&profile.travel_preferences)
            .map_err(|e| {
                anyhow::anyhow!("Failed to deserialize travel_preferences: {e}")
            })?;

    current.merge(updates);

    let new_json = current.to_json_value().map_err(|e| {
        anyhow::anyhow!("Failed to serialize travel_preferences: {e}")
    })?;

    let mut active: profile::ActiveModel = profile.into();
    active.travel_preferences = ActiveValue::Set(new_json);
    active.update(db).await?;

    Ok(current)
}

/// Standalone helper — finds profiles by a JSONB preference key/value pair.
///
/// Uses PostgreSQL's GIN-indexed `@>` containment operator for efficient lookup.
/// The JSONB filter object is built via `serde_json` to avoid injection.
pub async fn find_drivers_by_preference(
    db: &DatabaseConnection,
    category: &str,
    value: &str,
) -> utils::Result<Vec<profile::Model>> {
    // Build {"<category>": "<value>"} safely — serde_json handles all escaping.
    let mut map = serde_json::Map::new();
    map.insert(
        category.to_owned(),
        serde_json::Value::String(value.to_owned()),
    );
    let filter_json = serde_json::Value::Object(map).to_string();

    let results = profile::Entity::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT * FROM "profile" WHERE travel_preferences @> $1::jsonb"#,
            [filter_json.into()],
        ))
        .all(db)
        .await?;

    Ok(results)
}
