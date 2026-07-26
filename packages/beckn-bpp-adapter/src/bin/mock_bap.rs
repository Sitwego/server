//! Mock BAP — Step 10 rollout harness. **Dev/rollout tool only, not part of
//! the production adapter.**
//!
//! A throwaway Beckn Application Platform that exercises the real network path
//! end to end:
//!
//! 1. builds a `search` and signs the RAW body with the mock BAP's Ed25519 key,
//!    reusing the adapter's own signing profile (`crypto::signature::sign`) so
//!    the `Authorization` is byte-for-byte what a real BAP would send;
//! 2. POSTs it to the gateway's `/bg/search`;
//! 3. runs an HTTP server to receive the adapter's asynchronous `on_search`
//!    (Beckn callbacks land at `{bap_uri}/{on_action}`), and prints the catalog;
//! 4. with `FLOW=full` (Phase 2, the default) it then books the first catalog
//!    item straight against the BPP: `select` → `on_select`, `init` →
//!    `on_init` (order id), `confirm` → `on_confirm`, and finally waits for
//!    the unsolicited `on_status` announcing the assigned driver.
//!
//! ```bash
//! BAP_SIGNING_PRIVATE_KEY=... \
//! GATEWAY_URL=http://127.0.0.1:4030/bg \
//! cargo run -p beckn-bpp-adapter --bin mock_bap
//! ```
//!
//! For the gateway to accept the search, the mock BAP must be a registered +
//! SUBSCRIBED subscriber (the gateway verifies the BAP signature via the
//! registry). Register it with a `type=BAP` record, then approve it.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context as _, anyhow, bail};
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use beckn_bpp_adapter::crypto::signature::{sign, signing_key_from_b64};
use chrono::Utc;
use ed25519_dalek::SigningKey;
use serde_json::{Value, json};
use tokio::sync::mpsc;

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Everything needed to speak signed Beckn as the mock BAP.
struct Bap {
    subscriber_id: String,
    unique_key_id: String,
    bap_uri: String,
    domain: String,
    version: String,
    signing_key: SigningKey,
    transaction_id: String,
    client: reqwest::Client,
}

impl Bap {
    /// Serialize the body ONCE, sign those exact bytes, POST them to
    /// `{base}/{action}`, and return the raw `(status, body)`. Never
    /// re-serialized between signing and sending.
    async fn post_signed_raw(
        &self,
        base: &str,
        action: &str,
        body: &Value,
    ) -> anyhow::Result<(reqwest::StatusCode, String)> {
        let raw = serde_json::to_vec(body)?;
        let now = Utc::now().timestamp();
        let auth = sign(
            &raw,
            &self.subscriber_id,
            &self.unique_key_id,
            &self.signing_key,
            now - 5,
            now + 600,
        );
        let url = format!("{}/{action}", base.trim_end_matches('/'));
        tracing::info!("POST {url}");
        let resp = self
            .client
            .post(&url)
            .header("Authorization", &auth)
            .header("Content-Type", "application/json")
            .body(raw)
            .send()
            .await
            .with_context(|| format!("{action}: {base} unreachable"))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        tracing::info!("{action} answered HTTP {status}: {text}");
        Ok((status, text))
    }

    /// `post_signed_raw`, failing unless the target ACKed with a 2xx.
    async fn post_signed(
        &self,
        base: &str,
        action: &str,
        body: &Value,
    ) -> anyhow::Result<()> {
        let (status, text) = self.post_signed_raw(base, action, body).await?;
        if !status.is_success() {
            bail!("{action} was not ACKed (HTTP {status}): {text}");
        }
        Ok(())
    }

