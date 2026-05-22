use sea_orm::{
    ActiveValue, ActiveValue::Set, entity::prelude::*, sea_query::StringLen,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Default)]
#[sea_orm(table_name = "ipn")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(StringLen::N(26))", indexed)]
    pub id: String,
    #[sea_orm(unique, column_type = "String(StringLen::N(255))", indexed)]
    pub checkout_request_id: String,
    #[sea_orm(unique, column_type = "String(StringLen::N(255))", indexed)]
    pub merchant_request_id: String,
    pub amount: Option<Decimal>,
    #[sea_orm(column_type = "String(StringLen::N(50))")]
    pub mpesa_receipt_number: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(3))")]
    pub currency: String,
    #[sea_orm(column_type = "String(StringLen::N(20))")]
    pub payment_status: String,
    #[sea_orm(column_type = "String(StringLen::N(20))")]
    pub payment_method: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub transaction_date: i64,
    #[sea_orm(column_type = "String(StringLen::N(15))")]
    pub phone_number: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub result_desc: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(26))", indexed)]
    pub driver_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: ActiveValue::NotSet,
            checkout_request_id: ActiveValue::NotSet,
            merchant_request_id: ActiveValue::NotSet,
            amount: ActiveValue::NotSet,
            mpesa_receipt_number: ActiveValue::NotSet,
            currency: ActiveValue::NotSet,
            payment_status: ActiveValue::NotSet,
            payment_method: ActiveValue::NotSet,
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            transaction_date: ActiveValue::NotSet,
            phone_number: ActiveValue::NotSet,
            result_desc: ActiveValue::NotSet,
            driver_id: ActiveValue::NotSet,
        }
    }
}
