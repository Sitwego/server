use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "plans")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(StringLen::N(26))")]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(20))")]
    pub vehicle_type: VehicleType,
    #[sea_orm(column_type = "String(StringLen::N(50))")]
    pub plan_name: PlanName,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    pub cost: Decimal,
    #[sea_orm(column_type = "String(StringLen::N(10))")]
    pub billing_type: BillingType,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))", nullable)]
    pub max_charge: Option<Decimal>,
    #[sea_orm(nullable)]
    pub max_rides: Option<i32>,
    #[sea_orm(default_value = "false")]
    pub no_ride_no_charge: bool,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: OffsetDateTime,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: OffsetDateTime,
}

#[derive(
    Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
pub enum VehicleType {
    #[sea_orm(string_value = "TukTuk")]
    AutoRickshaw,
    #[sea_orm(string_value = "Taxi")]
    Taxi,
    #[sea_orm(string_value = "Bike")]
    Bike,
}

#[derive(
    Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(50))")]
pub enum PlanName {
    #[sea_orm(string_value = "Daily Unlimited")]
    DailyUnlimited,
    #[sea_orm(string_value = "Daily Per Ride")]
    DailyPerRide,
    #[sea_orm(string_value = "Free Trial")]
    FreeTrial,
}

#[derive(
    Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(10))")]
pub enum BillingType {
    #[sea_orm(string_value = "Per Day")]
    PerDay,
    #[sea_orm(string_value = "Per Ride")]
    PerRide,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::subscriptions::Entity")]
    Subscriptions,
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            created_at: sea_orm::Set(OffsetDateTime::now_utc()),
            updated_at: sea_orm::Set(OffsetDateTime::now_utc()),
            ..ActiveModelTrait::default()
        }
    }
}
