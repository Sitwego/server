use db_store::Database;
use redis_store::r_types::AppError;
use sea_orm::{
    ColumnTrait, EntityTrait, QueryFilter, QuerySelect, RelationTrait,
    SelectColumns,
};
use utils::hashing_algo::hash_value;

use crate::schemas::{customer, profile};
pub trait LoginCustomer {
    fn login_customer(
        &self,
        phone_number: &str,
        device_id: &str,
        password: Option<&str>,
    ) -> impl std::future::Future<
        Output = utils::Result<Option<customer::Model>, AppError>,
    > + Send;
}

impl LoginCustomer for Database {
    async fn login_customer(
        &self,
        phone_number: &str,
        _device_id: &str,
        _password: Option<&str>,
    ) -> utils::Result<Option<customer::Model>, AppError> {
        let resp = self
            .transaction(move |tx| async move {
                let phone_hash = hash_value(phone_number);
                let customer_opt = customer::Entity::find()
                    .join(
                        sea_orm::JoinType::LeftJoin,
                        customer::Relation::Profile.def(),
                    )
                    .filter(customer::Column::PhoneHash.eq(phone_hash))
                    .select_column(customer::Column::Id)
                    .select_column(profile::Column::FirstName)
                    .select_column(profile::Column::LastName)
                    .one(&*tx)
                    .await?;
                Ok(customer_opt)
            })
            .await
            .map_err(|er| AppError::DatabaseError(er.to_string()))?;

        Ok(resp)
    }
}
