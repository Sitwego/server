# Open mobility network — Registry + Gateway

The network infrastructure Sitwego's Beckn BPP adapter *joins* (spec §3).
There is no existing Beckn **mobility** network with a live registry in
Kenya, so we are founding one — and it only works as an open network if this
stack is credibly neutral:

- **Organisationally and operationally separate from Sitwego-the-app.** Own
  host, own deploy lifecycle, own on-call. A second operator or a county
  joins *the network*, not "Sitwego's network". (This directory lives in the
  backend repo for bootstrapping convenience; plan to move it to its own
  repo/org before Phase 3 onboards an external participant.)
- Sitwego is merely the network's **first BPP**; the Sitwego rider app will
  be its first BAP. Open by design, single-participant in practice, until a
  second participant onboards.

The services are the beckn-onix reference deployment (MIT), pinned to the
`main-pre-plugins` branch of `beckn/beckn-onix`:

| Service  | Image                | Port                  | Role |
|----------|----------------------|-----------------------|------|
| Registry | `fidedocker/registry`| 3030 (admin UI + API) | Subscriber directory + public-key store |
| Gateway  | `fidedocker/gateway` | 4030 (admin + protocol; BAPs POST `search` to `/bg/search`) | Broadcasts BAP `search` to every subscribed BPP via registry lookup |

(The images also open 3000/4000 internally — those are JDWP Java debug
ports, deliberately not published.)

The BPP adapter is **not** part of this stack (Inviolable Rule #1) — it is a
subscriber like any other.

## Bring-up (local dev)

```bash
cp network.env.example network.env    # edit names/URLs
./bootstrap.sh
```

`bootstrap.sh` renders the config templates, starts the registry, imports
the upstream role-permission sheet, seeds the network domains
(`NETWORK_DOMAINS` — the registry rejects `/register` for any domain not
in its `network_domains` table with "Invalid domain ..."), starts the
gateway, triggers the gateway's self-registration as the network's BG
(POST-login + `/bg/subscribe`), and verifies the BG record landed. Then,
in the registry admin UI (http://localhost:3030, `root`/`root`):

1. **Change the root password.** Immediately, even in dev.
2. *Admin → Network Participants*: confirm the gateway's network role is
   `SUBSCRIBED` (approve it if not).

## Onboarding Sitwego as the first BPP

1. **Mint the subscriber keys** (once; store in the secrets manager, never
   in the repo):

   ```bash
   cargo run -p beckn-bpp-adapter --bin keygen
   ```

   Prints `BECKN_SIGNING_{PRIVATE,PUBLIC}_KEY` (Ed25519) and
   `BECKN_ENCRYPTION_{PRIVATE,PUBLIC}_KEY` (X25519). The encryption pair is
   not used by this registry's onboarding (no challenge handshake — see
   below) but is part of the registry record and required by other Beckn
   networks; mint and keep it now.

2. **Register with the registry** (reads the adapter's normal env):

   ```bash
   BECKN_REGISTRY_URL=http://localhost:3030/subscribers \
   BECKN_SUBSCRIBER_ID=sitwego.mobility.ke \
   BECKN_SUBSCRIBER_URI=http://<adapter-host>:8092 \
   BECKN_SIGNING_PRIVATE_KEY=... \
   BECKN_ENCRYPTION_PUBLIC_KEY=... \
   cargo run -p beckn-bpp-adapter --bin onboard
   ```

3. **Approve it**: registry admin → *Network Participants* → the BPP record
   → network role → `SUBSCRIBED`. Only subscribed participants can transact.

4. Point the adapter at the network:

   ```bash
   BECKN_REGISTRY_URL=http://localhost:3030/subscribers   # /lookup appended by the adapter
   BECKN_GATEWAY_URL=http://localhost:4030/bg
   BECKN_VERIFY_SIGNATURES=true
   ```

### On the challenge handshake

The generic Beckn Subscribe API describes an encrypted-challenge
`/subscribe` → `/on_subscribe` handshake. This registry does not exercise
it: onboarding is `POST {registry}/register` followed by manual admin
approval, which is the correct trust ceremony for a founding network with
a human operator. If we ever join a network whose registry enforces the
challenge (e.g. ONDC-style), the adapter needs an `/on_subscribe` endpoint
that decrypts the challenge with the X25519 encryption key — tracked as
future work, deliberately unimplemented until a real registry exercises it.

### Key rotation

Registry records carry `valid_from`/`valid_until` (the onboard tool sets
~3 years) and a `unique_key_id`. To rotate: mint new keys with `keygen`,
register the new key under a bumped `unique_key_id` (e.g. `sitwego-key-2`),
deploy the adapter with the new private key + key id, then retire the old
record in the registry. Peers resolve keys per `(subscriber_id,
unique_key_id)` from the signature's `keyId`, so both keys verify during
the overlap.

## Production notes

- Same compose file behind Traefik: terminate TLS at Traefik and route
  `registry.<network-domain>` → :3030 and `gateway.<network-domain>` → :4030.
  Set the `*_PUBLIC_URL`s in `network.env` to the
  public hostnames before bootstrapping — the gateway advertises
  `GATEWAY_PUBLIC_URL` to BAPs via the registry record.
- The registry's H2 database lives in the `registry_database` volume —
  back it up; it *is* the network's subscriber directory.
- Keep this stack's lifecycle independent of the app's deploy pipeline.
