use std::sync::Arc;

use axum::{Extension, Json, extract::Path, http::StatusCode};

use payment::mpesa::{
    mpesa_instance::MpesaInstance, stk_push::TransactionType,
};
use redis_store::r_types::AppError;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, warn};
use url::Url;
use utils::Result;
use utils::executor::Executor;

use crate::{
    APIContext, api_responses::responces::Response, cache::keys::mpesa_c_key,
    queries::bussines::SubscriptionsPlans, types::DriverId,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct InputData {
    pub phone_number: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct DriverIdentityData {
    driver_id: DriverId,
    subscription_id: String,
}

#[derive(Debug, Deserialize)]
pub struct MpesaInputBody {
    phone_number: String,
    amount: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ChargePhoneNumberRes {
    pub checkout_request_id: String,
    pub customer_message: String,
    pub merchant_request_id: String,
    pub response_code: String,
    pub response_description: String,
}

#[axum_macros::debug_handler]
pub async fn charge_phone_number(
    Extension(driver_id): Extension<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
    Path(sub_id): Path<String>,
    Json(body): Json<MpesaInputBody>,
) -> Result<Response<ChargePhoneNumberRes>, AppError> {
    let consumer_key = std::env::var("MPESA_CONSUMER_KEY")
        .expect("MPESA_CONSUMER_KEY must be set");
    let consumer_secret = std::env::var("MPESA_CONSUMER_SECRET")
        .expect("MPESA_CONSUMER_SECRET must be set");
    let client = MpesaInstance::new(consumer_key, consumer_secret);

    let base_url = std::env::var("MPESA_CALLBACK_BASE_URL")
        .expect("MPESA_CALLBACK_BASE_URL must be set");
    let url = Url::parse(&format!("{}/mpesa/callback", base_url)).unwrap();
    let res = client
        .stk_push()
        .business_short_code(
            &*std::env::var("MPESA_SHORT_CODE")
                .expect("MPESA_SHORT_CODE must be set"),
        )
        .transaction_type(TransactionType::CustomerPayBillOnline)
        .amount(body.amount)
        .party_a(&*body.phone_number)
        .party_b(
            &*std::env::var("MPESA_SHORT_CODE")
                .expect("MPESA_SHORT_CODE must be set"),
        )
        .account_reference("Subscription Payment")
        .phone_number(&*body.phone_number)
        .transaction_desc("Description")
        .call_back_url(url)
        .build()
        .map_err(|err| AppError::InternalError(err.to_string()))?
        .call()
        .await?;

    ctx.redis
        .set_key::<DriverIdentityData>(
            &mpesa_c_key(&res.checkout_request_id, &res.merchant_request_id),
            DriverIdentityData {
                driver_id: DriverId(driver_id),
                subscription_id: sub_id,
            },
            ctx.config.exp_ttl,
        )
        .await
        .map_err(|err| {
            AppError::InternalError(format!(
                "Failed to set mpesa key {:?}",
                err
            ))
        })?;
    Ok(Response::OK(ChargePhoneNumberRes {
        checkout_request_id: res.checkout_request_id,
        customer_message: res.customer_message,
        merchant_request_id: res.merchant_request_id,
        response_code: res.response_code,
        response_description: res.response_description,
    }))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MpesResopnse {
    #[serde(rename = "Body")]
    pub body: Value,
}

pub async fn mpesa_callback_url(
    Extension(ctx): Extension<Arc<APIContext>>,
    Json(data): Json<MpesResopnse>,
) -> Result<StatusCode> {
    if let Some(data) = data.body.get("stkCallback") {
        let merchant_request_id: Option<String> = data
            .get("MerchantRequestID")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        let checkout_request_id: Option<String> = data
            .get("CheckoutRequestID")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        let result_code = data.get("ResultCode").and_then(|v| v.as_i64());
        let result_desc: Option<String> =
            data.get("ResultDesc").and_then(|v| v.as_str()).map(str::to_owned);

        let mut amount: Option<f64> = None;
        let mut mpesa_receipt: Option<String> = None;
        let mut transaction_date: Option<i64> = None;
        let mut phone_number: Option<i64> = None;

        if let Some(meta_data) = data
            .get("CallbackMetadata")
            .and_then(|m| m.get("Item").and_then(|i| i.as_array()))
        {
            for item in meta_data {
                if let (Some(name), value) = (
                    item.get("Name").and_then(|n| n.as_str()),
                    item.get("Value"),
                ) {
                    match name {
                        "Amount" => {
                            amount = value.as_ref().and_then(|v| v.as_f64())
                        }
                        "MpesaReceiptNumber" => {
                            mpesa_receipt = value
                                .as_ref()
                                .and_then(|v| v.as_str())
                                .map(str::to_owned)
                        }
                        "TransactionDate" => {
                            transaction_date =
                                value.as_ref().and_then(|v| v.as_i64())
                        }
                        "PhoneNumber" => {
                            phone_number =
                                value.as_ref().and_then(|v| v.as_i64())
                        }
                        _ => println!("Unknown metadata item: {:?}", item),
                    }
                }
            }
        }

        if result_code == Some(0) {
            let mpesa_key = mpesa_c_key(
                checkout_request_id.as_deref().unwrap(),
                merchant_request_id.as_deref().unwrap(),
            );
            // get driver_id
            // so as to be able to reset Subscriptions
            let driver_identity_data = ctx
                .redis
                .get_key::<DriverIdentityData>(&mpesa_key)
                .await
                .expect("mpesa callback:: failed to get driver id");
            info!(
                tag = "Payment MetaData",
                "Transaction details: Amount={:?}, MpesaReceiptNumber={:?}, TransactionDate={:?}, PhoneNumber={:?}, ResultDescription={:?}",
                &amount,
                &mpesa_receipt,
                &transaction_date,
                &phone_number,
                &result_desc
            );

            if let Some(driver_identity_data) = driver_identity_data {
                // here reset Subscriptions where driver_id?
                info!(
                    "Reseting driver subscription for driver_id {:?}",
                    driver_identity_data
                );
                let _ = ctx
                    .db
                    .reset_subscription(
                        driver_identity_data.driver_id.inner(),
                        &driver_identity_data.subscription_id,
                    )
                    .await;

                let db = ctx.db.clone();
                Executor.spawn_detached_task(async move {
                    warn!("Saving Mpesa transaction to the database");
                    let _ = db
                        .create_mpesa_transaction(
                            checkout_request_id.unwrap(),
                            merchant_request_id.unwrap(),
                            Some(Decimal::new(amount.unwrap() as i64, 0)),
                            mpesa_receipt,
                            transaction_date.unwrap(),
                            Some(phone_number.unwrap().to_string()),
                            result_desc,
                            driver_identity_data.driver_id.0,
                            "success".to_string(),
                        )
                        .await;
                });
            }
        } else {
            let status = mpesa_result_code_to_status(result_code);
            warn!(status);
        }
    }
    Ok(StatusCode::OK)
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PaymentResponse {
    id: Option<String>,
}
pub async fn confirm_payment(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(_driver_id): Extension<String>,
    Path(chekout_req_id): Path<String>,
) -> Result<Response<PaymentResponse>, AppError> {
    let resp = ctx
        .db
        .confirm_payment(&chekout_req_id)
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    if let Some(payment) = resp {
        return Ok(Response::OK(PaymentResponse {
            id: Some(payment.id),
        }));
    } else {
        return Ok(Response::OK(PaymentResponse { id: None }));
    }
}

pub fn mpesa_result_code_to_status(result_code: Option<i64>) -> &'static str {
    match result_code {
        Some(0) => "completed",   // success
        Some(2001) => "failed",   // wrong pin
        Some(1032) => "canceled", // canceled
        Some(1) => "failed",      // insufficient funds
        _ => "failed",            // Default for unknown codes or None
    }
}
