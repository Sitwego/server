use crate::types::TimeStamp;
use argon2::{
    Argon2, PasswordHasher, PasswordVerifier,
    password_hash::{PasswordHash, SaltString, rand_core::OsRng},
};
use chrono::{DateTime, ParseError, Utc};
use std::error::Error;

pub fn create_bucket_key(bucket_size: i64, TimeStamp(date): TimeStamp) -> i64 {
    date.timestamp() / bucket_size
}

pub fn ttl_to_datetime(ttl: &str) -> Result<DateTime<Utc>, ParseError> {
    DateTime::parse_from_rfc3339(ttl).map(|dt| dt.with_timezone(&Utc))
}

/// Hashes a password using Argon2id with OWASP-recommended parameters
///
/// # Arguments
/// * `password` - The password to hash as a string
///
/// # Returns
/// * `Result<String, Box<dyn Error>>` - The hashed password string on success, or an error
///
/// # Example
/// ```
/// let hashed = hash_password("my_secure_password").unwrap();
/// println!("Hashed password: {}", hashed);
/// ```
pub fn hash_password(password: &str) -> Result<String, Box<dyn Error>> {
    // Generate a random salt using a cryptographically secure RNG
    let salt = SaltString::generate(&mut OsRng);

    // Initialize Argon2 with default parameters
    let argon2 = Argon2::default();

    // Hash the password with OWASP-recommended minimum parameters:
    // - 19 MiB of memory (m_cost: 19456)
    // - 2 iterations (t_cost: 2)
    // - 1 degree of parallelism (p_cost: 1)
    // These are the defaults in Argon2::default() as of latest version

    let password_hash =
        argon2.hash_password(password.as_bytes(), &salt).map_err(|e| {
            Box::<dyn Error>::from(format!("Hashing failed: {}", e))
        })?;

    // Return the PHC string format which includes:
    // - Algorithm identifier
    // - Parameters
    // - Salt
    // - Hash
    Ok(password_hash.to_string())
}

// Optional: Function to verify a password against a hash
pub fn verify_password(
    password: &str,
    hash: &str,
) -> Result<bool, Box<dyn Error>> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| {
        Box::<dyn Error>::from(format!("Invalid hash format: {}", e))
    })?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_success() {
        let password = "TestPassword123!";
        let result = hash_password(password);

        assert!(result.is_ok(), "Hashing should succeed");
        let hash = result.unwrap();
        assert!(!hash.is_empty(), "Hash should not be empty");
        assert!(
            hash.starts_with("$argon2id$"),
            "Hash should use Argon2id identifier"
        );
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "CorrectHorseBatteryStaple";
        let hash = hash_password(password).unwrap();

        let verification = verify_password(password, &hash);
        assert!(verification.is_ok(), "Verification should succeed");
        assert!(verification.unwrap(), "Correct password should verify");
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "CorrectHorseBatteryStaple";
        let wrong_password = "WrongPassword";
        let hash = hash_password(password).unwrap();

        let verification = verify_password(wrong_password, &hash);
        assert!(verification.is_ok(), "Verification should succeed");
        assert!(
            !verification.unwrap(),
            "Incorrect password should not verify"
        );
    }

    #[test]
    fn test_empty_password() {
        let password = "";
        let result = hash_password(password);

        assert!(result.is_ok(), "Hashing empty password should succeed");
        let hash = result.unwrap();
        let verification = verify_password(password, &hash);
        assert!(
            verification.is_ok() && verification.unwrap(),
            "Empty password should verify"
        );
    }

    #[test]
    fn test_invalid_hash_format() {
        let password = "TestPassword123!";
        let invalid_hash = "not_a_valid_hash";

        let verification = verify_password(password, invalid_hash);
        assert!(verification.is_err(), "Invalid hash format should fail");
    }
}
