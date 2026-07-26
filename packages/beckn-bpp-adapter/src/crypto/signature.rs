//! Beckn HTTP-signature primitives.
//!
//! Beckn signs requests with a profile of the IETF HTTP Signatures draft:
//! the signing string is
//!
//! ```text
//! (created): <unix seconds>
//! (expires): <unix seconds>
//! digest: BLAKE-512=<base64 of BLAKE2b-512(raw request body)>
//! ```
//!
//! signed with Ed25519 and carried in the `Authorization` header as
//!
//! ```text
//! Signature keyId="{subscriber_id}|{unique_key_id}|ed25519",algorithm="ed25519",
//!           created="…",expires="…",headers="(created) (expires) digest",
//!           signature="<base64>"
//! ```
//!
//! The digest is computed over the RAW request bytes exactly as received — the
//! body is never re-serialized before hashing (Inviolable Rule #7).

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use blake2::{Blake2b512, Digest};
use ed25519_dalek::{
    Signature, Signer as _, SigningKey, Verifier as _, VerifyingKey,
};

/// The `headers` list Beckn signatures cover. Fixed by the signing profile.
pub const SIGNED_HEADERS: &str = "(created) (expires) digest";

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SignatureError {
    #[error("malformed signature header: {0}")]
    Malformed(String),
    #[error("unsupported signature algorithm `{0}` (expected ed25519)")]
    UnsupportedAlgorithm(String),
    #[error(
        "signature outside its validity window (created={created}, expires={expires}, now={now})"
    )]
    OutsideValidityWindow {
        created: i64,
        expires: i64,
        now: i64,
    },
    #[error("invalid Ed25519 public key")]
    InvalidPublicKey,
    #[error("signature does not verify against the raw request body")]
    VerificationFailed,
}

/// Base64 of the BLAKE2b-512 digest of the raw body bytes.
pub fn body_digest_b64(raw_body: &[u8]) -> String {
    let mut hasher = Blake2b512::new();
    hasher.update(raw_body);
    B64.encode(hasher.finalize())
}

/// The exact string that gets signed. Line breaks and spacing are part of the
/// signature — any deviation breaks interop.
pub fn signing_string(created: i64, expires: i64, digest_b64: &str) -> String {
    format!(
        "(created): {created}\n(expires): {expires}\ndigest: BLAKE-512={digest_b64}"
    )
}

/// Build a complete `Authorization` header value for `raw_body`.
pub fn sign(
    raw_body: &[u8],
    subscriber_id: &str,
    unique_key_id: &str,
    signing_key: &SigningKey,
    created: i64,
    expires: i64,
) -> String {
    let digest = body_digest_b64(raw_body);
    let message = signing_string(created, expires, &digest);
    let signature = B64.encode(signing_key.sign(message.as_bytes()).to_bytes());
    format!(
        "Signature keyId=\"{subscriber_id}|{unique_key_id}|ed25519\",\
         algorithm=\"ed25519\",created=\"{created}\",expires=\"{expires}\",\
         headers=\"{SIGNED_HEADERS}\",signature=\"{signature}\""
    )
}

/// Decode a base64 Ed25519 private key. Accepts either the 32-byte seed or the
/// 64-byte seed+public concatenation (both appear in the wild).
pub fn signing_key_from_b64(b64: &str) -> anyhow::Result<SigningKey> {
    let bytes = B64.decode(b64.trim())?;
    match bytes.len() {
        32 => {
            let seed: [u8; 32] = bytes.try_into().expect("length checked");
            Ok(SigningKey::from_bytes(&seed))
        }
        64 => {
            let pair: [u8; 64] = bytes.try_into().expect("length checked");
            SigningKey::from_keypair_bytes(&pair)
                .map_err(|e| anyhow::anyhow!("invalid Ed25519 keypair: {e}"))
        }
        n => {
            anyhow::bail!("Ed25519 private key must be 32 or 64 bytes, got {n}")
        }
    }
}

