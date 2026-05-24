use db_store::Database;
use redis_store::r_types::AppError;
use sea_orm::EntityTrait;
use serde_json::json;
use utils::{Result, gen_strings::ulid_string};

use crate::{
    schemas::{
        docs::{self, DriverDocumentType},
        driver_identity_documents,
        vehicle::{self, VehicleInfo},
    },
    types::DriverId,
};
#[derive(Debug)]
pub struct DriverIdentityInputs<'a> {
    pub id_number: &'a str,
    pub document_subtype: &'a str,
    pub file_id_front: &'a str,
    pub front_nonce: &'a [u8],
    pub front_encrypted_key: &'a [u8],
    pub back_nonce: &'a [u8],
    pub back_encrypted_key: &'a [u8],
    pub file_id_back: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct Metadata<'a> {
    pub exp: &'a str,
}
#[derive(Debug)]
pub struct DriverDocumentInput<'a> {
    pub file_id: &'a str,
    pub nonce: &'a [u8],
    pub encrypted_key: &'a [u8],
    pub expiry: &'a str,
    pub doc_type: DriverDocumentType,
}
pub trait Documents {
    fn save_driver_identity_documents(
        &self,
        driver_id: DriverId,
        driver_identity_docs: DriverIdentityInputs,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn save_driver_documents(
        &self,
        driver_id: DriverId,
        driver_docs: DriverDocumentInput,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn save_vehicle_info(
        &self,
        driver_id: DriverId,
        vehicle_info: VehicleInfo,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}
impl Documents for Database {
    async fn save_driver_documents<'a>(
        &self,
        driver_id: DriverId,
        driver_docs: DriverDocumentInput<'a>,
    ) -> Result<()> {
        self.transaction(move |tx| {
            let expiry = driver_docs.expiry;
            let file_id = driver_docs.file_id;
            let driver_id = driver_id.0.to_owned();
            let doc_type = driver_docs.doc_type.to_owned();
            let nonce = driver_docs.nonce.to_vec();
            let encrypted_key = driver_docs.encrypted_key.to_vec();
            Box::pin(async move {
                let _ = docs::Entity::insert(docs::ActiveModel {
                    driver_id: sea_orm::ActiveValue::Set(driver_id),
                    document_type: sea_orm::ActiveValue::Set(doc_type),
                    file_id: sea_orm::ActiveValue::Set(file_id.to_owned()),
                    nonce: sea_orm::ActiveValue::Set(nonce),
                    // KMS ciphertext blob — needed to recover the data key.
                    encrypted_key: sea_orm::ActiveValue::Set(encrypted_key),
                    metadata: sea_orm::ActiveValue::Set(
                        json!({"expiry": expiry.to_owned()}),
                    ),
                    ..Default::default()
                })
                .exec(&*tx)
                .await?;
                Ok(())
            })
        })
        .await?;
        Ok(())
    }

    async fn save_driver_identity_documents<'a>(
        &self,
        driver_id: DriverId,
        driver_identity_docs: DriverIdentityInputs<'a>,
    ) -> Result<()> {
        self.transaction(move |tx| {
            let front_nonce = driver_identity_docs.front_nonce.to_vec();
            let front_encrypted_key =
                driver_identity_docs.front_encrypted_key.to_vec();
            let back_nonce = driver_identity_docs.back_nonce.to_vec();
            let back_encrypted_key =
                driver_identity_docs.back_encrypted_key.to_vec();
            let driver_id = driver_id.inner().to_owned();
            Box::pin(async move {
                let _ = driver_identity_documents::Entity::insert(
                    driver_identity_documents::ActiveModel {
                        driver_id: sea_orm::ActiveValue::Set(driver_id),
                        id_number: sea_orm::ActiveValue::Set(
                            driver_identity_docs.id_number.to_owned(),
                        ),
                        document_subtype: sea_orm::ActiveValue::Set(
                            driver_identity_docs.document_subtype.to_owned(),
                        ),
                        file_id_front: sea_orm::ActiveValue::Set(
                            driver_identity_docs.file_id_front.to_owned(),
                        ),
                        front_nonce: sea_orm::ActiveValue::Set(front_nonce),
                        front_encrypted_key: sea_orm::ActiveValue::Set(
                            front_encrypted_key,
                        ),
                        back_nonce: sea_orm::ActiveValue::Set(back_nonce),
                        back_encrypted_key: sea_orm::ActiveValue::Set(
                            back_encrypted_key,
                        ),
                        file_id_back: sea_orm::ActiveValue::Set(
                            driver_identity_docs.file_id_back.to_owned(),
                        ),
                        ..Default::default()
                    },
                )
                .exec(&*tx)
                .await
                .expect("Failed to create DriverIdentityInputs");
                Ok(())
            })
        })
        .await?;
        Ok(())
    }

    async fn save_vehicle_info(
        &self,
        driver_id: DriverId,
        vehicle_info: VehicleInfo,
    ) -> Result<(), AppError> {
        self.transaction(move |tx| {
            let vehicle_type = vehicle_info.vehicle_type.to_owned();
            let make = vehicle_info.make.to_owned();
            let model = vehicle_info.model.to_owned();
            let vin = vehicle_info.vin.to_owned();
            let license_plate = vehicle_info.license_plate.to_owned();
            let color = vehicle_info.color.to_owned();
            let driver_id = driver_id.0.to_owned();
            Box::pin(async move {
                let _ = vehicle::Entity::insert(vehicle::ActiveModel {
                    id: sea_orm::ActiveValue::Set(ulid_string()),
                    color: sea_orm::ActiveValue::Set(color),
                    vehicle_type: sea_orm::ActiveValue::Set(vehicle_type),
                    plate_number: sea_orm::ActiveValue::Set(license_plate),
                    // category: todo!(),
                    capacity: sea_orm::ActiveValue::Set(Some(
                        vehicle_info.capacity,
                    )),
                    model: sea_orm::ActiveValue::Set(Some(model)),
                    y_manufacturing: sea_orm::ActiveValue::Set(Some(
                        vehicle_info.year,
                    )),
                    make: sea_orm::ActiveValue::Set(Some(make)),
                    vin: sea_orm::ActiveValue::Set(vin),
                    driver_id: sea_orm::ActiveValue::Set(driver_id),
                    ..Default::default()
                })
                .exec(&*tx)
                .await?;

                Ok(())
            })
        })
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))?;
        Ok(())
    }
}
