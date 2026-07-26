#!/usr/bin/env bash
# Bring up the open mobility network's Registry + Gateway.
#
# Mirrors the beckn-onix `main-pre-plugins` installer's sequence
# (install/beckn-onix.sh) with bind-mounted config instead of external
# volumes: render config → up registry → import role permissions → up
# gateway → gateway self-subscribes as BG. Idempotent; re-run freely.
#
# Usage:  cp network.env.example network.env   # edit values
#         ./bootstrap.sh
set -euo pipefail
cd "$(dirname "$0")"

[ -f network.env ] || {
    echo "network.env missing — cp network.env.example network.env and edit it" >&2
    exit 1
}
# shellcheck disable=SC1091
source network.env

# ── Split URLs into scheme/host/port the property files want ────────────────
url_scheme() { echo "$1" | sed -E 's#^(https?)://.*#\1#'; }
url_host() { echo "$1" | sed -E 's#^https?://([^:/]+).*#\1#'; }
url_port() {
    local p
    p=$(echo "$1" | sed -nE 's#^https?://[^:/]+:([0-9]+).*#\1#p')
    if [ -z "$p" ]; then
        [ "$(url_scheme "$1")" = "https" ] && p=443 || p=80
    fi
    echo "$p"
}

render() { # render <template> <dest>
    sed -e "s#{{NETWORK_NAME}}#${NETWORK_NAME}#g" \
        -e "s#{{REGISTRY_PUBLIC_URL}}#${REGISTRY_PUBLIC_URL}#g" \
        -e "s#{{REGISTRY_SCHEME}}#$(url_scheme "$REGISTRY_PUBLIC_URL")#g" \
        -e "s#{{REGISTRY_HOST}}#$(url_host "$REGISTRY_PUBLIC_URL")#g" \
        -e "s#{{REGISTRY_PORT}}#$(url_port "$REGISTRY_PUBLIC_URL")#g" \
        -e "s#{{GATEWAY_SCHEME}}#$(url_scheme "$GATEWAY_PUBLIC_URL")#g" \
        -e "s#{{GATEWAY_HOST}}#$(url_host "$GATEWAY_PUBLIC_URL")#g" \
        -e "s#{{GATEWAY_PORT}}#$(url_port "$GATEWAY_PUBLIC_URL")#g" \
        -e "s#{{GATEWAY_SUBSCRIBER_ID}}#${GATEWAY_SUBSCRIBER_ID}#g" \
        -e "s#{{COUNTRY_ISO3}}#${COUNTRY_ISO3}#g" \
        -e "s#{{COUNTRY_ISO2}}#${COUNTRY_ISO2}#g" \
        "$1" >"$2"
}

echo "▸ rendering config"
mkdir -p registry/config gateway/config/networks
render registry/swf.properties.template registry/config/swf.properties
render gateway/swf.properties.template gateway/config/swf.properties
render gateway/network.json.template \
    "gateway/config/networks/${NETWORK_NAME}.json"
# The bind mount shadows the image's whole overrideProperties/config dir,
# whose startup script also sources envvars (ports) and reads
# logger.properties — restore the vendored image defaults alongside our
# rendered swf.properties.
for svc in registry gateway; do
    cp "$svc/envvars.default" "$svc/config/envvars"
    cp "$svc/logger.properties.default" "$svc/config/logger.properties"
done

echo "▸ starting registry"
docker compose up -d registry

# The registry needs a moment before it accepts logins (upstream waits 10 s
# blindly; we poll).
echo "▸ waiting for registry login"
API_KEY=""
for _ in $(seq 1 30); do
    API_KEY=$(curl -s -H 'Content-Type: application/json' \
        -H 'Accept: application/json' \
        -d '{"Name":"root","Password":"root"}' \
        http://localhost:3030/login | tr -d '\n' |
        sed -nE 's/.*"api_key" *: *"([^"]+)".*/\1/p') || true
    [ -n "$API_KEY" ] && break
    sleep 3
