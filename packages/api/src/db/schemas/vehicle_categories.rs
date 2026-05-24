use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use time::OffsetDateTime;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "vehicle_categories")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub category: VehicleCategory,
    pub engine_size: String,
    pub example_cars: String,
    pub short_distance_kes_per_km: i32,
    pub long_distance_kes_per_km: i32,
    pub base_fare_kes: i32,
    pub min_fare: i32,
    pub per_min_rate: i32,
    pub waiting_per_minute_kes: i32,
    pub return_trip_discount: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_many = "super::vehicle_category_mappings::Entity",
        from = "Column::Category",
        to = "super::vehicle_category_mappings::Column::Category"
    )]
    VehicleCategoryMappings,
}

impl Related<super::vehicle_category_mappings::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::VehicleCategoryMappings.def()
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

// Enum matching your database data
#[derive(
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    EnumIter,
    EnumString,
    Display,
    DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "vehicle_category")]
#[derive(Default)]
pub enum VehicleCategory {
    #[sea_orm(string_value = "Swift")]
    #[default]
    Swift,
    #[sea_orm(string_value = "Standard")]
    Standard,
    #[sea_orm(string_value = "Comfort")]
    Comfort,
    #[sea_orm(string_value = "Xl")]
    Xl,
    #[sea_orm(string_value = "Executive")]
    Executive,
    #[sea_orm(string_value = "Bike")]
    Bike,
    #[sea_orm(string_value = "Women")]
    Women,
}

impl VehicleCategory {
    /// Returns all request categories that a driver of `self` category is
    /// eligible to serve. Bike and Women are exclusive — only an exact match.
    /// The remaining categories form a tier chain where a higher-tier driver
    /// can serve their own tier and all tiers below them:
    ///   Swift < Standard < Comfort < Xl < Executive
    ///
    /// - Swift driver   → Swift only
    /// - Standard driver → Swift, Standard
    /// - Comfort driver  → Swift, Standard, Comfort
    /// - Xl driver       → Swift, Standard, Comfort, Xl
    /// - Executive driver → Swift, Standard, Comfort, Xl, Executive
    pub fn eligible_serving_categories(self) -> Vec<VehicleCategory> {
        match self {
            VehicleCategory::Bike => vec![VehicleCategory::Bike],
            VehicleCategory::Women => vec![VehicleCategory::Women],
            VehicleCategory::Swift => vec![VehicleCategory::Swift],
            VehicleCategory::Standard => {
                vec![VehicleCategory::Swift, VehicleCategory::Standard]
            }
            VehicleCategory::Comfort => vec![
                VehicleCategory::Swift,
                VehicleCategory::Standard,
                VehicleCategory::Comfort,
            ],
            VehicleCategory::Xl => vec![
                VehicleCategory::Swift,
                VehicleCategory::Standard,
                VehicleCategory::Comfort,
                VehicleCategory::Xl,
            ],
            VehicleCategory::Executive => vec![
                VehicleCategory::Swift,
                VehicleCategory::Standard,
                VehicleCategory::Comfort,
                VehicleCategory::Xl,
                VehicleCategory::Executive,
            ],
        }
    }
}
