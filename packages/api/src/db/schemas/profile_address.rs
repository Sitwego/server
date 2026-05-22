use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "profile_address")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(StringLen::N(26))")]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub profile_id: String,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub street: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub city: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub state: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(20))", nullable)]
    pub zip: Option<String>,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: OffsetDateTime,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: OffsetDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::profile::Entity",
        from = "Column::ProfileId",
        to = "super::profile::Column::Id",
        on_delete = "Cascade"
    )]
    Profile,
}

impl Related<super::profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Profile.def()
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
