# Beckn BPP Adapter — Milestones

Progress log for the Beckn BPP adapter. We work **one step at a time and stop
for review** — see the full spec at
`C:\Users\user\Desktop\docs\beckn\beckn-bpp-adapter.md`.

- **Branch:** `feature/beckn-bpp-adapter` (off `main`)
- **Crate:** `packages/beckn-bpp-adapter` (workspace member; own binary + container)

## Inviolable rules (never break)

1. BPP **adapter**, never a Gateway/BG (Registry + Gateway are a separate ops
   stack — Step 8).
2. Beckn is **async**: ACK/NACK immediately, real payload later as `on_<action>`
   POSTed to `bap_uri`. Both halves required for every action.
3. Never modify the dispatch core (`packages/api/src/dispatch/`).
4. Idempotency on `(transaction_id, message_id)`.
5. `track` returns the existing SSE URL, never GPS coordinates.
6. Protocol fields live in `beckn_orders`, never on `rides`.
7. Verify signatures over the **raw** request bytes.
8. Don't invent Beckn schema — stop and ask.

## Steps

- [x] **Step 1 — Skeleton + context envelope + ACK/NACK** (+ core-version
      validation, folded in from the nammayatri reference)
- [x] **Step 2 — Correlation store + idempotency** (`beckn_orders`, `upsert_or_get`)
- [x] **Step 3 — Signing/verification, NATIVE** (decision: no onix sidecar)
- [x] **Step 4 — `search` → `on_search`** (discovery only, TRV10 catalog)
- [x] **Step 5 — `select`/`init` → `on_select`/`on_init`** (settlement decided: BPP + ON-FULFILLMENT)
- [x] **Step 6 — `confirm` + dispatch handoff** (dispatch timing decided: B, confirm-then-assign)
- [x] **Step 7 — `status`/`track`/`update`/`cancel` + unsolicited `on_status`/`on_cancel`** (decision B completed)
- [x] **Step 8 — Network infra** (beckn-onix Registry + Gateway vendored as a separate neutral stack + keygen/onboard tooling)
- [x] **Step 9 — Async-aware tests & monitoring** (PEL recovery, callback retry/backoff, DB tests, cancellation_terms, /metrics)
- [~] Step 10 — Rollout (Phase 1 discovery ✅ → Phase 2 full booking ✅ → Phase 3 external BAP)

---

## Step 1 — DONE (2026-06-28)

**Built:**
- New workspace member crate `beckn-bpp-adapter` with module layout
  `controllers/ callbacks/ context/ correlation/ crypto/ schemas/ adapters/
  validators/` (+ `config`, `app`, `main`). Later-step modules are documented
  stubs.
- `context/` — typed Beckn `context` parse/build. Models the required fields and
  preserves unknown spec fields verbatim via `#[serde(flatten)]` (no drop, no
  invent). `to_callback()` flips `action` → `on_action` and fills `bpp_id/uri`.
  Includes a minimal ISO-8601 duration parser for `ttl`.
- `schemas/` — `AckResponse` (`{message:{ack:{status}}, error?}`), Beckn error
  object, `IntoResponse`.
- `validators/` — single inbound gate: required fields → action match → ttl
  expiry. Step 3 adds signature verification here.
- `controllers/` — all 8 routes (`search…cancel`); parse raw `Bytes` (for
  raw-byte signature verify later), validate, ACK/NACK **only**.
- `callbacks/` — `CallbackClient` trait + `LoggingCallbackClient` (stub, wired)
  and `HttpCallbackClient` (real, wired in Step 4).
- `app.rs` router + `main.rs` binary. Config via `envy` (`BECKN_PORT`,
  `BECKN_SUBSCRIBER_ID`, `BECKN_SUBSCRIBER_URI`, `BECKN_DOMAIN`, `APP_ENV`).

**Refinement (from nammayatri `beckn-spec`):** added typed `context.version`
plus core-version validation in the inbound gate (order now: required → action →
version → ttl), config `BECKN_CORE_VERSION` (default `2.0.0`). `key` stays in
`extra` (signed by the onix sidecar in Step 3).

**Verified:** 17 tests pass, clippy clean, live `curl` smoke test:
- valid `search` → ACK
- expired ttl → NACK (`30008`)
- malformed context → NACK (`30001`)
- wrong action for route → NACK

**Run locally:**
```bash
cargo test -p beckn-bpp-adapter
BECKN_PORT=8099 APP_ENV=dev cargo run -p beckn-bpp-adapter
```

---

## Step 2 — DONE (2026-06-29)

**Built:**
- Migration `packages/api/migrations/202606290900000_beckn_orders.sql` — the
  `beckn_orders` table (protocol fields ONLY; none on `ride`). Columns:
  `id` (ULID PK), `transaction_id`, `message_id`, `beckn_order_id?`, `bap_id`,
  `bap_uri`, `network_id?`, `domain`, `beckn_status?`, `last_action`,
  `ride_id?` (FK→`ride(id)`, null until confirm), `created_at`, `updated_at`.
  `UNIQUE (transaction_id, message_id)` = idempotency anchor. Indexes on
  `transaction_id` and `ride_id`.
- `correlation/` — sea-orm `beckn_orders` entity + `CorrelationStore` trait
  (impl on `db_store::Database`, matching the api query-layer convention):
  - `upsert_or_get(NewBecknOrder) -> (BecknOrder, created: bool)` —
    INSERT … ON CONFLICT (transaction_id, message_id) DO NOTHING, then re-read
    the canonical row; `created` is true only for the inserting call.
  - `get_beckn_order(transaction_id, message_id)`.
- Added deps: `db_store`, `sea-orm`, `utils`.

**Migration applied to dev DB** by replicating sqlx's runner (ran the SQL +
recorded the row in `_sqlx_migrations` with the correct SHA-384), so the api
app's startup migrator treats it as already-applied. (No sqlx-cli installed.)

