//! `driver_referral_codes` — one immutable referral code per driver.
//!
//! Generated right after registration and never changed or reused. The code is
//! shared by the referrer and entered by a new driver at sign-up to create a
//! [`super::driver_referrals`] relationship.

use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "driver_referral_codes")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "String(StringLen::N(26))"
    )]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))", unique)]
    pub driver_id: String,
    #[sea_orm(column_type = "String(StringLen::N(12))", unique)]
    pub code: String,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::driver::Entity",
        from = "Column::DriverId",
        to = "super::driver::Column::Id",
        on_delete = "Cascade"
    )]
    Driver,
}

impl Related<super::driver::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Driver.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            created_at: Set(chrono::Utc::now().into()),
            ..ActiveModelTrait::default()
        }
    }
}
