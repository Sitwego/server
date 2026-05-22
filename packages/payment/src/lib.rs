pub mod mpesa;
pub mod request;

/// `Result` enum type alias
pub type MpesaResult<T> = utils::Result<T, redis_store::r_types::AppError>;
