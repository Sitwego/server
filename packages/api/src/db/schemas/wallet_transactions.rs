//! `wallet_transactions` — immutable, append-only wallet ledger.
//!
//! One signed row per balance movement, carrying the resulting `balance_after`.
//! `(reference, reference_id)` is UNIQUE (where `reference_id` is set), so a
//! given source — e.g. a referral — can credit the wallet at most once.

use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "wallet_transactions")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "String(StringLen::N(26))"
    )]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub wallet_id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub driver_id: String,
    /// Signed: positive = credit (money in), negative = debit (money out).
    #[sea_orm(column_type = "Decimal(Some((12, 2)))")]
    pub amount: Decimal,
    /// Wallet balance immediately after applying this row.
    #[sea_orm(column_type = "Decimal(Some((12, 2)))")]
    pub balance_after: Decimal,
    /// What caused the movement, e.g. `referral_reward`.
    #[sea_orm(column_type = "String(StringLen::N(40))")]
    pub reference: String,
    /// Optional pointer to the source row (e.g. a `driver_referrals` id).
    #[sea_orm(column_type = "String(StringLen::N(26))", nullable)]
    pub reference_id: Option<String>,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::driver_wallets::Entity",
        from = "Column::WalletId",
        to = "super::driver_wallets::Column::Id"
    )]
    Wallet,
}

impl Related<super::driver_wallets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Wallet.def()
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
