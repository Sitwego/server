use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "driver")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "String(StringLen::N(26))"
    )]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub password: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub phone_hash: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub email_hash: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub photo_id: Option<String>,
    pub is_logged_in: Option<bool>,
    pub activated: Option<bool>,
    pub has_onboarded: Option<bool>,
    pub photo_nonce: Option<Vec<u8>>,
    pub photo_encrypted_key: Option<Vec<u8>>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_one = "super::profile::Entity",
        from = "Column::Id",
        to = "super::profile::Column::Id"
    )]
    Profile,
    #[sea_orm(
        has_many = "super::driver_rating::Entity",
        from = "Column::Id",
        to = "super::driver_rating::Column::DriverId"
    )]
    DriverRating,
    #[sea_orm(
        has_many = "super::customer_rating::Entity",
        from = "Column::Id",
        to = "super::customer_rating::Column::DriverId"
    )]
    CustomerRating,
    #[sea_orm(has_one = "super::docs::Entity")]
    Docs,
    #[sea_orm(has_one = "super::vehicle::Entity")]
    Vehicle,
    #[sea_orm(has_one = "super::driver_stats::Entity")]
    DriverStats,
    #[sea_orm(has_one = "super::subscriptions::Entity")]
    Subscriptions,
    #[sea_orm(
        has_many = "super::vehicle_category_mappings::Entity",
        from = "Column::Id",
        to = "super::vehicle_category_mappings::Column::DriverId"
    )]
    VehicleCategoryMappings,
    #[sea_orm(
        has_many = "super::driver_earning::Entity",
        from = "Column::Id",
        to = "super::driver_earning::Column::DriverId"
    )]
    DriverEarnings,
}

impl Related<super::profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Profile.def()
    }
}
impl Related<super::driver_rating::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DriverRating.def()
    }
}

impl Related<super::customer_rating::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CustomerRating.def()
    }
}
impl Related<super::docs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Docs.def()
    }
}
impl Related<super::vehicle::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Vehicle.def()
    }
}
impl Related<super::driver_stats::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DriverStats.def()
    }
}

impl Related<super::driver_earning::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DriverEarnings.def()
    }
}
impl Related<super::subscriptions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscriptions.def()
    }
}

impl Related<super::vehicle_category_mappings::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::VehicleCategoryMappings.def()
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
