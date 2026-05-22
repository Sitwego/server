use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "profile")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Text")]
    pub id: String,
    pub nonce: Vec<u8>,
    pub encrypted_key: Vec<u8>,
    pub contact_data: Vec<u8>,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub gender: String,
    pub hometown: Option<String>,
    pub mobile_country_code: Option<String>,
    pub identifier: Option<String>,
    #[sea_orm(default_value = true)]
    pub is_new: bool,
    #[sea_orm(default_value = false)]
    pub verified: bool,
    pub device_token: Option<String>,
    pub whatsapp_notification_status: Option<String>,
    pub face_image_id: Option<String>,
    #[sea_orm(default_value = 0)]
    pub total_earned_coins: i32,
    #[sea_orm(default_value = 0)]
    pub used_coins: i32,
    pub registration_lat: Option<f64>,
    pub registration_lon: Option<f64>,
    pub client_device_type: Option<String>,
    pub client_device_id: Option<String>,
    pub backend_app_version: Option<String>,
    pub driver_tag: Option<Vec<String>>,
    pub dob: Option<Date>,
    pub bio: Option<String>,
    #[sea_orm(column_type = "JsonBinary", default_value = "{}")]
    pub travel_preferences: serde_json::Value,
    #[sea_orm(default_value = false)]
    pub google_linked: bool,
    pub google_email: Option<String>,
    pub id_token: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// ... Relations and ActiveModelBehavior ...

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::driver::Entity",
        from = "Column::Id",
        to = "super::driver::Column::Id"
    )]
    Driver,
    #[sea_orm(has_one = "super::customer::Entity")]
    Customer,
    #[sea_orm(has_one = "super::profile_address::Entity")]
    ProfileAddress,
}

impl Related<super::driver::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Driver.def()
    }
}

impl Related<super::customer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl Related<super::profile_address::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProfileAddress.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            created_at: Set(OffsetDateTime::now_utc()),
            updated_at: Set(OffsetDateTime::now_utc()),
            ..ActiveModelTrait::default()
        }
    }
}
