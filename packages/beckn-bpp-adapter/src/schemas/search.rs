//! Inbound `search` intent parsing.
//!
//! The BAP's discovery intent (`message.intent`) carries the trip as
//! fulfillment stops: a `START` and an `END`, each with `location.gps` in
//! `"lat, lon"` form (nammayatri `Intent`/`Stop`/`gpsToText`). We need both to
//! price a trip; anything else is a malformed search for a mobility BPP.

use serde_json::Value;

use crate::adapters::LatLon;

/// The parsed trip a search asks us to quote.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SearchIntent {
    pub pickup: LatLon,
    pub dropoff: LatLon,
}

/// Parse `"lat, lon"` (comma-separated, optional whitespace).
pub fn parse_gps(gps: &str) -> Result<LatLon, String> {
    let (lat, lon) = gps
        .split_once(',')
        .ok_or_else(|| format!("gps `{gps}` is not `lat, lon`"))?;
    let lat: f64 = lat
        .trim()
        .parse()
        .map_err(|_| format!("gps latitude `{lat}` is not a number"))?;
    let lon: f64 = lon
        .trim()
        .parse()
        .map_err(|_| format!("gps longitude `{lon}` is not a number"))?;
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        return Err(format!("gps `{gps}` out of range"));
    }
    Ok(LatLon { lat, lon })
}

/// Extract pickup/drop from `message.intent.fulfillment.stops`.
pub fn parse_search_intent(message: &Value) -> Result<SearchIntent, String> {
    let stops = message
        .get("intent")
        .and_then(|i| i.get("fulfillment"))
        .and_then(|f| f.get("stops"))
        .and_then(Value::as_array)
        .ok_or("message.intent.fulfillment.stops missing")?;

    let gps_of = |wanted: &str| -> Result<LatLon, String> {
        let stop = stops
            .iter()
            .find(|s| s.get("type").and_then(Value::as_str) == Some(wanted))
            .ok_or_else(|| format!("no `{wanted}` stop in intent"))?;
        let gps = stop
            .get("location")
            .and_then(|l| l.get("gps"))
            .and_then(Value::as_str)
            .ok_or_else(|| format!("`{wanted}` stop has no location.gps"))?;
        parse_gps(gps)
    };

    Ok(SearchIntent {
        pickup: gps_of("START")?,
        dropoff: gps_of("END")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_intent() {
        let message = serde_json::json!({
            "intent": { "fulfillment": { "stops": [
                { "type": "START",
                  "location": { "gps": "-1.286389, 36.817223" } },
                { "type": "END",
                  "location": { "gps": "-1.319167,36.925833" } }
            ]}}
        });
        let intent = parse_search_intent(&message).unwrap();
        assert_eq!(
            intent.pickup,
            LatLon {
                lat: -1.286389,
                lon: 36.817223
            }
        );
        assert_eq!(
            intent.dropoff,
            LatLon {
                lat: -1.319167,
                lon: 36.925833
            }
        );
    }

    #[test]
    fn missing_end_stop_is_an_error() {
        let message = serde_json::json!({
            "intent": { "fulfillment": { "stops": [
                { "type": "START",
                  "location": { "gps": "-1.28, 36.81" } }
            ]}}
        });
        assert!(parse_search_intent(&message).unwrap_err().contains("END"));
    }

    #[test]
    fn rejects_bad_gps() {
        assert!(parse_gps("not-gps").is_err());
        assert!(parse_gps("91.0, 36.8").is_err());
        assert!(parse_gps("-1.28; 36.81").is_err());
        assert!(parse_gps(" -1.28 , 36.81 ").is_ok());
    }
}
