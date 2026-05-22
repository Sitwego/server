// use std::error::Error;

use crate::{
    api::profile::ProfileCreateObject,
    helper,
    schemas::{
        driver::{self, ActiveModel, Entity},
        driver_stats, profile, subscriptions, vehicle,
        vehicle_category_mappings,
    },
    types::{DriverId, VehicleCategory},
};

use redis_store::r_types::AppError;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, SelectColumns,
    entity::prelude::Decimal, prelude::DateTimeLocal,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info};

use db_store::Database;
use sea_orm::{
    ActiveValue, ColumnTrait, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait,
};

use sea_orm::FromQueryResult;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DriverVehicleAndCategories {
    pub vehicle_id: Option<String>,
    pub vehicle_type: Option<String>,
    pub plate_number: Option<String>,
    pub color: Option<String>,
    pub capacity: Option<i32>,
    pub model: Option<String>,
    pub make: Option<String>,
    pub y_manufacturing: Option<i32>,
    pub categories: Vec<VehicleCategory>,
}

#[derive(Debug, FromQueryResult)]
pub struct DriverBundle {
    // From driver
    pub id: String,

    // From profile
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub face_image_id: Option<String>,
    pub verified: Option<bool>,
    pub contact_data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub encrypted_key: Vec<u8>,
    pub is_new: Option<bool>,

    // You can't directly join many-to-one like rating in partial result — see below

    // From vehicle
    pub plate_number: Option<String>,
    pub vehicle_type: Option<String>,
    pub color: Option<String>,

    pub rating: Option<f64>,
    pub total_ratings: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, FromQueryResult)]
pub struct DriverInfo {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub photo_id: String,
    pub rating: Option<Decimal>,
}

#[derive(Debug, FromQueryResult, Serialize, Deserialize, PartialEq)]
pub struct SimpleDriverProfile {
    pub driver_id: String,
    pub photo_id: Option<String>,
    pub contact_data: Option<Vec<u8>>,
    pub nonce: Option<Vec<u8>>,
    pub encrypted_key: Option<Vec<u8>>,
    pub first_name: Option<String>,
    pub is_new: Option<bool>,
    pub verified: Option<bool>,
    pub sub_id: Option<String>,
    pub plan_id: Option<String>,
    pub is_on_free_trial: Option<bool>,
    pub free_trial_end_date: Option<DateTimeLocal>,
    pub is_logged_in: Option<bool>,
    pub has_onboarded: Option<bool>,
    pub rating: Option<f64>,
    pub total_earnings: Option<f64>,
    pub total_rides: Option<i32>,
    pub activated: Option<bool>,
    pub amount_due: Option<Decimal>,
    pub is_plan_active: Option<bool>,
    pub last_billed_at: Option<DateTimeLocal>,
    pub plan_end_date: Option<DateTimeLocal>,
}
pub trait DriverQueries {
    fn create_driver_tx(
        &self,
        id: String,
        driver_profile: ProfileCreateObject,
    ) -> impl std::future::Future<Output = utils::Result<()>> + Send;
    fn set_driver_tx(
        &self,
    ) -> impl std::future::Future<Output = utils::Result<()>> + Send;
    fn set_driver_photo_tx(
        &self,
        driver_id: DriverId,
        photo_id: String,
        photo_nonce: &[u8],
        photo_encrypted_key: &[u8],
    ) -> impl std::future::Future<Output = utils::Result<()>> + Send;
    fn get_driver_tx(
        &self,
    ) -> impl std::future::Future<Output = utils::Result<()>> + Send;
    fn get_driver_simple_profile(
        &self,
        driver_id: &DriverId,
    ) -> impl std::future::Future<
        Output = utils::Result<
            Option<(SimpleDriverProfile, Vec<VehicleCategory>)>,
            AppError,
        >,
    > + Send;
    fn get_driver_rides(
        &self,
    ) -> impl std::future::Future<Output = utils::Result<()>> + Send;

