use sea_orm::entity::prelude::*;
use serde::Serialize;
use time::OffsetDateTime;
// use sea_orm::entity::prelude::Json;

#[derive(
    Clone, Debug, Default, PartialEq, Eq, DeriveEntityModel, Serialize,
)]
#[sea_orm(table_name = "ride_history")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub ride_id: String,
    pub driver_id: String,
    pub coordinates: String, // WKT representation, //TODO Change to JSON for flexibility
    #[sea_orm(default_value = "now")]
    pub updated_at: Option<OffsetDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ride::Entity",
        from = "Column::RideId",
        to = "super::ride::Column::Id",
        on_delete = "Cascade"
    )]
    Ride,

    #[sea_orm(
        belongs_to = "super::driver::Entity",
        from = "Column::DriverId",
        to = "super::driver::Column::Id",
        on_delete = "Cascade"
    )]
    Driver,
}

impl Related<super::ride::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ride.def()
    }
}

impl Related<super::driver::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Driver.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
