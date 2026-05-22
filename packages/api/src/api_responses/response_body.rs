use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use crate::api_responses::api_error::ApiErrorData;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiResponseBody<T: Serialize + PartialEq> {
    status_code: u16,
    data: T,
}

impl<T: Serialize + PartialEq> ApiResponseBody<T> {
    pub fn new(status_code: StatusCode, data: T) -> Self {
        Self {
            status_code: status_code.as_u16(),
            data,
        }
    }
}
impl<T: Serialize + PartialEq> IntoResponse for ApiResponseBody<T> {
    fn into_response(self) -> axum::response::Response {
        let body = Json(self.data);
        (StatusCode::from_u16(self.status_code).unwrap(), body).into_response()
    }
}

impl ApiResponseBody<ApiErrorData> {
    pub fn error(status_code: StatusCode, message: String) -> Self {
        let error_data = ApiErrorData { message };
        Self {
            status_code: status_code.as_u16(),
            data: error_data,
        }
    }
}
