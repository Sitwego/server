//! Driver referral programme — data access + business logic.
//!
//! Three responsibilities, kept here so they can be reused from the
//! registration path, the admin activation path, and the driver-facing
//! endpoints:
//!   1. Code generation — one immutable `TRN-XXXXXX` code per driver.
//!   2. Registration validation — create the `pending` relationship, rejecting
//!      self / duplicate / invalid-code referrals.
//!   3. Reward issuance — fired when the referred driver is activated; flips the
//!      referral to `rewarded`, writes the immutable ledger row, and applies the
//!      reward, all in ONE transaction so a referral is rewarded at most once.

use db_store::Database;
use nanoid::nanoid;
use redis_store::r_types::AppError;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, entity::prelude::*,
};
use serde::Serialize;
use utils::Result;
use utils::gen_strings::ulid_string;

use crate::schemas::driver_referrals::ReferralStatus;
use crate::schemas::referral_rewards::ReferralRewardType;
use crate::schemas::{
    driver_referral_codes, driver_referrals, profile, referral_rewards,
    subscriptions,
};

/// Build a fresh referral code, e.g. `TRN-A3K9X2`. Uppercase + human-readable.
/// `TRN-` (4) + 6 chars fits the `VARCHAR(12)` column.
pub fn generate_referral_code() -> String {
    let id = nanoid!(6, &nanoid::alphabet::SAFE);
    format!("TRN-{}", id.to_uppercase())
}

/// Aggregate counts for the referrer's dashboard.
#[derive(Debug, Serialize)]
pub struct ReferralStats {
    /// Referrals that have been rewarded.
    pub total_rewarded: i64,
    /// Completed (referred driver activated) but reward not yet issued.
    pub pending_reward: i64,
    /// Still pending — referred driver has not been activated yet.
    pub in_progress: i64,
    /// Sum of `subscription_days` rewards earned (in days).
    pub total_days_earned: i64,
}

/// One row of the referrer's referral history.
#[derive(Debug, Serialize)]
pub struct ReferralHistoryItem {
    pub driver_name: String,
    pub status: ReferralStatus,
    /// Reward magnitude in the unit implied by the type (days for
    /// subscription_days); `0` until the referral is rewarded.
    pub reward: i64,
    pub referred_at: DateTimeWithTimeZone,
}

/// Everything the caller needs to push the "you earned a reward" notification
/// after a successful reward issuance.
#[derive(Debug, Clone)]
pub struct RewardOutcome {
    /// The driver who gets rewarded (Driver A).
    pub referrer_id: String,
    /// Display name of the newly-activated referred driver (Driver B).
    pub referred_name: String,
    pub reward_type: ReferralRewardType,
    pub reward_value: Decimal,
}

pub trait ReferralQueries {
    /// Return the driver's referral code, generating + persisting it on first
    /// call. Idempotent: every later call returns the same code.
    fn get_or_create_referral_code(
        &self,
        driver_id: &str,
    ) -> impl std::future::Future<Output = Result<String>> + Send;

