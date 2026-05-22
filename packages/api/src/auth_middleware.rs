use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{HeaderName, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use utils::Error;

use crate::{
    APIContext,
    auth_token::{Claims, ValidationTokenError},
};

pub async fn auth_middleware(
    State(app_state): State<Arc<APIContext>>,
    mut req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(str::trim));

    match token {
        Some(token) => {
            let key = &app_state.config.jwt_secrete_key;
            match Claims::decode_token(token, key) {
                Ok(claims) => {
                    req.extensions_mut().insert(claims.sub);
                    next.run(req).await
                }
                Err(ValidationTokenError::Expired) => Error::Http(
                    StatusCode::UNAUTHORIZED,
                    "unauthorized".to_string(),
                    [(
                        HeaderName::from_static("x-expired-token"),
                        HeaderValue::from_static("true"),
                    )]
                    .into_iter()
                    .collect(),
                )
                .into_response(),
                Err(_err) => Error::http(
                    StatusCode::UNAUTHORIZED,
                    "unauthorized".to_string(),
                )
                .into_response(),
            }
        }
        None => (StatusCode::UNAUTHORIZED, "Missing token").into_response(),
    }
}
