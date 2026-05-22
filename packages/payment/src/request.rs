use std::str::FromStr;

use utils::http_reqwest::{Client, Error as HttpError};
use utils::http_reqwest::{
    HeaderMap as HeaderMapReqW, HeaderName, HeaderValue,
};

pub struct ReqwestClient {
    client: Client,
}

impl ReqwestClient {
    pub fn new(base_url: &str, api_token: &str) -> Self {
        let vanilla_client = utils::http_reqwest::ReqwClient::new();
        Self {
            client: Client::from(vanilla_client),
        }
    }

    #[cfg(feature = "reqwest-middleware")]
    pub fn new_with_retry() -> Self {
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
        }
    }
}
