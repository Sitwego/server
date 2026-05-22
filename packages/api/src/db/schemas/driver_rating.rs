use sea_orm::entity::Set;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Customer rates the driver after a ride.
#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "driver_rating")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "String(StringLen::N(26))"
    )]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub ride_id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub driver_id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub customer_id: String,
    pub rating_value: i32,
    #[sea_orm(nullable)]
    pub punctuality: Option<i32>,
    #[sea_orm(nullable)]
    pub driving_behavior: Option<i32>,
    #[sea_orm(nullable)]
    pub safety_compliance: Option<i32>,
    #[sea_orm(nullable)]
    pub vehicle_cleanliness: Option<i32>,
    #[sea_orm(column_type = "Text", nullable)]
    pub feedback_details: Option<String>,
    #[sea_orm(nullable)]
    pub was_offered_assistance: Option<bool>,
    #[sea_orm(column_type = "String(StringLen::N(26))", nullable)]
    pub attachment_id: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
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
        belongs_to = "super::customer::Entity",
        from = "Column::CustomerId",
        to = "super::customer::Column::Id"
    )]
    Customer,
    #[sea_orm(
        belongs_to = "super::ride::Entity",
        from = "Column::RideId",
        to = "super::ride::Column::Id"
    )]
    Ride,
}

impl Related<super::driver::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Driver.def()
    }
}

impl Related<super::customer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl Related<super::ride::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ride.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            created_at: Set(OffsetDateTime::now_utc()),
            updated_at: Set(OffsetDateTime::now_utc()),
            ..ActiveModelTrait::default()
        }
    }
}
