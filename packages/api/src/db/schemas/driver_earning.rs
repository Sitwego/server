use chrono_tz::Africa::Nairobi;
use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, Default, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "driver_earning")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Text")]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub driver_id: String,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    pub amount: Decimal,
    #[sea_orm(column_type = "Boolean", default_value = false)]
    pub is_discounted: bool,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))", default_value = "0.00")]
    pub discount: Decimal,
    #[sea_orm(
        column_type = "String(StringLen::N(3))",
        default_value = "'KES'"
    )]
    pub currency: String,
    #[sea_orm(
        column_type = "String(StringLen::N(20))",
        default_value = "'pending'"
    )]
    pub payment_status: String,
    #[sea_orm(
        column_type = "String(StringLen::N(50))",
        default_value = "'cash'"
    )]
    pub payment_method: String,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(column_type = "String(StringLen::N(26))", unique, indexed)]
    pub ride_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::driver::Entity",
        from = "Column::DriverId",
        to = "super::driver::Column::Id",
        on_delete = "Cascade"
    )]
    Driver,
    #[sea_orm(
        belongs_to = "super::ride::Entity",
        from = "Column::RideId",
        to = "super::ride::Column::Id",
        on_delete = "Cascade"
    )]
    Ride,
}

impl Related<super::driver::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Driver.def()
    }
}

impl Related<super::ride::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ride.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            created_at: Set(chrono::Utc::now()
                .with_timezone(&Nairobi)
                .fixed_offset()),
            updated_at: Set(chrono::Utc::now()
                .with_timezone(&Nairobi)
                .fixed_offset()),
            ..ActiveModelTrait::default()
        }
    }
}
