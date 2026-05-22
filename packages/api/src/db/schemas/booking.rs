use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
// use time::OffsetDateTime;

// Booking table mapped to a SeaORM entity
#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "booking")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(StringLen::N(26))")]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub customer_id: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub payment_method_id: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub payment_link: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub status: BookingStatus,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub driver_id: String,
    #[sea_orm(default_value = "false")]
    pub is_booking_updated: bool,
    pub start_time: DateTimeWithTimeZone,
    #[sea_orm(nullable)]
    pub return_time: Option<DateTimeWithTimeZone>,
    #[sea_orm(nullable)]
    pub round_trip: Option<bool>,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub from_location_id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub to_location_id: String,
    #[sea_orm(column_type = "Decimal(Some((30, 2)))")]
    pub estimated_fare: Decimal,
    #[sea_orm(column_type = "Double", nullable)]
    pub estimated_distance: Option<f64>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub distance_unit: Option<String>,
    #[sea_orm(nullable)]
    pub estimated_duration: Option<i32>,
    #[sea_orm(nullable)]
    pub estimated_static_duration: Option<i32>,
    #[sea_orm(column_type = "Decimal(Some((30, 2)))", nullable)]
    pub discount: Option<Decimal>,
    #[sea_orm(column_type = "Decimal(Some((30, 2)))")]
    pub estimated_total_fare: Decimal,
    pub is_scheduled: bool,
    #[sea_orm(
        column_type = "String(StringLen::N(255))",
        default_value = "ONE_WAY"
    )]
    pub fare_product_type: FareProductType,
    #[sea_orm(column_type = "Double", nullable)]
    pub distance_value: Option<f64>,
    #[sea_orm(column_type = "String(StringLen::N(36))", nullable)]
    pub stop_location_id: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(4))", nullable)]
    pub otp_code: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(column_type = "Text", nullable)]
    pub service_tier_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub service_tier_short_desc: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub payment_status: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub trip_category: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub initiated_by: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub currency: Option<String>,
    #[sea_orm(column_type = "Double", nullable)]
    pub estimated_distance_value: Option<f64>,
    #[sea_orm(nullable)]
    pub estimated_duration_value: Option<i32>,
}

// Enum for `status` field
#[derive(
    Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum BookingStatus {
    #[sea_orm(string_value = "PENDING")]
    Pending,
    #[sea_orm(string_value = "CONFIRMED")]
    Confirmed,
    // Add more variants as needed based on your application's status values
}

// Enum for `fare_product_type` field
#[derive(
    Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum FareProductType {
    #[sea_orm(string_value = "ONE_WAY")]
    OneWay,
    // Add other fare product types if applicable (e.g., "ROUND_TRIP")
}

// Relationships to other tables
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::profile::Entity",
        from = "Column::CustomerId",
        to = "super::profile::Column::Id"
    )]
    Customer,
    #[sea_orm(
        belongs_to = "super::profile::Entity",
        from = "Column::DriverId",
        to = "super::profile::Column::Id"
    )]
    Driver,
    #[sea_orm(
        belongs_to = "super::location::Entity",
        from = "Column::FromLocationId",
        to = "super::location::Column::Id",
        on_delete = "Cascade"
    )]
    FromLocation,
    #[sea_orm(
        belongs_to = "super::location::Entity",
        from = "Column::ToLocationId",
        to = "super::location::Column::Id",
        on_delete = "Cascade"
    )]
    ToLocation,
}

// Implement relationships for querying
impl Related<super::profile::Entity> for Entity {
    fn to() -> RelationDef {
        // Default to Customer relation; use Relation::Driver.def() for driver
        Relation::Customer.def()
    }
}

impl Related<super::location::Entity> for Entity {
    fn to() -> RelationDef {
        // Default to FromLocation; use Relation::ToLocation.def() for to_location
        Relation::FromLocation.def()
    }
}

// impl Related<super::ride::Entity> for Entity {
//     fn to() -> RelationDef {
//         Relation::Rides.def()
//     }
// }

// Enable ActiveModel functionality
impl ActiveModelBehavior for ActiveModel {}