/// A parsed `Authorization: Signature …` header.
#[derive(Debug, Clone, PartialEq)]
pub struct SignatureHeader {
    pub subscriber_id: String,
    pub unique_key_id: String,
    pub algorithm: String,
    pub created: i64,
    pub expires: i64,
    pub signature: Vec<u8>,
}

impl SignatureHeader {
    /// Parse the header value. Parameter values may be quoted or (for the
    /// integer timestamps, as the HTTP Signatures draft allows) bare.
    pub fn parse(header: &str) -> Result<Self, SignatureError> {
        let rest = header
            .trim()
            .strip_prefix("Signature")
            .ok_or_else(|| {
                SignatureError::Malformed(
                    "missing `Signature` scheme prefix".into(),
                )
            })?
            .trim_start();

        let mut key_id = None;
        let mut algorithm = None;
        let mut created = None;
        let mut expires = None;
        let mut headers = None;
        let mut signature = None;

        // Values are base64/pipe-delimited identifiers — none contain commas,
        // so splitting on ',' is safe.
        for part in rest.split(',') {
            let (k, v) = part.trim().split_once('=').ok_or_else(|| {
                SignatureError::Malformed(format!("bad parameter `{part}`"))
            })?;
            let v = v.trim().trim_matches('"');
            match k.trim() {
                "keyId" => key_id = Some(v.to_string()),
                "algorithm" => algorithm = Some(v.to_string()),
                "created" => created = Some(parse_ts(v)?),
                "expires" => expires = Some(parse_ts(v)?),
                "headers" => headers = Some(v.to_string()),
                "signature" => signature = Some(v.to_string()),
                // Unknown parameters are ignored (forward compatibility).
                _ => {}
            }
        }

        let key_id = key_id.ok_or_else(|| missing("keyId"))?;
        let mut key_parts = key_id.split('|');
        let subscriber_id = key_parts
            .next()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                SignatureError::Malformed("empty subscriber in keyId".into())
            })?
            .to_string();
        let unique_key_id = key_parts
            .next()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                SignatureError::Malformed(format!(
                    "keyId `{key_id}` is not `subscriber|ukid|algorithm`"
                ))
            })?
            .to_string();

        let headers = headers.ok_or_else(|| missing("headers"))?;
        if headers != SIGNED_HEADERS {
            return Err(SignatureError::Malformed(format!(
                "unsupported headers list `{headers}` (expected `{SIGNED_HEADERS}`)"
            )));
        }

        let signature = B64
            .decode(signature.ok_or_else(|| missing("signature"))?)
            .map_err(|e| {
                SignatureError::Malformed(format!("signature not base64: {e}"))
            })?;

        Ok(Self {
            subscriber_id,
            unique_key_id,
            algorithm: algorithm.ok_or_else(|| missing("algorithm"))?,
            created: created.ok_or_else(|| missing("created"))?,
            expires: expires.ok_or_else(|| missing("expires"))?,
            signature,
        })
    }

    /// Verify this signature over the RAW request bytes with the subscriber's
    /// base64 Ed25519 public key (as returned by the registry).
    pub fn verify(
        &self,
        raw_body: &[u8],
        public_key_b64: &str,
        now: i64,
    ) -> Result<(), SignatureError> {
        if !self.algorithm.eq_ignore_ascii_case("ed25519") {
            return Err(SignatureError::UnsupportedAlgorithm(
                self.algorithm.clone(),
            ));
        }
        if now < self.created || now > self.expires {
            return Err(SignatureError::OutsideValidityWindow {
                created: self.created,
                expires: self.expires,
                now,
            });
        }

        let key_bytes: [u8; 32] = B64
            .decode(public_key_b64.trim())
            .map_err(|_| SignatureError::InvalidPublicKey)?
            .try_into()
            .map_err(|_| SignatureError::InvalidPublicKey)?;
        let key = VerifyingKey::from_bytes(&key_bytes)
            .map_err(|_| SignatureError::InvalidPublicKey)?;

        let message = signing_string(
            self.created,
            self.expires,
            &body_digest_b64(raw_body),
        );
        let signature = Signature::from_slice(&self.signature)
            .map_err(|_| SignatureError::VerificationFailed)?;
        key.verify(message.as_bytes(), &signature)
            .map_err(|_| SignatureError::VerificationFailed)
    }
}

