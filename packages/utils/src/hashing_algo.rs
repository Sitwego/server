use aws_config::BehaviorVersion;
use aws_sdk_kms::{Client, primitives::Blob};
use chacha20poly1305::{
    ChaCha20Poly1305, KeyInit,
    aead::{Aead, Payload},
};
use lazy_static::lazy_static;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::error::Error as StdError;
use std::sync::Mutex;

#[derive(Debug)]
pub enum CryptoError {
    KmsError(String),
    CipherError(String),
    SerializationError(serde_json::Error),
    InvalidKeyLength(usize),
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::KmsError(e) => write!(f, "KMS error: {}", e),
            CryptoError::CipherError(e) => write!(f, "Cipher error: {}", e),
            CryptoError::SerializationError(e) => {
                write!(f, "Serialization error: {}", e)
            }
            CryptoError::InvalidKeyLength(len) => {
                write!(f, "Invalid key length: expected 32, got {}", len)
            }
        }
    }
}

impl StdError for CryptoError {}

impl From<serde_json::Error> for CryptoError {
    fn from(err: serde_json::Error) -> Self {
        CryptoError::SerializationError(err)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SensitiveData {
    pub email: String,
    pub phone: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptedRecord {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub key_id: String,
    /// KMS-encrypted copy of the data key. Store alongside ciphertext so the
    /// plaintext key can be recovered after a restart via kms:Decrypt.
    pub encrypted_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DecryptingRecord<'a> {
    pub ciphertext: &'a [u8],
    pub nonce: &'a [u8],
    pub key_id: String,
    pub encrypted_key: &'a [u8],
}

// Cache keyed by SHA-256 of the KMS ciphertext blob so that the same blob
// always maps to the same plaintext key regardless of restarts.
lazy_static! {
    static ref KEY_CACHE: Mutex<HashMap<String, [u8; 32]>> =
        Mutex::new(HashMap::new());
}

const KEY_LENGTH: usize = 32;
const AAD: &[u8] = b"production_data_v1";

fn blob_cache_key(blob: &[u8]) -> String {
    format!("{:x}", Sha256::digest(blob))
}

/// Generate a new data key from KMS. Returns (plaintext_key, encrypted_key_blob).
/// The blob must be stored persistently alongside the ciphertext so the key can
/// be recovered across restarts via `decrypt_envelope_key`.
async fn generate_envelope_key(
    kms_key_id: &str,
) -> Result<([u8; KEY_LENGTH], Vec<u8>), CryptoError> {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = Client::new(&config);

    let response = client
        .generate_data_key()
        .key_id(kms_key_id)
        .key_spec("AES_256".into())
        .send()
        .await
        .map_err(|e| {
            CryptoError::KmsError(format!("Failed to generate data key: {}", e))
        })?;

    let plaintext = response
        .plaintext
        .ok_or_else(|| CryptoError::KmsError("No plaintext key from KMS".into()))?;
    let blob = response
        .ciphertext_blob
        .ok_or_else(|| CryptoError::KmsError("No ciphertext blob from KMS".into()))?;

    let plaintext = plaintext.as_ref();
    if plaintext.len() != KEY_LENGTH {
        return Err(CryptoError::InvalidKeyLength(plaintext.len()));
    }

    let mut key = [0u8; KEY_LENGTH];
    key.copy_from_slice(plaintext);

    let blob_bytes = blob.into_inner();
    {
        let mut cache = KEY_CACHE.lock().map_err(|_| {
            CryptoError::CipherError("Mutex poisoned".to_string())
        })?;
        cache.insert(blob_cache_key(&blob_bytes), key);
    }

    Ok((key, blob_bytes))
}

/// Recover the plaintext data key by calling kms:Decrypt on the stored blob.
/// Uses an in-process cache so repeated decryptions within a process are free.
async fn decrypt_envelope_key(
    encrypted_key: &[u8],
) -> Result<[u8; KEY_LENGTH], CryptoError> {
    let cache_key = blob_cache_key(encrypted_key);

    {
        let cache = KEY_CACHE.lock().map_err(|_| {
            CryptoError::CipherError("Mutex poisoned".to_string())
        })?;
        if let Some(&key) = cache.get(&cache_key) {
            return Ok(key);
        }
    }

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = Client::new(&config);

    let response = client
        .decrypt()
        .ciphertext_blob(Blob::new(encrypted_key.to_vec()))
        .send()
        .await
        .map_err(|e| {
            CryptoError::KmsError(format!("Failed to decrypt data key: {}", e))
        })?;

    let plaintext = response
        .plaintext
        .ok_or_else(|| CryptoError::KmsError("No plaintext from KMS decrypt".into()))?;

    let plaintext = plaintext.as_ref();
    if plaintext.len() != KEY_LENGTH {
        return Err(CryptoError::InvalidKeyLength(plaintext.len()));
    }

    let mut key = [0u8; KEY_LENGTH];
    key.copy_from_slice(plaintext);

    {
        let mut cache = KEY_CACHE.lock().map_err(|_| {
            CryptoError::CipherError("Mutex poisoned".to_string())
        })?;
        cache.insert(cache_key, key);
    }

    Ok(key)
}

pub async fn encrypt_data(
    kms_key_id: &str,
    sensitive_data: &[u8],
) -> Result<EncryptedRecord, CryptoError> {
    let (key, encrypted_key) = generate_envelope_key(kms_key_id).await?;

    let mut nonce = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce);

    let cipher = ChaCha20Poly1305::new_from_slice(&key)
        .map_err(|e| CryptoError::CipherError(e.to_string()))?;
    let payload = Payload { msg: sensitive_data, aad: AAD };
    let ciphertext = cipher
        .encrypt(&nonce.into(), payload)
        .map_err(|e| CryptoError::CipherError(e.to_string()))?;

    Ok(EncryptedRecord {
        ciphertext,
        nonce: nonce.to_vec(),
        key_id: kms_key_id.to_owned(),
        encrypted_key,
    })
}

pub async fn decrypt_data<'a>(
    record: &'a DecryptingRecord<'a>,
) -> Result<Vec<u8>, CryptoError> {
    let key = decrypt_envelope_key(record.encrypted_key).await?;

    let cipher = ChaCha20Poly1305::new_from_slice(&key).map_err(|e| {
        CryptoError::CipherError(e.to_string())
    })?;

    if record.nonce.len() != 12 {
        return Err(CryptoError::CipherError(format!(
            "Invalid nonce length: expected 12, got {}",
            record.nonce.len()
        )));
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(record.nonce);

    let payload = Payload { msg: record.ciphertext, aad: AAD };
    let buf = cipher
        .decrypt((&nonce).into(), payload)
        .map_err(|e| CryptoError::CipherError(e.to_string()))?;
    Ok(buf)
}

pub async fn extract_contact_info(
    data: &[u8],
    nonce: &[u8],
    encrypted_key: &[u8],
) -> Result<(String, String), CryptoError> {
    let buffer = decrypt_data(&DecryptingRecord {
        ciphertext: data,
        nonce,
        key_id: String::new(),
        encrypted_key,
    })
    .await?;

    let sensitive_data = deserialize_data_from_slice(&buffer)?;
    Ok((sensitive_data.email, sensitive_data.phone))
}

pub fn deserialize_data_from_slice(
    buff: &[u8],
) -> Result<SensitiveData, CryptoError> {
    let data = serde_json::from_slice(buff).map_err(|e| {
        CryptoError::SerializationError(e)
    })?;
    Ok(data)
}

pub fn hash_value(value: &str) -> String {
    let mut hasher = Sha256::default();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    const KEY_ID: &str = "arn:"; // use proper key

    #[tokio::test]
    async fn test_encrypt_decrypt_fresh() {
        let data = SensitiveData {
            email: "test@example.com".to_string(),
            phone: "123-456-7890".to_string(),
        };
        let plaintext = serde_json::to_vec(&data).unwrap();
        {
            let mut cache = KEY_CACHE.lock().unwrap();
            cache.clear();
        }
        let encrypted = encrypt_data(KEY_ID, &plaintext).await.unwrap();

        let decrypted = decrypt_data(&DecryptingRecord {
            ciphertext: &encrypted.ciphertext,
            nonce: &encrypted.nonce,
            key_id: KEY_ID.to_string(),
            encrypted_key: &encrypted.encrypted_key,
        })
        .await
        .unwrap();

        let decrypted = deserialize_data_from_slice(&decrypted).unwrap();
        assert_eq!(data.email, decrypted.email);
        assert_eq!(data.phone, decrypted.phone);
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_cached() {
        let data = SensitiveData {
            email: "test@example.com".to_string(),
            phone: "123-456-7890".to_string(),
        };
        let plaintext = serde_json::to_vec(&data).unwrap();
        let encrypted = encrypt_data(KEY_ID, &plaintext).await.unwrap();

        // Second decrypt hits the in-process cache (no extra KMS call)
        let decrypted = decrypt_data(&DecryptingRecord {
            ciphertext: &encrypted.ciphertext,
            nonce: &encrypted.nonce,
            key_id: KEY_ID.to_string(),
            encrypted_key: &encrypted.encrypted_key,
        })
        .await
        .unwrap();

        let decrypted = deserialize_data_from_slice(&decrypted).unwrap();
        assert_eq!(data.email, decrypted.email);
        assert_eq!(data.phone, decrypted.phone);
    }

    #[test]
    fn test_hash_value() {
        let input1 = "hello";
        let expected_hash1 =
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        let result1 = hash_value(input1);
        assert_eq!(result1, expected_hash1);

        let result2 = hash_value(input1);
        assert_eq!(result1, result2);

        let input2 = "world";
        let result3 = hash_value(input2);
        assert_ne!(result1, result3);

        let input4 = "";
        let expected_hash4 =
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let result4 = hash_value(input4);
        assert_eq!(result4, expected_hash4);
    }
}
