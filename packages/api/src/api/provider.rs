use std::sync::Arc;

use axum::{Extension, Json, http::StatusCode};
use serde::Serialize;
use utils::Result;

use crate::APIContext;

#[derive(Debug, Serialize)]
pub struct LocationUpdateResponse {
    pub is_success: i32,
}

#[axum_macros::debug_handler]
pub async fn create_driver(
    Extension(_ctx): Extension<Arc<APIContext>>,
) -> Result<Json<LocationUpdateResponse>, StatusCode> {
    Ok(Json(LocationUpdateResponse { is_success: 1 }))
}
