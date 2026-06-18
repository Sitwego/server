//! Admin-plane queries for driver onboarding document verification.
//!
//! Drivers upload identity + typed documents during onboarding (see
//! `queries::docs`). An admin reviews the active version of each document and
//! approves or rejects it. Once every required document is approved the admin
//! confirms activation, which flips `driver.activated` (the hybrid flow).
//!
//! These run only behind the private admin plane — never on a public route.

use db_store::Database;
use sea_orm::FromQueryResult;
use sea_orm::{
    ActiveValue, ColumnTrait, IntoActiveModel, Iterable, PaginatorTrait,
    QueryFilter, QuerySelect, entity::prelude::*,
};
use serde::Serialize;
use utils::Result;

use crate::schemas::docs::{
    self, DocRequirement, DocumentReviewStatus, DriverDocumentType,
    VehicleClass,
};
use crate::schemas::{
    driver, driver_identity_documents, vehicle, vehicle_category_mappings,
};

/// The driver's vehicle class (Car / Bike / Auto), derived from the free-form
/// `vehicle.vehicle_type`. Drives which documents are required/hidden and how
/// they are labelled. Defaults to `Car` (the strictest set) when no vehicle
/// row exists yet.
async fn driver_vehicle_class(
    conn: &impl sea_orm::ConnectionTrait,
    driver_id: &str,
) -> Result<VehicleClass> {
    let v = vehicle::Entity::find()
        .filter(vehicle::Column::DriverId.eq(driver_id))
        .one(conn)
        .await?;
    Ok(v.map(|v| VehicleClass::from_vehicle_type(&v.vehicle_type))
        .unwrap_or(VehicleClass::Car))
}

/// Projection of the driver row's profile-photo review state.
#[derive(Debug, FromQueryResult)]
struct PhotoReviewRow {
    photo_id: Option<String>,
    photo_review_status: DocumentReviewStatus,
    photo_reviewed_by: Option<String>,
    photo_reviewed_at: Option<DateTimeWithTimeZone>,
    photo_reject_reason: Option<String>,
    created_at: DateTimeWithTimeZone,
}

/// Which table a review action targets. Both are keyed by a `BIGSERIAL` id, so
/// the id alone is ambiguous without the kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocKind {
    /// `driver_documents` — license, PSV badge/insurance, good conduct, etc.
    Document,
    /// `driver_identity_documents` — national ID / passport (front + back).
    Identity,
}

