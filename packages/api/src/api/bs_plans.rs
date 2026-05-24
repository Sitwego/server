use std::sync::Arc;

use crate::{
    APIContext, AppError, DriverId, Result,
    queries::bussines::{SubscriptionsData, SubscriptionsPlans},
    schemas::subscriptions::SubscriptionPlan,
};
use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{StatusCode, header::HeaderMap},
};
use serde::{Deserialize, Serialize};
use utils::gen_strings::ulid_string;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InputBody {}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QueryData {
    sub_id: Option<String>,
    ontrial: bool,
}

pub async fn create_bs_plan(
    _headers: HeaderMap,
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
    Query(q): Query<QueryData>,
    Path(plan_id): Path<String>,
    Json(_input_body): Json<InputBody>,
) -> Result<StatusCode, AppError> {
    // let vehicle_category = headers
    //     .get("vc")
    //     .and_then(|header_value| header_value.to_str().ok())
    //     .and_then(|vt_str| VehicleCategory::from_str(vt_str).ok())
    //     .ok_or(AppError::InternalError(
    //         "vt (VehicleCategory - Header) not found".to_string(),
    //     ))?;

    let id = if let Some(id) = q.sub_id {
        id
    } else {
        ulid_string()
    };

    let _ = ctx
        .db
        .create_bs_subscription(
            SubscriptionsData {
                driver_id: DriverId(driver_id.to_owned()),
                plan_id,
                free_trial_end_date: if q.ontrial {
                    // Some(OffsetDateTime::now_utc() + time::Duration::days(14))
                    Some(
                        (chrono::Utc::now() + chrono::Duration::days(14))
                            .into(),
                    )
                } else {
                    None
                },
                auto_pay_status:
                    crate::schemas::subscriptions::AutoPayStatus::NotSet,
                is_on_free_trial: q.ontrial,
                // set this to seven days from now
                // if on free trial do not set
                plan_end_date: if q.ontrial {
                    None
                } else {
                    Some(
                        (chrono::Utc::now() + chrono::Duration::days(7)).into(),
                    )
                },
                //set plan_start_date to now
                plan_start_date: (chrono::Utc::now()).into(),
            },
            id,
        )
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;

    Ok(StatusCode::OK)
}

pub async fn get_bs_subscription(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
) -> Result<Json<SubscriptionPlan>, AppError> {
    let subscription: Option<SubscriptionPlan> = ctx
        .db
        .get_bs_subscription(DriverId(driver_id.to_owned()))
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;

    if let Some(sub) = subscription {
        Ok(Json(sub))
    } else {
        Err(AppError::NotFound("Subscription not found".to_string()))
    }
}

pub async fn test_get_bs_plans(
    Extension(ctx): Extension<Arc<APIContext>>,
) -> Result<StatusCode, AppError> {
    ctx.db
        .update_due_amount()
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;
    Ok(StatusCode::OK)
}