    fn get_driver_profile_and_rating(
        &self,
        driver_id: DriverId,
    ) -> impl std::future::Future<Output = utils::Result<Option<DriverBundle>>> + Send;

    fn get_drivers_stats(
        &self,
        driver_ids: Vec<String>,
    ) -> impl Future<
        Output = utils::Result<HashMap<String, driver_stats::Model>>,
    > + Send;
    /// Returns `(nonce, encrypted_key)` stored at photo upload time.
    fn get_driver_photo_info(
        &self,
        driver_id: &str,
    ) -> impl std::future::Future<
        Output = utils::Result<(Vec<u8>, Vec<u8>), AppError>,
    > + Send;

    fn get_driver_info(
        &self,
        driver_id: &str,
    ) -> impl std::future::Future<
        Output = utils::Result<Option<DriverInfo>, AppError>,
    > + Send;

    fn set_has_onboarded(
        &self,
        driver_id: &str,
        has_onboarded: bool,
    ) -> impl std::future::Future<Output = utils::Result<(), AppError>> + Send;
    fn set_is_activated(
        &self,
        driver_id: &str,
        activated: bool,
    ) -> impl std::future::Future<Output = utils::Result<(), AppError>> + Send;

    /// Returns a short display name for the driver, e.g. `"David K."`.
    /// Falls back to `"Your driver"` if the profile row is missing.
    fn get_driver_display_name(
        &self,
        driver_id: &str,
    ) -> impl std::future::Future<Output = String> + Send;

    fn get_driver_categories(
        &self,
        driver_id: &DriverId,
    ) -> impl std::future::Future<
        Output = utils::Result<Vec<VehicleCategory>, AppError>,
    > + Send;

    fn get_driver_vehicle_and_categories(
        &self,
        driver_id: &DriverId,
    ) -> impl std::future::Future<
        Output = utils::Result<Option<DriverVehicleAndCategories>, AppError>,
    > + Send;

    fn set_driver_categories(
        &self,
        driver_id: &DriverId,
        vehicle_id: &str,
        categories: &[VehicleCategory],
    ) -> impl std::future::Future<Output = utils::Result<(), AppError>> + Send;

    fn login_driver(
        &self,
        phone_number: &str,
        device_id: Option<&str>,
        password: &str,
    ) -> impl std::future::Future<
        Output = utils::Result<Option<driver::Model>, AppError>,
    > + Send;
}

impl DriverQueries for Database {
    async fn create_driver_tx(
        &self,
        id: String,
        driver_profile: ProfileCreateObject,
    ) -> utils::Result<()> {
        let _ = self
            .transaction(move |tx| {
                let id = id.clone();
                let profile_obj = driver_profile.clone();
                async move {
                    let driver = Entity::insert(ActiveModel {
                        id: ActiveValue::set(id),
                        password: ActiveValue::set(profile_obj.password),
                        phone_hash: ActiveValue::set(profile_obj.phone_hash),
                        email_hash: ActiveValue::set(profile_obj.email_hash),
                        ..Default::default()
                    })
                    .exec(&*tx)
                    .await
                    .expect("Failed to insert driver into driver table");
                    Ok(driver)
                }
            })
            .await;
        Ok(())
    }

    async fn set_driver_tx(&self) -> utils::Result<()> {
        todo!()
    }

    async fn get_driver_tx(&self) -> utils::Result<()> {
        todo!()
    }

    async fn get_driver_rides(&self) -> utils::Result<()> {
        todo!()
    }

