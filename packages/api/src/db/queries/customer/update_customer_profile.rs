use async_trait::async_trait;
use db_store::Database;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter,
};
use time::{Date, OffsetDateTime};
use utils::Result;

use crate::schemas::{profile, profile_address};

// ── Profile update ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct UpdateRiderProfileInput {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub mobile_country_code: Option<String>,
    pub dob: Option<Date>,
}

#[async_trait]
pub trait UpdateCustomerProfile {
    async fn update_customer_profile(
        &self,
        profile_id: &str,
        input: UpdateRiderProfileInput,
    ) -> Result<()>;
}

#[async_trait]
impl UpdateCustomerProfile for Database {
    async fn update_customer_profile(
        &self,
        profile_id: &str,
        input: UpdateRiderProfileInput,
    ) -> Result<()> {
        let id = profile_id.to_owned();
        self.transaction(move |tx| {
            let id = id.clone();
            let input = input.clone();
            async move {
                let model = profile::Entity::find_by_id(&id)
                    .one(&*tx)
                    .await?
                    .ok_or_else(|| {
                    anyhow::anyhow!("Profile not found: {id}")
                })?;

                let mut active: profile::ActiveModel = model.into();

                if let Some(v) = input.first_name {
                    active.first_name = ActiveValue::Set(v);
                }
                if let Some(v) = input.last_name {
                    active.last_name = ActiveValue::Set(v);
                }
                if let Some(v) = input.mobile_country_code {
                    active.mobile_country_code = ActiveValue::Set(Some(v));
                }
                if let Some(v) = input.dob {
                    active.dob = ActiveValue::Set(Some(v));
                }
                active.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
                active.update(&*tx).await?;

                Ok(())
            }
        })
        .await
    }
}

// ── Google link upsert ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LinkGoogleInput {
    pub google_linked: bool,
    pub google_email: Option<String>,
    pub id_token: Option<String>,
}

#[async_trait]
pub trait LinkGoogleAccount {
    async fn link_google_account(
        &self,
        profile_id: &str,
        input: LinkGoogleInput,
    ) -> Result<()>;
}

#[async_trait]
impl LinkGoogleAccount for Database {
    async fn link_google_account(
        &self,
        profile_id: &str,
        input: LinkGoogleInput,
    ) -> Result<()> {
        let id = profile_id.to_owned();
        self.transaction(move |tx| {
            let id = id.clone();
            let input = input.clone();
            async move {
                let model = profile::Entity::find_by_id(&id)
                    .one(&*tx)
                    .await?
                    .ok_or_else(|| {
                    anyhow::anyhow!("Profile not found: {id}")
                })?;

                let mut active: profile::ActiveModel = model.into();
                active.google_linked = ActiveValue::Set(input.google_linked);
                active.google_email = ActiveValue::Set(input.google_email);
                active.id_token = ActiveValue::Set(input.id_token);
                active.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
                active.update(&*tx).await?;

                Ok(())
            }
        })
        .await
    }
}

// ── Address upsert ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AddressInput {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
}

#[async_trait]
pub trait UpsertCustomerAddress {
    async fn upsert_customer_address(
        &self,
        profile_id: &str,
        addr: AddressInput,
    ) -> Result<()>;
}

#[async_trait]
impl UpsertCustomerAddress for Database {
    async fn upsert_customer_address(
        &self,
        profile_id: &str,
        addr: AddressInput,
    ) -> Result<()> {
        let id = profile_id.to_owned();
        self.transaction(move |tx| {
            let id = id.clone();
            let addr = addr.clone();
            async move {
                let existing = profile_address::Entity::find()
                    .filter(profile_address::Column::ProfileId.eq(&id))
                    .one(&*tx)
                    .await?;

                match existing {
                    Some(row) => {
                        let mut a: profile_address::ActiveModel = row.into();
                        a.street = ActiveValue::Set(addr.street);
                        a.city = ActiveValue::Set(addr.city);
                        a.state = ActiveValue::Set(addr.state);
                        a.zip = ActiveValue::Set(addr.zip);
                        a.updated_at =
                            ActiveValue::Set(OffsetDateTime::now_utc());
                        a.update(&*tx).await?;
                    }
                    None => {
                        profile_address::Entity::insert(
                            profile_address::ActiveModel {
                                id: ActiveValue::Set(nanoid::nanoid!(26)),
                                profile_id: ActiveValue::Set(id.clone()),
                                street: ActiveValue::Set(addr.street),
                                city: ActiveValue::Set(addr.city),
                                state: ActiveValue::Set(addr.state),
                                zip: ActiveValue::Set(addr.zip),
                                ..Default::default()
                            },
                        )
                        .exec(&*tx)
                        .await?;
                    }
                }

                Ok(())
            }
        })
        .await
    }
}
