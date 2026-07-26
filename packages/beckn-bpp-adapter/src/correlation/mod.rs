//! Correlation store + idempotency (Step 2).
//!
//! Owns the `beckn_orders` table — the protocol↔internal mapping and the
//! idempotency ledger. Protocol fields live here, never on `ride` (Inviolable
//! Rule #6). `(transaction_id, message_id)` is the idempotency anchor: a retried
//! callback resolves to the existing row instead of acting twice (Rule #4).
//!
//! The repository is a trait implemented on the shared `db_store::Database`,
//! matching the `api` crate's query-layer convention.

use async_trait::async_trait;
use db_store::Database;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DbErr, EntityTrait, QueryFilter,
    sea_query::OnConflict,
};
use utils::Result;
use utils::gen_strings::ulid_string;

/// The `beckn_orders` entity. One row per inbound Beckn message.
pub mod beckn_orders {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(
        Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
    )]
    #[sea_orm(table_name = "beckn_orders")]
    pub struct Model {
        #[sea_orm(
            primary_key,
            auto_increment = false,
            column_type = "String(StringLen::N(26))"
        )]
        pub id: String,
        pub transaction_id: String,
        pub message_id: String,
        #[sea_orm(nullable)]
        pub beckn_order_id: Option<String>,
        pub bap_id: String,
        pub bap_uri: String,
        #[sea_orm(nullable)]
        pub network_id: Option<String>,
        pub domain: String,
        #[sea_orm(nullable)]
        pub beckn_status: Option<String>,
        pub last_action: String,
        #[sea_orm(column_type = "String(StringLen::N(26))", nullable)]
        pub ride_id: Option<String>,
        /// Quote agreed at init (whole KES) — confirm honours it instead of
        /// re-pricing.
        #[sea_orm(nullable)]
        pub quoted_fare: Option<i32>,
        /// The tier code the quote was for (e.g. `SWIFT`).
        #[sea_orm(nullable)]
        pub quoted_item_id: Option<String>,
        /// Network `FulfillmentState` code of the order's ride (on the
        /// `confirm` row): NEW → RIDE_ASSIGNED → … → RIDE_ENDED/RIDE_CANCELLED.
        #[sea_orm(nullable)]
        pub fulfillment_state: Option<String>,
        /// Driver/vehicle/OTP snapshot (serialized `AssignedInfo`) captured
        /// when dispatch assigns a driver.
        #[sea_orm(nullable)]
        pub assigned_json: Option<String>,
        /// The confirmed trip, `"lat, lon"` — lets later `on_status` orders
        /// repeat the stops without re-parsing the original confirm.
        #[sea_orm(nullable)]
        pub pickup_gps: Option<String>,
        #[sea_orm(nullable)]
        pub dropoff_gps: Option<String>,
        #[sea_orm(default_expr = "Expr::current_timestamp()")]
        pub created_at: DateTimeWithTimeZone,
        #[sea_orm(default_expr = "Expr::current_timestamp()")]
        pub updated_at: DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub use beckn_orders::Model as BecknOrder;

/// The fields needed to create a correlation record for an inbound message.
/// `ride_id`/`beckn_order_id` stay `None` until the order/ride exists.
#[derive(Debug, Clone)]
pub struct NewBecknOrder {
    pub transaction_id: String,
    pub message_id: String,
    pub bap_id: String,
    pub bap_uri: String,
    pub domain: String,
    pub last_action: String,
    pub network_id: Option<String>,
    pub beckn_order_id: Option<String>,
    pub beckn_status: Option<String>,
    pub ride_id: Option<String>,
    pub quoted_fare: Option<i32>,
    pub quoted_item_id: Option<String>,
    pub fulfillment_state: Option<String>,
    pub pickup_gps: Option<String>,
    pub dropoff_gps: Option<String>,
}