    /// A fresh context for `action` under transaction `txn`. `bpp` carries
    /// `(bpp_id, bpp_uri)` for the direct (post-discovery) legs.
    fn context(
        &self,
        action: &str,
        txn: &str,
        bpp: Option<(&str, &str)>,
    ) -> Value {
        let mut context = json!({
            "domain": self.domain,
            "action": action,
            "version": self.version,
            "bap_id": self.subscriber_id,
            "bap_uri": self.bap_uri,
            "transaction_id": txn,
            "message_id": uuid::Uuid::new_v4().to_string(),
            "timestamp": Utc::now().to_rfc3339(),
            "ttl": "PT30S",
        });
        if let Some((bpp_id, bpp_uri)) = bpp {
            context["bpp_id"] = json!(bpp_id);
            context["bpp_uri"] = json!(bpp_uri);
        }
        context
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt().with_target(false).init();

    let subscriber_id = env_or("BAP_SUBSCRIBER_ID", "bap.mock.ke");
    let unique_key_id = env_or("BAP_UNIQUE_KEY_ID", "bap-mock-key-1");
    let port: u16 = env_or("BAP_PORT", "9095").parse().context("BAP_PORT")?;
    let bap_uri = env_or("BAP_URI", &format!("http://127.0.0.1:{port}"));
    let gateway = env_or("GATEWAY_URL", "http://127.0.0.1:4030/bg");
    let domain = env_or("BECKN_DOMAIN", "mobility");
    let version = env_or("BECKN_CORE_VERSION", "2.0.0");
    let pickup = env_or("PICKUP_GPS", "-1.286389, 36.817223");
    let dropoff = env_or("DROPOFF_GPS", "-1.319167, 36.925833");
    let country = env_or("BECKN_COUNTRY", "KEN");
    // The founding-network Nairobi code (John, 2026-07-06): KE + county 047.
    let city = env_or("BECKN_CITY", "KE-047");
    let wait_secs: u64 = env_or("WAIT_SECS", "20").parse().unwrap_or(20);
    // "search" stops after on_search (the Phase 1 behaviour); "full" books.
    let flow = env_or("FLOW", "full");
    // How long to wait for the unsolicited on_status RIDE_ASSIGNED — covers
    // real dispatch time (offer TTL 20 s per candidate).
    let assign_wait: u64 =
        env_or("ASSIGN_WAIT_SECS", "180").parse().unwrap_or(180);
    let customer_name = env_or("CUSTOMER_NAME", "Amina Wanjiru");
    let customer_phone = env_or("CUSTOMER_PHONE", "+254700000001");

    let signing_private = std::env::var("BAP_SIGNING_PRIVATE_KEY").context(
        "BAP_SIGNING_PRIVATE_KEY is required — mint one with the keygen bin",
    )?;
    let signing_key = signing_key_from_b64(&signing_private)?;

    let bap = Bap {
        subscriber_id,
        unique_key_id,
        bap_uri,
        domain,
        version,
        signing_key,
        transaction_id: uuid::Uuid::new_v4().to_string(),
        client: reqwest::Client::new(),
    };

    let defaults = Defaults {
        pickup: pickup.clone(),
        dropoff: dropoff.clone(),
        country: country.clone(),
        city: city.clone(),
        customer_name: customer_name.clone(),
        customer_phone: customer_phone.clone(),
    };

    // FLOW=serve: stay up as a signing companion for Postman/curl instead of
    // running a one-shot flow.
    if flow == "serve" {
        return serve(bap, gateway, port, defaults).await;
    }

    // --- callback receiver: capture the adapter's on_* callbacks ---
    let (tx, mut rx) = mpsc::unbounded_channel::<Value>();
    let app = Router::new()
        // Beckn callbacks land at {bap_uri}/{on_action} — a single dynamic
        // segment catches on_search (and any other on_* that shows up).
        .route("/{action}", post(on_callback))
        .with_state(tx);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding BAP receiver on {addr}"))?;
    tracing::info!("mock BAP receiver on http://{addr} (awaiting callbacks)");
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("BAP receiver crashed: {e}");
        }
    });

    // --- 1. search, through the gateway ---
    let mut context = bap.context("search", &bap.transaction_id, None);
    // The gateway scopes its BPP multicast by `location.city.code` when present
    // (a BPP only receives searches for a city its operating region covers).
    // BECKN_CITY="" omits the city (and location) so the gateway falls back to
    // country+domain routing — useful before city operating-regions are seeded.
    if !country.is_empty() || !city.is_empty() {
        let mut location = serde_json::Map::new();
        if !country.is_empty() {
            location.insert("country".into(), json!({ "code": country }));
        }
        if !city.is_empty() {
            location.insert("city".into(), json!({ "code": city }));
        }
        context["location"] = Value::Object(location);
    }
    let stops = json!([
        { "type": "START", "location": { "gps": pickup } },
        { "type": "END", "location": { "gps": dropoff } },
    ]);
    let search = json!({
        "context": context,
        "message": { "intent": { "fulfillment": { "stops": stops } } }
    });
    tracing::info!(
        "search: txn={} pickup=({pickup}) dropoff=({dropoff})",
        bap.transaction_id
    );
    bap.post_signed(&gateway, "search", &search).await?;

    let on_search = wait_for(&mut rx, "on_search", wait_secs).await?;
    println!("\n===== on_search received =====");
    println!("{}", serde_json::to_string_pretty(&on_search)?);
    summarize(&on_search);

    if flow != "full" {
        return Ok(());
    }

    // --- 2. book the first catalog item, straight against the BPP ---
    let bpp_id = on_search["context"]["bpp_id"]
        .as_str()
        .context("on_search context.bpp_id missing")?
        .to_string();
    // The registry-facing bpp_uri may only resolve from inside Docker
    // (host.docker.internal); BPP_URI overrides it for host-side runs.
    let bpp_uri = std::env::var("BPP_URI").unwrap_or_else(|_| {
        on_search["context"]["bpp_uri"].as_str().unwrap_or_default().to_string()
    });
    if bpp_uri.is_empty() {
        bail!("no bpp_uri in on_search and no BPP_URI override");
    }
    let provider = &on_search["message"]["catalog"]["providers"][0];
    let provider_id = provider["id"]
        .as_str()
        .context("on_search has no provider")?
        .to_string();
    let item_id = match provider["items"][0]["id"].as_str() {
        Some(id) => id.to_string(),
        None => bail!(
            "catalog is EMPTY — no driver online near the pickup; put one \
             online and retry"
        ),
    };
    tracing::info!("booking item `{item_id}` from {provider_id} at {bpp_uri}");

    let order_core = json!({
        "provider": { "id": provider_id },
        "items": [ { "id": item_id } ],
        "fulfillments": [ { "id": item_id, "stops": stops } ],
    });

    // select → on_select
    let select = json!({
        "context": bap.context("select", &bap.transaction_id, Some((&bpp_id, &bpp_uri))),
        "message": { "order": order_core },
    });
    bap.post_signed(&bpp_uri, "select", &select).await?;
    let on_select = wait_for(&mut rx, "on_select", wait_secs).await?;
    print_quote("on_select", &on_select);

    // init → on_init (mints the durable order id)
    let init = json!({
        "context": bap.context("init", &bap.transaction_id, Some((&bpp_id, &bpp_uri))),
        "message": { "order": order_core },
    });
    bap.post_signed(&bpp_uri, "init", &init).await?;
    let on_init = wait_for(&mut rx, "on_init", wait_secs).await?;
    print_quote("on_init", &on_init);
    let order_id = on_init["message"]["order"]["id"]
        .as_str()
        .context("on_init carried no order.id")?
        .to_string();
    tracing::info!("order id minted: {order_id}");

    // confirm → on_confirm (+ the customer the driver will pick up)
    let mut confirm_order = order_core.clone();
    confirm_order["id"] = json!(order_id);
    confirm_order["fulfillments"][0]["customer"] = json!({
        "person": { "name": customer_name },
        "contact": { "phone": customer_phone },
    });
    let confirm = json!({
        "context": bap.context("confirm", &bap.transaction_id, Some((&bpp_id, &bpp_uri))),
        "message": { "order": confirm_order },
    });
    bap.post_signed(&bpp_uri, "confirm", &confirm).await?;
    let on_confirm = wait_for(&mut rx, "on_confirm", wait_secs).await?;
    println!("\n===== on_confirm received =====");
    println!("{}", serde_json::to_string_pretty(&on_confirm)?);

    // --- 3. wait for the unsolicited on_status with the assigned driver ---
    tracing::info!(
        "order {order_id} confirmed — waiting up to {assign_wait}s for the \
         driver assignment…"
    );
    let deadline =
        tokio::time::Instant::now() + Duration::from_secs(assign_wait);
    loop {
        let cb = match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Some(cb)) => cb,
            Ok(None) => bail!("receiver channel closed unexpectedly"),
            Err(_) => bail!(
                "no RIDE_ASSIGNED on_status within {assign_wait}s — check \
                 the adapter/api logs and that a driver is online + accepting"
            ),
        };
        let action =
            cb["context"]["action"].as_str().unwrap_or("<unknown>").to_string();
        println!("\n===== {action} received =====");
        println!("{}", serde_json::to_string_pretty(&cb)?);
        if let Some(err) = cb.get("error") {
            bail!("{action} carried an ERROR: {err}");
        }
        let fulfillment = &cb["message"]["order"]["fulfillments"][0];
        let state = fulfillment["state"]["descriptor"]["code"]
            .as_str()
            .unwrap_or_default();
        if action == "on_status" && state == "RIDE_ASSIGNED" {
            let agent = &fulfillment["agent"];
            println!("\n----- driver assigned -----");
            println!(
                "driver: {} ({})",
                agent["person"]["name"].as_str().unwrap_or("?"),
                agent["contact"]["phone"].as_str().unwrap_or("?"),
            );
            println!(
                "vehicle: {} {} {} [{}]",
                fulfillment["vehicle"]["color"].as_str().unwrap_or(""),
                fulfillment["vehicle"]["make"].as_str().unwrap_or(""),
                fulfillment["vehicle"]["model"].as_str().unwrap_or(""),
                fulfillment["vehicle"]["registration"].as_str().unwrap_or("?"),
            );
            // The ride OTP travels on the START stop's authorization.
            let otp = fulfillment["stops"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|s| s["type"].as_str() == Some("START"))
                .and_then(|s| s["authorization"]["token"].as_str());
            if let Some(otp) = otp {
                println!("ride OTP: {otp}");
            }
            println!("\nfull booking flow COMPLETE ✅");
            return Ok(());
        }
    }
}

