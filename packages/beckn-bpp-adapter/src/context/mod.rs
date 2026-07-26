//! The Beckn `context` envelope.
//!
//! Every Beckn message — inbound action and outbound `on_*` callback alike —
//! carries a `context`. This module parses it, validates it, and builds the
//! reply context for the matching callback.
//!
//! Field set is taken from the Beckn protocol `context` schema. We model the
//! fields the adapter actually reasons about as typed members and preserve any
//! additional spec fields verbatim via `extra` so a round-trip never drops data
//! (Inviolable Rule #8: do not invent — and do not silently discard — schema).

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The eight Beckn actions this BPP implements. Each has an `on_*` counterpart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Search,
    Select,
    Init,
    Confirm,
    Status,
    Track,
    Update,
    Cancel,
}

impl Action {
    /// The callback action name POSTed back to the BAP, e.g. `search` → `on_search`.
    pub fn callback_name(self) -> &'static str {
        match self {
            Action::Search => "on_search",
            Action::Select => "on_select",
            Action::Init => "on_init",
            Action::Confirm => "on_confirm",
            Action::Status => "on_status",
            Action::Track => "on_track",
            Action::Update => "on_update",
            Action::Cancel => "on_cancel",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Action::Search => "search",
            Action::Select => "select",
            Action::Init => "init",
            Action::Confirm => "confirm",
            Action::Status => "status",
            Action::Track => "track",
            Action::Update => "update",
            Action::Cancel => "cancel",
        }
    }
}

/// The Beckn `context` object.
///
/// Unknown/extra spec fields (e.g. `location`, `version`, `key`) are captured in
/// `extra` and re-emitted unchanged so we neither drop nor invent fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub domain: String,
    pub action: Action,
    pub bap_id: String,
    pub bap_uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bpp_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bpp_uri: Option<String>,
    pub transaction_id: String,
    pub message_id: String,
    pub timestamp: DateTime<Utc>,
    /// ISO-8601 duration (e.g. `PT30S`). Optional in the spec; when present it
    /// bounds how long this message stays valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>,
    /// Beckn core version, e.g. `2.0.0`. Validated against the version this BPP
    /// supports — nammayatri's `validateContext` rejects a mismatch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Any additional spec-defined context fields (e.g. `location`, `key`),
    /// preserved verbatim.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Reasons a context fails validation. Mapped to NACK errors by the caller.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ContextError {
    #[error("missing or empty required field: {0}")]
    MissingField(&'static str),
    #[error("context.action `{found}` does not match route `{expected}`")]
    ActionMismatch {
        expected: &'static str,
        found: String,
    },
    #[error("message has expired (timestamp + ttl is in the past)")]
    Expired,
    #[error("invalid ttl duration: {0}")]
    InvalidTtl(String),
    #[error(
        "unsupported core version `{found}` (this BPP supports `{expected}`)"
    )]
    UnsupportedVersion { expected: String, found: String },
    #[error("missing context.version")]
    MissingVersion,
}

impl Context {
    /// Structural validation of the required fields. Does NOT check ttl expiry —
    /// see [`Context::check_not_expired`] — so callers can report the precise
    /// failure.
    pub fn validate_required(&self) -> Result<(), ContextError> {
        if self.domain.trim().is_empty() {
            return Err(ContextError::MissingField("domain"));
        }
        if self.bap_id.trim().is_empty() {
            return Err(ContextError::MissingField("bap_id"));
        }
        if self.bap_uri.trim().is_empty() {
            return Err(ContextError::MissingField("bap_uri"));
        }
        if self.transaction_id.trim().is_empty() {
            return Err(ContextError::MissingField("transaction_id"));
        }
        if self.message_id.trim().is_empty() {
            return Err(ContextError::MissingField("message_id"));
        }
        Ok(())
    }

    /// Assert that `context.action` matches the route the request arrived on.
    pub fn check_action(&self, expected: Action) -> Result<(), ContextError> {
        if self.action == expected {
            Ok(())
        } else {
            Err(ContextError::ActionMismatch {
                expected: expected.as_str(),
                found: self.action.as_str().to_string(),
            })
        }
    }

    /// Assert the message's core version matches the one this BPP supports.
    pub fn check_version(&self, supported: &str) -> Result<(), ContextError> {
        match self.version.as_deref() {
            None => Err(ContextError::MissingVersion),
            Some(v) if v == supported => Ok(()),
            Some(v) => Err(ContextError::UnsupportedVersion {
                expected: supported.to_string(),
                found: v.to_string(),
            }),
        }
    }