/// Data-access for the correlation store.
#[async_trait]
pub trait CorrelationStore: Send + Sync {
    /// Insert a correlation record, or return the existing one if a row for
    /// `(transaction_id, message_id)` already exists. Never creates a duplicate
    /// and never runs a second side-effect for a replayed message.
    ///
    /// Returns the canonical row plus `created = true` iff this call inserted it.
    async fn upsert_or_get(
        &self,
        new: NewBecknOrder,
    ) -> Result<(BecknOrder, bool)>;

    /// Fetch the correlation record for a `(transaction_id, message_id)` pair.
    async fn get_beckn_order(
        &self,
        transaction_id: &str,
        message_id: &str,
    ) -> Result<Option<BecknOrder>>;

    /// Fetch the `init` record that minted `beckn_order_id` — the row carrying
    /// the quote a later `confirm` must honour.
    async fn find_init_by_order_id(
        &self,
        beckn_order_id: &str,
    ) -> Result<Option<BecknOrder>>;

    /// Record the quote delivered for a message (set after re-pricing, which
    /// happens after the idempotency insert). `confirm` reads it back instead
    /// of re-pricing.
    async fn set_quote(
        &self,
        transaction_id: &str,
        message_id: &str,
        fare: i32,
        item_id: &str,
    ) -> Result<()>;

    /// Fetch the `confirm` record for a Beckn order id — the row that carries
    /// the ride id, trip and lifecycle state `status`/`track`/`cancel` act on.
    async fn find_confirm_by_order_id(
        &self,
        beckn_order_id: &str,
    ) -> Result<Option<BecknOrder>>;

    /// Fetch the `confirm` record owning a ride id — how dispatch webhooks and
    /// ride lifecycle events find their way back to the Beckn order.
    async fn find_confirm_by_ride_id(
        &self,
        ride_id: &str,
    ) -> Result<Option<BecknOrder>>;

    /// Advance the order lifecycle on the `confirm` row owning `ride_id`:
    /// order status + fulfillment state, and (when a driver was just
    /// assigned) the driver snapshot.
    async fn set_fulfillment(
        &self,
        ride_id: &str,
        beckn_status: &str,
        fulfillment_state: &str,
        assigned_json: Option<String>,
    ) -> Result<()>;
}

#[async_trait]
impl CorrelationStore for Database {
    async fn upsert_or_get(
        &self,
        new: NewBecknOrder,
    ) -> Result<(BecknOrder, bool)> {
        let id = ulid_string();
        let model = beckn_orders::ActiveModel {
            id: Set(id.clone()),
            transaction_id: Set(new.transaction_id.clone()),
            message_id: Set(new.message_id.clone()),
            beckn_order_id: Set(new.beckn_order_id),
            bap_id: Set(new.bap_id),
            bap_uri: Set(new.bap_uri),
            network_id: Set(new.network_id),
            domain: Set(new.domain),
            beckn_status: Set(new.beckn_status),
            last_action: Set(new.last_action),
            ride_id: Set(new.ride_id),
            quoted_fare: Set(new.quoted_fare),
            quoted_item_id: Set(new.quoted_item_id),
            fulfillment_state: Set(new.fulfillment_state),
            pickup_gps: Set(new.pickup_gps),
            dropoff_gps: Set(new.dropoff_gps),
            // created_at / updated_at fall back to the DB default_expr.
            ..Default::default()
        };

        // INSERT ... ON CONFLICT (transaction_id, message_id) DO NOTHING.
        // A conflict surfaces as `RecordNotInserted`, which is the expected,
        // benign "already there" outcome — not an error.
        match beckn_orders::Entity::insert(model)
            .on_conflict(
                OnConflict::columns([
                    beckn_orders::Column::TransactionId,
                    beckn_orders::Column::MessageId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec(self.conn())
            .await
        {
            Ok(_) | Err(DbErr::RecordNotInserted) => {}
            Err(e) => return Err(e.into()),
        }

        // Re-read the canonical row (ours if we won the insert, otherwise the
        // pre-existing one). `created` is decided by whether the surviving row
        // carries the id we just minted.
        let row = self
            .get_beckn_order(&new.transaction_id, &new.message_id)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "beckn_orders row missing immediately after upsert_or_get"
                )
            })?;
        let created = row.id == id;
        Ok((row, created))
    }