/// Receive an `on_*` callback, ACK it, and forward the body to `main`.
async fn on_callback(
    State(tx): State<mpsc::UnboundedSender<Value>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let action = body
        .get("context")
        .and_then(|c| c.get("action"))
        .and_then(Value::as_str)
        .unwrap_or("<unknown>");
    tracing::info!("callback received: action={action}");
    let _ = tx.send(body);
    Json(json!({ "message": { "ack": { "status": "ACK" } } }))
}

/// Wait for the next callback whose `context.action` matches, printing (and
/// skipping) any other callback that arrives first.
async fn wait_for(
    rx: &mut mpsc::UnboundedReceiver<Value>,
    action: &str,
    wait_secs: u64,
) -> anyhow::Result<Value> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(wait_secs);
    loop {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Some(cb)) => {
                let got = cb["context"]["action"].as_str().unwrap_or_default();
                if got == action {
                    if let Some(err) = cb.get("error") {
                        bail!("{action} carried an ERROR: {err}");
                    }
                    return Ok(cb);
                }
                println!("\n(skipping interim callback `{got}`)");
                println!("{}", serde_json::to_string_pretty(&cb)?);
            }
            Ok(None) => bail!("receiver channel closed unexpectedly"),
            Err(_) => bail!(
                "no {action} within {wait_secs}s — check the adapter and \
                 gateway logs"
            ),
        }
    }
}

