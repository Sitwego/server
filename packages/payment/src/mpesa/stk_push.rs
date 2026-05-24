use crate::{
    MpesaResult,
    mpesa::mpesa_instance::{MpesaInstance, Request},
};
use chrono::DateTime;
use chrono::prelude::Local;
use derive_builder::Builder;
use openssl::base64;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use url::Url;
use utils::http_reqwest::Method;

fn serialize_utc_to_string<S>(
    date: &DateTime<Local>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = date.format("%Y%m%d%H%M%S").to_string();
    serializer.serialize_str(&s)
}
const REQUEST_URL: &str = "mpesa/stkpush/v1/processrequest";
pub static DEV_PASS_KEY: &str =
    "bfb279f9aa9bdbcf158e97dd71a467cd2e0c893059b10f78e6b72ada1ed2c919";

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct StkPush<'a> {
    #[builder(pattern = "immutable")]
    mpesa_instance: &'a MpesaInstance,
    business_short_code: &'a str,
    #[builder(setter(strip_option), default = "Some(DEV_PASS_KEY)")]
    password: Option<&'a str>,
    transaction_type: TransactionType,
    party_a: &'a str,
    amount: u32,
    party_b: &'a str,
    phone_number: &'a str,
    call_back_url: Url,
    account_reference: &'a str,
    #[builder(setter(strip_option), default)]
    transaction_desc: Option<&'a str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct StkPushReq<'a> {
    business_short_code: &'a str,
    password: String,
    #[serde(serialize_with = "serialize_utc_to_string")]
    timestamp: DateTime<Local>,
    transaction_type: TransactionType,
    pub amount: u32,
    party_a: &'a str,
    party_b: &'a str,
    phone_number: &'a str,
    #[serde(rename = "CallBackURL")]
    call_back_url: Url,
    account_reference: &'a str,
    transaction_desc: Option<&'a str>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct StkPushResponse {
    #[serde(rename = "CheckoutRequestID")]
    pub checkout_request_id: String,
    pub customer_message: String,
    #[serde(rename = "MerchantRequestID")]
    pub merchant_request_id: String,
    pub response_code: String,
    pub response_description: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionType {
    TransactionReversal,
    SalaryPayment,
    BusinessPayment,
    PromotionPayment,
    AccountBalance,
    CustomerPayBillOnline,
    TransactionStatusQuery,
    CheckIdentity,
    BusinessPayBill,
    BusinessBuyGoods,
    DisburseFundsToBusiness,
    BusinessToBusinessTransfer,
    BusinessTransferFromMMFToUtility,
}

impl Display for TransactionType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{self:?}")
    }
}

impl<'a> From<StkPush<'a>> for StkPushReq<'a> {
    fn from(v: StkPush<'a>) -> Self {
        Self {
            business_short_code: v.business_short_code,
            password: StkPush::pass_key(v.business_short_code, v.password),
            timestamp: chrono::Local::now(),
            transaction_type: v.transaction_type,
            amount: v.amount,
            party_a: v.party_a,
            party_b: v.party_b,
            phone_number: v.phone_number,
            call_back_url: v.call_back_url,
            account_reference: v.account_reference,
            transaction_desc: v.transaction_desc,
        }
    }
}

impl StkPushBuilder<'_> {}

impl<'a> StkPush<'a> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(c: &'a MpesaInstance) -> StkPushBuilder<'a> {
        StkPushBuilder::default().mpesa_instance(c)
    }

    pub fn pass_key(
        bs_short_code: &'a str,
        key: Option<&'a str>,
    ) -> std::string::String {
        let time = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
        base64::encode_block(
          format!(
            "{}{}{}",
            bs_short_code,
            key.unwrap_or(DEV_PASS_KEY),
            time,
          )
          .as_bytes()
        )
    }

    pub async fn call(self) -> MpesaResult<StkPushResponse> {
        self.mpesa_instance
            .send::<StkPushReq, _>(Request {
                method: Method::POST,
                path: REQUEST_URL,
                body: self.into(),
            })
            .await
    }
}
