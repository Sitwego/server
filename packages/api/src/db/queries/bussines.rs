use std::collections::HashMap;

use db_store::Database;
use pathfinding::num_traits::ToPrimitive;
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

    fn get_bs_subscription(
        &self,
        driver_id: DriverId,
    ) -> impl std::future::Future<
        Output = Result<Option<SubscriptionPlan>, AppError>,
    > + Send;

    fn update_due_amount(
        &self,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn reset_subscription(
        &self,
        driver_id: &str,
        sub_id: &str,
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
                    active_sub_model.update(&*tx).await?;
                }
                Ok(())
            })
        })
        .await?;
        Ok(())
    }

    async fn update_due_amount(&self) -> Result<()> {
        self.transaction(move |tx| {
            Box::pin(async move {
                // Fetch all active subscriptions with their plans
                let subs = subscriptions::Entity::find()
                    .find_also_related(plans::Entity)
                    .filter(
                        Condition::all()
                            .add(
                                Condition::any()
                                    .add(
                                        subscriptions::Column::PlanEndDate
                                            .gte(OffsetDateTime::now_utc()),
                                    )
                                    .add(
                                        subscriptions::Column::PlanEndDate
                                            .is_null(),
                                    ),
                            )
                            .add(
                                subscriptions::Column::IsPlanActive
                                    .eq(true),
                            ),
                    )
                    .all(&*tx)
                    .await?;

                if subs.is_empty() {
                    info!("No active subscriptions to process");
                    return Ok(());
                }

                let sub_len = subs.len();

                // Fix #8: filter_map → map (always returned Some, never filtered anything)
                let driver_ids = subs
                    .iter()
                    .map(|(sub, _)| sub.driver_id.clone())
                    .collect::<Vec<String>>();

                // Fix #4: capture now once and reuse throughout to avoid timestamp drift
                let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
                // Fix #6: renamed one_day_ago → lookback_start (window is 48h, not 1 day)
                let lookback_start = now - chrono::Duration::hours(48);
                let seven_days_ago = now - chrono::Duration::days(7);

                // Fix #9: removed dead code — driver_ids cannot be empty when subs is non-empty
                info!("Fetching rides for {} drivers", driver_ids.len());

                let rides = ride::Entity::find()
                    .filter(
                        Condition::all()
                            .add(ride::Column::DriverId.is_in(driver_ids))
                            .add(ride::Column::TripEndTime.gte(lookback_start))
                            .add(ride::Column::TripEndTime.lte(now))
                            .add(ride::Column::Status.eq("Completed"))
                            .add(ride::Column::IsRideDuringFreeTrial.eq(false))
                    )
                    .all(&*tx)
                    .await?;

                // Note: no early return on empty rides — PerDay(no_ride_no_charge=false)
                // must still charge active days even when there are no rides.
                info!("Fetched {} rides in the lookback window", rides.len());

                // Group rides by driver_id → UTC date → rides
                // Fix #11: date_naive() instead of naive_local().date() (idiomatic, avoids
                //          ambiguity with server local timezone)
                let mut rides_by_driver_by_date: HashMap<String, HashMap<ChronoDate, Vec<ride::Model>>> =
                    HashMap::new();
                for ride in rides {
                    if let Some(trip_end_time) = ride.trip_end_time {
                        let date: ChronoDate = trip_end_time.date_naive();
                        rides_by_driver_by_date
                            .entry(ride.driver_id.clone())
                            .or_default()
                            .entry(date)
                            .or_default()
                            .push(ride);
                    }
                }

                let mut updated_count = 0usize; // Fix #10: track actually-updated count

                for (sub, plan) in subs {
                    // Fix #5: handle missing plan gracefully instead of panicking with .expect()
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

                    // Skip if updated within the last 24 hours
                    if sub.updated_at >= now - chrono::Duration::hours(24) {
                        info!("Skipping driver {}: updated_at is within 24 hours", driver_id);
                        continue;
                    }

                    // Skip if plan hasn't started yet
                    if sub.plan_start_date > now {
                        info!(
                            "Skipping driver {}, plan {}: plan_start_date is in the future",
                            driver_id, plan_id
                        );
                        continue;
                    }

                    // Skip if on active free trial
                    let is_on_free_trial = sub.is_on_free_trial
                        && sub.free_trial_end_date.map_or(true, |end| end >= now);
                    if is_on_free_trial {
                        info!(
                            "Skipping driver {}, plan {}: on free trial",
                            driver_id, plan_id
                        );
                        continue;
                    }

                    // Accumulate on top of existing unpaid balance
                    let mut total_due = sub.amount_due.unwrap_or(Decimal::ZERO).to_f64().unwrap_or(0.0);

                    // Billing window starts at last_billed_at, defaulting to 7 days ago
                    let last_billed_date = sub.last_billed_at.unwrap_or(seven_days_ago);
                    let last_billed_naive = last_billed_date.date_naive();

                    let driver_rides = rides_by_driver_by_date.get(&driver_id);

                    let total_rides: usize = driver_rides
                        .map(|m| m.values().map(|v| v.len()).sum())
                        .unwrap_or(0);
                    info!(
                        "Driver {}, plan {}: {} rides in the lookback window",
                        driver_id, plan.id, total_rides
                    );

                    let cost = plan.cost.to_f64().unwrap_or(0.0);

                    match plan.billing_type {
                        BillingType::PerDay => {
                            match plan.no_ride_no_charge {
                                true => {
                                    // Only charge for days with rides that are after last_billed_naive
                                    // Fix #2: was counting all days in the map, ignoring last_billed_date
                                    if let Some(rides_by_day) = driver_rides {
                                        let unbilled_days = rides_by_day
                                            .keys()
                                            .filter(|date| **date > last_billed_naive)
                                            .count() as f64;

                                        if unbilled_days > 0.0 {
                                            total_due += unbilled_days * cost;
                                            info!(
                                                "Driver {}, plan {}: {} unbilled ride-days, total_due={}",
                                                driver_id, plan.id, unbilled_days, total_due
                                            );
                                            let mut active_model = sub.into_active_model();
                                            active_model.amount_due = ActiveValue::Set(Some(
                                                <Decimal as num_traits::FromPrimitive>::from_f64(total_due)
                                                    .unwrap_or(Decimal::ZERO),
                                            ));
                                            active_model.updated_at = ActiveValue::Set(now);
                                            active_model.update(&*tx).await?;
                                            updated_count += 1;
                                        } else {
                                            info!(
                                                "Driver {}: no new ride-days to bill (no_ride_no_charge=true)",
                                                driver_id
                                            );
                                        }
                                    } else {
                                        info!(
                                            "Driver {}: no rides in window, skipping (no_ride_no_charge=true)",
                                            driver_id
                                        );
                                    }
                                }
                                false => {
                                    // Fix #3: was just `continue` — charge for every active day
                                    // since last billing, regardless of whether rides occurred
                                    let days_since_billed = (now.date_naive() - last_billed_naive)
                                        .num_days()
                                        .max(0) as f64;

                                    if days_since_billed > 0.0 {
                                        total_due += days_since_billed * cost;
                                        info!(
                                            "Driver {}, plan {}: {} days since last billing, total_due={}",
                                            driver_id, plan.id, days_since_billed, total_due
                                        );
                                        let mut active_model = sub.into_active_model();
                                        active_model.amount_due = ActiveValue::Set(Some(
                                            <Decimal as num_traits::FromPrimitive>::from_f64(total_due)
                                                .unwrap_or(Decimal::ZERO),
                                        ));
                                        active_model.updated_at = ActiveValue::Set(now);
                                        active_model.update(&*tx).await?;
                                        updated_count += 1;
                                    } else {
                                        info!("Driver {}: already billed today, skipping", driver_id);
                                    }
                                }
                            }
                        }
                        BillingType::PerRide => {
                            let max_charge = plan.max_charge
                                .and_then(|v| v.to_f64())
                                .unwrap_or(f64::MAX);

                            if let Some(daily_rides) = driver_rides {
                                for (date, n_ride) in daily_rides {
                                    if n_ride.is_empty() {
                                        continue;
                                    }
                                    // Skip already-billed dates
                                    // Fix #2: was `< last_billed_naive` which would re-bill
                                    //         the last billed day on every subsequent run
                                    if *date <= last_billed_naive {
                                        info!(
                                            "Skipping driver {}, plan {}: date {} already billed (last_billed={})",
                                            driver_id, plan.id, date, last_billed_naive
                                        );
                                        continue;
                                    }
                                    let n_ride_count = n_ride.len() as f64;
                                    let max_rides = plan.max_rides.unwrap_or(i32::MAX) as f64;
                                    let daily_cost = (n_ride_count.min(max_rides) * cost).min(max_charge);
                                    info!(
                                        "Driver {}, plan {}: {} rides on {}, daily_cost={}",
                                        driver_id, plan.id, n_ride_count, date, daily_cost
                                    );
                                    total_due += daily_cost;
                                }

                                info!(
                                    "Driver {}, plan {}: total_due={}",
                                    driver_id, plan.id, total_due
                                );
                                let mut active_model = sub.into_active_model();
                                active_model.amount_due = ActiveValue::Set(Some(
                                    <Decimal as num_traits::FromPrimitive>::from_f64(total_due)
                                        .unwrap_or(Decimal::ZERO),
                                ));
                                active_model.updated_at = ActiveValue::Set(now);
                                active_model.update(&*tx).await?;
                                updated_count += 1;
                            } else {
                                info!("Driver {}: no rides in window for PerRide billing", driver_id);
                            }
                        }
                    }
                }

                // Fix #10: log actual updated count, not total fetched count
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
        Ok(self
            .transaction(move |tx| {
                let id = checkout_req_id.to_string();
                Box::pin(async move {
                    let payment = ipn::Entity::find()
                        .filter(ipn::Column::CheckoutRequestId.eq(id))
                        .one(&*tx)
                        .await?;
                    Ok(payment)
                })
            })
            .await?)
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
