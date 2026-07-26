//! Router assembly. One POST route per Beckn action; each returns ACK/NACK only.

use axum::Router;
use axum::routing::{get, post};

use crate::AppState;
use crate::controllers;
use crate::internal;

/// Build the adapter's Axum router: the eight inbound Beckn action routes,
/// plus the token-gated `/internal/dispatch/*` webhooks the api service
/// pushes dispatch outcomes to (keep `/internal/*` off the public ingress).
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(health))
        .route("/metrics", get(metrics_snapshot))
        .route("/search", post(controllers::search))
        .route("/select", post(controllers::select))
        .route("/init", post(controllers::init))
        .route("/confirm", post(controllers::confirm))
        .route("/status", post(controllers::status))
        .route("/track", post(controllers::track))
        .route("/update", post(controllers::update))
        .route("/cancel", post(controllers::cancel))
        .route(
            "/internal/dispatch/assigned",
            post(internal::dispatch_assigned),
        )
        .route("/internal/dispatch/failed", post(internal::dispatch_failed))
        .with_state(state)
}

async fn health() -> &'static str {
    "beckn-bpp-adapter ok"
}

/// Operational counters as JSON (Step 9). Read-only and unauthenticated —
/// counts only, no order data; keep it off the public ingress with
/// `/internal/*` anyway.
async fn metrics_snapshot(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> axum::Json<serde_json::Value> {
    axum::Json(state.metrics.snapshot())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::{
        CancelOutcome, DiscoveryAdapter, DispatchAdapter, LatLon, Quote,
        QuoteOption, RideDispatch,
    };
    use crate::callbacks::{
        Callback, CallbackClient, LoggingCallbackClient,
        RecordingCallbackClient,
    };
    use crate::config::Config;
    use crate::correlation::MemoryCorrelationStore;
    use crate::crypto::StaticRegistry;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use chrono::Utc;
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt; // for `oneshot`

    /// Discovery double returning a fixed quote.
    struct FakeDiscovery(Quote);

    #[async_trait::async_trait]
    impl DiscoveryAdapter for FakeDiscovery {
        async fn quote(
            &self,
            _pickup: LatLon,
            _dropoff: LatLon,
        ) -> anyhow::Result<Quote> {
            Ok(self.0.clone())
        }
    }

    /// Dispatch double recording every handoff and cancel.
    #[derive(Default)]
    struct FakeDispatch {
        rides: Mutex<Vec<RideDispatch>>,
        cancels: Mutex<Vec<String>>,
        /// Outcome the next `cancel_ride` reports; `None` = DispatchCancelled.
        cancel_outcome: Mutex<Option<CancelOutcome>>,
    }

    #[async_trait::async_trait]
    impl DispatchAdapter for FakeDispatch {
        async fn request_ride(
            &self,
            dispatch: RideDispatch,
        ) -> anyhow::Result<()> {
            self.rides.lock().unwrap().push(dispatch);
            Ok(())
        }

        async fn cancel_ride(
            &self,
            request_id: &str,
        ) -> anyhow::Result<CancelOutcome> {
            self.cancels.lock().unwrap().push(request_id.to_string());
            Ok(self
                .cancel_outcome
                .lock()
                .unwrap()
                .unwrap_or(CancelOutcome::DispatchCancelled))
        }
    }

    fn fake_quote() -> Quote {
        Quote {
            options: vec![
                QuoteOption {
                    tier_code: "SWIFT".into(),
                    tier_name: "Swift".into(),
                    vehicle_category: "CAB",
                    fare: 260.0,
                    base_fare: 100,
                    distance_fare: 120,
                    time_fare: 40,
                    waiting_fare: 0,
                },
                QuoteOption {
                    tier_code: "BIKE".into(),
                    tier_name: "Bike".into(),
                    vehicle_category: "TWO_WHEELER",
                    fare: 120.0,
                    base_fare: 50,
                    distance_fare: 50,
                    time_fare: 20,
                    waiting_fare: 0,
                },
            ],
            distance_km: 5.2,
            duration_s: 780,
        }
    }

    /// Token the tests use for the `/internal/dispatch/*` webhooks.
    const INTERNAL_TOKEN: &str = "test-internal-token";

    /// State with signature verification OFF — exercises the context gate and
    /// the async pipelines. Returns the dispatch recorder for inspection.
    fn state_with_dispatch(
        callbacks: Arc<dyn CallbackClient>,
    ) -> (AppState, Arc<FakeDispatch>) {
        let config = Config {
            beckn_verify_signatures: false,
            beckn_internal_token: Some(INTERNAL_TOKEN.to_string()),
            beckn_tracking_url_template: Some(
                "https://track.test/{ride_id}".to_string(),
            ),
            ..Config::default()
        };
        let dispatch = Arc::new(FakeDispatch::default());
        let state = AppState::new(
            config,
            callbacks,
            Arc::new(StaticRegistry::default()),
            Arc::new(MemoryCorrelationStore::default()),
            Arc::new(FakeDiscovery(fake_quote())),
            dispatch.clone(),
        );
        (state, dispatch)
    }

    fn state_with(callbacks: Arc<dyn CallbackClient>) -> AppState {
        state_with_dispatch(callbacks).0
    }

    fn test_state() -> AppState {
        state_with(Arc::new(LoggingCallbackClient))
    }

    fn search_body(ttl: &str, ts: chrono::DateTime<Utc>) -> String {
        serde_json::json!({
            "context": {
                "domain": "mobility",
                "action": "search",
                "bap_id": "bap.example.com",
                "bap_uri": "https://bap.example.com/beckn",
                "transaction_id": "txn-1",
                "message_id": "msg-1",
                "timestamp": ts.to_rfc3339(),
                "ttl": ttl,
                "version": "2.0.0",
            },
            "message": { "intent": { "fulfillment": { "stops": [
                { "type": "START",
                  "location": { "gps": "-1.286389, 36.817223" } },
                { "type": "END",
                  "location": { "gps": "-1.319167, 36.925833" } }
            ]}}}
        })
        .to_string()
    }

    async fn post_action(
        app: Router,
        path: &str,
        body: String,
    ) -> (StatusCode, serde_json::Value) {
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        let bytes =
            axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        (status, json)
    }

    async fn post_to(
        app: Router,
        body: String,
    ) -> (StatusCode, serde_json::Value) {
        post_action(app, "/search", body).await
    }

    async fn post(body: String) -> (StatusCode, serde_json::Value) {
        post_to(router(test_state()), body).await
    }

    #[tokio::test]
    async fn valid_search_returns_ack() {
        let (status, json) = post(search_body("PT30S", Utc::now())).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "ACK");
        assert!(json.get("error").is_none());
    }

    #[tokio::test]
    async fn expired_ttl_returns_nack() {
        let stale = Utc::now() - chrono::Duration::seconds(120);
        let (status, json) = post(search_body("PT1S", stale)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "NACK");
        assert_eq!(json["error"]["code"], "30008");
    }

    #[tokio::test]
    async fn malformed_context_returns_nack() {
        let bad = r#"{"context":{"action":"search"},"message":{}}"#.to_string();
        let (status, json) = post(bad).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "NACK");
        assert_eq!(json["error"]["code"], "30001");
    }

    // ---- Step 4: search → on_search ----

    fn recording_state()
    -> (AppState, tokio::sync::mpsc::UnboundedReceiver<Callback>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        (state_with(Arc::new(RecordingCallbackClient { tx })), rx)
    }

    #[tokio::test]
    async fn search_delivers_on_search_catalog() {
        let (state, mut rx) = recording_state();
        let (status, json) =
            post_to(router(state), search_body("PT30S", Utc::now())).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "ACK");

        // The callback is produced asynchronously after the ACK.
        let callback =
            tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
                .await
                .expect("on_search not sent within 2s")
                .expect("callback channel closed");

        assert_eq!(callback.action, "on_search");
        assert_eq!(callback.bap_uri, "https://bap.example.com/beckn");

        let ctx = &callback.payload["context"];
        assert_eq!(ctx["action"], "on_search");
        assert_eq!(ctx["transaction_id"], "txn-1");
        assert_eq!(ctx["message_id"], "msg-1");
        assert_eq!(ctx["bpp_id"], "sitwego.mobility.ke");

        let provider = &callback.payload["message"]["catalog"]["providers"][0];
        assert_eq!(provider["id"], "sitwego.mobility.ke");
        assert_eq!(provider["items"].as_array().unwrap().len(), 2);
        assert_eq!(provider["items"][0]["price"]["currency"], "KES");
        assert_eq!(
            provider["fulfillments"][0]["stops"][0]["location"]["gps"],
            "-1.286389, 36.817223"
        );
    }

    #[tokio::test]
    async fn replayed_search_sends_only_one_on_search() {
        let (state, mut rx) = recording_state();
        let app = router(state);
        let body = search_body("PT30S", Utc::now());

        // Same (transaction_id, message_id) twice — both ACK…
        let (s1, j1) = post_to(app.clone(), body.clone()).await;
        let (s2, j2) = post_to(app, body).await;
        assert_eq!((s1, s2), (StatusCode::OK, StatusCode::OK));
        assert_eq!(j1["message"]["ack"]["status"], "ACK");
        assert_eq!(j2["message"]["ack"]["status"], "ACK");

        // …but exactly ONE on_search goes out (Inviolable Rule #4).
        let first =
            tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
                .await
                .expect("first on_search not sent")
                .unwrap();
        assert_eq!(first.action, "on_search");
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        assert!(
            rx.try_recv().is_err(),
            "replayed search must not produce a second on_search"
        );
    }

    #[tokio::test]
    async fn search_without_stops_is_nacked() {
        let mut body: serde_json::Value =
            serde_json::from_str(&search_body("PT30S", Utc::now())).unwrap();
        body["message"] = serde_json::json!({ "intent": {} });
        let (status, json) = post(body.to_string()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "NACK");
        assert_eq!(json["error"]["code"], "30001");
    }

    // ---- Step 5: select/init → on_select/on_init ----

    fn order_body(action: &str, item_id: &str, msg_id: &str) -> String {
        serde_json::json!({
            "context": {
                "domain": "mobility",
                "action": action,
                "bap_id": "bap.example.com",
                "bap_uri": "https://bap.example.com/beckn",
                "transaction_id": "txn-1",
                "message_id": msg_id,
                "timestamp": Utc::now().to_rfc3339(),
                "ttl": "PT30S",
                "version": "2.0.0",
            },
            "message": { "order": {
                "items": [{ "id": item_id }],
                "fulfillments": [{ "stops": [
                    { "type": "START",
                      "location": { "gps": "-1.286389, 36.817223" } },
                    { "type": "END",
                      "location": { "gps": "-1.319167, 36.925833" } }
                ]}]
            }}
        })
        .to_string()
    }

    async fn next_callback(
        rx: &mut tokio::sync::mpsc::UnboundedReceiver<Callback>,
    ) -> Callback {
        tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
            .await
            .expect("callback not sent within 2s")
            .expect("callback channel closed")
    }

    #[tokio::test]
    async fn select_delivers_on_select_with_quote_and_payment() {
        let (state, mut rx) = recording_state();
        let (status, json) = post_action(
            router(state),
            "/select",
            order_body("select", "SWIFT", "msg-sel-1"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "ACK");

        let callback = next_callback(&mut rx).await;
        assert_eq!(callback.action, "on_select");
        assert_eq!(callback.payload["context"]["action"], "on_select");

        let order = &callback.payload["message"]["order"];
        // No durable order id at select time.
        assert!(order.get("id").is_none());
        assert_eq!(order["provider"]["id"], "sitwego.mobility.ke");
        assert_eq!(order["items"][0]["id"], "SWIFT");
        assert_eq!(order["quote"]["price"]["value"], "260");
        let titles: Vec<&str> = order["quote"]["breakup"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b["title"].as_str().unwrap())
            .collect();
        assert_eq!(
            titles,
            ["BASE_FARE", "DISTANCE_FARE", "RIDE_DURATION_FARE"]
        );
        // Settlement decision on the wire: rider pays driver directly.
        let payment = &order["payments"][0];
        assert_eq!(payment["collected_by"], "BPP");
        assert_eq!(payment["type"], "ON-FULFILLMENT");
        assert_eq!(payment["status"], "NOT-PAID");
    }

    #[tokio::test]
    async fn init_delivers_on_init_with_order_id() {
        let (state, mut rx) = recording_state();
        let (status, json) = post_action(
            router(state),
            "/init",
            order_body("init", "BIKE", "msg-init-1"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "ACK");

        let callback = next_callback(&mut rx).await;
        assert_eq!(callback.action, "on_init");
        let order = &callback.payload["message"]["order"];
        // on_init mints the durable Beckn order id confirm will act on.
        assert!(order["id"].as_str().is_some_and(|id| !id.is_empty()));
        assert_eq!(order["items"][0]["id"], "BIKE");
        assert_eq!(order["payments"][0]["collected_by"], "BPP");
    }

    #[tokio::test]
    async fn selecting_unavailable_item_sends_error_callback() {
        let (state, mut rx) = recording_state();
        let (status, json) = post_action(
            router(state),
            "/select",
            order_body("select", "EXECUTIVE", "msg-sel-2"),
        )
        .await;
        // Still ACKed — the failure is discovered after re-pricing, so it
        // travels asynchronously as `{context, error}`.
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "ACK");

        let callback = next_callback(&mut rx).await;
        assert_eq!(callback.action, "on_select");
        assert!(callback.payload.get("message").is_none());
        assert!(
            callback.payload["error"]["message"]
                .as_str()
                .unwrap()
                .contains("EXECUTIVE")
        );
    }

    #[tokio::test]
    async fn replayed_select_sends_only_one_on_select() {
        let (state, mut rx) = recording_state();
        let app = router(state);
        let body = order_body("select", "SWIFT", "msg-sel-3");

        let (s1, _) = post_action(app.clone(), "/select", body.clone()).await;
        let (s2, _) = post_action(app, "/select", body).await;
        assert_eq!((s1, s2), (StatusCode::OK, StatusCode::OK));

        let first = next_callback(&mut rx).await;
        assert_eq!(first.action, "on_select");
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        assert!(
            rx.try_recv().is_err(),
            "replayed select must not produce a second on_select"
        );
    }

    #[tokio::test]
    async fn select_without_order_is_nacked() {
        let mut body: serde_json::Value =
            serde_json::from_str(&order_body("select", "SWIFT", "msg-sel-4"))
                .unwrap();
        body["message"] = serde_json::json!({});
        let (status, json) =
            post_action(router(test_state()), "/select", body.to_string())
                .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "NACK");
        assert_eq!(json["error"]["code"], "30001");
    }

    // ---- Step 6: confirm → dispatch handoff + on_confirm ----

    fn confirm_body(order_id: &str, msg_id: &str) -> String {
        serde_json::json!({
            "context": {
                "domain": "mobility",
                "action": "confirm",
                "bap_id": "bap.example.com",
                "bap_uri": "https://bap.example.com/beckn",
                "transaction_id": "txn-1",
                "message_id": msg_id,
                "timestamp": Utc::now().to_rfc3339(),
                "ttl": "PT30S",
                "version": "2.0.0",
            },
            "message": { "order": {
                "id": order_id,
                "items": [{ "id": "SWIFT" }],
                "fulfillments": [{
                    "customer": {
                        "person": { "name": "Amina Wanjiru" },
                        "contact": { "phone": "+254700000001" }
                    },
                    "stops": [
                        { "type": "START",
                          "location": { "gps": "-1.286389, 36.817223" } },
                        { "type": "END",
                          "location": { "gps": "-1.319167, 36.925833" } }
                    ]
                }]
            }}
        })
        .to_string()
    }

    /// Run init on the app and return the order id minted in on_init.
    async fn init_order(
        app: Router,
        rx: &mut tokio::sync::mpsc::UnboundedReceiver<Callback>,
    ) -> String {
        let (status, json) = post_action(
            app,
            "/init",
            order_body("init", "SWIFT", "msg-init-pre"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "ACK");
        let callback = next_callback(rx).await;
        assert_eq!(callback.action, "on_init");
        callback.payload["message"]["order"]["id"]
            .as_str()
            .expect("on_init carries the order id")
            .to_string()
    }

    #[tokio::test]
    async fn confirm_dispatches_and_delivers_on_confirm() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let (state, dispatch) =
            state_with_dispatch(Arc::new(RecordingCallbackClient { tx }));
        let app = router(state);

        let order_id = init_order(app.clone(), &mut rx).await;

        let (status, json) = post_action(
            app,
            "/confirm",
            confirm_body(&order_id, "msg-confirm-1"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "ACK");

        let callback = next_callback(&mut rx).await;
        assert_eq!(callback.action, "on_confirm");
        let order = &callback.payload["message"]["order"];
        assert_eq!(order["id"], order_id.as_str());
        // Decision B: confirmed + allocating; driver comes via on_status later.
        assert_eq!(order["status"], "ACTIVE");
        assert_eq!(
            order["fulfillments"][0]["state"]["descriptor"]["code"],
            "NEW"
        );
        // The init-agreed fare is honoured, not re-priced.
        assert_eq!(order["quote"]["price"]["value"], "260");

        // Exactly one dispatch handoff, carrying the stored quote and the
        // real customer.
        let rides = dispatch.rides.lock().unwrap();
        assert_eq!(rides.len(), 1);
        assert_eq!(rides[0].fare, 260);
        assert_eq!(rides[0].tier_code, "SWIFT");
        assert_eq!(rides[0].customer_name, "Amina Wanjiru");
        assert_eq!(rides[0].customer_phone, "+254700000001");
        assert!(!rides[0].request_id.is_empty());
    }

    #[tokio::test]
    async fn replayed_confirm_never_dispatches_twice() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let (state, dispatch) =
            state_with_dispatch(Arc::new(RecordingCallbackClient { tx }));
        let app = router(state);

        let order_id = init_order(app.clone(), &mut rx).await;
        let body = confirm_body(&order_id, "msg-confirm-2");

        let (s1, _) = post_action(app.clone(), "/confirm", body.clone()).await;
        let (s2, _) = post_action(app, "/confirm", body).await;
        assert_eq!((s1, s2), (StatusCode::OK, StatusCode::OK));

        let first = next_callback(&mut rx).await;
        assert_eq!(first.action, "on_confirm");
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        assert!(rx.try_recv().is_err(), "no second on_confirm");

        // Rule #4, the critical property: ONE driver claim per confirm.
        assert_eq!(dispatch.rides.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn confirm_with_unknown_order_id_sends_error_and_no_dispatch() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let (state, dispatch) =
            state_with_dispatch(Arc::new(RecordingCallbackClient { tx }));
        let app = router(state);

        let (status, json) = post_action(
            app,
            "/confirm",
            confirm_body("01STRANGER", "msg-confirm-3"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "ACK");

        let callback = next_callback(&mut rx).await;
        assert_eq!(callback.action, "on_confirm");
        assert!(callback.payload.get("message").is_none());
        assert!(
            callback.payload["error"]["message"]
                .as_str()
                .unwrap()
                .contains("01STRANGER")
        );
        assert!(dispatch.rides.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn confirm_without_order_id_is_nacked() {
        let mut body: serde_json::Value =
            serde_json::from_str(&confirm_body("01X", "msg-confirm-4"))
                .unwrap();
        body["message"]["order"].as_object_mut().unwrap().remove("id");
        let (status, json) =
            post_action(router(test_state()), "/confirm", body.to_string())
                .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "NACK");
        assert_eq!(json["error"]["code"], "30001");
    }

    // ---- Step 7: status / track / cancel / update + dispatch webhooks ----

    fn order_ref_body(action: &str, order_id: &str, msg_id: &str) -> String {
        serde_json::json!({
            "context": {
                "domain": "mobility",
                "action": action,
                "bap_id": "bap.example.com",
                "bap_uri": "https://bap.example.com/beckn",
                "transaction_id": "txn-1",
                "message_id": msg_id,
                "timestamp": Utc::now().to_rfc3339(),
                "ttl": "PT30S",
                "version": "2.0.0",
            },
            "message": { "order_id": order_id }
        })
        .to_string()
    }

    /// Full quote→book flow: init then confirm. Returns the order id and the
    /// adapter-minted ride id (both callbacks are consumed).
    async fn confirmed_order(
        app: Router,
        rx: &mut tokio::sync::mpsc::UnboundedReceiver<Callback>,
        dispatch: &FakeDispatch,
    ) -> (String, String) {
        let order_id = init_order(app.clone(), rx).await;
        let (status, json) = post_action(
            app,
            "/confirm",
            confirm_body(&order_id, "msg-confirm-pre"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "ACK");
        let callback = next_callback(rx).await;
        assert_eq!(callback.action, "on_confirm");
        let ride_id = dispatch.rides.lock().unwrap()[0].request_id.clone();
        (order_id, ride_id)
    }

    async fn post_webhook(
        app: Router,
        path: &str,
        token: Option<&str>,
        body: serde_json::Value,
    ) -> StatusCode {
        let mut req = Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json");
        if let Some(token) = token {
            req = req.header("x-internal-token", token);
        }
        app.oneshot(req.body(Body::from(body.to_string())).unwrap())
            .await
            .unwrap()
            .status()
    }

    fn assigned_payload(ride_id: &str) -> serde_json::Value {
        serde_json::json!({
            "request_id": ride_id,
            "driver_id": "01DRIVER",
            "driver_name": "Otieno K",
            "driver_phone": "+254711000111",
            "vehicle_plate": "KDA 123X",
            "vehicle_model": "Vitz",
            "vehicle_color": "Silver",
            "otp": "4471",
        })
    }

    #[tokio::test]
    async fn status_replays_persisted_state() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let (state, dispatch) =
            state_with_dispatch(Arc::new(RecordingCallbackClient { tx }));
        let app = router(state);
        let (order_id, ride_id) =
            confirmed_order(app.clone(), &mut rx, &dispatch).await;

        let (status, json) = post_action(
            app,
            "/status",
            order_ref_body("status", &order_id, "msg-status-1"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "ACK");

        let callback = next_callback(&mut rx).await;
        assert_eq!(callback.action, "on_status");
        let order = &callback.payload["message"]["order"];
        assert_eq!(order["id"], order_id.as_str());
        assert_eq!(order["status"], "ACTIVE");
        // Still allocating: no driver has been announced yet.
        assert_eq!(
            order["fulfillments"][0]["state"]["descriptor"]["code"],
            "NEW"
        );
        assert_eq!(order["fulfillments"][0]["id"], ride_id.as_str());
        // The trip persisted at confirm is repeated.
        assert_eq!(
            order["fulfillments"][0]["stops"][0]["location"]["gps"],
            "-1.286389, 36.817223"
        );
        // The init-agreed fare came from the confirm row, not a re-price.
        assert_eq!(order["quote"]["price"]["value"], "260");
    }

    #[tokio::test]
    async fn status_of_unknown_order_sends_error_callback() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let (state, _) =
            state_with_dispatch(Arc::new(RecordingCallbackClient { tx }));
        let (status, json) = post_action(
            router(state),
            "/status",
            order_ref_body("status", "01STRANGER", "msg-status-2"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "ACK");
        let callback = next_callback(&mut rx).await;
        assert_eq!(callback.action, "on_status");
        assert!(
            callback.payload["error"]["message"]
                .as_str()
                .unwrap()
                .contains("01STRANGER")
        );
    }

    #[tokio::test]
    async fn assigned_webhook_pushes_unsolicited_ride_assigned() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let (state, dispatch) =
            state_with_dispatch(Arc::new(RecordingCallbackClient { tx }));
        let app = router(state);
        let (order_id, ride_id) =
            confirmed_order(app.clone(), &mut rx, &dispatch).await;

        let status = post_webhook(
            app.clone(),
            "/internal/dispatch/assigned",
            Some(INTERNAL_TOKEN),
            assigned_payload(&ride_id),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        // The unsolicited on_status: same transaction, FRESH message id.
        let callback = next_callback(&mut rx).await;
        assert_eq!(callback.action, "on_status");
        let ctx = &callback.payload["context"];
        assert_eq!(ctx["transaction_id"], "txn-1");
        assert_ne!(ctx["message_id"], "msg-confirm-pre");
        let order = &callback.payload["message"]["order"];
        assert_eq!(order["id"], order_id.as_str());
        assert_eq!(order["status"], "ACTIVE");
        let f = &order["fulfillments"][0];
        assert_eq!(f["state"]["descriptor"]["code"], "RIDE_ASSIGNED");
        assert_eq!(f["agent"]["person"]["name"], "Otieno K");
        assert_eq!(f["agent"]["contact"]["phone"], "+254711000111");
        assert_eq!(f["vehicle"]["registration"], "KDA 123X");
        assert_eq!(f["stops"][0]["authorization"]["token"], "4471");

        // The state persisted: a later `status` replays RIDE_ASSIGNED.
        let (_, json) = post_action(
            app,
            "/status",
            order_ref_body("status", &order_id, "msg-status-3"),
        )
        .await;
        assert_eq!(json["message"]["ack"]["status"], "ACK");
        let replay = next_callback(&mut rx).await;
        assert_eq!(
            replay.payload["message"]["order"]["fulfillments"][0]["state"]["descriptor"]
                ["code"],
            "RIDE_ASSIGNED"
        );
    }

    #[tokio::test]
    async fn dispatch_webhooks_require_the_internal_token() {
        let (state, _) = state_with_dispatch(Arc::new(LoggingCallbackClient));
        let app = router(state);
        let no_token = post_webhook(
            app.clone(),
            "/internal/dispatch/assigned",
            None,
            assigned_payload("01X"),
        )
        .await;
        assert_eq!(no_token, StatusCode::UNAUTHORIZED);
        let bad_token = post_webhook(
            app.clone(),
            "/internal/dispatch/assigned",
            Some("wrong"),
            assigned_payload("01X"),
        )
        .await;
        assert_eq!(bad_token, StatusCode::UNAUTHORIZED);
        // Right token but a ride no beckn order owns.
        let unknown = post_webhook(
            app,
            "/internal/dispatch/assigned",
            Some(INTERNAL_TOKEN),
            assigned_payload("01NOBODY"),
        )
        .await;
        assert_eq!(unknown, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn failed_webhook_pushes_on_cancel_by_provider() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let (state, dispatch) =
            state_with_dispatch(Arc::new(RecordingCallbackClient { tx }));
        let app = router(state);
        let (order_id, ride_id) =
            confirmed_order(app.clone(), &mut rx, &dispatch).await;

        let status = post_webhook(
            app,
            "/internal/dispatch/failed",
            Some(INTERNAL_TOKEN),
            serde_json::json!({
                "request_id": ride_id,
                "reason": "no driver accepted the booking",
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let callback = next_callback(&mut rx).await;
        assert_eq!(callback.action, "on_cancel");
        let order = &callback.payload["message"]["order"];
        assert_eq!(order["id"], order_id.as_str());
        assert_eq!(order["status"], "CANCELLED");
        assert_eq!(order["cancellation"]["cancelled_by"], "PROVIDER");
        assert_eq!(
            order["fulfillments"][0]["state"]["descriptor"]["code"],
            "RIDE_CANCELLED"
        );
    }

    #[tokio::test]
    async fn cancel_cancels_dispatch_and_delivers_on_cancel() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let (state, dispatch) =
            state_with_dispatch(Arc::new(RecordingCallbackClient { tx }));
        let app = router(state);
        let (order_id, ride_id) =
            confirmed_order(app.clone(), &mut rx, &dispatch).await;

        let body = order_ref_body("cancel", &order_id, "msg-cancel-1");
        let (status, json) =
            post_action(app.clone(), "/cancel", body.clone()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "ACK");

        let callback = next_callback(&mut rx).await;
        assert_eq!(callback.action, "on_cancel");
        let order = &callback.payload["message"]["order"];
        assert_eq!(order["status"], "CANCELLED");
        assert_eq!(order["cancellation"]["cancelled_by"], "CONSUMER");
        assert_eq!(dispatch.cancels.lock().unwrap().as_slice(), [ride_id]);

        // A replay is idempotent: ACK, but no second cancel or callback.
        let (s2, _) = post_action(app, "/cancel", body).await;
        assert_eq!(s2, StatusCode::OK);
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        assert!(rx.try_recv().is_err(), "no second on_cancel");
        assert_eq!(dispatch.cancels.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn cancel_of_in_progress_ride_sends_error_callback() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let (state, dispatch) =
            state_with_dispatch(Arc::new(RecordingCallbackClient { tx }));
        let app = router(state);
        let (order_id, _) =
            confirmed_order(app.clone(), &mut rx, &dispatch).await;
        *dispatch.cancel_outcome.lock().unwrap() =
            Some(CancelOutcome::ActiveRide);

        let (_, json) = post_action(
            app,
            "/cancel",
            order_ref_body("cancel", &order_id, "msg-cancel-2"),
        )
        .await;
        assert_eq!(json["message"]["ack"]["status"], "ACK");
        let callback = next_callback(&mut rx).await;
        assert_eq!(callback.action, "on_cancel");
        assert!(
            callback.payload["error"]["message"]
                .as_str()
                .unwrap()
                .contains("in progress")
        );
    }

    #[tokio::test]
    async fn track_returns_a_tracking_url_never_gps() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let (state, dispatch) =
            state_with_dispatch(Arc::new(RecordingCallbackClient { tx }));
        let app = router(state);
        let (order_id, ride_id) =
            confirmed_order(app.clone(), &mut rx, &dispatch).await;

        let (_, json) = post_action(
            app,
            "/track",
            order_ref_body("track", &order_id, "msg-track-1"),
        )
        .await;
        assert_eq!(json["message"]["ack"]["status"], "ACK");

        let callback = next_callback(&mut rx).await;
        assert_eq!(callback.action, "on_track");
        let tracking = &callback.payload["message"]["tracking"];
        assert_eq!(tracking["url"], format!("https://track.test/{ride_id}"));
        assert_eq!(tracking["status"], "ACTIVE");
        // Rule #5: a URL, never coordinates.
        assert!(tracking.get("location").is_none());
    }

    #[tokio::test]
    async fn track_without_template_sends_error_callback() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let dispatch = Arc::new(FakeDispatch::default());
        let state = AppState::new(
            Config {
                beckn_verify_signatures: false,
                beckn_internal_token: Some(INTERNAL_TOKEN.to_string()),
                beckn_tracking_url_template: None,
                ..Config::default()
            },
            Arc::new(RecordingCallbackClient { tx }),
            Arc::new(StaticRegistry::default()),
            Arc::new(MemoryCorrelationStore::default()),
            Arc::new(FakeDiscovery(fake_quote())),
            dispatch.clone(),
        );
        let app = router(state);
        let (order_id, _) =
            confirmed_order(app.clone(), &mut rx, &dispatch).await;

        let (_, json) = post_action(
            app,
            "/track",
            order_ref_body("track", &order_id, "msg-track-2"),
        )
        .await;
        assert_eq!(json["message"]["ack"]["status"], "ACK");
        let callback = next_callback(&mut rx).await;
        assert_eq!(callback.action, "on_track");
        assert!(
            callback.payload["error"]["message"]
                .as_str()
                .unwrap()
                .contains("not available")
        );
    }

    #[tokio::test]
    async fn update_is_nacked_as_unsupported() {
        let (status, json) = post_action(
            router(test_state()),
            "/update",
            order_ref_body("update", "01ORDER", "msg-update-1"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "NACK");
        assert!(
            json["error"]["message"]
                .as_str()
                .unwrap()
                .contains("not supported")
        );
    }

    #[tokio::test]
    async fn status_without_order_id_is_nacked() {
        let mut body: serde_json::Value = serde_json::from_str(
            &order_ref_body("status", "01X", "msg-status-4"),
        )
        .unwrap();
        body["message"] = serde_json::json!({});
        let (status, json) =
            post_action(router(test_state()), "/status", body.to_string())
                .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"]["ack"]["status"], "NACK");
        assert_eq!(json["error"]["code"], "30001");
    }

    // ---- Step 3: signature verification over the raw request bytes ----

    mod signed {
        use super::*;
        use base64::Engine;
        use base64::engine::general_purpose::STANDARD as B64;
        use ed25519_dalek::SigningKey;

        const BAP: &str = "bap.example.com";
        const UKID: &str = "bap-key-1";

        fn bap_key() -> SigningKey {
            SigningKey::from_bytes(&[42u8; 32])
        }

        /// State with verification ON and the test BAP's key registered.
        fn verifying_state() -> AppState {
            let registry = StaticRegistry::default().with_key(
                BAP,
                UKID,
                &B64.encode(bap_key().verifying_key().to_bytes()),
            );
            AppState::new(
                Config::default(), // beckn_verify_signatures: true
                Arc::new(LoggingCallbackClient),
                Arc::new(registry),
                Arc::new(MemoryCorrelationStore::default()),
                Arc::new(FakeDiscovery(fake_quote())),
                Arc::new(FakeDispatch::default()),
            )
        }

        fn authorization(body: &str, key: &SigningKey) -> String {
            let now = Utc::now().timestamp();
            crate::crypto::signature::sign(
                body.as_bytes(),
                BAP,
                UKID,
                key,
                now - 5,
                now + 60,
            )
        }

        async fn post_with_auth(
            body: String,
            auth: Option<String>,
        ) -> (StatusCode, Option<serde_json::Value>) {
            let app = router(verifying_state());
            let mut req = Request::builder()
                .method("POST")
                .uri("/search")
                .header("content-type", "application/json");
            if let Some(auth) = auth {
                req = req.header("authorization", auth);
            }
            let resp =
                app.oneshot(req.body(Body::from(body)).unwrap()).await.unwrap();
            let status = resp.status();
            let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap();
            (status, serde_json::from_slice(&bytes).ok())
        }

        #[tokio::test]
        async fn signed_request_is_acked() {
            let body = search_body("PT30S", Utc::now());
            let auth = authorization(&body, &bap_key());
            let (status, json) = post_with_auth(body, Some(auth)).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(json.unwrap()["message"]["ack"]["status"], "ACK");
        }

        #[tokio::test]
        async fn missing_signature_is_401() {
            let (status, _) =
                post_with_auth(search_body("PT30S", Utc::now()), None).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
        }

        #[tokio::test]
        async fn tampered_body_is_401() {
            let body = search_body("PT30S", Utc::now());
            let auth = authorization(&body, &bap_key());
            let tampered = body.replace("txn-1", "txn-2");
            let (status, _) = post_with_auth(tampered, Some(auth)).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
        }

        #[tokio::test]
        async fn unknown_subscriber_is_401() {
            // Valid signature, but signed by a key the registry doesn't know.
            let body = search_body("PT30S", Utc::now());
            let now = Utc::now().timestamp();
            let auth = crate::crypto::signature::sign(
                body.as_bytes(),
                "stranger.example.com",
                "k1",
                &SigningKey::from_bytes(&[9u8; 32]),
                now - 5,
                now + 60,
            );
            let (status, _) = post_with_auth(body, Some(auth)).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
        }

        #[tokio::test]
        async fn rejection_carries_www_authenticate_challenge() {
            let app = router(verifying_state());
            let resp = app
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/search")
                        .header("content-type", "application/json")
                        .body(Body::from(search_body("PT30S", Utc::now())))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            let challenge = resp
                .headers()
                .get("www-authenticate")
                .unwrap()
                .to_str()
                .unwrap();
            assert!(challenge.starts_with("Signature realm="));
            assert!(challenge.contains("(created) (expires) digest"));
        }

        #[tokio::test]
        async fn bap_id_must_match_signing_subscriber() {
            // Properly signed by BAP, but the context claims another bap_id.
            let body = search_body("PT30S", Utc::now())
                .replace("bap.example.com", "victim.example.com");
            let auth = authorization(&body, &bap_key());
            let (status, json) = post_with_auth(body, Some(auth)).await;
            assert_eq!(status, StatusCode::OK);
            let json = json.unwrap();
            assert_eq!(json["message"]["ack"]["status"], "NACK");
            assert_eq!(json["error"]["code"], "30001");
        }
    }
}
