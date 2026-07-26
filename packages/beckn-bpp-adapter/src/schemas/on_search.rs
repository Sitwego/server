//! The `on_search` catalog — the BPP's reply to a discovery `search`.
//!
//! Shapes mirror nammayatri's `BecknV2.OnDemand.Types` (Catalog / Provider /
//! Item / Fulfillment / Stop / Location / Price / Descriptor / Vehicle) with
//! `omitNothingFields` semantics: every optional field is skipped when absent,
//! nothing is invented (Inviolable Rule #8). Field-by-field provenance:
//! nammayatri `Beckn.OnDemand.Transformer.OnSearch`.

use serde::{Deserialize, Serialize};

use crate::adapters::{LatLon, Quote};

/// `message.catalog`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descriptor: Option<Descriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub providers: Option<Vec<Provider>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descriptor: Option<Descriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillments: Option<Vec<Fulfillment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Item>>,
    // `locations` and `payments` are deliberately absent for now: payment
    // terms await the 🔴 settlement decision (Step 5).
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Descriptor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_desc: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Fulfillment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Fulfillment mode. A one-way on-demand ride on the open network is
    /// `DELIVERY` (nammayatri `tripCategoryToFulfillmentType`).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub fulfillment_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stops: Option<Vec<Stop>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vehicle: Option<Vehicle>,
    /// The assigned driver (`on_status` RIDE_ASSIGNED on) — nammayatri
    /// `mkFulfillmentV2`'s `fulfillmentAgent`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<Agent>,
    /// The riding customer, echoed back on lifecycle callbacks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<Customer>,
    /// Ride lifecycle state (`on_confirm` on): `{descriptor: {code}}` with a
    /// network `FulfillmentState` code — NEW while allocating, RIDE_ASSIGNED
    /// once a driver claims, etc. Absent in the discovery catalog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<FulfillmentState>,
}

/// `fulfillment.state` wrapper (nammayatri `Spec.FulfillmentState`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descriptor: Option<Descriptor>,
}

/// `fulfillment.agent` — the driver serving the ride.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Agent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person: Option<Person>,
}

/// `fulfillment.customer`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Customer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person: Option<Person>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Person {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stop {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub stop_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    /// Ride-start authorization on the START stop (`{token, type: "OTP"}`) —
    /// present once a driver is assigned, per nammayatri `mkStopsOUS`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<Authorization>,
}

/// `stop.authorization` — the ride-start OTP handed to the BAP's customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authorization {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub authorization_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// `"lat, lon"`, 6 decimal places — nammayatri's `gpsToText` format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gps: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Vehicle {
    /// Network vehicle category code (CAB / AUTO_RICKSHAW / TWO_WHEELER).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    // `variant` is omitted: Sitwego doesn't track body types, and the spec
    // makes it optional — we don't claim what we don't know.
    /// Plate number — carried once a concrete vehicle is assigned (Step 7).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub make: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descriptor: Option<Descriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,
}

/// All numeric values are decimal strings on the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offered_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_value: Option<String>,
}

/// `gpsToText`: `"%.6f, %.6f"` as `lat, lon`.
pub fn gps_text(point: LatLon) -> String {
    format!("{:.6}, {:.6}", point.lat, point.lon)
}