/// Print the quote carried by an `on_select`/`on_init` order.
fn print_quote(label: &str, cb: &Value) {
    let order = &cb["message"]["order"];
    let price = &order["quote"]["price"];
    println!(
        "\n{label}: item {} quoted {} {}",
        order["items"][0]["id"].as_str().unwrap_or("?"),
        price["value"].as_str().unwrap_or("?"),
        price["currency"].as_str().unwrap_or("?"),
    );
    if let Some(breakup) = order["quote"]["breakup"].as_array() {
        for part in breakup {
            println!(
                "  - {}: {}",
                part["title"].as_str().unwrap_or("?"),
                part["price"]["value"].as_str().unwrap_or("?"),
            );
        }
    }
}

/// Print a one-line-per-item summary of the catalog (or the error callback).
fn summarize(cb: &Value) {
    if let Some(err) = cb.get("error") {
        println!("\n(callback carried an ERROR): {err}");
        return;
    }
    let providers = &cb["message"]["catalog"]["providers"];
    let Some(providers) = providers.as_array() else {
        println!("\n(no catalog.providers in on_search)");
        return;
    };
    println!("\n----- catalog summary -----");
    for p in providers {
        let pid = p["id"].as_str().unwrap_or("?");
        let items = p["items"].as_array().cloned().unwrap_or_default();
        println!("provider {pid}: {} item(s)", items.len());
        for it in &items {
            let code = it["descriptor"]["code"]
                .as_str()
                .or_else(|| it["id"].as_str())
                .unwrap_or("?");
            let val = it["price"]["value"].as_str().unwrap_or("?");
            let cur = it["price"]["currency"].as_str().unwrap_or("?");
            println!("  - {code}: {val} {cur}");
        }
    }
}

