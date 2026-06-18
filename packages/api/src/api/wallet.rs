//! Driver cash wallet (driver-facing read endpoints).
//!
//! Cash referral rewards land here (see `queries::wallet::credit_wallet`),
//! so the driver needs a way to see the balance and the ledger:
//!   * `GET /driver/wallet`              — current balance.
//!   * `GET /driver/wallet/transactions` — paginated ledger, newest first.

use std::sync::Arc;

use axum::{Extension, Json, extract::Query};
use redis_store::r_types::AppError;
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    APIContext, queries::wallet::WalletQueries, schemas::wallet_transactions,
};

#[derive(Debug, Serialize)]
pub struct WalletResponse {
    pub balance: Decimal,
    pub currency: &'static str,
}

/// `GET /driver/wallet`
///
/// The authenticated driver's wallet balance (zero when no wallet exists yet).
pub async fn get_wallet(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(profile_id): Extension<String>,
) -> Result<Json<WalletResponse>, AppError> {
    let balance = ctx
        .db
        .get_wallet_balance(&profile_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(Json(WalletResponse {
        balance,
        currency: "KES",
    }))
}

#[derive(Debug, Deserialize)]
pub struct LedgerQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

/// `GET /driver/wallet/transactions?limit=&offset=`
///
/// Paginated wallet ledger for the authenticated driver, newest first.
pub async fn get_wallet_transactions(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(profile_id): Extension<String>,
    Query(q): Query<LedgerQuery>,
) -> Result<Json<Vec<wallet_transactions::Model>>, AppError> {
    let limit = q.limit.unwrap_or(20).min(100);
    let offset = q.offset.unwrap_or(0);
    let txns = ctx
        .db
        .get_wallet_transactions(&profile_id, limit, offset)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(Json(txns))
}