    /// Validate ttl against `now`. No ttl ⇒ never expires.
    pub fn check_not_expired(
        &self,
        now: DateTime<Utc>,
    ) -> Result<(), ContextError> {
        let Some(ref ttl) = self.ttl else {
            return Ok(());
        };
        let dur = parse_iso8601_duration(ttl)
            .ok_or_else(|| ContextError::InvalidTtl(ttl.clone()))?;
        if self.timestamp + dur < now {
            Err(ContextError::Expired)
        } else {
            Ok(())
        }
    }

    /// Build the `context` for the `on_*` callback that answers this request.
    ///
    /// Per spec the callback echoes `transaction_id`/`message_id`, flips the
    /// action to its `on_*` form, stamps a fresh `timestamp`, and fills in our
    /// `bpp_id`/`bpp_uri`. `extra` is carried through so spec fields like
    /// `location`/`version` survive the round-trip.
    pub fn to_callback(&self, bpp_id: &str, bpp_uri: &str) -> CallbackContext {
        CallbackContext {
            domain: self.domain.clone(),
            action: self.action.callback_name(),
            bap_id: self.bap_id.clone(),
            bap_uri: self.bap_uri.clone(),
            bpp_id: bpp_id.to_string(),
            bpp_uri: bpp_uri.to_string(),
            transaction_id: self.transaction_id.clone(),
            message_id: self.message_id.clone(),
            timestamp: Utc::now(),
            version: self.version.clone(),
            extra: self.extra.clone(),
        }
    }
}

/// The `context` we emit on `on_*` callbacks. `action` is the `on_*` string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackContext {
    pub domain: String,
    pub action: &'static str,
    pub bap_id: String,
    pub bap_uri: String,
    pub bpp_id: String,
    pub bpp_uri: String,
    pub transaction_id: String,
    pub message_id: String,
    pub timestamp: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl CallbackContext {
    /// Build the context for an UNSOLICITED callback — one not answering any
    /// inbound message (e.g. `on_status` RIDE_ASSIGNED after decision B's
    /// deferred driver announcement). Per nammayatri's `buildOnStatusReqV2`
    /// it keeps the order's `transaction_id` but mints a fresh `message_id`.
    #[allow(clippy::too_many_arguments)]
    pub fn unsolicited(
        action: &'static str,
        domain: &str,
        bap_id: &str,
        bap_uri: &str,
        bpp_id: &str,
        bpp_uri: &str,
        transaction_id: &str,
        version: &str,
    ) -> Self {
        Self {
            domain: domain.to_string(),
            action,
            bap_id: bap_id.to_string(),
            bap_uri: bap_uri.to_string(),
            bpp_id: bpp_id.to_string(),
            bpp_uri: bpp_uri.to_string(),
            transaction_id: transaction_id.to_string(),
            message_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            version: Some(version.to_string()),
            extra: BTreeMap::new(),
        }
    }
}

