use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

// Location table mapped to a SeaORM entity
#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "location")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(StringLen::N(26))")]
    pub id: String,
    pub lat: f64,
    pub lon: f64,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub street: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub city: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub road: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub state: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub country: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub building: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub floor: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub door: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub area_code: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub area: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub ward: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub place_id: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub instructions: Option<String>,
    #[sea_orm(column_type = "Json", nullable)]
    pub extras: Option<Json>,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: OffsetDateTime,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: OffsetDateTime,
}

// Define relationships (none in this case as per the SQL)
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_one = "super::ride_request::Entity",
        from = "Column::Id",
        to = "super::ride_request::Column::FromLocationId"
    )]
    RideRequestFrom,

    #[sea_orm(
        has_one = "super::ride_request::Entity",
        from = "Column::Id",
        to = "super::ride_request::Column::ToLocationId"
    )]
    RideRequestTo,
}

// Enable ActiveModel functionality
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            created_at: Set(OffsetDateTime::now_utc()),
            updated_at: Set(OffsetDateTime::now_utc()),
            ..ActiveModelTrait::default()
        }
    }
}
