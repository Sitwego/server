use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "ride_fare")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(StringLen::N(26))")]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))", indexed)]
    pub ride_id: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub components: serde_json::Value,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))", default_value = "0.00")]
    pub total: Decimal,
    #[sea_orm(column_type = "Text")]
    pub status: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub reason: Option<String>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub recorded_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ride::Entity",
        from = "Column::RideId",
        to = "super::ride::Column::Id",
        on_delete = "Cascade"
    )]
    Ride,
}

impl Related<super::ride::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ride.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            components: Set(serde_json::json!({})),
            recorded_at: Set(chrono::Utc::now().fixed_offset()),
            ..ActiveModelTrait::default()
        }
    }
}
