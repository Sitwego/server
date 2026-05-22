use sea_orm::entity::prelude::*;
use serde::Serialize;

// use crate::types::VehicleCategory;

#[derive(
    Clone, Debug, Default, PartialEq, Eq, DeriveEntityModel, Serialize,
)]
#[sea_orm(table_name = "vehicle")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(StringLen::N(26))")]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(20))")]
    pub color: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub vehicle_type: String,
    #[sea_orm(column_type = "String(StringLen::N(20))")]
    pub plate_number: String,
    // #[sea_orm(
    //     column_type = "String(StringLen::N(26))",
    //     rs_type = "String",
    //     db_type = "Enum"
    // )]
    // pub category: VehicleCategory,
    #[sea_orm(column_type = "Integer", nullable)]
    pub capacity: Option<i32>,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub model: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub y_manufacturing: Option<i32>,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub make: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub vin: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(26))", indexed)]
    pub driver_id: String,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::driver::Entity",
        from = "Column::DriverId",
        to = "super::driver::Column::Id"
    )]
    Driver,
    #[sea_orm(
        has_many = "super::vehicle_category_mappings::Entity",
        from = "Column::Id",
        to = "super::vehicle_category_mappings::Column::VehicleId"
    )]
    VehicleCategoryMappings,
}
impl Related<super::driver::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Driver.def()
    }
}

impl Related<super::vehicle_category_mappings::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::VehicleCategoryMappings.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, serde::Deserialize)]
pub struct VehicleInfo {
    pub vehicle_type: String,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub vin: Option<String>,
    pub capacity: i32,
    pub license_plate: String,
    pub color: String,
}
