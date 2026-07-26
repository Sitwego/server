//! Mint the BPP's subscriber keypairs (Step 8). Run ONCE per key id:
//!
//! ```bash
//! cargo run -p beckn-bpp-adapter --bin keygen
//! ```
//!
//! Prints the Ed25519 signing pair (callbacks are signed with it; peers
//! resolve the public half from the registry) and the X25519 encryption
//! pair (published in the registry record; unused by our registry's
//! onboarding but required by the record and by challenge-style networks).
//!
//! Store the private halves in the secrets manager — never in the repo,
//! never in a committed .env.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use ed25519_dalek::SigningKey;
use rand::RngCore;
use x25519_dalek::{PublicKey, StaticSecret};

fn main() {
    let mut signing_seed = [0u8; 32];
    rand::rng().fill_bytes(&mut signing_seed);
    let signing = SigningKey::from_bytes(&signing_seed);

    let mut encryption_seed = [0u8; 32];
    rand::rng().fill_bytes(&mut encryption_seed);
    let encryption = StaticSecret::from(encryption_seed);

    println!("# Ed25519 signing pair — private half to the secrets manager");
    println!("BECKN_SIGNING_PRIVATE_KEY={}", B64.encode(signing_seed));
    println!(
        "BECKN_SIGNING_PUBLIC_KEY={}",
        B64.encode(signing.verifying_key().to_bytes())
    );
    println!("# X25519 encryption pair — registry record's encr_public_key");
    println!(
        "BECKN_ENCRYPTION_PRIVATE_KEY={}",
        B64.encode(encryption_seed)
    );
    println!(
        "BECKN_ENCRYPTION_PUBLIC_KEY={}",
        B64.encode(PublicKey::from(&encryption).to_bytes())
    );
}
