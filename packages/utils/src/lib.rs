pub mod collections;
pub mod executor;
pub mod gen_strings;
pub mod hashing_algo;
pub mod http_reqwest;
use axum::{
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
pub type Result<T, E = Error> = std::result::Result<T, E>;
pub enum Error {
    Http(StatusCode, String, HeaderMap),
    Database(sea_orm::error::DbErr),
    Internal(anyhow::Error),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err)
    }
}
impl From<sea_orm::error::DbErr> for Error {
    fn from(err: sea_orm::error::DbErr) -> Self {
        Self::Database(err)
    }
}

impl From<axum::http::Error> for Error {
    fn from(err: axum::http::Error) -> Self {
        Self::Internal(err.into())
    }
}

impl From<axum::Error> for Error {
    fn from(err: axum::Error) -> Self {
        Self::Internal(err.into())
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Http(code, message, headers) => {
                (code, headers, message).into_response()
            }
            Error::Database(db_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", &db_err))
                    .into_response()
            }
            Error::Internal(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", &error))
                    .into_response()
            }
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Http(code, message, _headers) => (code, message).fmt(f),
            Error::Database(error) => error.fmt(f),
            Error::Internal(error) => error.fmt(f),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Http(code, message, _) => {
                write!(f, "{code}: {message}")
            }
            Error::Database(error) => error.fmt(f),
            Error::Internal(error) => error.fmt(f),
        }
    }
}

impl Error {
    pub fn http(code: StatusCode, message: String) -> Self {
        Self::Http(code, message, HeaderMap::default())
    }
}

#[inline(always)]
pub fn meters_to_km(meters: f64) -> f64 {
    meters / 1000.0
}

#[inline(always)]
pub fn convert_seconds(seconds: u64) -> (u64, &'static str) {
    if seconds >= 3600 {
        (seconds / 3600, "hours")
    } else {
        (seconds / 60, "minutes")
    }
}

#[inline(always)]
pub fn seconds_to_minutes(seconds: u64) -> u64 {
    seconds / 60
}

#[inline(always)]
pub fn round_to_nearest_ten<T>(price: T) -> T
where
    T: num_traits::Float,
{
    (price / T::from(10.0).unwrap()).round() * T::from(10.0).unwrap()
}

/// Rounds a floating-point number to the 2nd decimal place
#[inline(always)]
pub fn round_to_2_decimal_places<T>(value: T) -> T
where
    T: num_traits::Float,
{
    (value * T::from(100.0).unwrap()).round() / T::from(100.0).unwrap()
}
