use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Intermediate stop of a ride request, ordered by `stop_order` (0-based).
/// Rides are capped at one stop at the API layer (`MAX_RIDE_STOPS`); this
/// table already supports N stops per request.
#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "ride_request_stops")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "String(StringLen::N(26))"
    )]
    pub ride_request_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub stop_order: i32,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub location_id: String,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: OffsetDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ride_request::Entity",
        from = "Column::RideRequestId",
        to = "super::ride_request::Column::Id",
        on_delete = "Cascade"
    )]
    RideRequest,
    #[sea_orm(
        belongs_to = "super::location::Entity",
        from = "Column::LocationId",
        to = "super::location::Column::Id",
        on_delete = "Cascade"
    )]
    Location,
}

impl Related<super::location::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Location.def()
    }
}

impl Related<super::ride_request::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RideRequest.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            created_at: Set(OffsetDateTime::now_utc()),
            ..ActiveModelTrait::default()
        }
    }
}
