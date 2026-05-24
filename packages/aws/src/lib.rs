use aws_sdk_s3::{
    Client, Config as AWSConfig,
    config::{Credentials, Region},
    primitives::ByteStream,
};
use std::io::Write;

use redis_store::r_types::*;
use serde::Deserialize;
use utils::{
    Result,
    hashing_algo::{DecryptingRecord, decrypt_data, encrypt_data},
};

#[derive(Deserialize, Debug, Clone)]
pub struct FilesLimit {
    pub min_file_size: usize,
    pub min_resolution: [usize; 2],
    pub max_mega_pixels: usize,
    pub max_pixel_side: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AwsCredentials {
    pub endpoint: Option<String>,
    pub path_style_buckets: Option<bool>,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub default_bucket: Option<String>,
}

/// Lets impl AwsCredentials
impl AwsCredentials {
    pub fn new(
        endpoint: Option<String>,
        path_style_buckets: Option<bool>,
        region: String,
        access_key_id: String,
        secret_access_key: String,
        default_bucket: Option<String>,
    ) -> Self {
        Self {
            endpoint,
            path_style_buckets,
            region,
            access_key_id,
            secret_access_key,
            default_bucket,
        }
    }
    /// Create S3 client from AwsCredentials
    fn s3_client(&self) -> Client {
        let aws_credentials = Credentials::new(
            self.access_key_id.clone(),
            self.secret_access_key.clone(),
            None,
            None,
            "default",
        );

        let config = AWSConfig::builder()
            .region(Region::new(self.region.clone()))
            .credentials_provider(aws_credentials)
            .build();
        Client::from_conf(config)
    }

    /// Encrypt and upload a file to S3.
    /// Returns `(nonce, encrypted_key)` — both must be stored in the DB so
    /// the file can be decrypted after a restart.
    pub async fn upload_file_to_s3(
        &self,
        buf: &[u8],
        path_id: &str,
        bucket_id: &str,
        kms_key_id: &str,
    ) -> Result<(Vec<u8>, Vec<u8>), AppError> {
        let client = self.s3_client();

        let encrypted_img = encrypt_data(kms_key_id, buf)
            .await
            .map_err(|err| AppError::InternalError(err.to_string()))?;

        let body: ByteStream =
            ByteStream::from(encrypted_img.ciphertext.clone());

        client
            .put_object()
            .bucket(bucket_id)
            .key(path_id)
            .body(body)
            .send()
            .await
            .map_err(|err| {
                println!("Error uploading file to S3: {:?}", err);
                AppError::InternalError(err.to_string())
            })?;

        Ok((encrypted_img.nonce, encrypted_img.encrypted_key))
    }

    /// Get and decrypt a file from S3.
    /// `encrypted_key` is the KMS ciphertext blob stored alongside the file record.
    pub async fn get_uploaded_file_from_s3(
        &self,
        path_id: &str,
        bucket: &str,
        encrypted_key: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>, AppError> {
        let client = self.s3_client();

        let mut file_data = client
            .get_object()
            .bucket(bucket)
            .key(path_id)
            .send()
            .await
            .map_err(|err| AppError::InternalError(err.to_string()))?;

        let mut buf = Vec::<u8>::new();
        while let Some(bytes) = file_data.body.next().await {
            let data = bytes.map_err(|err| {
                AppError::InternalError(format!(
                    "Error reading S3 ByteStream: {:?}",
                    err
                ))
            })?;
            buf.write_all(&data).unwrap();
        }

        let buf = decrypt_data(&DecryptingRecord {
            ciphertext: &buf,
            nonce,
            key_id: String::new(),
            encrypted_key,
        })
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;

        Ok(buf)
    }
}
