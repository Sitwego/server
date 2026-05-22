use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveValue,
    entity::{Set, prelude::*},
};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use time::OffsetDateTime;

// Enum definition
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
    Default,
    DeriveActiveEnum,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "ride_request_status"
)]
pub enum RideRequestStatus {
    #[sea_orm(string_value = "New")]
    #[default]
    New,
    #[sea_orm(string_value = "Accepted")]
    Accepted,
    #[sea_orm(string_value = "Inprogress")]
    Inprogress,
    #[sea_orm(string_value = "Completed")]
    Completed,
    #[sea_orm(string_value = "Canceled")]
    Canceled,
    #[sea_orm(string_value = "Expired")]
    Expired,
    #[sea_orm(string_value = "Arrived")]
    Arrived,
    #[sea_orm(string_value = "Waitingforrider")]
    Waitingforrider,
    #[sea_orm(string_value = "Failed")]
    Failed,
}
// Entity definition
#[derive(
    Clone, Debug, Default, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "ride_requests")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub driver_id: String,
    pub customer_id: String,
    #[sea_orm(column_type = "Double")]
    pub fare: Decimal,
    #[sea_orm(default_value = "New")]
    pub request_status: RideRequestStatus,
    pub estimated_distance_to_pickup: Option<f64>,
    pub estimated_duration_to_pickup: Option<i32>,
    pub estimated_distance: Option<f64>,
    pub estimated_duration: Option<i32>,
    pub search_request_valid_till: Option<DateTime<Utc>>,
    pub start_time: Option<OffsetDateTime>,
    pub from_location_id: String,
    pub to_location_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(6))")]
    pub otp: Option<String>,
    #[sea_orm(default_value = "false")]
    pub otp_verified: bool,
    #[sea_orm(column_type = "String(StringLen::N(6))")]
    pub end_otp: Option<String>,
    #[sea_orm(default_value = "false")]
    pub end_otp_verified: bool,
}

// Relations
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
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

impl Related<super::location::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FromLocation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: ActiveValue::NotSet,
            driver_id: ActiveValue::NotSet,
            customer_id: ActiveValue::NotSet,
            fare: ActiveValue::NotSet,
            request_status: ActiveValue::NotSet,
            estimated_distance_to_pickup: ActiveValue::NotSet,
            estimated_duration_to_pickup: ActiveValue::NotSet,
            estimated_distance: ActiveValue::NotSet,
            estimated_duration: ActiveValue::NotSet,
            search_request_valid_till: ActiveValue::NotSet,
            start_time: ActiveValue::NotSet,
            from_location_id: ActiveValue::NotSet,
            to_location_id: ActiveValue::NotSet,
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            message: ActiveValue::NotSet,
            otp: ActiveValue::NotSet,
            otp_verified: ActiveValue::NotSet,
            end_otp: ActiveValue::NotSet,
            end_otp_verified: ActiveValue::NotSet,
        }
    }
}
