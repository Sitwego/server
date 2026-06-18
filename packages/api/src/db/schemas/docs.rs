use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    EnumIter,
    EnumString,
    Display,
    DeriveActiveEnum,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "driver_document_type"
)]
#[derive(Default)]
pub enum DriverDocumentType {
    #[sea_orm(string_value = "DRIVING_LICENSE")]
    DrivingLicense,
    #[sea_orm(string_value = "PSV_BADGE")]
    PsvBadge,
    #[sea_orm(string_value = "PSV_INSURANCE")]
    PsvInsurance,
    #[sea_orm(string_value = "CERTIFICATE_OF_GOOD_CONDUCT")]
    CertificateOfGoodConduct,
    #[sea_orm(string_value = "VEHICLE_INSPECTION_STICKER")]
    VehicleInspectionSticker,
    #[sea_orm(string_value = "KRA")]
    Kra,
    #[sea_orm(string_value = "NONE")]
    #[default]
    None,
}

/// Broad vehicle classes that drive which documents a driver must provide and
/// how they are labelled. Derived from the free-form `vehicle.vehicle_type`
/// string captured at onboarding (e.g. "Taxi", "boda", "TukTuk") via
/// [`VehicleClass::from_vehicle_type`]. Bikes and three-wheelers share the same
/// reduced document set (no PSV badge / inspection sticker) but differ only in
/// naming, so most logic treats them together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleClass {
    /// Four-wheeled taxi — the full PSV document set applies.
    Car,
    /// Motorcycle / boda boda.
    Bike,
    /// Three-wheeled auto-rickshaw / tuk-tuk.
    Auto,
}

impl VehicleClass {
    /// Classify a free-form `vehicle_type` string. The onboarding app stores
    /// this verbatim (no enum), so we match loosely on keywords and fall back to
    /// `Car` for anything unrecognised (the strictest document set).
    pub fn from_vehicle_type(raw: &str) -> Self {
        let s = raw.to_ascii_lowercase();
        if s.contains("tuk")
            || s.contains("auto")
            || s.contains("rickshaw")
            || s.contains("bajaj")
        {
            VehicleClass::Auto
        } else if s.contains("bike") || s.contains("boda") || s.contains("moto")
        {
            VehicleClass::Bike
        } else {
            VehicleClass::Car
        }
    }

    /// Whether this is a two- or three-wheeler (bike or auto), which share the
    /// reduced document requirements.
    fn is_small(self) -> bool {
        matches!(self, VehicleClass::Bike | VehicleClass::Auto)
    }
}

/// How a document type applies to a given vehicle class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocRequirement {
    /// Must be uploaded and approved before the driver can be activated.
    Required,
    /// Collected and reviewable, but never blocks activation (e.g. KRA PIN).
    Optional,
    /// Not applicable to this vehicle class — hidden from the review UI.
    Hidden,
}

