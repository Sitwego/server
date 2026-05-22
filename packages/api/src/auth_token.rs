use anyhow::Result;
use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::{Duration, OffsetDateTime};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

const TOKEN_LIFETIME: Duration = Duration::seconds(7_776_000);
// const REFRESH_TOKEN_LIFETIME: Duration = Duration::seconds(7_776_000); // 90 days
impl Claims {
    pub fn create_token(key: &str, id: &str) -> Result<String> {
        let now = OffsetDateTime::now_utc();
        let exp = (now + TOKEN_LIFETIME).unix_timestamp() as usize;
        let claims = Self {
            sub: id.to_string(),
            exp,
            iat: now.unix_timestamp() as usize,
        };

        Ok(encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(key.as_ref()),
        )?)
    }

    pub fn decode_token(
        token: &str,
        key: &str,
    ) -> Result<Self, ValidationTokenError> {
        let key = DecodingKey::from_secret(key.as_ref());

        match decode::<Self>(token, &key, &Validation::default()) {
            Ok(token) => Ok(token.claims),
            Err(e) => {
                if e.kind()
                    == &jsonwebtoken::errors::ErrorKind::ExpiredSignature
                {
                    Err(ValidationTokenError::Expired)
                } else {
                    Err(ValidationTokenError::JwtError(e))
                }
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum ValidationTokenError {
    #[error("access token is expired")]
    Expired,
    #[error("access token validation error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
