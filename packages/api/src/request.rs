use std::str::FromStr;

use redis_store::r_types::AppError;
use utils::http_reqwest::{Client, Error as HttpError};
use utils::http_reqwest::{
    HeaderMap as HeaderMapReqW, HeaderName, HeaderValue,
};

pub struct RidesApiClient {
    client: Client,
    base_url: String,
    api_token: Option<String>,
}

impl RidesApiClient {
    // Vanilla client constructor
    pub fn new(base_url: &str, api_token: &str) -> Self {
        let vanilla_client = utils::http_reqwest::ReqwClient::new();
        Self {
            client: Client::from(vanilla_client),
            base_url: base_url.to_string(),
            api_token: Some(api_token.to_string()),
        }
    }

    #[cfg(feature = "reqwest-middleware")]
    pub fn new_with_retry(base_url: &str, api_token: Option<&str>) -> Self {
        use reqwest_retry::{
            Jitter, RetryTransientMiddleware, policies::ExponentialBackoff,
        };
        use utils::http_reqwest::{
            ClientBuilder, MiddlewareClient as ClientWithMiddleware,
        };
        let vanilla_client = utils::http_reqwest::ReqwClient::new();
        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(
                tokio::time::Duration::from_millis(30),
                tokio::time::Duration::from_millis(100),
            )
            .jitter(Jitter::Bounded)
            .build_with_max_retries(3);
        let middleware_client: ClientWithMiddleware =
            ClientBuilder::new(vanilla_client)
                // .with(TracingMiddleware::default())
                .with(RetryTransientMiddleware::new_with_policy(retry_policy))
                .build();

        Self {
            client: Client::from(middleware_client),
            base_url: base_url.to_string(),
            api_token: api_token.map(String::from),
        }
    }

    fn auth_headers(&self) -> Result<HeaderMapReqW, HttpError> {
        let mut headers = HeaderMapReqW::new();
        if let Some(api_token) = &self.api_token {
            headers.insert(
                HeaderName::from_str("Authorization").unwrap(),
                HeaderValue::from_str(&format!("Bearer {}", api_token))
                    .unwrap(),
            );
        }
        headers.insert(
            HeaderName::from_str("Content-Type").unwrap(),
            HeaderValue::from_str("application/json").unwrap(),
        );
        Ok(headers)
    }

    pub async fn get_ride_path_and_distance(
        &self,
        coordinates: &[(f64, f64)],
        end_point: &str,
    ) -> Result<String, AppError> {
        let coords_str = coordinates
            .iter()
            .map(|(lat, lon)| format!("{:.10},{:.10}", lat, lon))
            .collect::<Vec<_>>()
            .join(";");
        let url = format!(
            "{}/route/v1/driving/{}?{}",
            self.base_url, coords_str, end_point
        );

        let response = self
            .client
            .get(url)
            .headers(self.auth_headers().unwrap())
            .send()
            .await
            .map_err(|err| AppError::InternalError(err.to_string()))?;

        if response.status().is_success() {
            let route = response
                .text()
                .await
                .map_err(|err| AppError::InternalError(err.to_string()))?;
            Ok(route)
        } else {
            Err(AppError::InternalError(format!(
                "Failed to get route: {}",
                response.status()
            )))
        }
    }
}
