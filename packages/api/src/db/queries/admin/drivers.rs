//! Admin-plane queries for the driver directory (list / search).
//!
//! Reached only through the private admin listener. The `driver` row carries
//! status flags + the (encrypted) photo ref; the human-readable name lives on
//! the related `profile`, so we LEFT JOIN it. Contact data and the profile
//! photo are envelope-encrypted — decrypting them per row would mean a KMS call
//! per driver, so the list returns only cheap fields and the per-driver detail
//! view decrypts on demand.

use db_store::Database;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, FromQueryResult, JoinType, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait, entity::prelude::*,
};
use serde::Serialize;
use utils::Result;

use crate::schemas::{driver, profile, vehicle, vehicle_category_mappings};
use crate::types::VehicleCategory;

/// One row of the admin driver directory.
#[derive(Debug, Serialize, FromQueryResult)]
pub struct DriverListView {
    pub id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    /// S3 ref for the profile photo. The image itself is decrypted on demand by
    /// the detail view, never in the list.
    pub photo_id: Option<String>,
    pub activated: Option<bool>,
    pub has_onboarded: Option<bool>,
    pub created_at: DateTimeWithTimeZone,
}

/// Full row for the per-driver detail header. Carries the envelope-encrypted
/// contact blob so the handler can decrypt email/phone on demand (the same way
/// `captain::get_driver_simple_profile` does for the driver app).
#[derive(Debug, FromQueryResult)]
pub struct DriverDetailRow {
    pub id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub photo_id: Option<String>,
    pub activated: Option<bool>,
    pub has_onboarded: Option<bool>,
    pub created_at: DateTimeWithTimeZone,
    pub contact_data: Option<Vec<u8>>,
    pub nonce: Option<Vec<u8>>,
    pub encrypted_key: Option<Vec<u8>>,
}

/// Envelope-encryption material for a driver's profile photo. The S3 key is
/// `driver-docs/{driver_id}/{photo_id}`, mirroring `get_profile_photo`.
#[derive(Debug)]
pub struct DriverPhotoRef {
    pub photo_id: String,
    pub nonce: Vec<u8>,
    pub encrypted_key: Vec<u8>,
}

pub trait AdminDrivers {
    /// List drivers newest-first, joined to their profile for the display name.
    /// `limit`/`offset` page the result; name/ID filtering is done client-side
    /// in the admin UI over the returned page.
    fn list_drivers(
        &self,
        limit: u64,
        offset: u64,
    ) -> impl std::future::Future<Output = Result<Vec<DriverListView>>> + Send;

    /// Fetch one driver joined to their profile, including the encrypted contact
    /// blob (decrypted by the handler) — backs the detail header.
    fn get_driver_detail(
        &self,
        driver_id: &str,
    ) -> impl std::future::Future<Output = Result<Option<DriverDetailRow>>> + Send;

    /// S3 key + envelope material for a driver's profile photo, so the admin
    /// plane can stream a decrypted copy. `None` if no photo was uploaded.
    fn get_driver_photo_ref(
        &self,
        driver_id: &str,
    ) -> impl std::future::Future<Output = Result<Option<DriverPhotoRef>>> + Send;

    /// Set the categories a driver's vehicle QUALIFIES for, after the admin has
    /// reviewed the vehicle info + documents. This is the eligibility set; the
    /// driver later chooses which of these to actively serve. Replaces the
    /// existing set, preserving the driver's `is_active` choice for any category
    /// that is retained; newly-added categories start active.
    fn set_driver_qualifying_categories(
        &self,
        driver_id: &str,
        vehicle_id: &str,
        categories: &[VehicleCategory],
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}

impl AdminDrivers for Database {
    async fn list_drivers(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<DriverListView>> {
        let rows = driver::Entity::find()
            .select_only()
            .column(driver::Column::Id)
            .column_as(profile::Column::FirstName, "first_name")
            .column_as(profile::Column::LastName, "last_name")
            .column(driver::Column::PhotoId)
            .column(driver::Column::Activated)
            .column(driver::Column::HasOnboarded)
            .column(driver::Column::CreatedAt)
            .join(JoinType::LeftJoin, driver::Relation::Profile.def())
            .order_by_desc(driver::Column::CreatedAt)
            .limit(limit)
            .offset(offset)
            .into_model::<DriverListView>()
            .all(self.conn())
            .await?;

        Ok(rows)
    }

    async fn get_driver_detail(
        &self,
        driver_id: &str,
    ) -> Result<Option<DriverDetailRow>> {
        let row = driver::Entity::find_by_id(driver_id)
            .select_only()
            .column(driver::Column::Id)
            .column_as(profile::Column::FirstName, "first_name")
            .column_as(profile::Column::LastName, "last_name")
            .column(driver::Column::PhotoId)
            .column(driver::Column::Activated)
            .column(driver::Column::HasOnboarded)
            .column(driver::Column::CreatedAt)
            .column(profile::Column::ContactData)
            .column(profile::Column::Nonce)
            .column(profile::Column::EncryptedKey)
            .join(JoinType::LeftJoin, driver::Relation::Profile.def())
            .into_model::<DriverDetailRow>()
            .one(self.conn())
            .await?;

        Ok(row)
    }

    async fn get_driver_photo_ref(
        &self,
        driver_id: &str,
    ) -> Result<Option<DriverPhotoRef>> {
        let row =
            driver::Entity::find_by_id(driver_id).one(self.conn()).await?;

        Ok(row.and_then(|d| {
            match (d.photo_id, d.photo_nonce, d.photo_encrypted_key) {
                (Some(photo_id), Some(nonce), Some(encrypted_key)) => {
                    Some(DriverPhotoRef {
                        photo_id,
                        nonce,
                        encrypted_key,
                    })
                }
                _ => None,
            }
        }))
    }

    async fn set_driver_qualifying_categories(
        &self,
        driver_id: &str,
        vehicle_id: &str,
        categories: &[VehicleCategory],
    ) -> Result<()> {
        let driver_id = driver_id.to_string();
        let vehicle_id = vehicle_id.to_string();
        let categories = categories.to_vec();
        self.transaction(move |tx| {
            let driver_id = driver_id.clone();
            let vehicle_id = vehicle_id.clone();
            let categories = categories.clone();
            Box::pin(async move {
                // The vehicle must exist and belong to this driver — guards
                // against a stale/wrong vehicle_id before the FK constraint.
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

                // Preserve the driver's active choice for categories that are
                // being retained; a freshly-assigned category starts active so
                // it is immediately serveable.
                let prior: std::collections::BTreeMap<VehicleCategory, bool> =
                    vehicle_category_mappings::Entity::find()
                        .filter(
                            vehicle_category_mappings::Column::DriverId
                                .eq(&driver_id),
                        )
                        .all(&*tx)
                        .await?
                        .into_iter()
                        .map(|m| (m.category, m.is_active))
                        .collect();

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
                        .map(|cat| {
                            let is_active =
                                prior.get(&cat).copied().unwrap_or(true);
                            vehicle_category_mappings::ActiveModel {
                                vehicle_id: Set(vehicle_id.clone()),
                                category: Set(cat),
                                driver_id: Set(driver_id.clone()),
                                is_active: Set(is_active),
                                created_at: Set(now),
                                updated_at: Set(now),
                            }
                        })
                        .collect();

                vehicle_category_mappings::Entity::insert_many(models)
                    .exec(&*tx)
                    .await?;
                Ok(())
            })
        })
        .await
    }
}
