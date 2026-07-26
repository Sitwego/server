# Testing the Beckn BPP adapter from Postman

Postman can't do two things Beckn requires:

1. **Ed25519 signing** — every request must carry an HTTP-Signature
   `Authorization` computed over the *exact raw bytes* of the body
   (blake2b-512 digest, Ed25519). Postman scripts can't do this.
2. **Async callbacks** — the real reply to every action is an `on_*`
   POSTed later to the BAP's `bap_uri`. Postman can't receive those.

So Postman talks to the **mock-BAP harness**, which does both: it signs
and forwards your JSON to the real gateway/adapter, receives the
callbacks, and lets you poll them. Signature verification stays ON across
the whole network — you are testing the production path, not a bypass.

```
Postman ──plain JSON──▶ harness :9095 ──signed──▶ gateway :4030 ──▶ adapter :8092
   ▲                        │  ◀──────────── on_* callbacks ────────────┘
   └──── GET /callbacks ────┘
```

## Boot order (dev)

```bash
# 1. safiri network up (infra/beckn-network/), adapter + api running:
#    adapter:  env from scratchpad adapter.run.env  → :8092
#    api:      APP_ENV=dev + BECKN_INTERNAL_TOKEN + BECKN_ADAPTER_URL → :8090/:8091

# 2. the harness (this is the "Postman companion"):
FLOW=serve \
BAP_SIGNING_PRIVATE_KEY=<mock BAP key, scratchpad bap.keys.env> \
BPP_URI=http://127.0.0.1:8092 \
cargo run -p beckn-bpp-adapter --bin mock_bap
#   → http://127.0.0.1:9095

# 3. for bookings past `confirm`: put a driver online
python3 <scratchpad>/driver_sim.py   # goes online, accepts the offer
```

## Postman

Import `beckn-bpp.postman_collection.json`. One variable: `harness`
(default `http://127.0.0.1:9095`). Then run the requests top to bottom —
**no body editing needed**; the harness fills defaults from its session:

| Request | What happens | Where the reply lands |
|---|---|---|
| `1. search` | signed search via gateway, new transaction | `on_search` in `/callbacks` |
| `2. select` | first catalog item, direct to BPP | `on_select` (quote) |
| `3. init` | mints the order id (auto-harvested) | `on_init` |
| `4. confirm` | REAL dispatch handoff | `on_confirm`, then unsolicited `on_status` RIDE_ASSIGNED after a driver accepts |
| `5. status` / `6. track` / `7. cancel` | lifecycle ops on the session order | `on_status` / `on_track` / `on_cancel` |

Poll `callbacks (all)` after each action (replies are async — usually
under a second). `session` shows what the harness knows
(transaction/bpp/item/order). Any request body like `{"message": …}`
overrides the default.

## Gotchas

- **Empty catalog** (`items: []` in on_search): no driver online near the
  pickup — run `driver_sim.py`, wait ~10 s (location drainer), search again.
- A driver with an **open Accepted booking is invisible** to discovery
  (the api routes their locations to the on-ride path). Cancel the stale
  booking (`7. cancel`, or the api's internal cancel endpoint) first.
- The harness holds ONE session (one transaction at a time). A new
  `search` resets it.
- This is a **dev tool**: root-level mock BAP key, no auth on :9095.
  Never expose it beyond localhost.