**Verified:** 19 tests pass incl. two DB-backed integration tests —
`replaying_same_message_is_idempotent` (first `created`, replay not, same id,
exactly one row) and `distinct_messages_in_a_transaction_coexist`. Clippy clean;
no leftover rows. DB tests self-skip when `DATABASE_URL` is unset so the suite
still passes offline; run them with
`DATABASE_URL=… cargo test -p beckn-bpp-adapter`.

**Note:** `beckn_orders` is per-inbound-message (one row per
`(transaction_id, message_id)`), per the spec's idempotency anchor — not one row
per order. Order/ride fields fill in as the flow progresses.

## Step 3 — DONE (2026-07-02)

**Decision (John):** verify/sign **natively in Rust** instead of the beckn-onix
protocol-server sidecar. Rationale: controllers already capture raw `Bytes`;
nammayatri (our canonical reference) also verifies natively; fully
unit-testable offline; no extra container. We now own the Ed25519 key —
`BECKN_SIGNING_PRIVATE_KEY` comes from the secrets manager, never the repo.

**Built:**
- `crypto/signature.rs` — the Beckn HTTP-signature profile: signing string
  `(created)/(expires)/digest`, digest = base64(BLAKE2b-512(RAW body)), Ed25519
  via `ed25519-dalek`. Header build (`sign`) + parse (`SignatureHeader::parse`,
  quoted or bare timestamps) + `verify` (algorithm → validity window → digest →
  signature). Never re-serializes the body (Rule #7).
- `crypto/registry.rs` — `RegistryClient` trait; `HttpRegistryClient` resolves
  `signing_public_key` via `POST {BECKN_REGISTRY_URL}/lookup` with a 1h
  in-process cache (dashmap); `StaticRegistry` for tests/dev.
- `crypto/mod.rs` — `Signer` (our key from config) stamps `Authorization` onto
  outbound callbacks.
- `validators/` — `verify_signature(headers, raw_body, registry)`; rejection =
  **HTTP 401 + `WWW-Authenticate: Signature realm=…`** per the signing spec
  (no invented NACK body; reason logged server-side only).
- `controllers/` — handlers now take `HeaderMap` + `Bytes` and authenticate
  BEFORE parsing. Extra check: `context.bap_id` must equal the signature's
  keyId subscriber (blocks BAP impersonation by a valid network member) →
  NACK `30001`.
- `callbacks/` — `HttpCallbackClient` serializes the payload ONCE, signs those
  exact bytes, sends those exact bytes (+ warns loudly when unsigned).
- Config: `BECKN_VERIFY_SIGNATURES` (default ON), `BECKN_REGISTRY_URL`,
  `BECKN_GATEWAY_URL` (used from Step 4), `BECKN_UNIQUE_KEY_ID`,
  `BECKN_SIGNING_PRIVATE_KEY` (b64, 32-byte seed or 64-byte pair),
  `BECKN_SIGNATURE_VALIDITY_SECS` (600).
- Boot policy: verification ON + no registry URL → **dev degrades with a WARN,
  anything else refuses to boot** (verified live: prod boot bails, dev boots +
  ACKs with two loud WARNs).

**Verified:** 36 tests pass (17 new: sign/verify roundtrip, tamper, expiry,
wrong key, header parsing, signer config, and 6 router-level signed tests —
signed→ACK, missing/tampered/unknown-subscriber→401, WWW-Authenticate
challenge shape, bap_id-mismatch→NACK). Clippy clean.

**Deps added:** `ed25519-dalek`, `blake2` (+ workspace `base64`, `dashmap`).

## Step 4 — DONE (2026-07-02)

**Built (`search` → signed `on_search`, discovery only):**
- `adapters/` — `DiscoveryAdapter` trait + `SitwegoDiscovery`, the READ-ONLY
  quote pipeline reusing the rider app's exact flow: OSRM route
  (`api::request::RidesApiClient` + `parse_from_string`), nearby-driver
  categories (`api::dispatch::state_machine::find_nearest_driver`, Redis/H3,
  4 km radius), fares (`api::…::GetFairEstimates` on the shared `Database`).
  Adapter now depends on the `api` lib crate — calls it, never modifies it
  (Rule #3). Sitwego→Beckn vehicle mapping per nammayatri `castVariant`:
  Bike→`TWO_WHEELER` (NOT MOTORCYCLE), Auto→`AUTO_RICKSHAW`, all cab
  tiers→`CAB`.
- `schemas/search.rs` — intent parsing: `message.intent.fulfillment.stops`
  START/END, gps `"lat, lon"` validated/range-checked.
- `schemas/on_search.rs` — TRV10 catalog wire types + `build_catalog`, shapes
  field-by-field from nammayatri `Transformer/OnSearch.hs`: one fulfillment
  (type `DELIVERY`, stops, `vehicle.category`) + one item (descriptor
  code/name = tier, `fulfillment_ids`, price as decimal strings, KES) per
  available tier; item/fulfillment id = tier code (e.g. `SWIFT`) — Step 5's
  `select` echoes it and re-prices (no estimate state to expire). `payments`
  deliberately absent until the 🔴 settlement decision. `variant` omitted (we
  don't track body types — don't invent).
- `controllers/` — shared `gate()` (verify → parse → validate → bap_id pin);
  `search` now: gate → parse intent (bad intent = NACK 30001) → ACK → spawned
  async pipeline: correlation `upsert_or_get` (replay ⇒ SKIP, Rule #4) →
  discovery quote → catalog → `on_search` POST to `bap_uri`. No drivers ⇒
  honest empty catalog. Discovery failure ⇒ log, no callback.
- `correlation/` — `MemoryCorrelationStore` (same idempotency contract; dev
  boots without Postgres + handler tests).
- `main.rs` — real `HttpCallbackClient(signer)` wired (on_search goes out
  SIGNED); Postgres+Redis+OSRM wiring with dev-degrade/prod-refuse policy
  (env: `DATABASE_URL`, `ROUTES_API_URL`, `REDIS_HOST/PORT/CLUSTER_*`).
- api crate (additive only): `FareEstimate` fields made `pub`; 2 pre-existing
  clippy failures fixed (allow(too_many_arguments) on upsert_email_template,
  clone→deref in get_category_pricing).

**Verified:** 44 tests pass (8 new: catalog wire shape incl. gps/price/stop
formats, intent parsing/rejection, full router-level search→on_search flow
with recorded callback, replay sends exactly ONE on_search, missing-stops
NACK). `./lint.sh beckn-bpp-adapter api` clean. Live e2e against real
OSRM/Redis/drivers still pending (needs the full dev stack up) — covered by
fakes at the trait boundary.

## Step 5 — DONE (2026-07-03)

**🔴 Settlement decision (John, 2026-07-03): `collected_by = BPP`,
`PaymentType = ON-FULFILLMENT`.** The rider pays the driver directly at ride
end (cash/M-Pesa), exactly as today. No settlement infrastructure, no
BAP finder fee (`BUYER_FINDER_FEES_PERCENTAGE = "0"`), no SETTLEMENT_TERMS —
nothing moves BAP→BPP. Drivers keep 100%; Sitwego's business model is the
driver **subscription**, so per-ride commission is structurally zero on every
booking channel.

**Built (`select`/`init` → `on_select`/`on_init`):**
- `adapters/` — `QuoteOption` now carries the fare components
  (`base_fare`/`distance_fare`/`time_fare`/`waiting_fare`, whole KES) straight
  from `FareEstimate`; `time_fare` is the pre-discount total minus the named
  components. Components can sum below `fare` when the minimum fare applies.
- `schemas/order.rs` — inbound `parse_order_selection` (`order.items[0].id` =
  tier code + START/END stops of `order.fulfillments[0]`); outbound `Order`
  wire types per nammayatri `ACL.OnSelect`/`ACL.OnInit`: `provider{id}`,
  one fulfillment + one item (same shapes as on_search), `quote{breakup,
  price, ttl PT15M}` with `QuoteBreakupTitle` codes (BASE_FARE always;
  DISTANCE_FARE / RIDE_DURATION_FARE / WAITING_OR_PICKUP_CHARGES when > 0),
  and `payments[]` `{collected_by BPP, type ON-FULFILLMENT, status NOT-PAID,
  params{amount,currency}, tags[BUYER_FINDER_FEES→PERCENTAGE 0]}` (hyphenated
  wire enums; TagGroup = `{descriptor, display, list}`).
- `controllers/` — `select`/`init` handlers share `order_action` +
  `process_order(phase)`: gate → parse (bad order = NACK 30001) → ACK →
  async: idempotency (replay ⇒ SKIP) → re-price via discovery → tier missing
  from the fresh quote ⇒ **error callback** `{context, error}` (the async
  NACK; `Callback::error` added) → else `on_select`/`on_init`. `init` mints
  the durable Beckn order id (ulid) INSIDE the correlation insert so a
  retried init can never mint two ids; `on_init.order.id` = that id.
- `on_select` carries no order id (matches nammayatri — id appears at init).
  `cancellation_terms` deliberately omitted until cancel lands (Step 7/9).
  on_search catalog `payments`/`locations` left as-is: quote-phase payments
  now carry the terms; revisit only if a target network's policy demands
  catalog-level payments.

**Verified:** 53 tests pass (9 new: order parsing ×2, order wire shape incl.
breakup titles/payment terms/tags, on_init order id, router-level
select→on_select and init→on_init flows, unavailable-item error callback,
select replay sends exactly ONE on_select, missing-order NACK).
`./lint.sh beckn-bpp-adapter` clean. Live e2e still pending with the full
dev stack (same note as Step 4).

## Step 6 — DONE (2026-07-03)

**🔴 Dispatch-timing decision (John, 2026-07-03): Option B, confirm-then-assign.**
`on_confirm` goes out immediately ("confirmed, allocating a driver"): order
`status ACTIVE`, fulfillment state `NEW`. The driver is announced later via an
unsolicited `on_status` with `RIDE_ASSIGNED` + `fulfillment.agent` (Step 7).

**Built (`confirm` → dispatch handoff → `on_confirm`):**
- **api crate, additive (Rule #3 intact — dispatch core only CALLED):**
  `api/src/api/beckn_internal.rs` — a second private-plane router (same
  listener as the admin plane, `127.0.0.1:{admin_port}`), gated by a
  **dedicated** `BECKN_INTERNAL_TOKEN` (`constant_time_eq`; empty = plane
  disabled). `POST /internal/beckn/ride-request` replays the rider app's exact
  pre-dispatch flow: OSRM route → cache `ride_path_key_id` → 
  `register_ride_request` (duplicate request id refused — second idempotency
  layer) → `DispatchJob` → `enqueue_dispatch` → `202 {request_id}`.
  `main.rs` now merges admin/beckn routers per configured token.
- **Quote persistence:** migration `202607031300000_beckn_orders_quote.sql`
  adds `quoted_fare`/`quoted_item_id`; `init` stores them (`set_quote`) after
  re-pricing. `confirm` looks the quote up via `find_init_by_order_id`
  (transaction_id must match) and **never re-prices** — the BAP-echoed amount
  is never trusted.
- **FK fix:** migration `202607031400000_beckn_orders_ride_fk.sql` drops
  `beckn_orders.ride_id → ride(id)`. Under decision B the ride ulid is minted
  at confirm as the dispatch **request id**; the `ride` row only exists after
  a driver accepts (`start_ride` reuses `ride_request.id` as `ride.id`), so
  the FK cannot hold at confirm time. Soft reference + index instead.
  (Caught by live-booting the service, not by unit tests.)
- **Adapter:** `DispatchAdapter` trait + `HttpDispatchAdapter` (POSTs the
  api's `BecknRideRequest` with `x-internal-token`; env
  `SITWEGO_INTERNAL_API_URL` / `BECKN_INTERNAL_TOKEN` /
  `BECKN_NETWORK_RIDER_ID`, all-or-nothing, dev may run without).
  `controllers::confirm`: gate → parse (`order.id` mandatory, customer from
  `fulfillments[0].customer`) → ACK → async: **ride ulid minted INSIDE the
  idempotent insert** (Rule #4 — a retried confirm can never dispatch twice;
  replay ⇒ skip) → quote lookup (unknown order / no quote ⇒ error callback) →
  `dispatch.request_ride` (failure ⇒ error callback) → `on_confirm` (order
  `id` + `ACTIVE` + state `NEW` + quote/payments at the init-agreed fare).
  Real customer name/phone travel to dispatch via `RiderDataInfo` so drivers
  see them; the network-rider profile only satisfies the `ride.customer_id`
  FK.
- `main.rs` `redis_config` now mirrors the api's dev override (standalone
  localhost Redis when `APP_ENV=dev`).

**Verified:** 60 tests pass (7 new: confirm parse ×2 + wire shape;
router-level confirm→dispatch→on_confirm with a `FakeDispatch` recorder;
replayed confirm dispatches exactly ONCE; unknown order id ⇒ error callback +
zero dispatches; missing order ⇒ NACK). `./lint.sh beckn-bpp-adapter api`
clean. **Live-verified** (`cargo run`, api + adapter + callback sink):
confirm ACKs, correlation row inserted, quote found, HTTP handoff
authenticated against the live private plane, error callback delivered to
`bap_uri` when dispatch fails, replay skips with exactly one callback. Full
happy-path 202 still needs OSRM up (was down) + online drivers.

**Ops prerequisites for a real network booking (John):**
1. Create the **network rider profile row** (any ULID; dummy encrypted contact
   bytes are fine — the beckn path never decrypts it) and set
   `BECKN_NETWORK_RIDER_ID` to its id.
2. Env for the adapter: `SITWEGO_INTERNAL_API_URL=http://127.0.0.1:8091`,
   `BECKN_INTERNAL_TOKEN=<shared secret>` (same value in the api env).
3. `BECKN_PORT` must differ from the api's `:8090` when both run on one host
   (dev default collides; live test used `8092`).

## Step 7 — DONE (2026-07-04)

Completes dispatch-timing decision B: the driver promised by `on_confirm`
("confirmed, allocating") is now announced via the **unsolicited `on_status`
RIDE_ASSIGNED**, and the whole ride lifecycle flows to the BAP.

**Driver-assignment signal (api crate, additive):** there was no existing
event for "driver accepted" (the acceptance path has a literal
`TODO:: notify customer`), so `beckn_internal.rs` grew a per-booking watcher:
a SECOND subscription to the booking's dispatch broadcast channel (the core
is still only *called*). A raw `accepted` event is treated as a hint — the
watcher confirms against `ride_requests` (status `Accepted`, same driver,
retry loop) before acting, then enriches (driver name via `get_driver_info`,
phone via profile-blob decrypt `extract_contact_info`, vehicle via
`get_driver_vehicle_and_categories`, ride OTP from the request row) and
POSTs to the ADAPTER's webhook `/internal/dispatch/assigned`. Channel closed
or watch timeout → terminal DB check → `assigned` or
`/internal/dispatch/failed` ("no driver"). New api env: `BECKN_ADAPTER_URL`
(e.g. `http://127.0.0.1:8092`; empty = webhooks off, warn).
Also new: `POST /internal/beckn/ride-request/{id}/cancel` mirroring the
rider's two cancel paths — CancelRequest signal while dispatch is in flight,
else the customer-branch teardown of `cancel_ride` (delete row +
`ride_clean_up` + RideCanceledEvent). Outcomes: `dispatch_cancelled` /
`cancelled` / `already_closed` / `active_ride` (refused, v1 policy) /
`not_found`.

**Adapter:**
- Migration `202607041000000_beckn_orders_fulfillment.sql`: beckn_orders +
  `fulfillment_state`, `assigned_json`, `pickup_gps`, `dropoff_gps` (all
  protocol state stays on beckn_orders — Rule #6). Confirm rows now persist
  state `NEW`, the trip gps, and (copied from the init row) the quote.
- Webhooks `/internal/dispatch/assigned|failed` on the adapter router,
  gated by `BECKN_INTERNAL_TOKEN` (constant-time compare; keep `/internal/*`
  off the public ingress). assigned → persist RIDE_ASSIGNED + snapshot →
  unsolicited `on_status` (fresh uuid `message_id`, order's transaction_id,
  per nammayatri `buildOnStatusReqV2`); failed → `on_cancel` cancelled_by
  PROVIDER.
- **Ride lifecycle consumer** (`events.rs`): joins `swg:stream:ride_events`
  with its OWN consumer group `beckn-adapter-workers` (notif service's group
  unaffected), translates DriverArrivedEvent→RIDE_ARRIVED_PICKUP,
  RideStartEvent→RIDE_STARTED, RideEndEvent→RIDE_ENDED (order COMPLETE),
  RideCancelEvent→`on_cancel` (canceled_by 1 ⇒ CONSUMER, else PROVIDER);
  non-Beckn rides are acked and skipped.
- **Inbound actions** — `status`: replays persisted state (no re-pricing, no
  dispatch calls); order lookup requires the caller's bap_id AND
  transaction_id to match (an order id is not a capability). `track`:
  `tracking{url, status ACTIVE|INACTIVE}` from `BECKN_TRACKING_URL_TEMPLATE`
  (`{ride_id}` substituted); never GPS (Rule #5); unset template → error
  callback. `cancel`: calls the api cancel endpoint via `DispatchAdapter::
  cancel_ride`; success/not_found/already_closed → persist RIDE_CANCELLED +
  `on_cancel` cancelled_by CONSUMER; `active_ride` → error callback (v1:
  mid-ride cancels stay rider↔driver); cancel of a cancelled order re-sends
  `on_cancel`. `update`: **synchronous NACK** ("not supported") — the spec's
  update events (PAYMENT_COMPLETED / EDIT_LOCATION / ADD_STOP / EDIT_STOP)
  don't apply to cash-on-fulfilment v1.
- `on_status` order shape per nammayatri `tfAssignedReqToOrder` /
  `mkFulfillmentV2` / `mkStopsOUS`: fulfillment id = ride id, `agent{person
  {name}, contact{phone}}`, `vehicle{category, registration, model, color,
  make}`, START stop `authorization{token: <ride OTP>, type: OTP}`;
  cancelled orders carry `cancellation{cancelled_by: CONSUMER|PROVIDER}`.

**Verified:** 75 tests pass (15 new: status-order wire shapes ×2, order_id
parse, event translation, and router-level — status replay, unknown-order
error, assigned webhook → unsolicited RIDE_ASSIGNED (+persisted for later
status), webhook auth 401/404, failed webhook → on_cancel PROVIDER, cancel →
dispatch cancel + on_cancel + idempotent replay, active-ride refusal, track
URL + Rule #5, no-template error, update NACK, missing order_id NACK).
`./lint.sh beckn-bpp-adapter api` clean.

**Live-verified (2026-07-04, api :8090/:8091 + adapter :8092 + BAP sink,
real Postgres/Redis; migration auto-applied on boot):**
- `status` → `on_status` NEW with the persisted trip/fare;
- assigned webhook: 401 without token; with token → **unsolicited
  `on_status` RIDE_ASSIGNED** at the sink (agent name/phone, plate KDA 123X,
  OTP 4471 on the START stop's authorization);
- `track` → `on_track` `{url: https://track.sitwego.test/<ride_id>,
  status: ACTIVE}`, no coordinates;
- `update` → synchronous NACK;
- real `XADD swg:stream:ride_events` RideStartEvent → the consumer group
  translated it → unsolicited `on_status` RIDE_STARTED (driver snapshot
  still attached);
- `cancel` → adapter called the api's internal cancel over HTTP →
  `on_cancel` cancelled_by CONSUMER; row finished CANCELLED/RIDE_CANCELLED.
Test rows + stream entry cleaned up. Not live-tested: the assignment
watcher end-to-end (needs an online driver actually accepting).

## Pending 🔴 product notes for John (Step 7 policies to bless)

1. **Tracking URL**: no public tracking page exists yet (live tracking is
   app-internal gRPC). `on_track` serves `BECKN_TRACKING_URL_TEMPLATE`; the
   page behind it still needs building (or leave unset → "tracking not
   available" error callbacks).
2. **Mid-ride cancels refused** (`active_ride`): network customers cannot
   cancel once the trip started. OK?
3. **`update` NACKed as unsupported** (no PAYMENT_COMPLETED/EDIT_LOCATION/
   ADD_STOP/EDIT_STOP). OK for v1?

## Step 8 — DONE (2026-07-04)

The network we are FOUNDING (spec §3): no Beckn mobility network with a
live registry exists in Kenya, so the Registry + Gateway are our
deliverable — run as credibly-neutral infrastructure, separate from
Sitwego-the-app (Rule #1: the adapter stays a plain BPP subscriber).

**Built — `infra/beckn-network/` (separate stack, own lifecycle):**
- `docker-compose.yml`: beckn-onix reference deployment pinned to
  `beckn/beckn-onix@main-pre-plugins` — Registry `fidedocker/registry`
  (:3030 admin+API) + Gateway `fidedocker/gateway` (:4030 admin+protocol,
  BAPs POST `/bg/search`; the images' 3000/4000 are JDWP debug ports, not
  published), bind-mounted rendered config (no external-volume/busybox
  dance; bootstrap restores the image-default `envvars`/`logger.properties`
  the mount would otherwise shadow), named DB/log volumes.
- Config templates (verbatim upstream samples, localized): registry +
  gateway `swf.properties`, gateway network JSON. Network identity in
  `network.env` (name default `safiri`, KEN/KE, public URLs,
  gateway subscriber id).
- `bootstrap.sh`: render (+ restore image-default `envvars`/
  `logger.properties`) → up registry → poll JSON `/login` (root/root;
  response field is `api_key`) → import upstream `RolePermission.xlsx`
  via `/role_permissions/importxls` → seed `NETWORK_DOMAINS` via
  `/network_domains/save` (registry rejects `/register` for unseeded
  domains: "Invalid domain …") → up gateway → gateway self-subscribe
  (**POST**-login `:4030/login?name=…` — GET logins are ignored — then
  cookie `GET /bg/subscribe`) → poll `/subscribers/lookup {"type":"BG"}`
  until the BG record lands → prints the manual ceremony (change root
  password, approve participants).
- README: neutrality doctrine, bring-up, BPP onboarding, key rotation
  (bump `unique_key_id`, overlap windows), prod notes (Traefik, H2 volume
  backup, move to own repo/org before Phase 3).

**Built — adapter onboarding tooling:**
- `keygen` bin: mints the Ed25519 signing pair + X25519 encryption pair
  (base64 env lines; private halves → secrets manager).
- `onboard` bin + `crypto/onboarding.rs`: `POST {registry}/register` with
  the exact beckn-onix installer payload (subscriber_id, pub_key_id =
  unique_key_id, subscriber_url, domain, encr/signing public keys,
  valid_from now−1d, valid_until +3y, type BPP, country, status
  SUBSCRIBED). New config: `BECKN_ENCRYPTION_PUBLIC_KEY`, `BECKN_COUNTRY`
  (default KEN), `BECKN_REGISTRY_API_KEY` (optional).
- Verified our Step-3 `/lookup` client against the official registry
  contract (POST `{base}/lookup`, `subscriber_id` + `unique_key_id`,
  array reply with `signing_public_key`). **Ops note:** with this
  registry, `BECKN_REGISTRY_URL` must be `http://<host>:3030/subscribers`.

**Onboarding ceremony (deliberate):** this registry has NO encrypted
challenge — register + manual admin approval (network role → SUBSCRIBED)
is the trust ceremony for a founding network. `/on_subscribe` challenge
decryption is tracked as future work for challenge-enforcing registries
(ONDC-style); the X25519 pair is minted now so the record is ready.

**Verified:** 76 tests (payload-contract test added), lint clean, `keygen`
run live. **Live network pass (2026-07-06):** from-zero `docker compose
down -v && ./bootstrap.sh` is fully green with no manual steps (registry
up, role permissions imported, `mobility` domain seeded, gateway up and
its BG record confirmed in the registry); `onboard` with throwaway keys →
HTTP 200, record created `status INITIATED` (registry correctly overrides
requested SUBSCRIBED pending admin approval); `POST /subscribers/lookup
{subscriber_id, unique_key_id}` returns the exact minted
`signing_public_key` — the Step-3 verify client's contract holds against
the real registry. Live-only findings fixed: bind mount shadowed the
image's `envvars`/`logger.properties` (registry crashlooped); login reply
field is `api_key` not `ApiKey`; gateway login must be POST; ports
3000/4000 are JDWP debug ports (unpublished; real ports 3030/4030,
gateway protocol at `/bg/search`); `*_PUBLIC_URL`s must resolve from
inside the containers (gateway calls the registry at
`REGISTRY_PUBLIC_URL`) — local dev uses `host.docker.internal` +
compose `extra_hosts`; network domains must be seeded before register.
Test record `bpp.sitwego.test` (throwaway keys) left in the dev registry;
stack left running for John.

**🔴 Architecture note for John — classic vs "Beckn Fabric":** the
beckn-onix repo ARCHIVED the registry+gateway deployment
(`main-pre-plugins`) in Aug 2025; upstream now pushes "Beckn Fabric"
(NFH docs: ONIX plugin adapters, hosted DeDi Registry, Catalog Service +
Discovery Service, `discover`/`on_discover` lifecycle). We deliberately
stay CLASSIC (per spec §3): (1) Steps 1–7 are built and live-verified
against classic Beckn v2 + TRV10; (2) production mobility networks
(Namma Yatri / ONDC) run classic gateway-broadcast `search` precisely
because ride supply/pricing is real-time — Fabric's publish-to-catalog
discovery doesn't fit fares that depend on live driver positions and
routing; (3) Fabric is starter-kit/testnet maturity. Tracked as a
possible future migration; the adapter's clean protocol boundary keeps
dispatch integration untouched if we ever switch.

## Step 9 — DONE (2026-07-06)

Async-aware hardening; all verified live against dev Postgres + Redis
(adapter booted on :8092, exercised with curl, then stopped).

- **PEL recovery** (`events.rs`): `handle_ride_event` now returns the ack
  decision — transient DB failures (correlation lookup / `set_fulfillment`)
  leave the message in the PEL instead of acking it away; a spawned
  `beckn-recovery` consumer runs `redis_store::events::run_recovery_pass`
  every 30 s, reclaiming messages idle >60 s (idle > one 5 s BLOCK cycle so
  live in-flight messages are never stolen). Replays are safe:
  `set_fulfillment` is idempotent, duplicate `on_status` is within Beckn's
  at-least-once semantics. New live-Redis integration test
  (`test_recovery_pass_reclaims_unacked_messages`, `#[ignore]`-gated like
  the rest of redis_store's) covers deliver-without-ack → reclaim → replay
  → ack; ran green against local Redis. Live check: the consumer group
  shows both `beckn-0` and `beckn-recovery`, lag/pending 0.
- **Live-found consumer bug, fixed in `redis_store::events`**: a blocked
  `XREADGROUP` that expires with NO messages returns RESP nil, which fred
  reports as `Parse Error: Cannot convert to map` — the loop logged an
  ERROR every ~6 s on an idle stream (and slept 1 s, slowing real
  consumption). Now recognized as idle and skipped silently. Only the
  adapter uses `run_event_consumer`, so no other service's behavior
  changes.
- **Callback retry/backoff** (`callbacks/mod.rs`): `HttpCallbackClient`
  retries transport errors and transient statuses (408/429/5xx) up to 3
  attempts with exponential backoff (500 ms base); other 4xx are
  rejections and never retried. Signed ONCE — every retry re-sends the
  identical bytes+header inside the signature validity window (Rule #7).
  Client now has a 10 s request timeout (a hung BAP can't pin the task).
  Unit tests with an in-process axum BAP: fail-twice-then-succeed,
  4xx-no-retry, exhaustion, transport retry. Verified live: dead
  `bap_uri` → attempt 1 → 500 ms → attempt 2 → 1 s → attempt 3 → give-up,
  `callbacks.failed` = 1 in /metrics.
- **DB integration tests** (`correlation/mod.rs`, same connect()/skip
  pattern as Step 2): quote lifecycle (`set_quote` →
  `find_init_by_order_id`), fulfillment lifecycle
  (`find_confirm_by_{ride,order}_id`, `set_fulfillment` advancing
  status/state, `assigned_json` captured at RIDE_ASSIGNED and PRESERVED by
  later updates that carry none, init row untouched by ride-keyed
  updates). Green against the dev database.
- **cancellation_terms** (`schemas/order.rs`): `CancellationTerm`/`Fee`
  per nammayatri `Spec.CancellationTerm`/`Spec.Fee`
  (`cancellation_fee.amount{currency,value}`, optional
  `fulfillment_state`, `reason_required`). Policy: free cancellation, no
  reason required — explicit zero fee so BAPs can render it positively.
  Attached from `on_init` on and on `on_confirm` (nammayatri: none on
  `on_select`, terms at init/confirm); lifecycle orders don't repeat them.
- **Watcher timeout review** (api crate, comment only): 600 s
  `BECKN_WATCH_TIMEOUT` confirmed correct = 2× the dispatch state
  machine's own 300 s deadline (set at construction in `ride_request.rs`:
  20 s offer TTL, 20 attempts cap, +15 s grace); normal exit is channel
  `Closed`, the guard only fires if the state machine wedges. Comment now
  cites the concrete numbers.
- **Metrics** (`metrics.rs` + `GET /metrics`): plain atomics as JSON — no
  Prometheus stack exists in the workspace yet, so counters only, swap for
  a real exporter later. `inbound.accepted/rejected` (counted at the
  shared `gate`), `callbacks.delivered/failed` (a `CountingCallbacks`
  decorator wraps whatever client `AppState::new` receives),
  `ride_events.handled/requeued/recovered`. Live-verified: garbage POST →
  rejected+1, valid search → accepted+1, dead-BAP callback → failed+1.
- Suite: 85 adapter tests + 31 redis_store green; `./lint.sh` clean for
  beckn-bpp-adapter, redis_store, api. Note for later: with OSRM down,
  `search`/`select` log "no on_search/on_select sent" and send nothing —
  correct for search (gateway broadcast), debatable for select (could send
  an error callback); revisit if BAP timeout behavior warrants it.

## Step 10 — Phase 1 IN PROGRESS (2026-07-06)

First real network-path transaction against the founded `safiri` network
(registry :3030 + gateway :4030 from Step 8, OSRM :5000, dev PG/Redis).

**Built:**
- `src/bin/mock_bap.rs` — throwaway BAP (dev tool only). Signs a `search`
  over the RAW body with `crypto::signature::sign` (byte-identical to a
  real BAP), POSTs it to the gateway `/bg/search`, and runs an axum server
  to receive the adapter's async `on_search` at `{bap_uri}/{on_action}`.
  `BECKN_CITY=""` omits `location.city` (see routing finding below).
- Scripted registry approval: `scratchpad/approve.py` echoes a network
  role's edit form back with `STATUS=SUBSCRIBED` (scoped to the
  `/network_roles/save` form so the `_FORM_DIGEST` + hidden fields are the
  right ones). Turns the "manual admin ceremony" into a repeatable step.

**Verified live:**
- keygen → onboard registered the BPP `sitwego.mobility.ke` (INITIATED),
  a `type=BAP` `bap.mock.ke` registered via curl; both approved to
  SUBSCRIBED by the script; both resolve by `(subscriber_id,
  unique_key_id)` and now appear in `{type:BPP}` / `{domain:mobility}`
  lookups.
- Adapter boots on :8092 with `BECKN_VERIFY_SIGNATURES=true`, real signing
  key, DB+Redis+OSRM discovery wired; **reachable from the gateway
  container** at `http://host.docker.internal:8092`.
- Mock BAP → gateway `/bg/search`: **gateway verifies the BAP's Ed25519
  signature and ACKs (HTTP 200)** — the BAP→gateway leg with real
  signing/verification works over the wire.

**Live-found + fixed — onboard timestamp bug:** `build_register_payload`
emitted `to_rfc3339()` (9-digit nanoseconds). This registry MISPARSES the
fractional-seconds component as MILLISECONDS and adds it to the instant, so
`.361673673` shoved `valid_from` ~4.2 days into the FUTURE — the key read
"not yet valid" and validity-window lookups (which the gateway uses to pick
BPPs) SKIPPED the BPP, while `(subscriber_id,unique_key_id)` lookups (no
window filter) still found it. Fixed to `SecondsFormat::Secs` (no
fraction); test asserts no `.` in the timestamps. Re-onboard fixed the
record (note: re-register leaves a harmless duplicate INITIATED role the
gateway's SUBSCRIBED filter ignores).

**City-scoped routing (was the blocker, now solved):** the gateway
multicasts a `search` only to BPPs whose operating region covers the
search's `location.city.code`. Its recipient lookup is
`{country, domain, location:{city:{code}}, type:BPP, status:SUBSCRIBED}`; a
search with NO city produces no recipient lookup at all. The registry's geo
tables ship seeded with INDIA only (one state, "Karnataka"), no Kenyan
city. **John's founding-network decision: Nairobi city code = `KE-047`**
(KE + Nairobi's county number 047). Seeded via the registry admin (Country
→ State → City → the BPP's network-role operating region):
- Kenya country already present (id 32).
- Created state "Nairobi", `CODE=47` — the states `CODE` column is
  `VARCHAR(2)`, so a 2-char code is mandatory (KE-30/KE-047 are rejected
  "value too long").
- Created city "Nairobi", `CODE=KE-047`, under that state (cities require a
  state — "State is mandatory").
- Registered the BPP network role's operating region → COUNTRY=Kenya,
  CITY=Nairobi. Confirmed: the gateway's exact recipient lookup with
  `location.city.code=KE-047` now returns `sitwego.mobility.ke`.
(Scripted with `scratchpad/reg.py` — a reusable scoped form-echo helper.)

**✅ FIRST FULL NETWORK-PATH TRANSACTION (2026-07-06):** mock BAP →
signed `/bg/search` → **gateway verifies the BAP Ed25519 sig + ACK** →
multicasts to the BPP → **adapter verifies the forwarded signature + ACK**
(inbound.accepted=1, no NACK — proves the gateway forwards the BAP's
`Authorization` UNCHANGED, so the Step-3 bap_id pin holds) → read-only
`find_nearest_driver` (Radius 4000 m) → **signed `on_search` delivered to
the BAP** (callbacks.delivered=1). The BAP received a well-formed TRV10
catalog: context echoes txn/msg, `bpp_id=sitwego.mobility.ke`, `location`
(KE-047/KEN) preserved. Real Ed25519/blake2b verification INBOUND +
signing OUTBOUND, both directions, over the real gateway.

Catalog had **0 items** — expected: no drivers are online near Nairobi CBD
in dev Redis ("honest empty catalog", Step 4). A non-empty catalog needs a
real online driver — that's Phase 2's "e2e with an online driver".

**Left running for the continue:** adapter on :8092 (env in
`scratchpad/adapter.run.env`), safiri stack up. Keys for this run are in
`scratchpad/{adapter,bap}.keys.env` (dev throwaway — private halves never
committed). Registry now holds: BPP `sitwego.mobility.ke` (SUBSCRIBED,
KE-047 operating region), BAP `bap.mock.ke` (SUBSCRIBED), Kenya state/city
Nairobi.

## Step 10 — Phase 2 ✅ DONE (2026-07-07): first full network BOOKING

The complete booking flow ran end to end over the real gateway with a real
(simulated-app) driver — every leg through production code paths:

```
mock BAP ──signed search──▶ gateway /bg/search ──▶ adapter :8092
  ◀── on_search: SWIFT 943 KES (catalog NON-empty: driver online)
  ──signed select──▶ adapter  ◀── on_select (quote + breakup)
  ──signed init───▶ adapter   ◀── on_init  (order id minted)
  ──signed confirm▶ adapter ──▶ api :8091 /internal/beckn/ride-request (202)
  ◀── on_confirm (ACTIVE / fulfillment NEW)          │ dispatch core
       driver app accepts ◀─offer (ride_requests row)┘
  api assignment watcher confirms Accepted in DB ──▶ adapter
       /internal/dispatch/assigned (shared token)
  ◀── UNSOLICITED on_status RIDE_ASSIGNED  (~4 s after confirm)
       agent: Henry Thiong'o +2547…, vehicle Red Nissan Dayz KDP 439T,
       ride OTP on the START stop authorization, fulfillment id = ride id
```

Evidence: adapter metrics `inbound.accepted=4, callbacks.delivered=5,
rejected=failed=0`; `beckn_orders` confirm row `ACTIVE / RIDE_ASSIGNED`
with ride_id + quoted_fare 943; `ride_requests` row `Accepted` w/ OTP.
**First live exercise of the Step-7 assignment watcher** — broadcast
accept hint → DB-confirmed Accepted → webhook → on_status, all real.

### Ops provisioning done this session (Step 6 prereqs)

- Network-rider profile row `01KWY43N8HW9J1SD6H14CRP5Z1` ("Safiri
  Network", dummy 1-byte encrypted contact blobs) → `BECKN_NETWORK_RIDER_ID`.
- `BECKN_INTERNAL_TOKEN` minted (openssl rand, dev throwaway in
  `scratchpad/{adapter,api}.run.env`), shared api ↔ adapter.
- api booted `:8090` + private plane `:8091` with `BECKN_ADAPTER_URL=
  http://127.0.0.1:8092`; **api needs `APP_ENV=dev` exported** or it tries
  the production Redis cluster from `.env`.

### Live-found + fixed: blocking XREADGROUP starved the shared Redis pool

First Phase-2 search NACKed internally: discovery `mgeo_search` failed with
fred's *"Error sending command while connection is blocked"*. The Step-7
ride-event consumer parks a blocking `XREADGROUP … BLOCK 5000` on the
shared 10-connection pool; fred round-robins commands onto the blocked
connection and rejects them (~1-in-10 per call — Phase 1 simply got
lucky). Fix in adapter `main.rs`: the consumer gets its OWN 2-connection
`RedisConnectionPool`; the request path never shares a connection with a
blocking read. (api is unaffected — it never runs blocking reads on its
pool.)

### Test harness (scratchpad, dev-only)

- `mock_bap.rs` extended: `FLOW=full` (default) books the first catalog
  item after on_search — select/init/confirm signed over raw bytes,
  then waits for the unsolicited RIDE_ASSIGNED on_status (`ASSIGN_WAIT_SECS`).
  `BPP_URI` overrides the registry bpp_uri (host.docker.internal doesn't
  resolve on the host). `BECKN_CITY` default now `KE-047`.
- `scratchpad/driver_sim.py`: simulated driver app using ONLY real public
  API endpoints — mints the driver JWT (dev `JWT_SECRETE_KEY`), `POST
  /go-online` (vc header), periodic `/update-location-coordinates` (feeds
  the real location drainer → geo buckets), polls `ride_requests` for the
  offer, `POST /accept-ride-request/{id}/{vc}/accept`. Driver: Henry
  Thiong'o `01KQ77PWS2FWH7KMRWVEAVBJD1` (activated + vehicle + Swift cat).

State left after the run: adapter :8092 + api :8090/:8091 running, Henry
taken offline (`/go-offline`), booking left `Accepted` (ride never started
— driver app would `create-ride` with the OTP; lifecycle events were
already live-verified in Step 7). 85/85 tests, lint clean.

## Postman workflow (added 2026-07-07, after Phase 2)

Postman can't Ed25519-sign raw bodies or receive async callbacks, so
`mock_bap` gained **`FLOW=serve`**: a long-lived signing companion on
:9095. Postman POSTs plain JSON to `/send/{action}`; the harness signs
the exact bytes, forwards (search → gateway, everything else → BPP),
stores every `on_*` callback for `GET /callbacks` polling, and keeps a
session (transaction id, bpp, first catalog item, order id) so the ENTIRE
flow runs with empty request bodies. `GET /session`, `DELETE /callbacks`,
`{"message": …}` overrides. Collection + docs:
`packages/beckn-bpp-adapter/postman/` — verified live end to end
(search→select→init→confirm→RIDE_ASSIGNED→status/track/cancel, all via
curl exactly as Postman would send them).

Live gotcha found while testing: a driver holding an open `Accepted`
booking is INVISIBLE to discovery — the api routes their location updates
to the on-ride path, so they never reach the nearby-driver geo buckets.
Cancel stale bookings (harness `cancel` or the api's internal cancel)
before expecting them in a catalog. (Both Phase-2 test bookings were
cancelled through the real cancel path — `on_cancel cancelled_by=CONSUMER`
delivered; `ride_requests` clean.)

## Next: Phase 3

External/real BAP onboarding: another party's BAP (or a second machine)
onboards to the safiri registry and books against sitwego.mobility.ke.
Candidate cleanups first: registry's stale duplicate INITIATED role for
sitwego.mobility.ke; tracking page behind BECKN_TRACKING_URL_TEMPLATE.
