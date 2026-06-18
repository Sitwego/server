//! `driver_referrals` — one row per referral relationship and its lifecycle.
//!
//! Created `Pending` at the referred driver's registration, advanced to
//! `Completed` when that driver is activated (KYC approved / go-live), then to
//! `Rewarded` once the referrer's reward has been issued. `referred_id` is
//! UNIQUE so a driver can be referred at most once.

use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "driver_referrals")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "String(StringLen::N(26))"
    )]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub referrer_id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))", unique)]
    pub referred_id: String,
    #[sea_orm(column_type = "String(StringLen::N(12))")]
    pub code_used: String,
    pub status: ReferralStatus,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub referred_at: DateTimeWithTimeZone,
    #[sea_orm(nullable)]
    pub completed_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(nullable)]
    pub rewarded_at: Option<DateTimeWithTimeZone>,
}

/// Referral lifecycle. Maps the Postgres `referral_status` enum; the
/// `string_value`s MUST match the DB labels exactly.
#[derive(
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    EnumIter,
    EnumString,
    Display,
    DeriveActiveEnum,
    Default,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "referral_status")]
pub enum ReferralStatus {
    #[sea_orm(string_value = "pending")]
    #[default]
    Pending,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "rewarded")]
    Rewarded,
    #[sea_orm(string_value = "expired")]
    Expired,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::driver::Entity",
        from = "Column::ReferrerId",
        to = "super::driver::Column::Id"
    )]
    Referrer,
}

impl Related<super::referral_rewards::Entity> for Entity {
    fn to() -> RelationDef {
        super::referral_rewards::Relation::Referral.def().rev()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            referred_at: Set(chrono::Utc::now().into()),
            status: Set(ReferralStatus::Pending),
            ..ActiveModelTrait::default()
        }
    }
}