/// Flattened view of one document for the admin review UI. Identity docs carry
/// a back image + id number; typed docs carry `metadata` (e.g. expiry).
#[derive(Debug, Serialize)]
pub struct DriverDocumentView {
    pub id: i64,
    pub kind: &'static str,
    /// Human-readable document name for the review UI (e.g. "PSV Badge").
    pub doc_type: String,
    /// Whether this document must be approved before the driver can be
    /// activated. Optional docs (e.g. KRA) are reviewable but never block.
    pub required: bool,
    pub file_id: String,
    pub file_id_back: Option<String>,
    pub id_number: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub version: i32,
    pub review_status: DocumentReviewStatus,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTimeWithTimeZone>,
    pub reject_reason: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

/// Everything needed to fetch + decrypt one document image from S3. The S3 key
/// is reconstructed as `driver-docs/{driver_id}/{file_id}` (see `api::docs`).
#[derive(Debug)]
pub struct DocBlobRef {
    pub driver_id: String,
    pub file_id: String,
    pub nonce: Vec<u8>,
    pub encrypted_key: Vec<u8>,
}

pub trait AdminDocuments {
    /// List the **active** documents (both tables) for a driver, with their
    /// current review state, ordered identity-first then typed docs.
    fn list_driver_documents(
        &self,
        driver_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<DriverDocumentView>>> + Send;

    /// Record an admin's approve/reject decision on a single document.
    /// On `Approved` any prior reject reason is cleared; `Rejected` stores the
    /// reason shown to the driver so they can re-upload.
    fn review_document(
        &self,
        kind: DocKind,
        doc_id: i64,
        status: DocumentReviewStatus,
        reviewed_by: &str,
        reject_reason: Option<&str>,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Record an approve/reject decision on a driver's profile photo. The photo
    /// has no document row, so its review state lives on the driver record.
    fn review_driver_photo(
        &self,
        driver_id: &str,
        status: DocumentReviewStatus,
        reviewed_by: &str,
        reject_reason: Option<&str>,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Flip `driver.activated`. Called when the admin confirms a driver is
    /// cleared to go live (hybrid: docs reviewed first, then explicit confirm).
    fn activate_driver(
        &self,
        driver_id: &str,
        activate: bool,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Reasons a driver cannot yet be activated. An empty vec means the driver
    /// is clear to go live: the profile photo, every active identity document,
    /// and every *required* typed document must be APPROVED, and the admin must
    /// have assigned at least one vehicle category. Each entry is a
    /// human-readable blocker for the admin UI.
    fn driver_activation_blockers(
        &self,
        driver_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<String>>> + Send;

    /// Look up the S3 key + envelope-encryption material for one document image
    /// so the admin plane can stream a decrypted copy. For identity docs `back`
    /// selects the reverse side; it is ignored for typed docs.
    fn get_document_blob_ref(
        &self,
        kind: DocKind,
        doc_id: i64,
        back: bool,
    ) -> impl std::future::Future<Output = Result<Option<DocBlobRef>>> + Send;
}

impl AdminDocuments for Database {
    async fn list_driver_documents(
        &self,
        driver_id: &str,
    ) -> Result<Vec<DriverDocumentView>> {
        let conn = self.conn();

        let class = driver_vehicle_class(conn, driver_id).await?;

        let identity = driver_identity_documents::Entity::find()
            .filter(driver_identity_documents::Column::DriverId.eq(driver_id))
            .filter(driver_identity_documents::Column::IsActive.eq(true))
            .all(conn)
            .await?;

        let typed = docs::Entity::find()
            .filter(docs::Column::DriverId.eq(driver_id))
            .filter(docs::Column::IsActive.eq(true))
            .all(conn)
            .await?;

        // The profile photo lives on the driver row (one per driver), not a
        // documents table — pull its review state so it can be reviewed too.
        let photo = driver::Entity::find_by_id(driver_id)
            .select_only()
            .column(driver::Column::PhotoId)
            .column(driver::Column::PhotoReviewStatus)
            .column(driver::Column::PhotoReviewedBy)
            .column(driver::Column::PhotoReviewedAt)
            .column(driver::Column::PhotoRejectReason)
            .column(driver::Column::CreatedAt)
            .into_model::<PhotoReviewRow>()
            .one(conn)
            .await?;

        let mut out: Vec<DriverDocumentView> =
            Vec::with_capacity(identity.len() + typed.len() + 1);

        // Photo first — it's the primary identity check in onboarding.
        if let Some(p) = photo
            && let Some(photo_id) = p.photo_id
        {
            out.push(DriverDocumentView {
                id: 0,
                kind: "PHOTO",
                doc_type: "Profile photo".to_string(),
                required: true,
                file_id: photo_id,
                file_id_back: None,
                id_number: None,
                metadata: None,
                version: 1,
                review_status: p.photo_review_status,
                reviewed_by: p.photo_reviewed_by,
                reviewed_at: p.photo_reviewed_at,
                reject_reason: p.photo_reject_reason,
                created_at: p.created_at,
            });
        }

        for d in identity {
            out.push(DriverDocumentView {
                id: d.id,
                kind: "IDENTITY",
                doc_type: d.document_subtype,
                required: true,
                file_id: d.file_id_front,
                file_id_back: Some(d.file_id_back),
                id_number: Some(d.id_number),
                metadata: None,
                version: d.version,
                review_status: d.review_status,
                reviewed_by: d.reviewed_by,
                reviewed_at: d.reviewed_at,
                reject_reason: d.reject_reason,
                created_at: d.created_at,
            });
        }

        for d in typed {
            // Hidden docs don't apply to this vehicle class (e.g. PSV badge for
            // a bike) — leave them out of the review UI entirely.
            let requirement = d.document_type.requirement_for(class);
            if requirement == DocRequirement::Hidden {
                continue;
            }
            out.push(DriverDocumentView {
                id: d.id,
                kind: "DOCUMENT",
                doc_type: d.document_type.label_for(class).to_string(),
                required: requirement == DocRequirement::Required,
                file_id: d.file_id,
                file_id_back: None,
                id_number: None,
                metadata: Some(d.metadata),
                version: d.version,
                review_status: d.review_status,
                reviewed_by: d.reviewed_by,
                reviewed_at: d.reviewed_at,
                reject_reason: d.reject_reason,
                created_at: d.created_at,
            });
        }

        Ok(out)
    }

    async fn review_document(
        &self,
        kind: DocKind,
        doc_id: i64,
        status: DocumentReviewStatus,
        reviewed_by: &str,
        reject_reason: Option<&str>,
    ) -> Result<()> {
        // Reject reason only makes sense on a rejection; clear it otherwise so
        // a re-approved doc doesn't keep showing a stale reason to the driver.
        let reason = match status {
            DocumentReviewStatus::Rejected => {
                reject_reason.map(|s| s.to_owned())
            }
            _ => None,
        };
        let reviewed_by = reviewed_by.to_owned();

        self.transaction(move |tx| {
            // The closure is `Fn` (re-run on serialization retry), so clone the
            // owned values per-invocation instead of moving the captures out.
            let reason = reason.clone();
            let reviewed_by = reviewed_by.clone();
            Box::pin(async move {
                let now = chrono::Utc::now().into();
                match kind {
                    DocKind::Document => {
                        if let Some(doc) =
                            docs::Entity::find_by_id(doc_id).one(&*tx).await?
                        {
                            let mut m = doc.into_active_model();
                            m.review_status = ActiveValue::Set(status);
                            m.reviewed_by = ActiveValue::Set(Some(reviewed_by));
                            m.reviewed_at = ActiveValue::Set(Some(now));
                            m.reject_reason = ActiveValue::Set(reason);
                            m.updated_at = ActiveValue::Set(now);
                            m.update(&*tx).await?;
                        }
                    }
                    DocKind::Identity => {
                        if let Some(doc) =
                            driver_identity_documents::Entity::find_by_id(
                                doc_id,
                            )
                            .one(&*tx)
                            .await?
                        {
                            let mut m = doc.into_active_model();
                            m.review_status = ActiveValue::Set(status);
                            m.reviewed_by = ActiveValue::Set(Some(reviewed_by));
                            m.reviewed_at = ActiveValue::Set(Some(now));
                            m.reject_reason = ActiveValue::Set(reason);
                            m.updated_at = ActiveValue::Set(now);
                            m.update(&*tx).await?;
                        }
                    }
                }
                Ok(())
            })
        })
        .await
    }

    async fn review_driver_photo(
        &self,
        driver_id: &str,
        status: DocumentReviewStatus,
        reviewed_by: &str,
        reject_reason: Option<&str>,
    ) -> Result<()> {
        let reason = match status {
            DocumentReviewStatus::Rejected => {
                reject_reason.map(|s| s.to_owned())
            }
            _ => None,
        };
        let reviewed_by = reviewed_by.to_owned();

        self.transaction(move |tx| {
            // `Fn` closure (re-run on retry) — clone owned captures per call.
            let reason = reason.clone();
            let reviewed_by = reviewed_by.clone();
            Box::pin(async move {
                if let Some(d) =
                    driver::Entity::find_by_id(driver_id).one(&*tx).await?
                {
                    let mut m = d.into_active_model();
                    m.photo_review_status = ActiveValue::Set(status);
                    m.photo_reviewed_by = ActiveValue::Set(Some(reviewed_by));
                    m.photo_reviewed_at =
                        ActiveValue::Set(Some(chrono::Utc::now().into()));
                    m.photo_reject_reason = ActiveValue::Set(reason);
                    m.updated_at =
                        ActiveValue::Set(time::OffsetDateTime::now_utc());
                    m.update(&*tx).await?;
                }
                Ok(())
            })
        })
        .await
    }

    async fn activate_driver(
        &self,
        driver_id: &str,
        activate: bool,
    ) -> Result<()> {
        self.transaction(move |tx| {
            Box::pin(async move {
                if let Some(d) =
                    driver::Entity::find_by_id(driver_id).one(&*tx).await?
                {
                    let mut m = d.into_active_model();
                    m.activated = ActiveValue::Set(Some(activate));
                    m.updated_at =
                        ActiveValue::Set(time::OffsetDateTime::now_utc());
                    m.update(&*tx).await?;
                }
                Ok(())
            })
        })
        .await
    }

    async fn driver_activation_blockers(
        &self,
        driver_id: &str,
    ) -> Result<Vec<String>> {
        let conn = self.conn();
        let mut blockers: Vec<String> = Vec::new();

        let class = driver_vehicle_class(conn, driver_id).await?;

        // Profile photo — must be present and approved.
        match driver::Entity::find_by_id(driver_id).one(conn).await? {
            Some(d) if d.photo_id.is_some() => {
                if d.photo_review_status != DocumentReviewStatus::Approved {
                    blockers.push("Profile photo not approved".to_string());
                }
            }
            _ => blockers.push("Profile photo not uploaded".to_string()),
        }

        // Identity documents — at least one active, and none left unapproved.
        let identity = driver_identity_documents::Entity::find()
            .filter(driver_identity_documents::Column::DriverId.eq(driver_id))
            .filter(driver_identity_documents::Column::IsActive.eq(true))
            .all(conn)
            .await?;
        if identity.is_empty() {
            blockers.push("Identity document not uploaded".to_string());
        } else if identity
            .iter()
            .any(|d| d.review_status != DocumentReviewStatus::Approved)
        {
            blockers.push("Identity document not approved".to_string());
        }

        // Typed documents — every required type must have an approved active doc.
        let typed = docs::Entity::find()
            .filter(docs::Column::DriverId.eq(driver_id))
            .filter(docs::Column::IsActive.eq(true))
            .all(conn)
            .await?;
        for ty in DriverDocumentType::iter() {
            // Only Required docs block; Optional + Hidden (for this class) skip.
            if ty.requirement_for(class) != DocRequirement::Required {
                continue;
            }
            let matching: Vec<&docs::Model> =
                typed.iter().filter(|d| d.document_type == ty).collect();
            if matching.is_empty() {
                blockers.push(format!("{} not uploaded", ty.label_for(class)));
            } else if matching
                .iter()
                .any(|d| d.review_status != DocumentReviewStatus::Approved)
            {
                blockers.push(format!("{} not approved", ty.label_for(class)));
            }
        }

        // At least one qualifying vehicle category must be assigned.
        let category_count = vehicle_category_mappings::Entity::find()
            .filter(vehicle_category_mappings::Column::DriverId.eq(driver_id))
            .count(conn)
            .await?;
        if category_count == 0 {
            blockers.push("No vehicle category assigned".to_string());
        }

        Ok(blockers)
    }

    async fn get_document_blob_ref(
        &self,
        kind: DocKind,
        doc_id: i64,
        back: bool,
    ) -> Result<Option<DocBlobRef>> {
        let conn = self.conn();
        let blob = match kind {
            DocKind::Document => docs::Entity::find_by_id(doc_id)
                .one(conn)
                .await?
                .map(|d| DocBlobRef {
                    driver_id: d.driver_id,
                    file_id: d.file_id,
                    nonce: d.nonce,
                    encrypted_key: d.encrypted_key,
                }),
            DocKind::Identity => {
                driver_identity_documents::Entity::find_by_id(doc_id)
                    .one(conn)
                    .await?
                    .map(|d| {
                        if back {
                            DocBlobRef {
                                driver_id: d.driver_id,
                                file_id: d.file_id_back,
                                nonce: d.back_nonce,
                                encrypted_key: d.back_encrypted_key,
                            }
                        } else {
                            DocBlobRef {
                                driver_id: d.driver_id,
                                file_id: d.file_id_front,
                                nonce: d.front_nonce,
                                encrypted_key: d.front_encrypted_key,
                            }
                        }
                    })
            }
        };
        Ok(blob)
    }
}