    async fn get_beckn_order(
        &self,
        transaction_id: &str,
        message_id: &str,
    ) -> Result<Option<BecknOrder>> {
        let row = beckn_orders::Entity::find()
            .filter(beckn_orders::Column::TransactionId.eq(transaction_id))
            .filter(beckn_orders::Column::MessageId.eq(message_id))
            .one(self.conn())
            .await?;
        Ok(row)
    }

    async fn find_init_by_order_id(
        &self,
        beckn_order_id: &str,
    ) -> Result<Option<BecknOrder>> {
        let row = beckn_orders::Entity::find()
            .filter(beckn_orders::Column::BecknOrderId.eq(beckn_order_id))
            .filter(beckn_orders::Column::LastAction.eq("init"))
            .one(self.conn())
            .await?;
        Ok(row)
    }

    async fn set_quote(
        &self,
        transaction_id: &str,
        message_id: &str,
        fare: i32,
        item_id: &str,
    ) -> Result<()> {
        beckn_orders::Entity::update_many()
            .col_expr(
                beckn_orders::Column::QuotedFare,
                sea_orm::sea_query::Expr::value(fare),
            )
            .col_expr(
                beckn_orders::Column::QuotedItemId,
                sea_orm::sea_query::Expr::value(item_id),
            )
            .col_expr(
                beckn_orders::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(
                    chrono::Utc::now().fixed_offset(),
                ),
            )
            .filter(beckn_orders::Column::TransactionId.eq(transaction_id))
            .filter(beckn_orders::Column::MessageId.eq(message_id))
            .exec(self.conn())
            .await?;
        Ok(())
    }

    async fn find_confirm_by_order_id(
        &self,
        beckn_order_id: &str,
    ) -> Result<Option<BecknOrder>> {
        let row = beckn_orders::Entity::find()
            .filter(beckn_orders::Column::BecknOrderId.eq(beckn_order_id))
            .filter(beckn_orders::Column::LastAction.eq("confirm"))
            .one(self.conn())
            .await?;
        Ok(row)
    }

    async fn find_confirm_by_ride_id(
        &self,
        ride_id: &str,
    ) -> Result<Option<BecknOrder>> {
        let row = beckn_orders::Entity::find()
            .filter(beckn_orders::Column::RideId.eq(ride_id))
            .filter(beckn_orders::Column::LastAction.eq("confirm"))
            .one(self.conn())
            .await?;
        Ok(row)
    }

    async fn set_fulfillment(
        &self,
        ride_id: &str,
        beckn_status: &str,
        fulfillment_state: &str,
        assigned_json: Option<String>,
    ) -> Result<()> {
        let mut update = beckn_orders::Entity::update_many()
            .col_expr(
                beckn_orders::Column::BecknStatus,
                sea_orm::sea_query::Expr::value(beckn_status),
            )
            .col_expr(
                beckn_orders::Column::FulfillmentState,
                sea_orm::sea_query::Expr::value(fulfillment_state),
            )
            .col_expr(
                beckn_orders::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(
                    chrono::Utc::now().fixed_offset(),
                ),
            );
        if let Some(json) = assigned_json {
            update = update.col_expr(
                beckn_orders::Column::AssignedJson,
                sea_orm::sea_query::Expr::value(json),
            );
        }
        update
            .filter(beckn_orders::Column::RideId.eq(ride_id))
            .filter(beckn_orders::Column::LastAction.eq("confirm"))
            .exec(self.conn())
            .await?;
        Ok(())
    }
}

/// In-memory correlation store with the same idempotency contract as the
/// Postgres one. For dev boots without a database and for handler tests —
/// records vanish on restart, so production must use the real store.
#[derive(Default)]
pub struct MemoryCorrelationStore {
    rows: std::sync::Mutex<
        std::collections::HashMap<(String, String), BecknOrder>,
    >,
}

