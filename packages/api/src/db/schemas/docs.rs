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
    None,
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
    pub is_active: bool,
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
impl Default for DriverDocumentType {
    fn default() -> Self {
        DriverDocumentType::None
    }
}
impl ActiveModelBehavior for ActiveModel {}