// ===== FLOW=serve: signing companion for Postman/curl =====
//
// Postman can neither Ed25519-sign request bodies nor receive asynchronous
// Beckn callbacks. In serve mode this binary stays up and does both:
//
//   POST /send/{action}   sign + forward to the gateway (search) or the BPP
//                         (select/init/confirm/status/track/cancel). An empty
//                         body sends a sensible default built from the session
//                         (see below); `{"message": …}` overrides it.
//   GET  /callbacks       every on_* received so far (?action=on_search filters)
//   DELETE /callbacks     clear them
//   GET  /session         the facts accumulated from callbacks so far
//   POST /{on_action}     the Beckn callback receiver itself (the adapter
//                         calls this, not you)
//
// The session makes the whole flow drivable with EMPTY bodies: `search`
// resets it and mints a transaction id; `on_search` records bpp_id/bpp_uri
// and the first catalog item; `on_init`/`on_confirm` record the order id.

/// Trip + customer defaults, from the same env vars the one-shot flows use.
#[derive(Clone)]
struct Defaults {
    pickup: String,
    dropoff: String,
    country: String,
    city: String,
    customer_name: String,
    customer_phone: String,
}

/// What we have learned from callbacks in the current transaction.
#[derive(Default, Clone, serde::Serialize)]
struct Session {
    transaction_id: Option<String>,
    bpp_id: Option<String>,
    bpp_uri: Option<String>,
    provider_id: Option<String>,
    item_id: Option<String>,
    order_id: Option<String>,
}

struct Harness {
    bap: Bap,
    gateway: String,
    /// `BPP_URI` env — overrides the registry-facing bpp_uri from on_search
    /// (host.docker.internal does not resolve on the host).
    bpp_uri_override: Option<String>,
    defaults: Defaults,
    callbacks: Mutex<Vec<Value>>,
    session: Mutex<Session>,
}

