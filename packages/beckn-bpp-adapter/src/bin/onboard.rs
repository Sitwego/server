//! Register this BPP with the network registry (Step 8).
//!
//! ```bash
//! BECKN_REGISTRY_URL=http://localhost:3030/subscribers \
//! BECKN_SUBSCRIBER_URI=http://<adapter-host>:8092 \
//! BECKN_SIGNING_PRIVATE_KEY=... \
//! BECKN_ENCRYPTION_PUBLIC_KEY=... \
//! cargo run -p beckn-bpp-adapter --bin onboard
//! ```
//!
//! Sends the beckn-onix `POST {registry}/register` record; a registry admin
//! must then approve the participant (set its network role to SUBSCRIBED)
//! before it can transact — that manual step is the network's onboarding
//! ceremony, not an error.

use anyhow::Context;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use beckn_bpp_adapter::config::Config;
use beckn_bpp_adapter::crypto::onboarding::build_register_payload;
use beckn_bpp_adapter::crypto::signature::signing_key_from_b64;
use chrono::Utc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let config = Config::from_env()?;

    let registry = config
        .beckn_registry_url
        .clone()
        .context("BECKN_REGISTRY_URL is required (e.g. http://localhost:3030/subscribers)")?;
    let signing_private = config.beckn_signing_private_key.clone().context(
        "BECKN_SIGNING_PRIVATE_KEY is required — mint one with the keygen bin",
    )?;
    let encr_public = config
        .beckn_encryption_public_key
        .clone()
        .context("BECKN_ENCRYPTION_PUBLIC_KEY is required — mint one with the keygen bin")?;

    let signing_public = B64.encode(
        signing_key_from_b64(&signing_private)?.verifying_key().to_bytes(),
    );
    let payload = build_register_payload(
        &config,
        &signing_public,
        &encr_public,
        Utc::now(),
    );

    let url = format!("{}/register", registry.trim_end_matches('/'));
    println!("registering {} at {url}", config.beckn_subscriber_id);
    let mut request = reqwest::Client::new().post(&url).json(&payload);
    if let Some(api_key) = &config.beckn_registry_api_key {
        request = request.header("ApiKey", api_key);
    }
    let response = request.send().await.context("registry unreachable")?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    println!("registry answered HTTP {status}");
    if !body.trim().is_empty() {
        println!("{body}");
    }
    if !status.is_success() {
        anyhow::bail!("registration rejected");
    }

    println!();
    println!("Registered. Now have the registry admin approve it:");
    println!(
        "  Admin → Network Participants → {} → network role → SUBSCRIBED",
        config.beckn_subscriber_id
    );
    Ok(())
}
