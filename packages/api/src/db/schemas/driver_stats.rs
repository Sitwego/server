use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "driver_stats")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(StringLen::N(26))")]
    pub driver_id: String,
    pub idle_since: DateTime<Utc>,
    pub total_rides: i32,
    pub total_earnings: f64,
    pub bonus_earned: f64,
    pub late_night_trips: i32,
    pub earnings_missed: f64,
    pub total_distance: f64,
    pub rides_cancelled: Option<i32>,
    pub total_rides_assigned: Option<i32>,
    pub coin_coverted_to_cash_left: Option<Decimal>,
    pub total_coins_converted_cash: Option<Decimal>,
    #[sea_orm(column_type = "String(StringLen::N(20))")]
    pub distance_unit: Option<String>,
    pub rating: Option<Decimal>,
    pub total_ratings: Option<i32>,
    pub total_rating_score: Option<f64>,
    pub is_valid_rating: Option<bool>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub fav_rider_count: i32,
    pub total_payout_earnings: Option<Decimal>,
    pub total_valid_activated_rides: Option<i32>,
    pub total_referral_counts: Option<i32>,
    pub total_payout_amount_paid: Option<Decimal>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::driver::Entity",
        from = "Column::DriverId",
        to = "super::driver::Column::Id"
    )]
    Driver,
}

impl Related<super::driver::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Driver.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