async fn serve(
    bap: Bap,
    gateway: String,
    port: u16,
    defaults: Defaults,
) -> anyhow::Result<()> {
    let harness = Arc::new(Harness {
        bap,
        gateway,
        bpp_uri_override: std::env::var("BPP_URI").ok(),
        defaults,
        callbacks: Mutex::new(Vec::new()),
        session: Mutex::new(Session::default()),
    });
    let app = Router::new()
        .route("/send/{action}", post(send_action))
        .route("/callbacks", get(get_callbacks).delete(clear_callbacks))
        .route("/session", get(get_session))
        .route("/{action}", post(serve_callback))
        .with_state(harness);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding BAP harness on {addr}"))?;
    tracing::info!(
        "mock BAP harness on http://{addr} — POST /send/{{action}}, \
         GET /callbacks, GET /session"
    );
    axum::serve(listener, app).await?;
    Ok(())
}

async fn send_action(
    State(harness): State<Arc<Harness>>,
    Path(action): Path<String>,
    body: Option<Json<Value>>,
) -> (axum::http::StatusCode, Json<Value>) {
    let body = body.map(|Json(v)| v).unwrap_or_else(|| json!({}));
    match build_and_send(&harness, &action, &body).await {
        Ok(reply) => (axum::http::StatusCode::OK, Json(reply)),
        Err(e) => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

/// Assemble the context (+ default message when none was supplied), sign the
/// exact bytes, forward, and echo everything back so the caller sees what
/// went over the wire.
async fn build_and_send(
    harness: &Harness,
    action: &str,
    body: &Value,
) -> anyhow::Result<Value> {
    let d = &harness.defaults;
    let default_stops = json!([
        { "type": "START", "location": { "gps": d.pickup } },
        { "type": "END", "location": { "gps": d.dropoff } },
    ]);

    let (target, context, message) = if action == "search" {
        // A search opens a fresh transaction and resets the session.
        let txn = uuid::Uuid::new_v4().to_string();
        *harness.session.lock().unwrap() = Session {
            transaction_id: Some(txn.clone()),
            ..Session::default()
        };
        let mut context = harness.bap.context("search", &txn, None);
        if !d.country.is_empty() || !d.city.is_empty() {
            let mut location = serde_json::Map::new();
            if !d.country.is_empty() {
                location.insert("country".into(), json!({ "code": d.country }));
            }
            if !d.city.is_empty() {
                location.insert("city".into(), json!({ "code": d.city }));
            }
            context["location"] = Value::Object(location);
        }
        let message = body.get("message").cloned().unwrap_or_else(|| {
            json!({ "intent": { "fulfillment": { "stops": default_stops } } })
        });
        (harness.gateway.clone(), context, message)
    } else {
        let session = harness.session.lock().unwrap().clone();
        let txn = session.transaction_id.clone().ok_or_else(|| {
            anyhow!("no open transaction — POST /send/search first")
        })?;
        let bpp_id = session
            .bpp_id
            .clone()
            .ok_or_else(|| anyhow!("no bpp_id yet — wait for on_search"))?;
        let bpp_uri = harness
            .bpp_uri_override
            .clone()
            .or_else(|| session.bpp_uri.clone())
            .ok_or_else(|| {
                anyhow!("no bpp_uri — wait for on_search or set BPP_URI")
            })?;
        let context =
            harness.bap.context(action, &txn, Some((&bpp_id, &bpp_uri)));
        let message = match body.get("message") {
            Some(message) => message.clone(),
            None => default_message(action, &session, &default_stops, d)?,
        };
        (bpp_uri, context, message)
    };

    let full = json!({ "context": context, "message": message });
    let (status, text) =
        harness.bap.post_signed_raw(&target, action, &full).await?;
    let reply =
        serde_json::from_str::<Value>(&text).unwrap_or_else(|_| json!(text));
    Ok(json!({
        "target": format!("{}/{action}", target.trim_end_matches('/')),
        "request": full,
        "response": { "http_status": status.as_u16(), "body": reply },
        "hint": "poll GET /callbacks for the asynchronous on_* reply",
    }))
}

/// The empty-body default `message` for each action, built from the session.
fn default_message(
    action: &str,
    session: &Session,
    default_stops: &Value,
    d: &Defaults,
) -> anyhow::Result<Value> {
    let order_core = || -> anyhow::Result<Value> {
        let item_id = session.item_id.clone().ok_or_else(|| {
            anyhow!(
                "no catalog item in session (empty catalog?) — put a driver \
                 online, search again, or supply a message"
            )
        })?;
        Ok(json!({
            "provider": { "id": session.provider_id },
            "items": [ { "id": item_id } ],
            "fulfillments": [ { "id": item_id, "stops": default_stops } ],
        }))
    };
    let order_id = || {
        session.order_id.clone().ok_or_else(|| {
            anyhow!("no order id in session — init/confirm first")
        })
    };
    match action {
        "select" | "init" => Ok(json!({ "order": order_core()? })),
        "confirm" => {
            let mut order = order_core()?;
            order["id"] = json!(order_id()?);
            order["fulfillments"][0]["customer"] = json!({
                "person": { "name": d.customer_name },
                "contact": { "phone": d.customer_phone },
            });
            Ok(json!({ "order": order }))
        }
        "status" | "track" | "cancel" => Ok(json!({ "order_id": order_id()? })),
        other => Err(anyhow!(
            "no default body for `{other}` — supply {{\"message\": …}}"
        )),
    }
}

/// Serve-mode callback receiver: ACK, store, and harvest session facts.
async fn serve_callback(
    State(harness): State<Arc<Harness>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let action = body["context"]["action"].as_str().unwrap_or("<unknown>");
    tracing::info!("callback received: action={action}");
    {
        let mut session = harness.session.lock().unwrap();
        match action {
            "on_search" => {
                if let Some(id) = body["context"]["bpp_id"].as_str() {
                    session.bpp_id = Some(id.to_string());
                }
                if let Some(uri) = body["context"]["bpp_uri"].as_str() {
                    session.bpp_uri = Some(uri.to_string());
                }
                let provider = &body["message"]["catalog"]["providers"][0];
                if let Some(id) = provider["id"].as_str() {
                    session.provider_id = Some(id.to_string());
                }
                if let Some(id) = provider["items"][0]["id"].as_str() {
                    session.item_id = Some(id.to_string());
                }
            }
            "on_init" | "on_confirm" => {
                if let Some(id) = body["message"]["order"]["id"].as_str() {
                    session.order_id = Some(id.to_string());
                }
            }
            _ => {}
        }
    }
    harness.callbacks.lock().unwrap().push(body);
    Json(json!({ "message": { "ack": { "status": "ACK" } } }))
}

async fn get_callbacks(
    State(harness): State<Arc<Harness>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Value> {
    let wanted = params.get("action").map(String::as_str);
    let callbacks: Vec<Value> = harness
        .callbacks
        .lock()
        .unwrap()
        .iter()
        .filter(|cb| match wanted {
            Some(action) => cb["context"]["action"].as_str() == Some(action),
            None => true,
        })
        .cloned()
        .collect();
    Json(json!({ "count": callbacks.len(), "callbacks": callbacks }))
}

async fn clear_callbacks(State(harness): State<Arc<Harness>>) -> Json<Value> {
    let mut callbacks = harness.callbacks.lock().unwrap();
    let cleared = callbacks.len();
    callbacks.clear();
    Json(json!({ "cleared": cleared }))
}

async fn get_session(State(harness): State<Arc<Harness>>) -> Json<Value> {
    let session = harness.session.lock().unwrap().clone();
    Json(serde_json::to_value(session).unwrap_or_default())
}