/// Build the `on_search` catalog for a quote: one fulfillment + one item per
/// bookable tier, linked by the tier code, exactly as nammayatri links each
/// pricing's estimate id.
pub fn build_catalog(
    provider_id: &str,
    provider_name: &str,
    currency: &str,
    quote: &Quote,
    pickup: LatLon,
    dropoff: LatLon,
) -> Catalog {
    let stops = vec![
        Stop {
            stop_type: Some("START".to_string()),
            location: Some(Location {
                gps: Some(gps_text(pickup)),
                address: None,
            }),
            ..Default::default()
        },
        Stop {
            stop_type: Some("END".to_string()),
            location: Some(Location {
                gps: Some(gps_text(dropoff)),
                address: None,
            }),
            ..Default::default()
        },
    ];

    let fulfillments = quote
        .options
        .iter()
        .map(|opt| Fulfillment {
            id: Some(opt.tier_code.clone()),
            fulfillment_type: Some("DELIVERY".to_string()),
            stops: Some(stops.clone()),
            vehicle: Some(Vehicle {
                category: Some(opt.vehicle_category.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    let items = quote
        .options
        .iter()
        .map(|opt| {
            // Whole-KES fares as decimal strings; a single-point estimate, so
            // value/offered/min/max coincide.
            let fare = format!("{}", opt.fare.round() as i64);
            Item {
                id: Some(opt.tier_code.clone()),
                descriptor: Some(Descriptor {
                    code: Some(opt.tier_code.clone()),
                    name: Some(opt.tier_name.clone()),
                    short_desc: None,
                }),
                fulfillment_ids: Some(vec![opt.tier_code.clone()]),
                price: Some(Price {
                    currency: Some(currency.to_string()),
                    value: Some(fare.clone()),
                    offered_value: Some(fare.clone()),
                    minimum_value: Some(fare.clone()),
                    maximum_value: Some(fare),
                }),
            }
        })
        .collect::<Vec<_>>();

    Catalog {
        descriptor: Some(Descriptor {
            code: None,
            name: Some(provider_name.to_string()),
            short_desc: None,
        }),
        providers: Some(vec![Provider {
            id: Some(provider_id.to_string()),
            descriptor: Some(Descriptor {
                code: None,
                name: Some(provider_name.to_string()),
                short_desc: None,
            }),
            fulfillments: Some(fulfillments),
            items: Some(items),
        }]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::QuoteOption;

    fn sample_quote() -> Quote {
        Quote {
            options: vec![
                QuoteOption {
                    tier_code: "SWIFT".into(),
                    tier_name: "Swift".into(),
                    vehicle_category: "CAB",
                    fare: 260.4,
                    base_fare: 100,
                    distance_fare: 120,
                    time_fare: 40,
                    waiting_fare: 0,
                },
                QuoteOption {
                    tier_code: "BIKE".into(),
                    tier_name: "Bike".into(),
                    vehicle_category: "TWO_WHEELER",
                    fare: 120.0,
                    base_fare: 50,
                    distance_fare: 50,
                    time_fare: 20,
                    waiting_fare: 0,
                },
            ],
            distance_km: 5.2,
            duration_s: 780,
        }
    }

    #[test]
    fn gps_text_is_lat_lon_6dp() {
        let nairobi = LatLon {
            lat: -1.286389,
            lon: 36.817223,
        };
        assert_eq!(gps_text(nairobi), "-1.286389, 36.817223");
    }

    #[test]
    fn catalog_wire_shape() {
        let pickup = LatLon {
            lat: -1.286389,
            lon: 36.817223,
        };
        let dropoff = LatLon {
            lat: -1.319167,
            lon: 36.925833,
        };
        let catalog = build_catalog(
            "sitwego.mobility.ke",
            "Sitwego",
            "KES",
            &sample_quote(),
            pickup,
            dropoff,
        );
        let v = serde_json::to_value(&catalog).unwrap();

        assert_eq!(v["descriptor"]["name"], "Sitwego");
        let provider = &v["providers"][0];
        assert_eq!(provider["id"], "sitwego.mobility.ke");

        // One item + one fulfillment per tier, linked by id.
        assert_eq!(provider["items"].as_array().unwrap().len(), 2);
        assert_eq!(provider["fulfillments"].as_array().unwrap().len(), 2);
        let item = &provider["items"][0];
        assert_eq!(item["id"], "SWIFT");
        assert_eq!(item["descriptor"]["code"], "SWIFT");
        assert_eq!(item["descriptor"]["name"], "Swift");
        assert_eq!(item["fulfillment_ids"][0], "SWIFT");
        // Rounded whole-KES decimal string.
        assert_eq!(item["price"]["currency"], "KES");
        assert_eq!(item["price"]["value"], "260");

        let fulfillment = &provider["fulfillments"][0];
        assert_eq!(fulfillment["id"], "SWIFT");
        assert_eq!(fulfillment["type"], "DELIVERY");
        assert_eq!(fulfillment["vehicle"]["category"], "CAB");
        let stops = fulfillment["stops"].as_array().unwrap();
        assert_eq!(stops[0]["type"], "START");
        assert_eq!(stops[0]["location"]["gps"], "-1.286389, 36.817223");
        assert_eq!(stops[1]["type"], "END");

        // Bike maps to TWO_WHEELER (nammayatri castVariant), not MOTORCYCLE.
        assert_eq!(
            provider["fulfillments"][1]["vehicle"]["category"],
            "TWO_WHEELER"
        );

        // Nothing we didn't set leaks onto the wire.
        assert!(item.get("payment_ids").is_none());
        assert!(fulfillment.get("agent").is_none());
    }
}
