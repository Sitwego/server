use std::collections::HashMap;

use db_store::Database;
use redis_store::r_types::AppError;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, entity::prelude::*};
use sea_orm::{
    Condition, FromQueryResult, IntoActiveModel, JoinType, QueryOrder,
    QuerySelect, SelectColumns,
};

use serde::Serialize;
use time::OffsetDateTime;

use tracing::info;
use utils::Result;
use utils::gen_strings::ulid_string;

use crate::schemas::plans::{BillingType, PlanName, VehicleType};
use crate::schemas::subscriptions::{AutoPayStatus, SubscriptionPlan};
use crate::schemas::{ipn, payment_authorizations, plans};
use crate::schemas::{ride, subscriptions};
use crate::types::DriverId;

pub trait SubscriptionsPlans {
    fn create_bs_plan(
        &self,
        sub_plan_data: SubPlanData,
    ) -> impl std::future::Future<Output = Result<String>> + Send;

    fn create_bs_subscription(
        &self,
        subscription_data: SubscriptionsData,
        id: String,
    ) -> impl std::future::Future<Output = Result<subscriptions::Model>> + Send;

    /// Start a driver's free trial as part of admin activation. The driver does
    /// not pick a plan during onboarding, so the admin supplies `plan_id` here;
    /// this creates the subscription on a `trial_days` free trial with the plan
    /// marked active, going through the same [`create_bs_subscription`] upsert
    /// path the public endpoint uses. On re-activation it reuses the driver's
    /// existing subscription id (driver_id is UNIQUE) so the upsert UPDATES it.
    fn start_driver_trial(
        &self,
        driver_id: &str,
        plan_id: &str,
        trial_days: i64,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn get_bs_subscription(
        &self,
        driver_id: DriverId,
    ) -> impl std::future::Future<
        Output = Result<Option<SubscriptionPlan>, AppError>,
    > + Send;

    /// All available billing plans, for the admin to pick from when activating a
    /// driver (the trial is started on the chosen plan).
    fn list_plans(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<plans::Model>>> + Send;

    fn update_due_amount(
        &self,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn reset_subscription(
        &self,
        driver_id: &str,
        sub_id: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Admin: suspend/reactivate a driver's billing by flipping `is_plan_active`
    /// — the single gate the accrual job now respects. Suspending stops further
    /// accrual; reactivating resumes it from the watermark.
    fn set_plan_active(
        &self,
        driver_id: &str,
        active: bool,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Admin: directly set a subscription's `amount_due` (e.g. a manual credit
    /// or correction). Advances `last_accrued_at` to now so the next accrual run
    /// does not re-bill rides already accounted for in this adjustment.
    fn admin_adjust_amount_due(
        &self,
        sub_id: &str,
        amount_due: Decimal,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn confirm_payment(
        &self,
        chekout_req_id: &str,
    ) -> impl std::future::Future<Output = Result<Option<ipn::Model>>> + Send;

    fn get_driver_subscription_status(
        &self,
        driver_id: &str,
    ) -> impl std::future::Future<
        Output = Result<Option<SubscriptionStatus>, AppError>,
    > + Send;

    #[allow(clippy::too_many_arguments)]
    fn create_mpesa_transaction(
        &self,
        checkout_request_id: String,
        merchant_request_id: String,
        amount: Option<Decimal>,
        mpesa_receipt_number: Option<String>,
        transaction_date: i64,
        phone_number: Option<String>,
        result_desc: Option<String>,
        driver_id: String,
        payment_status: String,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}

pub struct SubPlanData {
    pub cost: Decimal,
    pub plans: (VehicleType, PlanName, BillingType),
    pub max_charge: Option<Decimal>,
    pub max_rides: Option<i32>,
}

pub struct SubscriptionsData {
    pub driver_id: DriverId,
    pub plan_id: String,
    pub free_trial_end_date: Option<DateTimeWithTimeZone>,
    pub auto_pay_status: AutoPayStatus,
    pub plan_end_date: Option<DateTimeWithTimeZone>,
    pub plan_start_date: DateTimeWithTimeZone,
    pub is_on_free_trial: bool,
}

#[derive(Debug, FromQueryResult, Serialize, PartialEq)]
pub struct SubscriptionStatus {
    pub is_plan_active: bool,
    pub is_on_free_trial: bool,
    pub free_trial_end_date: Option<DateTimeWithTimeZone>,
    pub plan_end_date: Option<DateTimeWithTimeZone>,
    pub amount_due: Option<Decimal>,
    pub last_billed_at: Option<DateTimeWithTimeZone>,
    pub auto_pay_status: AutoPayStatus,
}

#[derive(Debug, FromQueryResult)]
pub struct SubscriptionResultData {
    pub id: String,
    pub driver_id: String,
    pub plan_id: String,
    pub plan_end_date: Option<DateTimeWithTimeZone>,
    pub plan_name: String,
    pub cost: Decimal,
}
impl SubscriptionsPlans for Database {
    async fn create_bs_subscription(
        &self,
        subscription_data: SubscriptionsData,
        id: String,
    ) -> Result<subscriptions::Model> {
        let subscription = self
            .transaction(move |tx| {
                let id = id.clone();
                let driver_id = subscription_data.driver_id.clone();
                let plan_id = subscription_data.plan_id.clone();
                async move {
                    let payment_authorization =
                        payment_authorizations::ActiveModel {
                            id: ActiveValue::Set(ulid_string()),
                            setup_date: ActiveValue::Set(
                                chrono::Utc::now().into(),
                            ),
                        }
                        .insert(&*tx)
                        .await?;
                    let subscription = subscriptions::Entity::insert(
                        subscriptions::ActiveModel {
                            id: ActiveValue::Set(id),
                            driver_id: ActiveValue::Set(driver_id.0),
                            plan_id: ActiveValue::Set(plan_id),
                            payment_auth_id: ActiveValue::Set(
                                payment_authorization.id,
                            ),
                            payment_auth_setup_date: ActiveValue::Set(Some(
                                payment_authorization.setup_date,
                            )),
                            is_on_free_trial: ActiveValue::Set(
                                subscription_data.is_on_free_trial,
                            ),
                            auto_pay_status: ActiveValue::Set(
                                subscription_data.auto_pay_status,
                            ),
                            free_trial_end_date: ActiveValue::Set(
                                subscription_data.free_trial_end_date,
                            ),
                            plan_end_date: ActiveValue::Set(
                                subscription_data.plan_end_date,
                            ),
                            plan_start_date: ActiveValue::Set(
                                subscription_data.plan_start_date,
                            ),
                            amount_due: ActiveValue::Set(Some(Decimal::ZERO)),
                            is_plan_active: ActiveValue::Set(true),
                            ..Default::default()
                        },
                    )
                    .on_conflict(
                        OnConflict::columns([subscriptions::Column::Id])
                            .update_columns([
                                subscriptions::Column::PlanId,
                                subscriptions::Column::IsOnFreeTrial,
                                subscriptions::Column::AutoPayStatus,
                                subscriptions::Column::PaymentAuthSetupDate,
                                subscriptions::Column::PaymentAuthId,
                                // subscriptions::Column::FreeTrialEndDate,
                                subscriptions::Column::UpdatedAt,
                                subscriptions::Column::PlanEndDate,
                                subscriptions::Column::PlanStartDate,
                            ])
                            .to_owned(),
                    )
                    .exec_with_returning(&*tx)
                    .await?;
                    Ok(subscription)
                }
            })
            .await?;
        Ok(subscription)
    }

    async fn start_driver_trial(
        &self,
        driver_id: &str,
        plan_id: &str,
        trial_days: i64,
    ) -> Result<()> {
        // driver_id is UNIQUE on subscriptions. On a first activation there is no
        // row; on re-activation reuse the existing id so the upsert UPDATES it
        // (a fresh id would collide with the UNIQUE(driver_id) constraint).
        let existing_id = subscriptions::Entity::find()
            .filter(subscriptions::Column::DriverId.eq(driver_id))
            .one(self.conn())
            .await?
            .map(|s| s.id)
            .unwrap_or_else(ulid_string);

        let now = chrono::Utc::now();
        let trial_end = (now + chrono::Duration::days(trial_days)).into();
        // Mirrors the public create-subscriptions-plan?ontrial=true path.
        self.create_bs_subscription(
            SubscriptionsData {
                driver_id: DriverId(driver_id.to_owned()),
                plan_id: plan_id.to_owned(),
                free_trial_end_date: Some(trial_end),
                auto_pay_status: AutoPayStatus::NotSet,
                is_on_free_trial: true,
                // Anchor the billing window to the trial end (14 days). When the
                // trial lapses, renewal logic extends it by a billing cycle.
                plan_end_date: Some(trial_end),
                plan_start_date: now.into(),
            },
            existing_id,
        )
        .await?;
        Ok(())
    }

    async fn list_plans(&self) -> Result<Vec<plans::Model>> {
        let plans = plans::Entity::find()
            .order_by_asc(plans::Column::VehicleType)
            .order_by_asc(plans::Column::Cost)
            .all(self.conn())
            .await?;
        Ok(plans)
    }

    async fn create_bs_plan(
        &self,
        sub_plan_data: SubPlanData,
    ) -> Result<String> {
        let plan_id = self
            .transaction(move |tx| {
                let (vehicle_type, plan_name, billing_type) =
                    sub_plan_data.plans.clone();
                let id = ulid_string();
                async move {
                    let plan_id = plans::Entity::insert(plans::ActiveModel {
                        id: ActiveValue::Set(id),
                        vehicle_type: ActiveValue::Set(vehicle_type),
                        plan_name: ActiveValue::Set(plan_name),
                        cost: ActiveValue::Set(sub_plan_data.cost),
                        billing_type: ActiveValue::Set(billing_type),
                        max_charge: ActiveValue::Set(sub_plan_data.max_charge),
                        max_rides: ActiveValue::Set(sub_plan_data.max_rides),
                        ..Default::default()
                    })
                    .exec_with_returning(&*tx)
                    .await?
                    .id;
                    Ok(plan_id)
                }
            })
            .await?;
        Ok(plan_id)
    }

    async fn get_bs_subscription(
        &self,
        driver_id: DriverId,
    ) -> Result<Option<SubscriptionPlan>, AppError> {
        let subscription = self
            .transaction(move |tx| {
                let driver_id = driver_id.clone();
                async move {
                    let subscription = sea_orm::QuerySelect::join(
                        subscriptions::Entity::find().filter(
                            Condition::all()
                                .add(
                                    subscriptions::Column::DriverId
                                        .eq(driver_id.0),
                                )
                                .add(
                                    Condition::any()
                                        .add(
                                            subscriptions::Column::PlanEndDate
                                                .gt(OffsetDateTime::now_utc()),
                                        )
                                        .add(
                                            subscriptions::Column::PlanEndDate
                                                .is_null(),
                                        ),
                                )
                                .add(
                                    subscriptions::Column::AutoPayStatus.is_in(
                                        vec![
                                            AutoPayStatus::Enabled,
                                            AutoPayStatus::PendingActivation,
                                            AutoPayStatus::NotSet,
                                        ],
                                    ),
                                ),
                        ),
                        JoinType::InnerJoin,
                        subscriptions::Relation::Plans.def(),
                    )
                    .select_only()
                    .column_as(subscriptions::Column::Id, "sub_id")
                    .column_as(subscriptions::Column::DriverId, "driver_id")
                    .column_as(subscriptions::Column::PlanId, "plan_id")
                    .column_as(
                        subscriptions::Column::AutoPayStatus,
                        "auto_pay_status",
                    )
                    .column_as(plans::Column::VehicleType, "plan_vehicle_type")
                    .column_as(plans::Column::PlanName, "plan_name")
                    .column_as(plans::Column::Cost, "plan_cost")
                    .column_as(plans::Column::BillingType, "billing_type")
                    .column_as(
                        subscriptions::Column::PaymentAuthSetupDate,
                        "payment_auth_setup_date",
                    )
                    .column_as(
                        subscriptions::Column::PlanEndDate,
                        "plan_end_date",
                    )
                    .order_by_desc(subscriptions::Column::CreatedAt)
                    .into_model::<SubscriptionPlan>()
                    .one(&*tx)
                    .await?;

                    Ok(subscription)
                }
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;
        Ok(subscription)
    }

    async fn reset_subscription(&self, _: &str, sub_id: &str) -> Result<()> {
        self.transaction(move |tx| {
            Box::pin(async move {
                // first check if subscription exist
                let sub =
                    subscriptions::Entity::find_by_id(sub_id).one(&*tx).await?;
                if let Some(subscription) = sub {
                    let mut active_sub_model = subscription.into_active_model();
                    let now = chrono::Utc::now().into();
                    active_sub_model.amount_due =
                        ActiveValue::Set(Some(Decimal::ZERO));
                    active_sub_model.updated_at = ActiveValue::Set(now);
                    active_sub_model.plan_end_date =
                        ActiveValue::Set(Some(now + chrono::Duration::days(7)));
                    active_sub_model.plan_start_date = ActiveValue::Set(now);
                    active_sub_model.last_billed_at =
                        ActiveValue::Set(Some(now));
                    // Move the accrual watermark to now: rides up to this point
                    // were already billed and are now paid, so the next accrual
                    // run must start fresh from here and not re-charge them.
                    active_sub_model.last_accrued_at =
                        ActiveValue::Set(Some(now));
                    active_sub_model.update(&*tx).await?;
                }
                Ok(())
            })
        })
        .await?;
        Ok(())
    }

    async fn set_plan_active(
        &self,
        driver_id: &str,
        active: bool,
    ) -> Result<()> {
        self.transaction(move |tx| {
            Box::pin(async move {
                let subs = subscriptions::Entity::find()
                    .filter(subscriptions::Column::DriverId.eq(driver_id))
                    .all(&*tx)
                    .await?;
                let now = chrono::Utc::now().into();
                for sub in subs {
                    let mut active_model = sub.into_active_model();
                    active_model.is_plan_active = ActiveValue::Set(active);
                    active_model.updated_at = ActiveValue::Set(now);
                    active_model.update(&*tx).await?;
                }
                Ok(())
            })
        })
        .await
    }

    async fn admin_adjust_amount_due(
        &self,
        sub_id: &str,
        amount_due: Decimal,
    ) -> Result<()> {
        self.transaction(move |tx| {
            Box::pin(async move {
                if let Some(sub) =
                    subscriptions::Entity::find_by_id(sub_id).one(&*tx).await?
                {
                    let now = chrono::Utc::now().into();
                    let mut active_model = sub.into_active_model();
                    active_model.amount_due =
                        ActiveValue::Set(Some(amount_due));
                    active_model.last_accrued_at = ActiveValue::Set(Some(now));
                    active_model.updated_at = ActiveValue::Set(now);
                    active_model.update(&*tx).await?;
                }
                Ok(())
            })
        })
        .await
    }

    async fn update_due_amount(&self) -> Result<()> {
        self.transaction(move |tx| {
            Box::pin(async move {
                // This job normally runs once a day (midnight, Africa/Nairobi)
                // and accrues the cost of each driver's rides on top of the
                // existing amount_due. It does NOT advance last_billed_at — that
                // only moves when the driver actually pays.
                //
                // The billing window for each subscription is anchored to its own
                // `last_accrued_at` watermark (the instant up to which charges have
                // been accrued), NOT to a fixed `now - 24h`. This makes the job
                // self-healing: if a run is missed (server restart / cron downtime),
                // the next run's window still stretches back to the last successful
                // billing, so no ride-day is ever lost or double-counted regardless
                // of when or in what order the job actually fires.
                //
                // `last_accrued_at` is advanced ONLY here and by payment-reset, so —
                // unlike `updated_at` — it can't drift when unrelated subscription
                // fields change. Existing rows were backfilled to deploy time by
                // the migration; the `unwrap_or(updated_at)` below is just a safety
                // net should the column ever be NULL (e.g. a freshly inserted sub).
                // Gate accrual on `is_plan_active` alone. We deliberately do NOT
                // gate on plan_end_date: that field is only pushed forward on
                // payment, so using it here would stop accruing for a driver who
                // keeps driving but hasn't paid yet — they'd ride free once the
                // window lapsed. Not paying must grow the balance, not freeze it.
                // Stopping a driver's billing is an explicit is_plan_active = false
                // (suspension / cancellation), not a date silently sliding past.
                let subs = subscriptions::Entity::find()
                    .find_also_related(plans::Entity)
                    .filter(subscriptions::Column::IsPlanActive.eq(true))
                    .all(&*tx)
                    .await?;

                if subs.is_empty() {
                    info!("No active subscriptions to process");
                    return Ok(());
                }

                let sub_len = subs.len();

                let driver_ids = subs
                    .iter()
                    .map(|(sub, _)| sub.driver_id.clone())
                    .collect::<Vec<String>>();

                // Capture now once and reuse throughout to avoid timestamp drift.
                let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

                // Lower bound for the ride query: the oldest per-sub watermark.
                // Each subscription is then filtered to rides after its own
                // watermark below, so this only bounds how far back we fetch.
                let oldest_watermark = subs
                    .iter()
                    .map(|(sub, _)| sub.last_accrued_at.unwrap_or(sub.updated_at))
                    .min()
                    .unwrap_or(now);

                info!("Fetching rides for {} drivers", driver_ids.len());

                let rides = ride::Entity::find()
                    .filter(
                        Condition::all()
                            .add(ride::Column::DriverId.is_in(driver_ids))
                            .add(ride::Column::TripEndTime.gt(oldest_watermark))
                            .add(ride::Column::TripEndTime.lte(now))
                            .add(ride::Column::Status.eq("Completed"))
                            .add(ride::Column::IsRideDuringFreeTrial.eq(false))
                    )
                    .all(&*tx)
                    .await?;

                info!("Fetched {} rides since {}", rides.len(), oldest_watermark);

                // Collect each driver's completed-ride end times so we can window
                // them per-subscription against that sub's own watermark.
                let mut ride_times_by_driver: HashMap<String, Vec<chrono::DateTime<chrono::FixedOffset>>> =
                    HashMap::new();
                for ride in rides {
                    if let Some(trip_end_time) = ride.trip_end_time {
                        ride_times_by_driver
                            .entry(ride.driver_id.clone())
                            .or_default()
                            .push(trip_end_time);
                    }
                }

                let mut updated_count = 0usize;

                for (sub, plan) in subs {
                    // Handle missing plan gracefully instead of panicking.
                    let plan = match plan {
                        Some(p) => p,
                        None => {
                            info!(
                                "Skipping driver {}: no associated plan found",
                                sub.driver_id
                            );
                            continue;
                        }
                    };

                    let driver_id = sub.driver_id.clone();
                    let plan_id = sub.plan_id.clone();

                    // Skip if plan hasn't started yet.
                    if sub.plan_start_date > now {
                        info!(
                            "Skipping driver {}, plan {}: plan_start_date is in the future",
                            driver_id, plan_id
                        );
                        continue;
                    }

                    // Skip if on active free trial.
                    let is_on_free_trial = sub.is_on_free_trial
                        && sub.free_trial_end_date.is_none_or(|end| end >= now);
                    if is_on_free_trial {
                        info!(
                            "Skipping driver {}, plan {}: on free trial",
                            driver_id, plan_id
                        );
                        continue;
                    }

                    // Window this sub's rides to those after its own watermark and
                    // bucket them by UTC date. A multi-day window (e.g. after a
                    // missed run) yields multiple days, each billed in full.
                    let watermark = sub.last_accrued_at.unwrap_or(sub.updated_at);
                    let mut rides_per_day: HashMap<chrono::NaiveDate, i64> =
                        HashMap::new();
                    if let Some(times) = ride_times_by_driver.get(&driver_id) {
                        for t in times {
                            if *t > watermark {
                                *rides_per_day.entry(t.date_naive()).or_default() += 1;
                            }
                        }
                    }

                    // Business rule: no ride, no charge — applies to every plan
                    // type. A driver is only billed for days they actually drove.
                    if rides_per_day.is_empty() {
                        info!("Driver {}: no new rides since last billing, no charge", driver_id);
                        continue;
                    }

                    let cost = plan.cost;
                    let existing_due = sub.amount_due.unwrap_or(Decimal::ZERO);
                    let ride_count: i64 = rides_per_day.values().sum();

                    let charge = match plan.billing_type {
                        // Flat per active day: one day's price for each distinct
                        // day the driver drove, regardless of rides taken that day.
                        BillingType::PerDay => {
                            cost * Decimal::from(rides_per_day.len() as i64)
                        }
                        // Per ride, billed per day: cap rides at max_rides and the
                        // charge at max_charge for each day, then sum across days.
                        BillingType::PerRide => {
                            let max_rides =
                                plan.max_rides.map(|m| m as i64).unwrap_or(i64::MAX);
                            rides_per_day.values().fold(Decimal::ZERO, |acc, &n| {
                                let chargeable = n.min(max_rides);
                                let day_charge = Decimal::from(chargeable) * cost;
                                let day_charge = match plan.max_charge {
                                    Some(cap) => day_charge.min(cap),
                                    None => day_charge,
                                };
                                acc + day_charge
                            })
                        }
                    };

                    let total_due = existing_due + charge;
                    info!(
                        "Driver {}, plan {}: {} rides over {} day(s), charge={}, total_due={}",
                        driver_id, plan.id, ride_count, rides_per_day.len(), charge, total_due
                    );

                    let mut active_model = sub.into_active_model();
                    active_model.amount_due = ActiveValue::Set(Some(total_due));
                    active_model.updated_at = ActiveValue::Set(now);
                    // Advance the watermark so the next run starts after this point.
                    active_model.last_accrued_at = ActiveValue::Set(Some(now));
                    active_model.update(&*tx).await?;
                    updated_count += 1;
                }

                info!("Processed {} subscriptions, updated {}", sub_len, updated_count);

                Ok(())
            })
        })
        .await
    }

    async fn get_driver_subscription_status(
        &self,
        driver_id: &str,
    ) -> Result<Option<SubscriptionStatus>, AppError> {
        let sub_data = subscriptions::Entity::find()
            .select_only()
            .filter(subscriptions::Column::DriverId.eq(driver_id))
            .select_column_as(
                subscriptions::Column::IsPlanActive,
                "is_plan_active",
            )
            .select_column_as(
                subscriptions::Column::IsOnFreeTrial,
                "is_on_free_trial",
            )
            .select_column_as(
                subscriptions::Column::FreeTrialEndDate,
                "free_trial_end_date",
            )
            .select_column_as(
                subscriptions::Column::PlanEndDate,
                "plan_end_date",
            )
            .select_column_as(subscriptions::Column::AmountDue, "amount_due")
            .select_column_as(
                subscriptions::Column::LastBilledAt,
                "last_billed_at",
            )
            .select_column_as(
                subscriptions::Column::AutoPayStatus,
                "auto_pay_status",
            )
            .into_model::<SubscriptionStatus>()
            .one(self.conn())
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;
        Ok(sub_data)
    }

    async fn confirm_payment(
        &self,
        checkout_req_id: &str,
    ) -> Result<Option<ipn::Model>> {
        self.transaction(move |tx| {
            let id = checkout_req_id.to_string();
            Box::pin(async move {
                let payment = ipn::Entity::find()
                    .filter(ipn::Column::CheckoutRequestId.eq(id))
                    .one(&*tx)
                    .await?;
                Ok(payment)
            })
        })
        .await
    }

    async fn create_mpesa_transaction(
        &self,
        checkout_request_id: String,
        merchant_request_id: String,
        amount: Option<Decimal>,
        mpesa_receipt_number: Option<String>,
        transaction_date: i64,
        phone_number: Option<String>,
        result_desc: Option<String>,
        driver_id: String,
        payment_status: String,
    ) -> Result<(), AppError> {
        let active_model = ipn::ActiveModel {
            currency: ActiveValue::Set("KES".to_string()),
            payment_method: ActiveValue::Set("mpesa".to_string()),
            id: ActiveValue::Set(ulid_string()),
            checkout_request_id: ActiveValue::Set(checkout_request_id),
            merchant_request_id: ActiveValue::Set(merchant_request_id),
            amount: ActiveValue::Set(amount),
            mpesa_receipt_number: ActiveValue::Set(mpesa_receipt_number),
            transaction_date: ActiveValue::Set(transaction_date),
            phone_number: ActiveValue::Set(phone_number),
            result_desc: ActiveValue::Set(result_desc),
            driver_id: ActiveValue::Set(driver_id),
            payment_status: ActiveValue::Set(payment_status),
            ..Default::default()
        };
        // Box::pin heap-allocates SeaORM's large generic insert future, breaking
        // the state machine inlining that causes stack overflow on Tokio worker threads.
        Box::pin(active_model.insert(self.conn()))
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;
        Ok(())
    }
}