#[async_trait]
impl CorrelationStore for MemoryCorrelationStore {
    async fn upsert_or_get(
        &self,
        new: NewBecknOrder,
    ) -> Result<(BecknOrder, bool)> {
        let key = (new.transaction_id.clone(), new.message_id.clone());
        let mut rows = self.rows.lock().expect("correlation lock poisoned");
        if let Some(existing) = rows.get(&key) {
            return Ok((existing.clone(), false));
        }
        let now = chrono::Utc::now().fixed_offset();
        let row = BecknOrder {
            id: ulid_string(),
            transaction_id: new.transaction_id,
            message_id: new.message_id,
            beckn_order_id: new.beckn_order_id,
            bap_id: new.bap_id,
            bap_uri: new.bap_uri,
            network_id: new.network_id,
            domain: new.domain,
            beckn_status: new.beckn_status,
            last_action: new.last_action,
            ride_id: new.ride_id,
            quoted_fare: new.quoted_fare,
            quoted_item_id: new.quoted_item_id,
            fulfillment_state: new.fulfillment_state,
            assigned_json: None,
            pickup_gps: new.pickup_gps,
            dropoff_gps: new.dropoff_gps,
            created_at: now,
            updated_at: now,
        };
        rows.insert(key, row.clone());
        Ok((row, true))
    }

    async fn get_beckn_order(
        &self,
        transaction_id: &str,
        message_id: &str,
    ) -> Result<Option<BecknOrder>> {
        let rows = self.rows.lock().expect("correlation lock poisoned");
        Ok(rows
            .get(&(transaction_id.to_string(), message_id.to_string()))
            .cloned())
    }

    async fn find_init_by_order_id(
        &self,
        beckn_order_id: &str,
    ) -> Result<Option<BecknOrder>> {
        let rows = self.rows.lock().expect("correlation lock poisoned");
        Ok(rows
            .values()
            .find(|row| {
                row.beckn_order_id.as_deref() == Some(beckn_order_id)
                    && row.last_action == "init"
            })
            .cloned())
    }

    async fn set_quote(
        &self,
        transaction_id: &str,
        message_id: &str,
        fare: i32,
        item_id: &str,
    ) -> Result<()> {
        let mut rows = self.rows.lock().expect("correlation lock poisoned");
        if let Some(row) =
            rows.get_mut(&(transaction_id.to_string(), message_id.to_string()))
        {
            row.quoted_fare = Some(fare);
            row.quoted_item_id = Some(item_id.to_string());
            row.updated_at = chrono::Utc::now().fixed_offset();
        }
        Ok(())
    }

    async fn find_confirm_by_order_id(
        &self,
        beckn_order_id: &str,
    ) -> Result<Option<BecknOrder>> {
        let rows = self.rows.lock().expect("correlation lock poisoned");
        Ok(rows
            .values()
            .find(|row| {
                row.beckn_order_id.as_deref() == Some(beckn_order_id)
                    && row.last_action == "confirm"
            })
            .cloned())
    }

    async fn find_confirm_by_ride_id(
        &self,
        ride_id: &str,
    ) -> Result<Option<BecknOrder>> {
        let rows = self.rows.lock().expect("correlation lock poisoned");
        Ok(rows
            .values()
            .find(|row| {
                row.ride_id.as_deref() == Some(ride_id)
                    && row.last_action == "confirm"
            })
            .cloned())
    }

