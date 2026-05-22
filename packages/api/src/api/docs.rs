use std::{io::Read, str::FromStr, sync::Arc};

use axum::{
    Extension, Json,
    extract::Path,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use nanoid::nanoid;
use redis_store::r_types::AppError;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use tempfile::NamedTempFile;

use tokio::time::Instant;
use tracing::info;
use utils::Result;

use crate::{
    APIContext,
    queries::{
        docs::{Documents, DriverIdentityInputs},
        drivers::DriverQueries,
    },
    schemas::{docs::DriverDocumentType, vehicle::VehicleInfo},
    types::DriverId,
};

/// Header value used for cache control
pub static CACHE_CONTROL: &str = "public, max-age=604800, must-revalidate";

/// Request body for upload
#[derive(TryFromMultipart)]
pub struct UploadPayload {
    #[allow(dead_code)]
    #[form_data(limit = "unlimited")] // handled by axum
    file: FieldData<NamedTempFile>,
}

/// Successful upload response
#[derive(Serialize, Debug)]
pub struct UploadResponse {
    /// ID to attach uploaded file to object
    id: String,
    nonce: Vec<u8>,
    /// KMS ciphertext blob — the client must send this back when associating
    /// the file with a document record so it can be stored alongside the nonce.
    encrypted_key: Vec<u8>,
}

pub async fn handle_docs_upload(
    Extension(client_id): Extension<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
    TypedMultipart(UploadPayload { mut file }): TypedMultipart<UploadPayload>,
) -> Result<Json<UploadResponse>, AppError> {
    // Keep track of processing time
    let now = Instant::now();
    let bucket_id = &ctx.config.bucket;
    let key_id = &ctx.config.kms_key_id;
    //get file name or name it otherwise
    // let file_name = file.metadata.file_name.unwrap_or("un-named-file".to_string());

    // create a buffer to hold image data
    let mut buf = Vec::<u8>::new();

    //read file and write to buffer
    file.contents.read_to_end(&mut buf).map_err(|error| {
        AppError::InternalError(format!(
            "Failed to find nearest driver: {:?}",
            error
        ))
    })?;

    let original_file_size = buf.len();
    // create a file hash
    let file_hash = {
        let mut hasher = sha2::Sha256::new();
        hasher.update(&buf);
        hasher.finalize()
    };

    //TODO::
    let nid = nanoid!(10); // make file_id hash unique if uploading same image.

    // create a file id for storing in db
    let file_id = format!("driver-docs/{client_id}/{file_hash:02x}-{nid}");

    let id = format!("{file_hash:02x}-{nid}");

    let time_taken = Instant::now() - now;
    info!(
        "took {:?} time for processing file {:?}, length {:?}",
        time_taken, file_id, original_file_size
    );

    let now = Instant::now();
    let aws_instance = ctx.config.aws_credentials();
    // upload_file_to_s3 now returns both the nonce (ChaCha20 IV) and the
    // encrypted_key blob (KMS-encrypted data key). Both are sent to the client
    // and must be stored in the DB when the document record is saved.
    let (nonce, encrypted_key) = aws_instance
        .upload_file_to_s3(&buf, &file_id, bucket_id, key_id)
        .await?;
    let time_taken = Instant::now() - now;

    info!(
        "Time taken to upload to S3 bucket is {:?} Nonce {:?}",
        time_taken, nonce
    );

    Ok(Json(UploadResponse {
        id,
        nonce,
        encrypted_key,
    }))
}

#[derive(Debug, Deserialize)]
pub struct DriverIdentityDocuments {
    pub id_number: String,
    pub file_id_front: String,
    pub front_nonce: Vec<u8>,
    pub front_encrypted_key: Vec<u8>,
    pub back_nonce: Vec<u8>,
    pub back_encrypted_key: Vec<u8>,
    pub file_id_back: String,
}

// #[axum_macros::debug_handler]
pub async fn save_driver_identity_documents(
    Extension(client_id): Extension<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
    Path(id_type): Path<String>,
    Json(body): Json<DriverIdentityDocuments>,
) -> Result<StatusCode, AppError> {
    let _ = ctx
        .db
        .save_driver_identity_documents(
            DriverId(client_id),
            DriverIdentityInputs {
                id_number: &body.id_number,
                document_subtype: &id_type,
                file_id_front: &body.file_id_front,
                front_nonce: &body.front_nonce,
                front_encrypted_key: &body.front_encrypted_key,
                back_nonce: &body.back_nonce,
                back_encrypted_key: &body.back_encrypted_key,
                file_id_back: &body.file_id_back,
            },
        )
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub struct DriverDocuments {
    pub id: String,
    pub nonce: Vec<u8>,
    pub encrypted_key: Vec<u8>,
    pub expiry: String,
}

pub async fn save_driver_documents(
    Extension(client_id): Extension<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
    Path(doc_type): Path<String>,
    Json(body): Json<DriverDocuments>,
) -> Result<StatusCode, AppError> {
    let doc_type = DriverDocumentType::from_str(&doc_type)
        .ok()
        .ok_or(AppError::InternalError("Missing body doc_type".to_string()))?;
    let _ = ctx
        .db
        .save_driver_documents(
            DriverId(client_id),
            crate::queries::docs::DriverDocumentInput {
                file_id: &body.id,
                nonce: &body.nonce,
                encrypted_key: &body.encrypted_key,
                expiry: &body.expiry,
                doc_type,
            },
        )
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn save_vehicle_info(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(client_id): Extension<String>,
    Json(body): Json<VehicleInfo>,
) -> Result<StatusCode, AppError> {
    let _ = ctx
        .db
        .save_vehicle_info(DriverId(client_id), body)
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn get_driver_document(
    Extension(client_id): Extension<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
    Path(path_id): Path<String>,
) -> Result<Response, AppError> {
    let bucket = &ctx.config.bucket;
    let path_id = format!("driver-docs/{client_id}/{path_id}");
    // TODO: look up (nonce, encrypted_key) from driver_documents by file_id
    // before this endpoint can work correctly.
    let aws_instance = ctx.config.aws_credentials();
    let res = aws_instance
        .get_uploaded_file_from_s3(&path_id, bucket, &[], &[])
        .await
        .map(|data: Vec<u8>| {
            (
                [
                    (header::CONTENT_TYPE, "image/jpeg".to_string()),
                    (header::CONTENT_DISPOSITION, "inline".to_owned()),
                    (header::CACHE_CONTROL, CACHE_CONTROL.to_owned()),
                ],
                data,
            )
                .into_response()
        })?;

    Ok(res)
}

pub async fn get_profile_photo(
    Extension(ctx): Extension<Arc<APIContext>>,
    Path((ulid, path_id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let bucket = &ctx.config.bucket;
    let path_id = format!("driver-docs/{ulid}/{path_id}");
    // Retrieve both the nonce and the encrypted_key blob stored at upload time.
    let (nonce, encrypted_key) = ctx.db.get_driver_photo_info(&ulid).await?;
    let aws_instance = ctx.config.aws_credentials();
    let res = aws_instance
        .get_uploaded_file_from_s3(&path_id, bucket, &encrypted_key, &nonce)
        .await
        .map(|data: Vec<u8>| {
            (
                [
                    (header::CONTENT_TYPE, "image/jpeg".to_string()),
                    (header::CONTENT_DISPOSITION, "attachment".to_owned()),
                    (header::CACHE_CONTROL, CACHE_CONTROL.to_owned()),
                ],
                data,
            )
                .into_response()
        })?;

    Ok(res)
}