/// Minimal ISO-8601 *duration* parser covering the time component Beckn uses
/// for ttl: `PnYnMnDTnHnMnS`. We only need the `T...` (hours/minutes/seconds)
/// portion in practice (e.g. `PT30S`, `PT1M`, `PT1H30M`); date components are
/// accepted and converted with calendar-agnostic approximations (D = 24h).
/// Returns `None` on anything malformed.
pub fn parse_iso8601_duration(s: &str) -> Option<chrono::Duration> {
    let s = s.trim();
    let rest = s.strip_prefix('P')?;
    if rest.is_empty() {
        return None;
    }

    let (date_part, time_part) = match rest.split_once('T') {
        Some((d, t)) => (d, t),
        None => (rest, ""),
    };

    let mut total = chrono::Duration::zero();
    let mut saw_any = false;

    // Date component: only D (and W) carry into a fixed duration sensibly.
    let mut num = String::new();
    for c in date_part.chars() {
        if c.is_ascii_digit() {
            num.push(c);
        } else {
            let v: i64 = num.parse().ok()?;
            num.clear();
            saw_any = true;
            match c {
                'D' => total += chrono::Duration::days(v),
                'W' => total += chrono::Duration::weeks(v),
                // Years/months are not fixed-length; reject rather than guess.
                'Y' | 'M' => return None,
                _ => return None,
            }
        }
    }
    if !num.is_empty() {
        return None; // trailing digits with no unit
    }

    num.clear();
    for c in time_part.chars() {
        if c.is_ascii_digit() {
            num.push(c);
        } else {
            let v: i64 = num.parse().ok()?;
            num.clear();
            saw_any = true;
            match c {
                'H' => total += chrono::Duration::hours(v),
                'M' => total += chrono::Duration::minutes(v),
                'S' => total += chrono::Duration::seconds(v),
                _ => return None,
            }
        }
    }
    if !num.is_empty() {
        return None;
    }

    if saw_any { Some(total) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_json(action: &str, ttl: &str) -> serde_json::Value {
        serde_json::json!({
            "domain": "mobility",
            "action": action,
            "bap_id": "bap.example.com",
            "bap_uri": "https://bap.example.com/beckn",
            "transaction_id": "txn-1",
            "message_id": "msg-1",
            "timestamp": "2026-06-28T10:00:00Z",
            "ttl": ttl,
            "version": "2.0.0",
            // a spec field we don't model explicitly — must survive round-trip:
            "location": { "country": { "code": "KEN" }, "city": { "code": "std:254" } }
        })
    }

    #[test]
    fn parses_and_roundtrips_preserving_extra() {
        let json = sample_json("search", "PT30S");
        let ctx: Context = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(ctx.action, Action::Search);
        assert_eq!(ctx.transaction_id, "txn-1");
        assert!(ctx.extra.contains_key("location"));

        let back = serde_json::to_value(&ctx).unwrap();
        assert_eq!(back["location"], json["location"]);
        assert_eq!(back["action"], "search");
        assert_eq!(back["ttl"], "PT30S");
        assert_eq!(back["version"], "2.0.0");
    }

    #[test]
    fn version_match_and_mismatch() {
        let ctx: Context =
            serde_json::from_value(sample_json("search", "PT30S")).unwrap();
        assert_eq!(ctx.check_version("2.0.0"), Ok(()));
        assert!(matches!(
            ctx.check_version("1.1.0"),
            Err(ContextError::UnsupportedVersion { .. })
        ));

        let mut no_ver = sample_json("search", "PT30S");
        no_ver.as_object_mut().unwrap().remove("version");
        let ctx: Context = serde_json::from_value(no_ver).unwrap();
        assert_eq!(
            ctx.check_version("2.0.0"),
            Err(ContextError::MissingVersion)
        );
    }

    #[test]
    fn callback_flips_action_and_fills_bpp() {
        let ctx: Context =
            serde_json::from_value(sample_json("search", "PT30S")).unwrap();
        let cb =
            ctx.to_callback("sitwego.mobility.ke", "https://sitwego.ke/beckn");
        assert_eq!(cb.action, "on_search");
        assert_eq!(cb.bpp_id, "sitwego.mobility.ke");
        assert_eq!(cb.transaction_id, "txn-1");
        assert!(cb.extra.contains_key("location"));
    }

    #[test]
    fn missing_required_field_is_rejected() {
        let mut json = sample_json("search", "PT30S");
        json["bap_uri"] = serde_json::Value::String(String::new());
        let ctx: Context = serde_json::from_value(json).unwrap();
        assert_eq!(
            ctx.validate_required(),
            Err(ContextError::MissingField("bap_uri"))
        );
    }

    #[test]
    fn action_mismatch_detected() {
        let ctx: Context =
            serde_json::from_value(sample_json("select", "PT30S")).unwrap();
        assert!(matches!(
            ctx.check_action(Action::Search),
            Err(ContextError::ActionMismatch { .. })
        ));
    }

    #[test]
    fn ttl_expiry_is_enforced() {
        let ctx: Context =
            serde_json::from_value(sample_json("search", "PT30S")).unwrap();
        // timestamp is 10:00:00Z, ttl 30s → expires 10:00:30Z.
        let after = "2026-06-28T10:05:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(ctx.check_not_expired(after), Err(ContextError::Expired));

        let within = "2026-06-28T10:00:10Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(ctx.check_not_expired(within), Ok(()));
    }

    #[test]
    fn no_ttl_never_expires() {
        let mut json = sample_json("search", "PT30S");
        json.as_object_mut().unwrap().remove("ttl");
        let ctx: Context = serde_json::from_value(json).unwrap();
        let far_future =
            "2099-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(ctx.check_not_expired(far_future), Ok(()));
    }

    #[test]
    fn duration_parser_cases() {
        assert_eq!(
            parse_iso8601_duration("PT30S"),
            Some(chrono::Duration::seconds(30))
        );
        assert_eq!(
            parse_iso8601_duration("PT1M"),
            Some(chrono::Duration::minutes(1))
        );
        assert_eq!(
            parse_iso8601_duration("PT1H30M"),
            Some(chrono::Duration::hours(1) + chrono::Duration::minutes(30))
        );
        assert_eq!(
            parse_iso8601_duration("P1D"),
            Some(chrono::Duration::days(1))
        );
        assert_eq!(parse_iso8601_duration(""), None);
        assert_eq!(parse_iso8601_duration("30S"), None); // missing P
        assert_eq!(parse_iso8601_duration("PT"), None); // no components
        assert_eq!(parse_iso8601_duration("PT1Y"), None); // not fixed length here
    }
}