fn parse_ts(v: &str) -> Result<i64, SignatureError> {
    v.parse().map_err(|_| {
        SignatureError::Malformed(format!("`{v}` is not a unix timestamp"))
    })
}

fn missing(param: &str) -> SignatureError {
    SignatureError::Malformed(format!("missing `{param}` parameter"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn public_key_b64(key: &SigningKey) -> String {
        B64.encode(key.verifying_key().to_bytes())
    }

    #[test]
    fn signing_string_shape_is_exact() {
        assert_eq!(
            signing_string(100, 200, "abc"),
            "(created): 100\n(expires): 200\ndigest: BLAKE-512=abc"
        );
    }

    #[test]
    fn digest_is_blake2b_512() {
        // 64-byte digest → 88 base64 chars; differs per input.
        let d = body_digest_b64(b"hello");
        assert_eq!(d.len(), 88);
        assert_ne!(d, body_digest_b64(b"hello!"));
    }

    #[test]
    fn sign_verify_roundtrip() {
        let key = test_key();
        let body = br#"{"context":{"action":"search"}}"#;
        let header = sign(body, "bap.example.com", "key1", &key, 100, 200);

        let parsed = SignatureHeader::parse(&header).unwrap();
        assert_eq!(parsed.subscriber_id, "bap.example.com");
        assert_eq!(parsed.unique_key_id, "key1");
        parsed.verify(body, &public_key_b64(&key), 150).unwrap();
    }

    #[test]
    fn tampered_body_fails() {
        let key = test_key();
        let header =
            sign(b"original", "bap.example.com", "key1", &key, 100, 200);
        let parsed = SignatureHeader::parse(&header).unwrap();
        assert_eq!(
            parsed.verify(b"tampered", &public_key_b64(&key), 150),
            Err(SignatureError::VerificationFailed)
        );
    }

    #[test]
    fn outside_validity_window_fails() {
        let key = test_key();
        let body = b"x";
        let header = sign(body, "bap.example.com", "key1", &key, 100, 200);
        let parsed = SignatureHeader::parse(&header).unwrap();
        let pk = public_key_b64(&key);
        assert!(matches!(
            parsed.verify(body, &pk, 201),
            Err(SignatureError::OutsideValidityWindow { .. })
        ));
        assert!(matches!(
            parsed.verify(body, &pk, 99),
            Err(SignatureError::OutsideValidityWindow { .. })
        ));
    }

    #[test]
    fn wrong_key_fails() {
        let header =
            sign(b"body", "bap.example.com", "key1", &test_key(), 100, 200);
        let parsed = SignatureHeader::parse(&header).unwrap();
        let other = SigningKey::from_bytes(&[9u8; 32]);
        assert_eq!(
            parsed.verify(b"body", &public_key_b64(&other), 150),
            Err(SignatureError::VerificationFailed)
        );
    }

    #[test]
    fn parses_unquoted_timestamps() {
        // The HTTP Signatures draft allows bare integers for created/expires.
        let key = test_key();
        let quoted = sign(b"b", "s.example.com", "k", &key, 111, 222);
        let bare = quoted
            .replace("created=\"111\"", "created=111")
            .replace("expires=\"222\"", "expires=222");
        let parsed = SignatureHeader::parse(&bare).unwrap();
        assert_eq!(parsed.created, 111);
        assert_eq!(parsed.expires, 222);
    }

    #[test]
    fn rejects_non_signature_scheme() {
        assert!(matches!(
            SignatureHeader::parse("Bearer abc123"),
            Err(SignatureError::Malformed(_))
        ));
    }

    #[test]
    fn rejects_bad_key_id() {
        let header = "Signature keyId=\"no-pipes-here\",algorithm=\"ed25519\",\
                      created=\"1\",expires=\"2\",\
                      headers=\"(created) (expires) digest\",signature=\"AA==\"";
        assert!(matches!(
            SignatureHeader::parse(header),
            Err(SignatureError::Malformed(_))
        ));
    }
}