done
[ -n "$API_KEY" ] || {
    echo "registry never accepted the default login — check 'docker logs beckn-registry'" >&2
    exit 1
}

# Role permissions: same import the upstream installer performs
# (scripts/registry_role_permissions.sh). The xlsx is fetched from the
# pinned upstream branch, not committed here as a binary.
if [ ! -f RolePermission.xlsx ]; then
    echo "▸ fetching RolePermission.xlsx (beckn-onix@main-pre-plugins)"
    curl -sfLo RolePermission.xlsx \
        https://raw.githubusercontent.com/beckn/beckn-onix/main-pre-plugins/install/scripts/RolePermission.xlsx
fi
echo "▸ importing role permissions"
curl -s -o /dev/null -w '  importxls → HTTP %{http_code}\n' \
    -H "ApiKey: $API_KEY" \
    -F "datafile=@RolePermission.xlsx" \
    http://localhost:3030/role_permissions/importxls || true

# Network domains: the registry rejects any /register whose domain is not
# in its network_domains table ("Invalid domain ..."). The upstream
# installer expects you to create them by hand in the admin UI; they are
# network config, so we seed them here.
echo "▸ seeding network domains"
EXISTING=$(curl -s -H "ApiKey: $API_KEY" -H 'Accept: application/json' \
    http://localhost:3030/network_domains)
for domain in ${NETWORK_DOMAINS:-mobility}; do
    if echo "$EXISTING" | grep -q "\"name\" : \"$domain\""; then
        echo "  domain '$domain' already present"
        continue
    fi
    curl -s -o /dev/null -w "  domain '$domain' → HTTP %{http_code}\n" \
        -H "ApiKey: $API_KEY" -H 'Content-Type: application/json' \
        -H 'Accept: application/json' -d "{\"name\":\"$domain\"}" \
        http://localhost:3030/network_domains/save
done

echo "▸ starting gateway"
docker compose up -d gateway
sleep 10

# Gateway self-registers with the registry as the network's BG
# (upstream scripts/register_gateway.sh: POST-login on the gateway admin —
# the framework ignores GET logins — then GET /bg/subscribe with the
# session cookie).
echo "▸ gateway self-subscribe"
curl -s -o /dev/null --request POST --cookie-jar /tmp/beckn-gw-cookies.txt \
    "http://localhost:4030/login?name=root&password=root&_LOGIN=Login"
curl -s --cookie /tmp/beckn-gw-cookies.txt -o /dev/null \
    -w '  /bg/subscribe → HTTP %{http_code}\n' \
    "http://localhost:4030/bg/subscribe"
rm -f /tmp/beckn-gw-cookies.txt

# The BG record only appears after the gateway's async call back to the
# registry — verify it landed.
echo "▸ verifying BG record in registry"
BG_SEEN=""
for _ in $(seq 1 10); do
    if curl -s -H 'Content-Type: application/json' -d '{"type":"BG"}' \
        http://localhost:3030/subscribers/lookup | grep -q subscriber_id; then
        BG_SEEN=yes
        break
    fi
    sleep 3
done
if [ -n "$BG_SEEN" ]; then
    echo "  gateway is registered as BG"
else
    echo "  WARNING: no BG record yet — check 'docker logs beckn-gateway'" >&2
fi

cat <<EOF

Network is up.
  Registry admin : http://localhost:3030   (root / root — CHANGE THIS)
  Gateway        : http://localhost:4030   (admin + protocol at /bg/search)

Next:
  1. Log into the registry admin and change the root password.
  2. Approve the gateway: Admin → Network Participants → set the BG's
     network role to SUBSCRIBED (if not already).
  3. Onboard Sitwego as the first BPP:
       cargo run -p beckn-bpp-adapter --bin keygen        # mint keys once
       cargo run -p beckn-bpp-adapter --bin onboard       # register
     then approve it the same way (see README).
EOF