    async fn login_driver(
        &self,
        phone_number: &str,
        _device_id: Option<&str>,
        password: &str,
    ) -> utils::Result<Option<driver::Model>, AppError> {
        let found_result = self
            .transaction(move |tx| {
                let phone_has = utils::hashing_algo::hash_value(phone_number);
                // get driver by phone hash
                // then verify password
                async move {
                    let driver_opt = driver::Entity::find()
                        .filter(driver::Column::PhoneHash.eq(phone_has))
                        .one(&*tx)
                        .await?;
                    match driver_opt {
                        Some(driver_model) => {
                            let is_valid = helper::verify_password(
                                password,
                                &driver_model.password,
                            )
                            .map_err(|e| {
                                AppError::InternalError(format!(
                                    "Failed to verify password: {:?}",
                                    e
                                ))
                            })?;
                            if is_valid {
                                Ok(Some(driver_model))
                            } else {
                                Ok(None)
                            }
                        }
                        None => Ok(None),
                    }
                }
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;
        Ok(found_result)
    }

    async fn get_driver_profile_and_rating(
        &self,
        driver_id: DriverId,
    ) -> utils::Result<Option<DriverBundle>> {
        let driver_profile = self
            .transaction(move |tx| {
                let driver_id = driver_id.0.to_owned();
                Box::pin(async move {
                    let result = driver::Entity::find()
                        .select_only()
                        // Join profile and vehicle
                        .join(
                            JoinType::LeftJoin,
                            driver::Relation::Profile.def(),
                        )
                        .join(
                            JoinType::LeftJoin,
                            driver::Relation::Vehicle.def(),
                        )
                        .join(
                            JoinType::LeftJoin,
                            driver::Relation::DriverStats.def(),
                        )
                        // Select fields from driver
                        .column(driver::Column::Id)
                        .column_as(driver::Column::PhotoId, "face_image_id")
                        // Select fields from profile
                        .column_as(profile::Column::FirstName, "first_name")
                        .column_as(profile::Column::LastName, "last_name")
                        .column_as(profile::Column::ContactData, "contact_data")
                        .column_as(profile::Column::Nonce, "nonce")
                        .column_as(
                            profile::Column::EncryptedKey,
                            "encrypted_key",
                        )
                        .column_as(profile::Column::IsNew, "is_new")
                        .column_as(profile::Column::Verified, "verified")
                        // Select fields from vehicle
                        .column_as(vehicle::Column::PlateNumber, "plate_number")
                        .column_as(vehicle::Column::VehicleType, "vehicle_type")
                        .column_as(vehicle::Column::Color, "color")
                        // Select fields from vehicle
                        .column_as(
                            driver_stats::Column::TotalRatingScore,
                            "rating",
                        )
                        .column_as(
                            driver_stats::Column::TotalRatings,
                            "total_ratings",
                        )
                        .filter(driver::Column::Id.eq(driver_id))
                        .into_model::<DriverBundle>()
                        .one(&*tx)
                        .await?;

                    Ok(result)
                })
            })
            .await?;
        Ok(driver_profile)
    }

    async fn get_drivers_stats(
        &self,
        driver_ids: Vec<String>,
    ) -> utils::Result<HashMap<String, driver_stats::Model>> {
        let response = self
            .transaction(move |tx| {
                let driver_ids = driver_ids.clone();
                async move {
                    let stats = driver_stats::Entity::find()
                        .filter(
                            driver_stats::Column::DriverId.is_in(driver_ids),
                        )
                        .all(&*tx)
                        .await?;
                    let stats_map = stats
                        .into_iter()
                        .map(|s| (s.driver_id.clone(), s))
                        .collect::<HashMap<String, driver_stats::Model>>();
                    Ok(stats_map)
                }
            })
            .await?;
        Ok(response)
    }

    async fn set_driver_photo_tx(
        &self,
        driver_id: DriverId,
        photo_id: String,
        photo_nonce: &[u8],
        photo_encrypted_key: &[u8],
    ) -> utils::Result<()> {
        let _ = self
            .transaction(move |tx| {
                let driver_id = driver_id.0.to_owned();
                let photo_id = photo_id.to_owned();
                let photo_nonce = photo_nonce.to_vec();
                let photo_encrypted_key = photo_encrypted_key.to_vec();

                Box::pin(async move {
                    let _ = driver::ActiveModel {
                        id: Set(driver_id),
                        photo_id: Set(Some(photo_id)),
                        photo_nonce: Set(Some(photo_nonce)),
                        // Store the KMS ciphertext blob so the photo can be
                        // decrypted after a restart via kms:Decrypt.
                        photo_encrypted_key: Set(Some(photo_encrypted_key)),
                        ..Default::default()
                    }
                    .update(&*tx)
                    .await
                    .map_err(|err| utils::Error::Database(err))?;
                    Ok(())
                })
            })
            .await?;
        Ok(())
    }

    async fn get_driver_info(
        &self,
        driver_id: &str,
    ) -> utils::Result<Option<DriverInfo>, AppError> {
        let driver_info = self
            .transaction(move |tx| {
                let driver_id = driver_id.to_string();
                async move {
                    let result = driver::Entity::find()
                        .select_only()
                        .join(
                            JoinType::LeftJoin,
                            driver::Relation::Profile.def(),
                        )
                        .join(
                            JoinType::LeftJoin,
                            driver::Relation::DriverStats.def(),
                        )
                        .select_column_as(driver::Column::PhotoId, "photo_id")
                        .select_column_as(
                            profile::Column::FirstName,
                            "first_name",
                        )
                        .select_column_as(
                            profile::Column::LastName,
                            "last_name",
                        )
                        .column_as(driver_stats::Column::Rating, "rating")
                        .filter(driver::Column::Id.eq(driver_id))
                        .into_model::<DriverInfo>()
                        .one(&*tx)
                        .await?;

                    Ok(result)
                }
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;

        Ok(driver_info)
    }

    async fn get_driver_simple_profile(
        &self,
        driver_id: &DriverId,
    ) -> utils::Result<
        Option<(SimpleDriverProfile, Vec<VehicleCategory>)>,
        AppError,
    > {
        let simple_profile = driver::Entity::find_by_id(&driver_id.0)
            .select_only()
            .join(JoinType::LeftJoin, driver::Relation::Profile.def())
            .join(JoinType::LeftJoin, driver::Relation::DriverStats.def())
            .join(JoinType::LeftJoin, driver::Relation::Subscriptions.def())
            .select_column_as(driver::Column::Id, "driver_id")
            .select_column_as(profile::Column::ContactData, "contact_data")
            .select_column_as(profile::Column::Nonce, "nonce")
            .select_column_as(profile::Column::EncryptedKey, "encrypted_key")
            .select_column_as(driver::Column::PhotoId, "photo_id")
            .select_column_as(driver::Column::Activated, "activated")
            .select_column_as(driver::Column::HasOnboarded, "has_onboarded")
            .select_column_as(driver::Column::IsLoggedIn, "is_logged_in")
            .select_column_as(driver_stats::Column::TotalRatingScore, "rating")
            .select_column_as(
                driver_stats::Column::TotalEarnings,
                "total_earnings",
            )
            .select_column_as(driver_stats::Column::TotalRides, "total_rides")
            .select_column_as(profile::Column::FirstName, "first_name")
            .select_column_as(profile::Column::IsNew, "is_new")
            .select_column_as(profile::Column::Verified, "verified")
            .select_column_as(subscriptions::Column::Id, "sub_id")
            .select_column_as(subscriptions::Column::PlanId, "plan_id")
            .select_column_as(
                subscriptions::Column::FreeTrialEndDate,
                "free_trial_end_date",
            )
            .select_column_as(
                subscriptions::Column::IsOnFreeTrial,
                "is_on_free_trial",
            )
            .select_column_as(subscriptions::Column::AmountDue, "amount_due")
            .select_column_as(
                subscriptions::Column::IsPlanActive,
                "is_plan_active",
            )
            .select_column_as(
                subscriptions::Column::LastBilledAt,
                "last_billed_at",
            )
            .select_column_as(
                subscriptions::Column::PlanEndDate,
                "plan_end_date",
            )
            .into_model::<SimpleDriverProfile>()
            .one(self.conn())
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;

        let Some(mut driver_profile) = simple_profile else {
            return Ok(None);
        };

        let categories = vehicle_category_mappings::Entity::find()
            .select_only()
            .filter(
                vehicle_category_mappings::Column::DriverId.eq(&driver_id.0),
            )
            .column(vehicle_category_mappings::Column::Category)
            .into_tuple::<VehicleCategory>()
            .all(self.conn())
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;

        Ok(Some((driver_profile, categories)))
    }
    async fn get_driver_photo_info(
        &self,
        driver_id: &str,
    ) -> utils::Result<(Vec<u8>, Vec<u8>), AppError> {
        let row = self
            .transaction(move |tx| {
                Box::pin(async move {
                    driver::Entity::find()
                        .filter(driver::Column::Id.eq(driver_id))
                        .one(&*tx)
                        .await
                        .map_err(utils::Error::Database)
                })
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;

        match row {
            Some(m) => {
                let nonce = m.photo_nonce.ok_or_else(|| {
                    AppError::NotFound("Photo nonce not found".to_string())
                })?;
                let encrypted_key = m.photo_encrypted_key.ok_or_else(|| {
                    AppError::NotFound(
                        "Photo encrypted key not found".to_string(),
                    )
                })?;
                Ok((nonce, encrypted_key))
            }
            None => Err(AppError::NotFound("Photo not found!".to_string())),
        }
    }

    async fn set_has_onboarded(
        &self,
        driver_id: &str,
        has_onboarded: bool,
    ) -> utils::Result<(), AppError> {
        let _ = self
            .transaction(move |tx| {
                let driver_id = driver_id.to_owned();
                Box::pin(async move {
                    let _ = driver::ActiveModel {
                        id: Set(driver_id),
                        has_onboarded: Set(Some(has_onboarded)),
                        ..Default::default()
                    }
                    .update(&*tx)
                    .await
                    .map_err(|err| utils::Error::Database(err))?;
                    Ok(())
                })
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;
        Ok(())
    }
    async fn get_driver_categories(
        &self,
        driver_id: &DriverId,
    ) -> utils::Result<Vec<VehicleCategory>, AppError> {
        let driver_id = driver_id.0.clone();
        let categories = self
            .transaction(move |tx| {
                let driver_id = driver_id.clone();
                Box::pin(async move {
                    let rows = vehicle_category_mappings::Entity::find()
                        .filter(
                            vehicle_category_mappings::Column::DriverId
                                .eq(&driver_id),
                        )
                        .order_by_asc(
                            vehicle_category_mappings::Column::CreatedAt,
                        )
                        .all(&*tx)
                        .await?;
                    Ok(rows.into_iter().map(|r| r.category).collect())
                })
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;
        Ok(categories)
    }

    async fn get_driver_vehicle_and_categories(
        &self,
        driver_id: &DriverId,
    ) -> utils::Result<Option<DriverVehicleAndCategories>, AppError> {
        #[derive(Debug, FromQueryResult)]
        struct VehicleRow {
            id: String,
            vehicle_type: String,
            plate_number: String,
            color: String,
            capacity: Option<i32>,
            y_manufacturing: Option<i32>,
            model: Option<String>,
            make: Option<String>,
        }

        let driver_id = driver_id.0.clone();

        let vehicle_opt = vehicle::Entity::find()
            .select_only()
            .column(vehicle::Column::Id)
            .column(vehicle::Column::VehicleType)
            .column(vehicle::Column::PlateNumber)
            .column(vehicle::Column::Color)
            .column(vehicle::Column::Capacity)
            .column(vehicle::Column::Model)
            .column(vehicle::Column::Make)
            .column(vehicle::Column::YManufacturing)
            .filter(vehicle::Column::DriverId.eq(&driver_id))
            .into_model::<VehicleRow>()
            .one(self.conn())
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;

        let Some(v) = vehicle_opt else {
            return Ok(None);
        };

        let categories = vehicle_category_mappings::Entity::find()
            .filter(vehicle_category_mappings::Column::DriverId.eq(&driver_id))
            .order_by_asc(vehicle_category_mappings::Column::CreatedAt)
            .all(self.conn())
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?
            .into_iter()
            .map(|m| m.category)
            .collect();

        Ok(Some(DriverVehicleAndCategories {
            vehicle_id: Some(v.id),
            vehicle_type: Some(v.vehicle_type),
            plate_number: Some(v.plate_number),
            y_manufacturing: v.y_manufacturing,
            color: Some(v.color),
            capacity: v.capacity,
            model: v.model,
            make: v.make,
            categories,
        }))
    }

    async fn set_driver_categories(
        &self,
        driver_id: &DriverId,
        vehicle_id: &str,
        categories: &[VehicleCategory],
    ) -> utils::Result<(), AppError> {
        let driver_id = driver_id.0.clone();
        let vehicle_id = vehicle_id.to_string();
        let categories = categories.to_vec();
        self.transaction(move |tx| {
            let driver_id = driver_id.clone();
            let vehicle_id = vehicle_id.clone();
            let categories = categories.clone();
            Box::pin(async move {
                // Verify the vehicle exists and belongs to this driver before
                // touching the mappings table — catches stale/wrong vehicle_id
                // from the client before hitting the FK constraint.
                let vehicle_exists = vehicle::Entity::find_by_id(&vehicle_id)
                    .filter(vehicle::Column::DriverId.eq(&driver_id))
                    .one(&*tx)
                    .await?
                    .is_some();
                if !vehicle_exists {
                    return Err(utils::Error::Http(
                        axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                        format!(
                            "Vehicle {} not found for this driver",
                            vehicle_id
                        ),
                        hyper::HeaderMap::new(),
                    ));
                }

                vehicle_category_mappings::Entity::delete_many()
                    .filter(
                        vehicle_category_mappings::Column::DriverId
                            .eq(&driver_id),
                    )
                    .exec(&*tx)
                    .await?;

                if categories.is_empty() {
                    return Ok(());
                }
                let now = chrono::Utc::now().fixed_offset();
                let models: Vec<vehicle_category_mappings::ActiveModel> =
                    categories
                        .into_iter()
                        .map(|cat| vehicle_category_mappings::ActiveModel {
                            vehicle_id: Set(vehicle_id.clone()),
                            category: Set(cat),
                            driver_id: Set(driver_id.clone()),
                            created_at: Set(now),
                            updated_at: Set(now),
                        })
                        .collect();

                vehicle_category_mappings::Entity::insert_many(models)
                    .exec(&*tx)
                    .await?;
                Ok(())
            })
        })
        .await
        .map_err(|err| {
            error!("Failed to set driver categories  {:?}", err);
            AppError::DatabaseError(err.to_string())
        })?;
        Ok(())
    }

    async fn get_driver_display_name(&self, driver_id: &str) -> String {
        profile::Entity::find_by_id(driver_id)
            .one(self.conn())
            .await
            .ok()
            .flatten()
            .map(|p| {
                let initial = p
                    .last_name
                    .chars()
                    .next()
                    .map(|c| format!(" {}.", c))
                    .unwrap_or_default();
                format!("{}{}", p.first_name, initial)
            })
            .unwrap_or_else(|| "Your driver".to_string())
    }

    async fn set_is_activated(
        &self,
        driver_id: &str,
        activated: bool,
    ) -> utils::Result<(), AppError> {
        let _ = self
            .transaction(move |tx| {
                let driver_id = driver_id.to_owned();
                Box::pin(async move {
                    let _ = driver::ActiveModel {
                        id: Set(driver_id),
                        activated: Set(Some(activated)),
                        ..Default::default()
                    }
                    .update(&*tx)
                    .await
                    .map_err(|err| utils::Error::Database(err))?;
                    Ok(())
                })
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;
        Ok(())
    }
}
