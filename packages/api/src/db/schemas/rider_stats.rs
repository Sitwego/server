use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "rider_stats")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub customer_id: String,
    pub total_rides: i32,
    pub total_spent: f64,
    pub total_distance: f64,
    pub rides_cancelled: i32,
    pub total_coins_earned: i32,
    pub total_coins_spent: i32,
    pub rating: Decimal,
    pub total_ratings: i32,
    pub total_rating_score: f64,
    #[sea_orm(nullable)]
    pub is_valid_rating: Option<bool>,
    pub fav_driver_count: i32,
    pub total_referral_counts: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::customer::Entity",
        from = "Column::CustomerId",
        to = "super::customer::Column::Id"
    )]
    Customer,
}

impl Related<super::customer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
