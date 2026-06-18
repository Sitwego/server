//! `referral_rewards` — immutable reward ledger (insert-only).
//!
//! One row per reward issued to a referrer, written in the same transaction
//! that flips the [`super::driver_referrals`] row to `Rewarded`. `referral_id`
//! is UNIQUE, so a referral can be rewarded at most once.

use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "referral_rewards")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "String(StringLen::N(26))"
    )]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))", unique)]
    pub referral_id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub driver_id: String,
    pub reward_type: ReferralRewardType,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    pub reward_value: Decimal,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub issued_at: DateTimeWithTimeZone,
}

/// Kind of reward. Maps the Postgres `referral_reward_type` enum; the
/// `string_value`s MUST match the DB labels exactly. The magnitude lives in
/// `reward_value` (days for `SubscriptionDays`, KES for `CashCredit`, unused
/// for `Badge`).
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
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "referral_reward_type"
)]
pub enum ReferralRewardType {
    #[sea_orm(string_value = "subscription_days")]
    SubscriptionDays,
    // cash_credit is the primary reward type (default).
    #[sea_orm(string_value = "cash_credit")]
    #[default]
    CashCredit,
    #[sea_orm(string_value = "badge")]
    Badge,
}

impl ReferralRewardType {
    /// Parse the `REFERRAL_REWARD_TYPE` env value (the DB labels). Unknown
    /// values fall back to the default (`cash_credit`) rather than erroring —
    /// a config typo must not block driver activation.
    pub fn from_config(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "subscription_days" => Self::SubscriptionDays,
            "badge" => Self::Badge,
            "cash_credit" => Self::CashCredit,
            other => {
                if !other.is_empty() {
                    tracing::warn!(
                        raw = other,
                        "unknown REFERRAL_REWARD_TYPE, using cash_credit"
                    );
                }
                Self::CashCredit
            }
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::driver_referrals::Entity",
        from = "Column::ReferralId",
        to = "super::driver_referrals::Column::Id"
    )]
    Referral,
}

impl Related<super::driver_referrals::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Referral.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            issued_at: Set(chrono::Utc::now().into()),
            ..ActiveModelTrait::default()
        }
    }
}
