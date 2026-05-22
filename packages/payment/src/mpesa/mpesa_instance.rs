use std::time::Duration;

use lazy_static::lazy_static;
use moka::future::Cache;
use redis_store::r_types::AppError;
use secrecy::{ExposeSecret, SecretBox};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_aux::field_attributes::deserialize_number_from_string;
use tokio::sync::Mutex;
use utils::http_reqwest::{Method, ReqwClient};

use crate::{
    MpesaResult,
    mpesa::stk_push::{StkPush, StkPushBuilder},
};

pub struct Request<Body: Serialize + Send> {
    pub method: Method,
    pub path: &'static str,
    pub body: Body,
}

lazy_static! {
    static ref SIMPLI_CACHE: Cache<String, String> = Cache::builder()
        .max_capacity(1)
        .time_to_live(Duration::from_secs(3599))
        .build();
}

const CARGO_PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
const END_POINT: &str = "oauth/v1/generate?grant_type=client_credentials";
#[derive(Debug)]
pub struct MpesaInstance {
    pub consumer_key: SecretBox<String>,
    pub consumer_secret: SecretBox<String>,
    pub initiator_password: Mutex<Option<SecretBox<String>>>,
    pub(crate) http_client: ReqwClient,
    pub(crate) base_url: String,
    pub certificate: String,
}

impl MpesaInstance {
    pub fn new<T: Into<String>>(consumer_key: T, consumer_secret: T) -> Self {
        let key = SecretBox::new(Box::new(consumer_key.into()));
        let secret = SecretBox::new(Box::new(consumer_secret.into()));

        let http_client = ReqwClient::builder()
            .connect_timeout(Duration::from_secs(10))
            .user_agent(format!("{CARGO_PACKAGE_VERSION}"))
            .build()
            .expect("Faild to create http_client");
        Self {
            consumer_key: key,
            consumer_secret: secret,
            initiator_password: Mutex::new(None),
            http_client,
            base_url: std::env::var("MPESA_BASE_URL")
                .unwrap_or_else(|_| "https://sandbox.safaricom.co.ke".to_string()),
            certificate: "".to_string(),
        }
    }

    pub async fn set_initiator_password<S: Into<String>>(&self, pass: S) {
        let initiator_password = SecretBox::new(Box::new(pass.into()));
        *self.initiator_password.lock().await = Some(initiator_password)
    }

    pub async fn get_auth(&self) -> MpesaResult<String> {
        let key = self.consumer_key.expose_secret();
        let token = SIMPLI_CACHE.get(key).await;

        if let Some(token) = token {
            return Ok(token);
        }
        let token = auth(&self).await?; // this returns token successfuly

        println!(" saving token {:?}", key);
        SIMPLI_CACHE.insert(key.to_owned(), token.clone()).await;
        println!("saved token {} for key {}", token, key);
        Ok(token)
    }

    pub async fn send<Req, Res>(&self, req: Request<Req>) -> MpesaResult<Res>
    where
        Req: Serialize + Send,
        Res: DeserializeOwned,
    {
        let url = format!("{}/{}", self.base_url, req.path);

        let response = self
            .http_client
            .request(req.method, url)
            .bearer_auth(self.get_auth().await?)
            .json(&req.body)
            .send()
            .await
            .map_err(|err| AppError::InternalError(err.to_string()))?;

        if response.status().is_success() {
            let body = response
                .json()
                .await
                .map_err(|err| AppError::InternalError(err.to_string()))?;
            Ok(body)
        } else {
            Err(AppError::InternalError("Mpesa StkPush Error!".to_string()))
        }
    }

    pub fn stk_push(&self) -> StkPushBuilder {
        StkPush::new(self)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct AuthResponse {
    pub access_token: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub expires_in: u64,
}

pub async fn auth(mpesa_instance: &MpesaInstance) -> MpesaResult<String> {
    let uri = format!("{}/{}", mpesa_instance.base_url, END_POINT);
    let key = mpesa_instance.consumer_key.expose_secret();
    let secret = mpesa_instance.consumer_secret.expose_secret();

    let response = mpesa_instance
        .http_client
        .get(&uri)
        .basic_auth(key, Some(secret))
        .send()
        .await
        .map_err(|err| {
            println!("Error!{:?}", err);
            AppError::InternalError(err.to_string())
        })?;
    if response.status().is_success() {
        let value = response
            .json::<AuthResponse>()
            .await
            .map_err(|err| AppError::InternalError(err.to_string()))?;
        return Ok(value.access_token);
    }
    Err(AppError::InternalError("Mpesa Auth Faild!!".to_string()))
}
