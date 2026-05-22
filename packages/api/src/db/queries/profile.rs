use anyhow::Context;
use db_store::Database;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter,
};
use time::Date;
use tracing::info;

use crate::schemas::{customer, driver};
use crate::{
    api::profile::ProfileCreateObject,
    schemas::profile::{self, ActiveModel},
    types::ProfileId,
};

use super::driver_stats::DriverStatsQueries;
use super::drivers::DriverQueries;

pub struct PersonalDetailsUpdate {
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub date_of_birth: Date,
}

pub trait ProfileQueries {
    fn get_profile(
        &self,
    ) -> impl std::future::Future<Output = utils::Result<()>> + Send;

    fn create_profile(
        &self,
        profile_obj: ProfileCreateObject,
        is_driver: bool,
    ) -> impl std::future::Future<Output = utils::Result<ProfileId>> + Send;
    fn check_hash_exists(
        &self,
        phone_hash: String,
        email_hash: String,
        is_driver: bool,
    ) -> impl std::future::Future<Output = utils::Result<bool>> + Send;

    fn update_driver_personal_details(
        &self,
        profile_id: &str,
        details: PersonalDetailsUpdate,
    ) -> impl std::future::Future<Output = utils::Result<()>> + Send;

    fn update_device_info(
        &self,
        profile_id: &str,
        device_type: String,
        device_token: String,
    ) -> impl std::future::Future<Output = utils::Result<()>> + Send;
}

impl ProfileQueries for Database {
    async fn get_profile(&self) -> utils::Result<()> {
        let _ = self
            .transaction(|tx| async move {
                let profile =
                    profile::Entity::find_by_id("1").one(&*tx).await.context(
                        "Failed to check for existing profile in profile table",
                    )?;
                Ok(profile)
            })
            .await;
        Ok(())
    }

    async fn create_profile(
        &self,
        profile_obj: ProfileCreateObject,
        is_driver: bool,
    ) -> utils::Result<ProfileId> {
        let profile = profile_obj.clone();
        let id = self
            .transaction(move |tx| {
                let p_obj = profile_obj.clone();
                println!("{:?}", p_obj.contact_data);
                async move {
                    let profile =
                        profile::Entity::insert(profile::ActiveModel {
                            id: ActiveValue::set(p_obj.id),
                            nonce: ActiveValue::set(p_obj.nonce),
                            // KMS ciphertext blob — required to recover the
                            // data key after a restart (envelope encryption).
                            encrypted_key: ActiveValue::set(
                                p_obj.encrypted_key,
                            ),
                            contact_data: ActiveValue::Set(p_obj.contact_data),
                            first_name: ActiveValue::Set(p_obj.first_name),
                            last_name: ActiveValue::Set(p_obj.last_name),
                            gender: ActiveValue::Set(p_obj.gender),
                            mobile_country_code: ActiveValue::Set(Some(
                                p_obj.mobile_country_code,
                            )),
                            ..Default::default()
                        })
                        .exec_with_returning(&*tx)
                        .await?;
                    info!("CREATE PROFILE{:?}", profile.contact_data);
                    Ok(ProfileId(profile.id))
                }
            })
            .await?;

        let profile_id = id.clone();
        if is_driver {
            self.create_driver_tx(id.0.clone(), profile).await?;
            self.create_driver_stats_tx(profile_id.0.to_owned()).await?;
        } else {
            let _ = self
                .transaction(move |tx| {
                    let id = profile_id.clone();
                    let customer_profile = profile.clone();
                    async move {
                        let customer =
                            customer::Entity::insert(customer::ActiveModel {
                                id: ActiveValue::set(id.0.clone()),
                                password: ActiveValue::set(
                                    customer_profile.password,
                                ),
                                email_hash: ActiveValue::set(
                                    customer_profile.email_hash,
                                ),
                                phone_hash: ActiveValue::set(
                                    customer_profile.phone_hash,
                                ),
                                ..Default::default()
                            })
                            .exec(&*tx)
                            .await
                            .expect(
                                "Failed to insert customer into customer table",
                            );
                        Ok(customer)
                    }
                })
                .await;
        }
        Ok(id)
    }

    async fn update_driver_personal_details(
        &self,
        profile_id: &str,
        details: PersonalDetailsUpdate,
    ) -> utils::Result<()> {
        let id = profile_id.to_owned();
        self.transaction(move |tx| {
            let id = id.clone();
            let details = PersonalDetailsUpdate {
                first_name: details.first_name.clone(),
                middle_name: details.middle_name.clone(),
                last_name: details.last_name.clone(),
                date_of_birth: details.date_of_birth,
            };
            async move {
                let profile = profile::Entity::find_by_id(&id)
                    .one(&*tx)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("Profile not found: {id}")
                    })?;

                let mut active: profile::ActiveModel = profile.into();
                active.first_name = ActiveValue::Set(details.first_name);
                active.middle_name = ActiveValue::Set(details.middle_name);
                active.last_name = ActiveValue::Set(details.last_name);
                active.dob = ActiveValue::Set(Some(details.date_of_birth));
                active.update(&*tx).await?;
                Ok(())
            }
        })
        .await
    }

    async fn update_device_info(
        &self,
        profile_id: &str,
        device_type: String,
        device_token: String,
    ) -> utils::Result<()> {
        let id = profile_id.to_owned();
        self.transaction(move |tx| {
            let id = id.clone();
            let device_type = device_type.clone();
            let device_token = device_token.clone();
            async move {
                let profile = profile::Entity::find_by_id(&id)
                    .one(&*tx)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("Profile not found: {id}")
                    })?;

                let mut active: profile::ActiveModel = profile.into();
                active.client_device_type = ActiveValue::Set(Some(device_type));
                active.device_token = ActiveValue::Set(Some(device_token));
                active.update(&*tx).await?;
                Ok(())
            }
        })
        .await
    }

    async fn check_hash_exists(
        &self,
        phone_hash: String,
        email_hash: String,
        is_driver: bool,
    ) -> utils::Result<bool> {
        match is_driver {
            true => {
                match self
                    .transaction(move |tx| {
                        let phone_hash = phone_hash.clone();
                        let email_hash = email_hash.clone();
                        async move {
                            let profile = driver::Entity::find()
                .filter(
                  driver::Column::EmailHash
                    .eq(email_hash)
                    .or(driver::Column::PhoneHash.eq(phone_hash)),
                )
                .one(&*tx)
                .await
                .context(
                  "Failed to check for existing profile in profile table",
                )?;
                            Ok(profile)
                        }
                    })
                    .await?
                    .is_some()
                {
                    true => Ok(true),
                    false => Ok(false),
                }
            }
            false => {
                match self
                    .transaction(move |tx| {
                        let phone_hash = phone_hash.clone();
                        let email_hash = email_hash.clone();
                        async move {
                            let profile = customer::Entity::find()
                .filter(
                  customer::Column::EmailHash
                    .eq(email_hash)
                    .or(customer::Column::PhoneHash.eq(phone_hash)),
                )
                .one(&*tx)
                .await
                .context(
                  "Failed to check for existing profile in profile table",
                )?;
                            Ok(profile)
                        }
                    })
                    .await?
                    .is_some()
                {
                    true => Ok(true),
                    false => Ok(false),
                }
            }
        }
    }
}