    async fn set_fulfillment(
        &self,
        ride_id: &str,
        beckn_status: &str,
        fulfillment_state: &str,
        assigned_json: Option<String>,
    ) -> Result<()> {
        let mut rows = self.rows.lock().expect("correlation lock poisoned");
        if let Some(row) = rows.values_mut().find(|row| {
            row.ride_id.as_deref() == Some(ride_id)
                && row.last_action == "confirm"
        }) {
            row.beckn_status = Some(beckn_status.to_string());
            row.fulfillment_state = Some(fulfillment_state.to_string());
            if assigned_json.is_some() {
                row.assigned_json = assigned_json;
            }
            row.updated_at = chrono::Utc::now().fixed_offset();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Statement};

    /// Connect to the dev DB; skip the test (returning `None`) when there is no
    /// database to talk to or the migration hasn't been applied, so the suite
    /// still passes offline.
    async fn connect() -> Option<Database> {
        let url = std::env::var("DATABASE_URL").ok()?;
        let db = Database::new(
            db_store::ConnectOptions::new(url),
            utils::executor::Executor,
        )
        .await
        .ok()?;
        // require the table to exist
        let exists = db
            .conn()
            .query_one(Statement::from_string(
                db.conn().get_database_backend(),
                "SELECT to_regclass('public.beckn_orders') IS NOT NULL AS ok"
                    .to_owned(),
            ))
            .await
            .ok()??;
        let ok: bool = exists.try_get("", "ok").ok()?;
        if ok { Some(db) } else { None }
    }

    fn sample(txn: &str, msg: &str) -> NewBecknOrder {
        NewBecknOrder {
            transaction_id: txn.to_string(),
            message_id: msg.to_string(),
            bap_id: "bap.example.com".to_string(),
            bap_uri: "https://bap.example.com/beckn".to_string(),
            domain: "mobility".to_string(),
            last_action: "search".to_string(),
            network_id: None,
            beckn_order_id: None,
            beckn_status: None,
            ride_id: None,
            quoted_fare: None,
            quoted_item_id: None,
            fulfillment_state: None,
            pickup_gps: None,
            dropoff_gps: None,
        }
    }

    #[tokio::test]
    async fn replaying_same_message_is_idempotent() {
        let Some(db) = connect().await else {
            eprintln!("skipping: no DATABASE_URL / beckn_orders table");
            return;
        };

        // Unique ids so repeated test runs don't collide.
        let txn = ulid_string();
        let msg = ulid_string();

        let (first, created1) =
            db.upsert_or_get(sample(&txn, &msg)).await.unwrap();
        assert!(created1, "first upsert should insert");

        // Replay the SAME (transaction_id, message_id): no new row, same id.
        let (second, created2) =
            db.upsert_or_get(sample(&txn, &msg)).await.unwrap();
        assert!(!created2, "replay must NOT insert a second row");
        assert_eq!(first.id, second.id, "replay must return the same row");

        // Prove there is exactly one row at the data layer.
        let count = beckn_orders::Entity::find()
            .filter(beckn_orders::Column::TransactionId.eq(&txn))
            .filter(beckn_orders::Column::MessageId.eq(&msg))
            .all(db.conn())
            .await
            .unwrap()
            .len();
        assert_eq!(count, 1, "exactly one row for the (txn, msg) pair");

        // Cleanup.
        beckn_orders::Entity::delete_by_id(first.id)
            .exec(db.conn())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn quote_is_persisted_and_found_by_order_id() {
        let Some(db) = connect().await else {
            eprintln!("skipping: no DATABASE_URL / beckn_orders table");
            return;
        };
        let txn = ulid_string();
        let msg = ulid_string();
        let order_id = ulid_string();

        let mut init = sample(&txn, &msg);
        init.last_action = "init".to_string();
        init.beckn_order_id = Some(order_id.clone());
        let (row, created) = db.upsert_or_get(init).await.unwrap();
        assert!(created);
        assert_eq!(row.quoted_fare, None);

        // Quote lands after re-pricing (post-insert, like the init handler).
        db.set_quote(&txn, &msg, 450, "SWIFT").await.unwrap();

        // confirm's read path: the init row for this beckn_order_id carries
        // the agreed quote.
        let found = db
            .find_init_by_order_id(&order_id)
            .await
            .unwrap()
            .expect("init row must be findable by beckn_order_id");
        assert_eq!(found.id, row.id);
        assert_eq!(found.quoted_fare, Some(450));
        assert_eq!(found.quoted_item_id.as_deref(), Some("SWIFT"));

        beckn_orders::Entity::delete_by_id(row.id)
            .exec(db.conn())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn fulfillment_lifecycle_on_the_confirm_row() {
        let Some(db) = connect().await else {
            eprintln!("skipping: no DATABASE_URL / beckn_orders table");
            return;
        };
        let txn = ulid_string();
        let order_id = ulid_string();
        let ride_id = ulid_string();

        // The confirm row owns the ride; an init row for the same order must
        // never be touched by ride-keyed lifecycle updates.
        let mut init = sample(&txn, &ulid_string());
        init.last_action = "init".to_string();
        init.beckn_order_id = Some(order_id.clone());
        let (init_row, _) = db.upsert_or_get(init).await.unwrap();

        let mut confirm = sample(&txn, &ulid_string());
        confirm.last_action = "confirm".to_string();
        confirm.beckn_order_id = Some(order_id.clone());
        confirm.ride_id = Some(ride_id.clone());
        confirm.beckn_status = Some("ACTIVE".to_string());
        confirm.fulfillment_state = Some("NEW".to_string());
        let (confirm_row, _) = db.upsert_or_get(confirm).await.unwrap();

        // Both webhook and lifecycle consumers resolve the ride this way.
        let by_ride = db
            .find_confirm_by_ride_id(&ride_id)
            .await
            .unwrap()
            .expect("confirm row must be findable by ride_id");
        assert_eq!(by_ride.id, confirm_row.id);
        let by_order = db
            .find_confirm_by_order_id(&order_id)
            .await
            .unwrap()
            .expect("confirm row must be findable by beckn_order_id");
        assert_eq!(by_order.id, confirm_row.id);

        // Driver assigned: status advances and the snapshot is captured.
        let snapshot = r#"{"request_id":"x","driver_name":"Test"}"#;
        db.set_fulfillment(
            &ride_id,
            "ACTIVE",
            "RIDE_ASSIGNED",
            Some(snapshot.to_string()),
        )
        .await
        .unwrap();
        let row = db.find_confirm_by_ride_id(&ride_id).await.unwrap().unwrap();
        assert_eq!(row.beckn_status.as_deref(), Some("ACTIVE"));
        assert_eq!(row.fulfillment_state.as_deref(), Some("RIDE_ASSIGNED"));
        assert_eq!(row.assigned_json.as_deref(), Some(snapshot));

        // Later lifecycle events pass no snapshot — the assigned driver must
        // survive so on_status keeps reporting the agent.
        db.set_fulfillment(&ride_id, "COMPLETE", "RIDE_ENDED", None)
            .await
            .unwrap();
        let row = db.find_confirm_by_ride_id(&ride_id).await.unwrap().unwrap();
        assert_eq!(row.beckn_status.as_deref(), Some("COMPLETE"));
        assert_eq!(row.fulfillment_state.as_deref(), Some("RIDE_ENDED"));
        assert_eq!(
            row.assigned_json.as_deref(),
            Some(snapshot),
            "assigned_json must be preserved when the update carries none"
        );

        // The init row is untouched by all of it.
        let init_after = db
            .get_beckn_order(&init_row.transaction_id, &init_row.message_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(init_after.beckn_status, init_row.beckn_status);
        assert_eq!(init_after.fulfillment_state, init_row.fulfillment_state);

        for id in [init_row.id, confirm_row.id] {
            beckn_orders::Entity::delete_by_id(id)
                .exec(db.conn())
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn distinct_messages_in_a_transaction_coexist() {
        let Some(db) = connect().await else {
            eprintln!("skipping: no DATABASE_URL / beckn_orders table");
            return;
        };
        let txn = ulid_string();
        let (a, _) =
            db.upsert_or_get(sample(&txn, &ulid_string())).await.unwrap();
        let (b, _) =
            db.upsert_or_get(sample(&txn, &ulid_string())).await.unwrap();
        assert_ne!(a.id, b.id, "different message_ids are different rows");

        beckn_orders::Entity::delete_by_id(a.id).exec(db.conn()).await.unwrap();
        beckn_orders::Entity::delete_by_id(b.id).exec(db.conn()).await.unwrap();
    }
}
