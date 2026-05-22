use sea_orm::{FromQueryResult, Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use super::plans::{BillingType, PlanName, VehicleType};

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "subscriptions")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(StringLen::N(26))")]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub driver_id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub plan_id: String,
    #[sea_orm(column_type = "String(StringLen::N(26))")]
    pub payment_auth_id: String,
    #[sea_orm(nullable)]
    pub payment_auth_setup_date: Option<DateTimeWithTimeZone>,
    #[sea_orm(nullable)]
    pub plan_end_date: Option<DateTimeWithTimeZone>,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTimeWithTimeZone,
    pub auto_pay_status: AutoPayStatus,
    #[sea_orm(column_type = "Text", default_value = "SUBSCRIPTION")]
    pub service_name: String,
    #[sea_orm(nullable)]
    pub last_payment_link_sent_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(default_value = "true")]
    pub enable_service_usage_charge: bool,
    #[sea_orm(default_value = "true")]
    pub is_on_free_trial: bool,
    #[sea_orm(default_value = "false")]
    pub is_plan_active: bool,
    pub plan_start_date: DateTimeWithTimeZone,
    #[sea_orm(nullable)]
    pub is_category_level_subscription_enabled: Option<bool>,
    #[sea_orm(nullable)]
    pub free_trial_end_date: Option<DateTimeWithTimeZone>,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))", nullable)]
    pub amount_due: Option<Decimal>,
    #[sea_orm(nullable)]
    pub last_billed_at: Option<DateTimeWithTimeZone>,
}

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
    Default,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "auto_pay_status")]
pub enum AutoPayStatus {
    #[sea_orm(string_value = "enabled")]
    Enabled,
    #[sea_orm(string_value = "disabled")]
    Disabled,
    #[sea_orm(string_value = "pending_activation")]
    PendingActivation,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "suspended")]
    Suspended,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
    #[sea_orm(string_value = "not_set")]
    #[default]
    NotSet,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::plans::Entity",
        from = "Column::PlanId",
        to = "super::plans::Column::Id"
    )]
    Plans,
    #[sea_orm(
        belongs_to = "super::payment_authorizations::Entity",
        from = "Column::PaymentAuthId",
        to = "super::payment_authorizations::Column::Id"
    )]
    PaymentAuthorizations,
    #[sea_orm(
        belongs_to = "super::driver::Entity",
        from = "Column::DriverId",
        to = "super::driver::Column::Id",
        on_delete = "Cascade"
    )]
    Driver,
}

impl Related<super::plans::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plans.def()
    }
}

impl Related<super::payment_authorizations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PaymentAuthorizations.def()
    }
}

impl Related<super::driver::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Driver.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            ..ActiveModelTrait::default()
        }
    }
}
#[derive(Clone, Debug, PartialEq, FromQueryResult, Serialize, Deserialize)]
pub struct SubscriptionPlan {
    pub sub_id: String,
    pub driver_id: String,
    pub plan_id: String,
    pub auto_pay_status: AutoPayStatus,
    pub plan_vehicle_type: VehicleType,
    pub plan_name: PlanName,
    pub billing_type: BillingType,
    pub plan_cost: Decimal,
    pub payment_auth_setup_date: DateTimeLocal,
    pub plan_end_date: DateTimeLocal,
}
