//! Driver cash wallet — balance read + transaction-safe crediting.
//!
//! The core primitive is [`credit_wallet`], which runs against an existing
//! connection/transaction so callers (e.g. referral reward issuance) can credit
//! the wallet in the SAME transaction as their own work. It get-or-creates the
//! wallet, moves the balance, and appends one immutable ledger row.

use db_store::Database;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, QuerySelect, entity::prelude::*,
};
use utils::Result;
use utils::gen_strings::ulid_string;

use crate::schemas::{driver_wallets, wallet_transactions};

/// Credit (positive `amount`) or debit (negative) a driver's wallet, writing a
/// ledger row carrying the resulting balance. Runs on the supplied connection
/// so it composes inside a larger transaction. Get-or-creates the wallet.
///
/// `reference` describes the source (e.g. `"referral_reward"`); `reference_id`
/// points at the source row and, when set, is UNIQUE per reference in the DB —
/// so a sourced credit applied twice fails the second time rather than
/// double-crediting.
pub async fn credit_wallet(
    conn: &impl sea_orm::ConnectionTrait,
    driver_id: &str,
    amount: Decimal,
    reference: &str,
    reference_id: Option<&str>,
    now: DateTimeWithTimeZone,
) -> Result<Decimal> {
    // Get-or-create the wallet row.
    let wallet = match driver_wallets::Entity::find()
        .filter(driver_wallets::Column::DriverId.eq(driver_id))
        .one(conn)
        .await?
    {
        Some(w) => w,
        None => {
            driver_wallets::ActiveModel {
                id: Set(ulid_string()),
                driver_id: Set(driver_id.to_owned()),
                balance: Set(Decimal::ZERO),
                currency: Set("KES".to_owned()),
                created_at: Set(now),
                updated_at: Set(now),
            }
            .insert(conn)
            .await?
        }
    };

    let new_balance = wallet.balance + amount;

    // Ledger row first — its UNIQUE(reference, reference_id) is the idempotency
    // guard, so a duplicate sourced credit errors here before the balance moves.
    wallet_transactions::ActiveModel {
        id: Set(ulid_string()),
        wallet_id: Set(wallet.id.clone()),
        driver_id: Set(driver_id.to_owned()),
        amount: Set(amount),
        balance_after: Set(new_balance),
        reference: Set(reference.to_owned()),
        reference_id: Set(reference_id.map(str::to_owned)),
        created_at: Set(now),
    }
    .insert(conn)
    .await?;

    let mut m = wallet.into_active_model();
    m.balance = Set(new_balance);
    m.updated_at = Set(now);
    m.update(conn).await?;

    Ok(new_balance)
}

/// Read side of the wallet, for a driver-facing balance/ledger endpoint.
pub trait WalletQueries {
    /// Current wallet balance (zero when the driver has no wallet yet).
    fn get_wallet_balance(
        &self,
        driver_id: &str,
    ) -> impl std::future::Future<Output = Result<Decimal>> + Send;

    /// Paginated wallet ledger, newest first.
    fn get_wallet_transactions(
        &self,
        driver_id: &str,
        limit: u64,
        offset: u64,
    ) -> impl std::future::Future<
        Output = Result<Vec<wallet_transactions::Model>>,
    > + Send;
}

impl WalletQueries for Database {
    async fn get_wallet_balance(&self, driver_id: &str) -> Result<Decimal> {
        let balance = driver_wallets::Entity::find()
            .filter(driver_wallets::Column::DriverId.eq(driver_id))
            .one(self.conn())
            .await?
            .map(|w| w.balance)
            .unwrap_or(Decimal::ZERO);
        Ok(balance)
    }

    async fn get_wallet_transactions(
        &self,
        driver_id: &str,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<wallet_transactions::Model>> {
        let txns = wallet_transactions::Entity::find()
            .filter(wallet_transactions::Column::DriverId.eq(driver_id))
            .order_by_desc(wallet_transactions::Column::CreatedAt)
            .limit(limit)
            .offset(offset)
            .all(self.conn())
            .await?;
        Ok(txns)
    }
}
