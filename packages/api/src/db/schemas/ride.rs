use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
// use time::OffsetDateTime;

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "ride")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(StringLen::N(26))")]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub customer_id: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub status: String,
    #[sea_orm(column_type = "String(StringLen::N(26))", indexed)]
    pub driver_id: String,
    #[sea_orm(column_type = "String(StringLen::N(4))")]
    pub otp: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub end_otp: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub tracking_url: String,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))", nullable)]
    pub fare: Option<Decimal>,
    #[sea_orm(column_type = "Text", nullable)]
    pub currency: Option<String>,
    #[sea_orm(column_type = "Double", default_value = 0)]
    pub traveled_distance: f64,
    #[sea_orm(column_type = "Integer", nullable)]
    pub chargeable_distance: Option<i32>,
    #[sea_orm(column_type = "Text", nullable)]
    pub distance_unit: Option<String>,
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub driver_arrival_time: Option<DateTimeWithTimeZone>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub trip_start_time: DateTimeWithTimeZone,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub trip_end_time: Option<DateTimeWithTimeZone>,
    #[sea_orm(column_type = "Double", nullable)]
    pub trip_start_lat: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub trip_start_lon: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub trip_end_lat: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub trip_end_lon: Option<f64>,
    #[sea_orm(column_type = "Boolean", nullable)]
    pub pickup_drop_outside_of_threshold: Option<bool>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(column_type = "Boolean", nullable)]
    pub driver_deviated_to_toll_route: Option<bool>,
    #[sea_orm(column_type = "Boolean", nullable)]
    pub driver_deviated_from_route: Option<bool>,
    #[sea_orm(column_type = "Integer", nullable)]
    pub number_of_snap_to_road_calls: Option<i32>,
    #[sea_orm(column_type = "Integer", nullable)]
    pub number_of_osrm_snap_to_road_calls: Option<i32>,
    #[sea_orm(column_type = "Integer", nullable)]
    pub number_of_self_tuned: Option<i32>,
    #[sea_orm(column_type = "Boolean", nullable)]
    pub number_of_deviation: Option<bool>,
    #[sea_orm(column_type = "Integer", nullable)]
    pub ui_distance_calculation_with_accuracy: Option<i32>,
    #[sea_orm(column_type = "Integer", nullable)]
    pub ui_distance_calculation_without_accuracy: Option<i32>,
    #[sea_orm(column_type = "Boolean", nullable)]
    pub is_free_ride: Option<bool>,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))", nullable)]
    pub estimated_toll_charges: Option<Decimal>,
    pub estimated_toll_names: Option<Vec<String>>,
    #[sea_orm(column_type = "Boolean", default_value = false)]
    pub safety_alert_triggered: bool,
    #[sea_orm(column_type = "Boolean", nullable)]
    pub enable_frequent_location_updates: Option<bool>,
    #[sea_orm(column_type = "String(StringLen::N(26))", nullable)]
    pub ride_ended_by: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub trip_category: Option<String>,
    #[sea_orm(column_type = "Boolean", default_value = false)]
    pub online_payment: bool,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))", nullable)]
    pub cancellation_fee_if_cancelled: Option<Decimal>,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))", nullable)]
    pub tip_amount: Option<Decimal>,
    pub ride_tags: Option<Vec<String>>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ride_type: Option<String>,
    #[sea_orm(column_type = "Boolean", default_value = false)]
    pub is_ride_during_free_trial: bool,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub ride_extras: Option<serde_json::Value>,
}

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
        belongs_to = "super::profile::Entity",
        from = "Column::RideEndedBy",
        to = "super::profile::Column::Id"
    )]
    RideEndedBy,
    #[sea_orm(
        has_one = "super::driver_earning::Entity",
        from = "Column::Id",
        to = "super::driver_earning::Column::RideId"
    )]
    DriverEarning,
    #[sea_orm(
        has_many = "super::ride_fare::Entity",
        from = "Column::Id",
        to = "super::ride_fare::Column::RideId"
    )]
    RideFare,
}

impl Related<super::profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl Related<super::driver_earning::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DriverEarning.def()
    }
}

impl Related<super::ride_fare::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RideFare.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
