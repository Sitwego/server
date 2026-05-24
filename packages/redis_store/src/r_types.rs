use std::ops::Sub;

use axum::{http::StatusCode, response::IntoResponse};
use dashmap::DashMap;
use fred::{error::Error as FredRedisError, types::geo::GeoValue};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[shared_macro::impl_getter]
#[derive(Deserialize, Serialize, Clone, Debug, Copy)]
pub struct Radius(pub f64);
pub type DriverLocationMap = DashMap<String, Vec<GeoValue>>;
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Copy)]
#[shared_macro::impl_getter]
pub struct Latitude(pub f64);
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Copy)]
#[shared_macro::impl_getter]
pub struct Longitude(pub f64);

#[derive(Debug, Error)]
pub enum RedisError {
    #[error("Redis connection error: {0}")]
    RedisConnectionError(FredRedisError),
    #[error("Redis Error setnx failed: {0}")]
    SetnxFailed(FredRedisError),
    #[error("Redis PubsubPublish error: {0}")]
    RedisPublishError(FredRedisError),
    #[error("Redis Stream error: {0}")]
    RedisStreamError(FredRedisError),
    #[error("Redis GeoAdd error: {0}")]
    RedisGeoAddError(FredRedisError),

    #[error("Redis scriptLoad error: {0}")]
    RedisScriptLoadError(FredRedisError),
    #[error("Redis delete_key failed: {0}")]
    DeleteFailed(FredRedisError),
    #[error("Redis TTl commad failed: {0}")]
    TTLFailed(FredRedisError),
    #[error("Unkown error: {0}")]
    AppError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Redis Set error: {0}")]
    SetError(String),
    #[error("Redis error: {0}")]
    RedisDefaultError(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq)]
pub struct GeoPoint {
    pub lat: Latitude,
    pub lon: Longitude,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Database Error: {0}")]
    DatabaseError(String),
    #[error("Validation Error: {0}")]
    ValidationError(String),
    #[error("Lock contention: resource is busy")]
    LockContention,
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Gone: {0}")]
    Gone(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            AppError::LockContention => (
                StatusCode::CONFLICT,
                "Resource is busy, try again".to_string(),
            ),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Gone(msg) => (StatusCode::GONE, msg),
        };
        (status, error_message).into_response()
    }
}

impl From<AppError> for utils::Error {
    fn from(err: AppError) -> Self {
        utils::Error::Internal(anyhow::anyhow!(err.to_string()))
    }
}

impl From<FredRedisError> for RedisError {
    fn from(err: FredRedisError) -> Self {
        RedisError::RedisConnectionError(err)
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        AppError::InternalError(err.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum Ttl {
    Timetolive(i64),
    NoExpiry,
    NoKeyFound,
}

impl Sub for Latitude {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Latitude(self.0 - other.0)
    }
}

impl Sub for Longitude {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Longitude(self.0 - other.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocationEvent {
    pub entity_id: String,
    pub ride_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: i64,
    pub accuracy: f32,
    pub speed: i32,
    pub bearing: f32,
}
