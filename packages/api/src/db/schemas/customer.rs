use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "customer")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub password: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub phone_hash: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub email_hash: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::profile::Entity",
        from = "Column::Id",
        to = "super::profile::Column::Id"
    )]
    Profile,
    #[sea_orm(has_one = "super::rider_stats::Entity")]
    RiderStats,
}

impl Related<super::profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Profile.def()
    }
}

impl Related<super::rider_stats::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RiderStats.def()
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
