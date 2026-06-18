use sea_orm::entity::prelude::*;
use serde::Serialize;

use super::docs::DocumentReviewStatus;

#[derive(
    Clone, Debug, Default, PartialEq, Eq, DeriveEntityModel, Serialize,
)]
#[sea_orm(table_name = "driver_identity_documents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(column_type = "String(StringLen::N(26))", indexed)]
    pub driver_id: String,
    #[sea_orm(column_type = "Text", indexed)]
    pub id_number: String,
    #[sea_orm(column_type = "Text")]
    pub document_subtype: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub file_id_front: String,
    pub front_nonce: Vec<u8>,
    pub front_encrypted_key: Vec<u8>,
    pub back_nonce: Vec<u8>,
    pub back_encrypted_key: Vec<u8>,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub file_id_back: String,
    pub version: i32,
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
