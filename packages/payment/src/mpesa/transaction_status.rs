use crate::{
    MpesaResult,
    mpesa::mpesa_instance::{MpesaInstance, Request},
};
use derive_builder::Builder;
use serde::{Deserialize, Serialize, Serializer};
use url::Url;
use utils::http_reqwest::Method;

const REQUEST_URL: &str = "mpesa/transactionstatus/v1/query";
const COMMAND_ID: &str = "TransactionStatusQuery";

/// Type of organization/party receiving the transaction, sent to M-Pesa as the
/// numeric `IdentifierType` string it expects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierType {
    /// A customer phone number.
    Msisdn,
    /// A Buy Goods till number.
    TillNumber,
    /// An organization short code (paybill / business short code).
    ShortCode,
}

impl IdentifierType {
    pub fn code(&self) -> &'static str {
        match self {
            IdentifierType::Msisdn => "1",
            IdentifierType::TillNumber => "2",
            IdentifierType::ShortCode => "4",
        }
    }
}

impl Serialize for IdentifierType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.code())
    }
}

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct TransactionStatus<'a> {
    #[builder(pattern = "immutable")]
    mpesa_instance: &'a MpesaInstance,
    /// The name of the initiator initiating the request.
    initiator: &'a str,
    /// Unique identifier of the transaction on M-Pesa (e.g. the receipt number).
    transaction_id: &'a str,
    /// Organization/MSISDN receiving the transaction (usually the short code).
    party_a: &'a str,
    #[builder(default = "IdentifierType::ShortCode")]
    identifier_type: IdentifierType,
    /// Endpoint that receives the final transaction status result.
    result_url: Url,
    /// Endpoint that receives the request if it times out on the M-Pesa side.
    queue_time_out_url: Url,
    /// Alternative identifier used when `transaction_id` is not available.
    #[builder(setter(strip_option), default)]
    original_conversation_id: Option<&'a str>,
    #[builder(default = "\"OK\"")]
    remarks: &'a str,
    #[builder(default = "\"OK\"")]
    occasion: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TransactionStatusReq<'a> {
    initiator: &'a str,
    security_credential: String,
    #[serde(rename = "CommandID")]
    command_id: &'static str,
    #[serde(rename = "TransactionID")]
    transaction_id: &'a str,
    #[serde(
        rename = "OriginatorConversationID",
        skip_serializing_if = "Option::is_none"
    )]
    original_conversation_id: Option<&'a str>,
    party_a: &'a str,
    identifier_type: IdentifierType,
    #[serde(rename = "ResultURL")]
    result_url: Url,
    #[serde(rename = "QueueTimeOutURL")]
    queue_time_out_url: Url,
    remarks: &'a str,
    occasion: &'a str,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TransactionStatusResponse {
    #[serde(rename = "OriginatorConversationID")]
    pub originator_conversation_id: String,
    #[serde(rename = "ConversationID")]
    pub conversation_id: String,
    pub response_code: String,
    pub response_description: String,
}