    /// Whether a referral code exists. Used to pre-validate the code a new
    /// driver entered BEFORE their profile is created, so an invalid code can
    /// reject the registration cleanly.
    fn referral_code_exists(
        &self,
        code: &str,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    /// Validate a referral code entered at registration and persist the
    /// `pending` relationship. Rejects self-referral, an already-referred
    /// driver, and unknown codes (all as `ValidationError`).
    fn create_referral(
        &self,
        referred_id: &str,
        code: &str,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;

    /// Reward the referrer (if any) when `referred_id` is activated. No-op when
    /// the driver was not referred or the referral is already rewarded —
    /// returns `Some` only when a reward was freshly issued, so the caller knows
    /// to notify. Transaction-safe and idempotent.
    fn reward_referral_on_activation(
        &self,
        referred_id: &str,
        reward_type: ReferralRewardType,
        reward_value: Decimal,
    ) -> impl std::future::Future<Output = Result<Option<RewardOutcome>>> + Send;

    /// Aggregate stats for the referrer's dashboard.
    fn get_referral_stats(
        &self,
        driver_id: &str,
    ) -> impl std::future::Future<Output = Result<ReferralStats>> + Send;

    /// Paginated referral history (newest first).
    fn get_referral_history(
        &self,
        driver_id: &str,
        limit: u64,
        offset: u64,
    ) -> impl std::future::Future<Output = Result<Vec<ReferralHistoryItem>>> + Send;
}

impl ReferralQueries for Database {
    async fn get_or_create_referral_code(
        &self,
        driver_id: &str,
    ) -> Result<String> {
        let conn = self.conn();

        // One code per driver (UNIQUE driver_id) — return it if it exists.
        if let Some(existing) = driver_referral_codes::Entity::find()
            .filter(driver_referral_codes::Column::DriverId.eq(driver_id))
            .one(conn)
            .await?
        {
            return Ok(existing.code);
        }

        // Generate + insert, retrying on the (astronomically rare) code
        // collision. On any insert error we re-check for a code created by a
        // concurrent request for the same driver before giving up.
        for _ in 0..5 {
            let code = generate_referral_code();
            let model = driver_referral_codes::ActiveModel {
                id: Set(ulid_string()),
                driver_id: Set(driver_id.to_owned()),
                code: Set(code.clone()),
                ..Default::default()
            };
            match model.insert(conn).await {
                Ok(m) => return Ok(m.code),
                Err(_) => {
                    if let Some(existing) =
                        driver_referral_codes::Entity::find()
                            .filter(
                                driver_referral_codes::Column::DriverId
                                    .eq(driver_id),
                            )
                            .one(conn)
                            .await?
                    {
                        return Ok(existing.code);
                    }
                    // else: code collision — loop and regenerate.
                }
            }
        }

        Err(utils::Error::Internal(anyhow::anyhow!(
            "failed to generate a unique referral code"
        )))
    }

    async fn referral_code_exists(&self, code: &str) -> Result<bool> {
        let code = code.trim().to_uppercase();
        let found = driver_referral_codes::Entity::find()
            .filter(driver_referral_codes::Column::Code.eq(code))
            .count(self.conn())
            .await?;
        Ok(found > 0)
    }

    async fn create_referral(
        &self,
        referred_id: &str,
        code: &str,
    ) -> Result<(), AppError> {
        let conn = self.conn();
        let code = code.trim().to_uppercase();

        let referral_code = driver_referral_codes::Entity::find()
            .filter(driver_referral_codes::Column::Code.eq(&code))
            .one(conn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| {
                AppError::ValidationError("Invalid referral code".into())
            })?;

        // Self-referral guard.
        if referral_code.driver_id == referred_id {
            return Err(AppError::ValidationError(
                "You cannot use your own referral code".into(),
            ));
        }

        // Duplicate-referral guard (a friendly error ahead of the
        // UNIQUE(referred_id) constraint that ultimately enforces it).
        let already = driver_referrals::Entity::find()
            .filter(driver_referrals::Column::ReferredId.eq(referred_id))
            .one(conn)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        if already.is_some() {
            return Err(AppError::ValidationError(
                "This driver has already used a referral code".into(),
            ));
        }

        driver_referrals::ActiveModel {
            id: Set(ulid_string()),
            referrer_id: Set(referral_code.driver_id),
            referred_id: Set(referred_id.to_owned()),
            code_used: Set(code),
            status: Set(ReferralStatus::Pending),
            ..Default::default()
        }
        .insert(conn)
        .await
        .map_err(|e| {
            // A race that lost the UNIQUE(referred_id) check above surfaces here.
            AppError::ValidationError(format!(
                "Could not apply referral code: {e}"
            ))
        })?;

        Ok(())
    }

    async fn reward_referral_on_activation(
        &self,
        referred_id: &str,
        reward_type: ReferralRewardType,
        reward_value: Decimal,
    ) -> Result<Option<RewardOutcome>> {
        let referred_id = referred_id.to_owned();

        self.transaction(move |tx| {
            // `Fn` closure (re-run on serialization retry) — clone owned
            // captures per invocation.
            let referred_id = referred_id.clone();
            Box::pin(async move {
                let Some(referral) = driver_referrals::Entity::find()
                    .filter(
                        driver_referrals::Column::ReferredId.eq(&referred_id),
                    )
                    .one(&*tx)
                    .await?
                else {
                    // Driver was not referred — nothing to do.
                    return Ok(None);
                };

                // Idempotent: only Pending/Completed referrals are rewardable.
                // Already-rewarded or expired ones short-circuit (no duplicate
                // ledger row, no duplicate notification).
                if !matches!(
                    referral.status,
                    ReferralStatus::Pending | ReferralStatus::Completed
                ) {
                    return Ok(None);
                }

                let now: DateTimeWithTimeZone = chrono::Utc::now().into();
                let referrer_id = referral.referrer_id.clone();
                let referral_id = referral.id.clone();

                // 1. Immutable ledger row (UNIQUE(referral_id) is the hard
                //    once-only guard).
                referral_rewards::ActiveModel {
                    id: Set(ulid_string()),
                    referral_id: Set(referral.id.clone()),
                    driver_id: Set(referrer_id.clone()),
                    reward_type: Set(reward_type),
                    reward_value: Set(reward_value),
                    ..Default::default()
                }
                .insert(&*tx)
                .await?;

                // 2. Advance the referral lifecycle to rewarded.
                let completed_at = referral.completed_at.or(Some(now));
                let mut rm = referral.into_active_model();
                rm.status = Set(ReferralStatus::Rewarded);
                rm.completed_at = Set(completed_at);
                rm.rewarded_at = Set(Some(now));
                rm.update(&*tx).await?;

                // 3. Apply the reward:
                //    - cash_credit (the primary type) credits the referrer's
                //      wallet in this same transaction.
                //    - subscription_days extends their subscription window.
                //    - badge is recorded in the ledger only (no balance store).
                match reward_type {
                    ReferralRewardType::CashCredit => {
                        crate::queries::wallet::credit_wallet(
                            &*tx,
                            &referrer_id,
                            reward_value,
                            "referral_reward",
                            Some(&referral_id),
                            now,
                        )
                        .await?;
                    }
                    ReferralRewardType::SubscriptionDays => {
                        apply_subscription_days(
                            &*tx,
                            &referrer_id,
                            reward_value,
                            now,
                        )
                        .await?;
                    }
                    ReferralRewardType::Badge => {}
                }

                // Referred driver's name for the notification copy.
                let referred_name =
                    profile::Entity::find_by_id(referred_id.clone())
                        .one(&*tx)
                        .await?
                        .map(|p| {
                            format!("{} {}", p.first_name, p.last_name)
                                .trim()
                                .to_string()
                        })
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| "A driver".to_string());

                Ok(Some(RewardOutcome {
                    referrer_id,
                    referred_name,
                    reward_type,
                    reward_value,
                }))
            })
        })
        .await
    }

    async fn get_referral_stats(
        &self,
        driver_id: &str,
    ) -> Result<ReferralStats> {
        let conn = self.conn();

        let count_status = |status: ReferralStatus| async move {
            driver_referrals::Entity::find()
                .filter(driver_referrals::Column::ReferrerId.eq(driver_id))
                .filter(driver_referrals::Column::Status.eq(status))
                .count(conn)
                .await
        };

        let total_rewarded =
            count_status(ReferralStatus::Rewarded).await? as i64;
        let pending_reward =
            count_status(ReferralStatus::Completed).await? as i64;
        let in_progress = count_status(ReferralStatus::Pending).await? as i64;

        // Sum subscription-day rewards in Rust — a referrer has at most a
        // handful of reward rows, so this avoids SQL aggregate plumbing.
        let rewards = referral_rewards::Entity::find()
            .filter(referral_rewards::Column::DriverId.eq(driver_id))
            .filter(
                referral_rewards::Column::RewardType
                    .eq(ReferralRewardType::SubscriptionDays),
            )
            .all(conn)
            .await?;
        let total_days_earned: i64 = rewards
            .iter()
            .filter_map(|r| i64::try_from(r.reward_value.trunc()).ok())
            .sum();

        Ok(ReferralStats {
            total_rewarded,
            pending_reward,
            in_progress,
            total_days_earned,
        })
    }

    async fn get_referral_history(
        &self,
        driver_id: &str,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<ReferralHistoryItem>> {
        let conn = self.conn();

        let referrals = driver_referrals::Entity::find()
            .filter(driver_referrals::Column::ReferrerId.eq(driver_id))
            .order_by_desc(driver_referrals::Column::ReferredAt)
            .limit(limit)
            .offset(offset)
            .all(conn)
            .await?;

        if referrals.is_empty() {
            return Ok(Vec::new());
        }

        let referred_ids: Vec<String> =
            referrals.iter().map(|r| r.referred_id.clone()).collect();
        let referral_ids: Vec<String> =
            referrals.iter().map(|r| r.id.clone()).collect();

        // Names of the referred drivers (profile.id == driver.id).
        let profiles = profile::Entity::find()
            .filter(profile::Column::Id.is_in(referred_ids))
            .all(conn)
            .await?;
        let name_by_id: std::collections::HashMap<String, String> = profiles
            .into_iter()
            .map(|p| {
                let name = format!("{} {}", p.first_name, p.last_name)
                    .trim()
                    .to_string();
                (p.id, name)
            })
            .collect();

        // Reward magnitude per referral (only present once rewarded).
        let rewards = referral_rewards::Entity::find()
            .filter(referral_rewards::Column::ReferralId.is_in(referral_ids))
            .all(conn)
            .await?;
        let reward_by_referral: std::collections::HashMap<String, i64> =
            rewards
                .into_iter()
                .map(|r| {
                    (
                        r.referral_id,
                        i64::try_from(r.reward_value.trunc()).unwrap_or(0),
                    )
                })
                .collect();

        let items = referrals
            .into_iter()
            .map(|r| ReferralHistoryItem {
                driver_name: name_by_id
                    .get(&r.referred_id)
                    .filter(|s| !s.is_empty())
                    .cloned()
                    .unwrap_or_else(|| "A driver".to_string()),
                status: r.status,
                reward: reward_by_referral.get(&r.id).copied().unwrap_or(0),
                referred_at: r.referred_at,
            })
            .collect();

        Ok(items)
    }
}

/// Extend a referrer's subscription by `reward_value` days. Anchors off the
/// later of the current end date and now, so an already-expired subscription
/// still gets a full grant from today. No-op when the referrer has no
/// subscription row yet (the ledger still records the reward).
async fn apply_subscription_days(
    conn: &impl sea_orm::ConnectionTrait,
    referrer_id: &str,
    reward_value: Decimal,
    now: DateTimeWithTimeZone,
) -> Result<()> {
    let days = i64::try_from(reward_value.trunc()).unwrap_or(0);
    if days <= 0 {
        return Ok(());
    }
    let extend = chrono::Duration::days(days);

    let Some(sub) = subscriptions::Entity::find()
        .filter(subscriptions::Column::DriverId.eq(referrer_id))
        .one(conn)
        .await?
    else {
        return Ok(());
    };

    let new_plan_end = sub.plan_end_date.unwrap_or(now).max(now) + extend;
    let new_trial_end = sub.free_trial_end_date.map(|d| d.max(now) + extend);

    let mut m = sub.into_active_model();
    m.plan_end_date = Set(Some(new_plan_end));
    if let Some(t) = new_trial_end {
        m.free_trial_end_date = Set(Some(t));
    }
    m.updated_at = Set(now);
    m.update(conn).await?;

    Ok(())
}
