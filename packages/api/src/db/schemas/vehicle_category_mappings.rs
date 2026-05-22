use sea_orm::entity::prelude::*;

use crate::types::VehicleCategory;

// Define the Model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "vehicle_category_mappings")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "String(StringLen::N(26))"
    )]
    pub vehicle_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub category: VehicleCategory,
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "String(StringLen::N(26))",
        indexed
    )]
    pub driver_id: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

// Define Relations
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::vehicle::Entity",
        from = "Column::VehicleId",
        to = "super::vehicle::Column::Id",
        on_delete = "Cascade"
    )]
    Vehicle,
    #[sea_orm(
        belongs_to = "super::vehicle_categories::Entity",
        from = "Column::Category",
        to = "super::vehicle_categories::Column::Category",
        on_delete = "Cascade"
    )]
    VehicleCategory,
    #[sea_orm(
        belongs_to = "super::driver::Entity",
        from = "Column::DriverId",
        to = "super::driver::Column::Id",
        on_delete = "Cascade"
    )]
    Driver,
}

// Implement Related for easier querying
impl Related<super::vehicle::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Vehicle.def()
    }
}

impl Related<super::vehicle_categories::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::VehicleCategory.def()
    }
}

impl Related<super::driver::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Driver.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