impl<'a> TransactionStatus<'a> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(c: &'a MpesaInstance) -> TransactionStatusBuilder<'a> {
        TransactionStatusBuilder::default().mpesa_instance(c)
    }

    pub async fn call(self) -> MpesaResult<TransactionStatusResponse> {
        let security_credential =
            self.mpesa_instance.security_credential().await?;

        let req = TransactionStatusReq {
            initiator: self.initiator,
            security_credential,
            command_id: COMMAND_ID,
            transaction_id: self.transaction_id,
            original_conversation_id: self.original_conversation_id,
            party_a: self.party_a,
            identifier_type: self.identifier_type,
            result_url: self.result_url,
            queue_time_out_url: self.queue_time_out_url,
            remarks: self.remarks,
            occasion: self.occasion,
        };

        self.mpesa_instance
            .send::<TransactionStatusReq, _>(Request {
                method: Method::POST,
                path: REQUEST_URL,
                body: req,
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_expected_mpesa_fields() {
        let req = TransactionStatusReq {
            initiator: "testapi",
            security_credential: "encrypted".to_string(),
            command_id: COMMAND_ID,
            transaction_id: "OEI2AK4Q16",
            original_conversation_id: None,
            party_a: "600782",
            identifier_type: IdentifierType::ShortCode,
            result_url: Url::parse("https://example.com/result").unwrap(),
            queue_time_out_url: Url::parse("https://example.com/timeout")
                .unwrap(),
            remarks: "OK",
            occasion: "OK",
        };

        let value = serde_json::to_value(&req).unwrap();

        assert_eq!(value["Initiator"], "testapi");
        assert_eq!(value["SecurityCredential"], "encrypted");
        assert_eq!(value["CommandID"], "TransactionStatusQuery");
        assert_eq!(value["TransactionID"], "OEI2AK4Q16");
        assert_eq!(value["PartyA"], "600782");
        assert_eq!(value["IdentifierType"], "4");
        assert_eq!(value["ResultURL"], "https://example.com/result");
        assert_eq!(value["QueueTimeOutURL"], "https://example.com/timeout");
        assert_eq!(value["Remarks"], "OK");
        assert_eq!(value["Occasion"], "OK");
        // Omitted when None.
        assert!(value.get("OriginatorConversationID").is_none());
    }

    #[test]
    fn serializes_original_conversation_id_when_present() {
        let req = TransactionStatusReq {
            initiator: "testapi",
            security_credential: "encrypted".to_string(),
            command_id: COMMAND_ID,
            transaction_id: "OEI2AK4Q16",
            original_conversation_id: Some("AG_20190826_0000777ab7d848b9"),
            party_a: "600782",
            identifier_type: IdentifierType::Msisdn,
            result_url: Url::parse("https://example.com/result").unwrap(),
            queue_time_out_url: Url::parse("https://example.com/timeout")
                .unwrap(),
            remarks: "OK",
            occasion: "OK",
        };

        let value = serde_json::to_value(&req).unwrap();

        assert_eq!(
            value["OriginatorConversationID"],
            "AG_20190826_0000777ab7d848b9"
        );
        assert_eq!(value["IdentifierType"], "1");
    }

    /// Live smoke test against the Safaricom Daraja sandbox. Ignored by default
    /// because it needs real credentials and network access. Run with:
    ///
    /// ```sh
    /// MPESA_CONSUMER_KEY=... MPESA_CONSUMER_SECRET=... \
    /// MPESA_SHORT_CODE=600XXX MPESA_INITIATOR_NAME=testapi \
    /// MPESA_INITIATOR_PASSWORD='Safaricom999!*!' \
    /// MPESA_CERT_PATH=./certs/mpesa_sandbox.cer \
    /// cargo test -p payment -- --ignored --nocapture transaction_status
    /// ```
    ///
    /// `MPESA_TEST_TRANSACTION_ID` overrides the transaction id being queried;
    /// the ACK (`ResponseCode == "0"`) only confirms the request was accepted —
    /// the real status is delivered asynchronously to `ResultURL`.
    #[tokio::test]
    #[ignore = "hits the M-Pesa sandbox; requires live credentials"]
    async fn sandbox_transaction_status_smoke() {
        let consumer_key = std::env::var("MPESA_CONSUMER_KEY")
            .expect("MPESA_CONSUMER_KEY must be set");
        let consumer_secret = std::env::var("MPESA_CONSUMER_SECRET")
            .expect("MPESA_CONSUMER_SECRET must be set");
        let short_code = std::env::var("MPESA_SHORT_CODE")
            .expect("MPESA_SHORT_CODE must be set");
        let initiator = std::env::var("MPESA_INITIATOR_NAME")
            .expect("MPESA_INITIATOR_NAME must be set");
        let transaction_id = std::env::var("MPESA_TEST_TRANSACTION_ID")
            .unwrap_or_else(|_| "OEI2AK4Q16".to_string());

        let client = MpesaInstance::new(consumer_key, consumer_secret);

        // The SecurityCredential must be non-empty base64 (reads
        // MPESA_INITIATOR_PASSWORD + MPESA_CERT_PATH under the hood).
        let credential = client
            .security_credential()
            .await
            .expect("failed to build security credential");
        assert!(!credential.is_empty(), "security credential was empty");

        let result_url =
            Url::parse("https://example.com/mpesa/status/result").unwrap();
        let queue_url =
            Url::parse("https://example.com/mpesa/status/timeout").unwrap();

        let res = client
            .transaction_status()
            .initiator(&*initiator)
            .transaction_id(&*transaction_id)
            .party_a(&*short_code)
            .identifier_type(IdentifierType::ShortCode)
            .result_url(result_url)
            .queue_time_out_url(queue_url)
            .build()
            .expect("failed to build transaction status request")
            .call()
            .await
            .expect("transaction status call failed");

        println!("transaction status ACK: {res:?}");
        assert_eq!(res.response_code, "0", "unexpected ACK: {res:?}");
    }
}
