use db_store::Database;
use redis_store::r_types::AppError;
use rust_decimal::Decimal;
use sea_orm::ActiveValue::Set;
use utils::gen_strings::ulid_string;

use crate::schemas::ipn::ActiveModel;

pub trait InstantPaymentNotification {
    type Error: Into<AppError>;

    #[allow(clippy::too_many_arguments)]
    fn create_transaction(
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

impl InstantPaymentNotification for Database {
    type Error = AppError;
    async fn create_transaction(
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
    ) -> Result<(), Self::Error> {
        let _active_model = ActiveModel {
            id: Set(ulid_string()),
            checkout_request_id: Set(checkout_request_id),
            merchant_request_id: Set(merchant_request_id),
            amount: Set(amount),
            mpesa_receipt_number: Set(mpesa_receipt_number),
            transaction_date: Set(transaction_date),
            phone_number: Set(phone_number),
            result_desc: Set(result_desc),
            driver_id: Set(driver_id),
            payment_status: Set(payment_status),
            ..Default::default()
        };
        // Box::pin heap-allocates SeaORM's large generic insert future, breaking
        // the state machine inlining that causes stack overflow on Tokio worker threads.
        // Box::pin(active_model.insert(self.conn()))
        //     .await
        //     .map_err(|err| AppError::DatabaseError(err.to_string()))?;
        Ok(())
    }
}