impl DriverDocumentType {
    /// Vehicle-class-aware display name shown in the admin review UI. Bikes and
    /// three-wheelers use motorcycle/auto wording for the licence + insurance;
    /// everything else reads as for a taxi.
    pub fn label_for(&self, class: VehicleClass) -> &'static str {
        match self {
            Self::DrivingLicense => match class {
                VehicleClass::Car => "Driving License",
                _ => "Motorcycle Driving Licence (Class A)",
            },
            Self::PsvInsurance => match class {
                VehicleClass::Car => "PSV Insurance",
                _ => "Valid Motorcycle/Auto Insurance",
            },
            Self::PsvBadge => "PSV Badge",
            Self::CertificateOfGoodConduct => "Certificate of Good Conduct",
            Self::VehicleInspectionSticker => "Vehicle Inspection Sticker",
            Self::Kra => "KRA PIN",
            Self::None => "Document",
        }
    }

    /// How this document type applies to a vehicle class — the onboarding matrix:
    ///
    /// | Document            | Car      | Bike / Auto |
    /// |---------------------|----------|-------------|
    /// | Driving Licence     | required | required    |
    /// | Good Conduct        | required | required    |
    /// | PSV Badge           | required | hidden      |
    /// | Insurance           | optional | optional    |
    /// | Inspection Sticker  | optional | hidden      |
    /// | KRA PIN             | optional | optional    |
    ///
    /// Insurance and the inspection sticker are collected and reviewable but not
    /// yet enforced (to be made `Required` later), so they never block the gate.
    pub fn requirement_for(&self, class: VehicleClass) -> DocRequirement {
        match self {
            Self::DrivingLicense | Self::CertificateOfGoodConduct => {
                DocRequirement::Required
            }
            // PSV badge stays a hard requirement for taxis; hidden for two/three
            // wheelers which don't carry one.
            Self::PsvBadge => {
                if class.is_small() {
                    DocRequirement::Hidden
                } else {
                    DocRequirement::Required
                }
            }
            // Inspection sticker only applies to cars, and is optional for now.
            Self::VehicleInspectionSticker => {
                if class.is_small() {
                    DocRequirement::Hidden
                } else {
                    DocRequirement::Optional
                }
            }
            // Insurance + KRA are collected for every class but never block yet.
            Self::PsvInsurance | Self::Kra => DocRequirement::Optional,
            Self::None => DocRequirement::Hidden,
        }
    }
}

/// Admin verification state for an uploaded driver document. A document is
/// `Pending` until an admin approves or rejects it during onboarding review.
#[derive(
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    EnumIter,
    EnumString,
    Display,
    DeriveActiveEnum,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "document_review_status"
)]
// Serialize to the same UPPERCASE tokens as the DB enum (and the `kind` field),
// so the admin BFF/UI see one consistent contract. Without this, serde would
// emit the Rust variant names ("Pending", …) and diverge from the DB values.
#[serde(rename_all = "UPPERCASE")]
#[derive(Default)]
pub enum DocumentReviewStatus {
    #[sea_orm(string_value = "PENDING")]
    #[default]
    Pending,
    #[sea_orm(string_value = "APPROVED")]
    Approved,
    #[sea_orm(string_value = "REJECTED")]
    Rejected,
}
#[derive(
    Clone, Debug, Default, PartialEq, Eq, DeriveEntityModel, Serialize,
)]
#[sea_orm(table_name = "driver_documents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(column_type = "String(StringLen::N(26))", indexed)]
    pub driver_id: String,
    pub document_type: DriverDocumentType,
    pub version: i32,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub file_id: String,
    pub nonce: Vec<u8>,
    pub encrypted_key: Vec<u8>,
    #[sea_orm(column_type = "JsonBinary", default_value = "{}")]
    pub metadata: Json,
    #[sea_orm(default_value = "true")]
    pub is_active: bool,
    /// Admin verification state — defaults to `Pending` on upload.
    pub review_status: DocumentReviewStatus,
    /// Admin id (ULID) that approved/rejected; `None` while pending.
    #[sea_orm(column_type = "String(StringLen::N(26))", nullable)]
    pub reviewed_by: Option<String>,
    #[sea_orm(nullable)]
    pub reviewed_at: Option<DateTimeWithTimeZone>,
    /// Reason shown to the driver when a document is rejected.
    #[sea_orm(column_type = "Text", nullable)]
    pub reject_reason: Option<String>,
    #[sea_orm(
        column_type = "TimestampWithTimeZone",
        default_value = "CURRENT_TIMESTAMP"
    )]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(
        column_type = "TimestampWithTimeZone",
        default_value = "CURRENT_TIMESTAMP",
        on_update = "CURRENT_TIMESTAMP"
    )]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::driver::Entity",
        from = "Column::DriverId",
        to = "super::driver::Column::Id",
        on_delete = "Cascade"
    )]
    Driver,
}

impl Related<super::driver::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Driver.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
