//! `select`/`init` orders — inbound parsing and the `on_select`/`on_init`
//! reply order.
//!
//! Shapes mirror nammayatri's provider-side transforms (`Beckn.ACL.OnSelect`,
//! `Beckn.ACL.OnInit` over `BecknV2.OnDemand.Types`), with `omitNothingFields`
//! semantics — optional fields are skipped when absent, nothing is invented
//! (Inviolable Rule #8). The reply `order` carries `provider{id}`,
//! `fulfillments`, `items`, `quote{breakup, price, ttl}` and `payments`;
//! `on_init` additionally carries the order `id`.
//!
//! Payment terms encode John's settlement decision (2026-07-03): the rider
//! pays the driver directly when the ride completes — `collected_by = "BPP"`,
//! `type = "ON-FULFILLMENT"`, `status = "NOT-PAID"` until then. No money moves
//! BAP→BPP, so there are no SETTLEMENT_TERMS, and the buyer-finder fee is an
//! explicit zero: drivers keep 100%, Sitwego's revenue is the driver
//! subscription, not a per-ride cut.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::adapters::{LatLon, QuoteOption};
use crate::schemas::on_search::{
    Agent, Authorization, Contact, Descriptor, Fulfillment, FulfillmentState,
    Item, Location, Person, Price, Stop, Vehicle, gps_text,
};
use crate::schemas::search::parse_gps;

/// How long the quoted price stays honoured (ISO-8601). Fares are re-priced on
/// every quote-phase action, so a short window is honest.
pub const QUOTE_TTL: &str = "PT15M";

/// What an inbound `select`/`init` asks for: one of our catalog items (a tier
/// code from `on_search`) plus the trip it was quoted for.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderSelection {
    /// The chosen item id — a tier code we emitted in `on_search` (e.g. `SWIFT`).
    pub item_id: String,
    pub pickup: LatLon,
    pub dropoff: LatLon,
}

/// Extract the selection from `message.order`: the first item's id and the
/// START/END stops of the first fulfillment (the BAP echoes our catalog).
pub fn parse_order_selection(
    message: &Value,
) -> Result<OrderSelection, String> {
    let order = message.get("order").ok_or("message.order missing")?;

    let item_id = order
        .get("items")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("id"))
        .and_then(Value::as_str)
        .ok_or("order.items[0].id missing")?
        .to_string();

    let stops = order
        .get("fulfillments")
        .and_then(Value::as_array)
        .and_then(|fulfillments| fulfillments.first())
        .and_then(|fulfillment| fulfillment.get("stops"))
        .and_then(Value::as_array)
        .ok_or("order.fulfillments[0].stops missing")?;

    let gps_of = |wanted: &str| -> Result<LatLon, String> {
        let stop = stops
            .iter()
            .find(|s| s.get("type").and_then(Value::as_str) == Some(wanted))
            .ok_or_else(|| format!("no `{wanted}` stop in order"))?;
        let gps = stop
            .get("location")
            .and_then(|l| l.get("gps"))
            .and_then(Value::as_str)
            .ok_or_else(|| format!("`{wanted}` stop has no location.gps"))?;
        parse_gps(gps)
    };

    Ok(OrderSelection {
        item_id,
        pickup: gps_of("START")?,
        dropoff: gps_of("END")?,
    })
}

/// The `message.order` we emit on `on_select`/`on_init`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Present from `on_init` on — the durable Beckn order id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Order lifecycle (`on_confirm` on): a network `OrderStatus` code,
    /// e.g. `ACTIVE`. Absent during the quote phase.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillments: Option<Vec<Fulfillment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Item>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<Quotation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payments: Option<Vec<Payment>>,
    /// Who cancelled, on `on_cancel` / cancelled `on_status` orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellation: Option<Cancellation>,
    /// Our cancellation policy, declared from `on_init` on (nammayatri
    /// attaches terms at init/confirm, none on `on_select`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellation_terms: Option<Vec<CancellationTerm>>,
}

/// `order.cancellation_terms[]` (nammayatri `Spec.CancellationTerm`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancellationTerm {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellation_fee: Option<Fee>,
    /// The fulfillment state the term applies to; absent = the whole order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_state: Option<FulfillmentState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_required: Option<bool>,
}

/// `Fee` (nammayatri `Spec.Fee`): a flat `amount` or a `percentage`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fee {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Price>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<String>,
}

/// Sitwego's cancellation policy: the rider may cancel free of charge and
/// needs no reason — declared as an explicit zero fee rather than an absent
/// one so BAPs can render "free cancellation" positively.
pub fn build_cancellation_terms(currency: &str) -> Vec<CancellationTerm> {
    vec![CancellationTerm {
        cancellation_fee: Some(Fee {
            amount: Some(Price {
                currency: Some(currency.to_string()),
                value: Some("0".to_string()),
                offered_value: None,
                minimum_value: None,
                maximum_value: None,
            }),
            percentage: None,
        }),
        fulfillment_state: None,
        reason_required: Some(false),
    }]
}

/// `order.cancellation` (nammayatri `Spec.Cancellation`): `cancelled_by` is a
/// network `CancellationSource` — `CONSUMER` | `PROVIDER`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cancellation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled_by: Option<String>,
}

/// `order.provider` on quote-phase callbacks only names the subscriber
/// (nammayatri `mkProvider` — id alone, no catalog repeat).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// `order.quote`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakup: Option<Vec<BreakupItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakupItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,
}

/// `order.payments[]` — the settlement terms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collected_by: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub payment_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<PaymentParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<TagGroup>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descriptor: Option<Descriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<Vec<Tag>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descriptor: Option<Descriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<bool>,
}

fn code_descriptor(code: &str) -> Descriptor {
    Descriptor {
        code: Some(code.to_string()),
        name: None,
        short_desc: None,
    }
}

/// Sitwego's payment terms: rider pays the driver on fulfilment, buyer-finder
/// fee zero (see module docs). Tag layout per nammayatri's `mkPayment`.
pub fn build_payment(currency: &str, amount: &str) -> Payment {
    Payment {
        collected_by: Some("BPP".to_string()),
        payment_type: Some("ON-FULFILLMENT".to_string()),
        status: Some("NOT-PAID".to_string()),
        params: Some(PaymentParams {
            amount: Some(amount.to_string()),
            currency: Some(currency.to_string()),
        }),
        tags: Some(vec![TagGroup {
            descriptor: Some(code_descriptor("BUYER_FINDER_FEES")),
            display: Some(false),
            list: Some(vec![Tag {
                descriptor: Some(code_descriptor(
                    "BUYER_FINDER_FEES_PERCENTAGE",
                )),
                value: Some("0".to_string()),
                display: Some(false),
            }]),
        }]),
    }
}

/// Build the reply `order` for `on_select` (`order_id = None`) or `on_init`
/// (`order_id = Some`): the re-priced tier as one fulfillment + one item, the
/// quote with its component breakup, and the payment terms.
pub fn build_order(
    provider_id: &str,
    currency: &str,
    option: &QuoteOption,
    pickup: LatLon,
    dropoff: LatLon,
    order_id: Option<String>,
) -> Order {
    let fare = format!("{}", option.fare.round() as i64);
    let money = |amount: i64| Price {
        currency: Some(currency.to_string()),
        value: Some(amount.to_string()),
        offered_value: None,
        minimum_value: None,
        maximum_value: None,
    };

    // Component breakup with network `QuoteBreakupTitle` codes. BASE_FARE is
    // always present; zero-valued components are dropped. The components can
    // sum below the quote price when the category minimum fare applies.
    let mut breakup = vec![BreakupItem {
        title: Some("BASE_FARE".to_string()),
        price: Some(money(option.base_fare as i64)),
    }];
    for (title, amount) in [
        ("DISTANCE_FARE", option.distance_fare),
        ("RIDE_DURATION_FARE", option.time_fare),
        ("WAITING_OR_PICKUP_CHARGES", option.waiting_fare),
    ] {
        if amount > 0 {
            breakup.push(BreakupItem {
                title: Some(title.to_string()),
                price: Some(money(amount as i64)),
            });
        }
    }

    let fulfillment = Fulfillment {
        id: Some(option.tier_code.clone()),
        fulfillment_type: Some("DELIVERY".to_string()),
        stops: Some(vec![
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
        ]),
        vehicle: Some(Vehicle {
            category: Some(option.vehicle_category.to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let item = Item {
        id: Some(option.tier_code.clone()),
        descriptor: Some(Descriptor {
            code: Some(option.tier_code.clone()),
            name: Some(option.tier_name.clone()),
            short_desc: None,
        }),
        fulfillment_ids: Some(vec![option.tier_code.clone()]),
        price: Some(Price {
            currency: Some(currency.to_string()),
            value: Some(fare.clone()),
            offered_value: Some(fare.clone()),
            minimum_value: None,
            maximum_value: None,
        }),
    };

    // Terms appear once there is a durable order to cancel (on_init on);
    // on_select is a quote, not yet a cancellable commitment.
    let cancellation_terms =
        order_id.is_some().then(|| build_cancellation_terms(currency));

    Order {
        id: order_id,
        status: None,
        provider: Some(ProviderRef {
            id: Some(provider_id.to_string()),
        }),
        fulfillments: Some(vec![fulfillment]),
        items: Some(vec![item]),
        quote: Some(Quotation {
            breakup: Some(breakup),
            price: Some(Price {
                currency: Some(currency.to_string()),
                value: Some(fare.clone()),
                offered_value: Some(fare.clone()),
                minimum_value: None,
                maximum_value: None,
            }),
            ttl: Some(QUOTE_TTL.to_string()),
        }),
        payments: Some(vec![build_payment(currency, &fare)]),
        cancellation: None,
        cancellation_terms,
    }
}

/// What an inbound `confirm` commits to: the order minted at init, the trip,
/// and the customer riding it (shown to the driver).
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmOrder {
    /// The Beckn order id we issued in `on_init`.
    pub order_id: String,
    pub pickup: LatLon,
    pub dropoff: LatLon,
    /// `fulfillments[0].customer.person.name` — optional on the wire; we fall
    /// back to a generic label so a privacy-lean BAP still gets a ride.
    pub customer_name: Option<String>,
    /// `fulfillments[0].customer.contact.phone`.
    pub customer_phone: Option<String>,
}

/// Extract the commitment from `message.order`. The order id is mandatory —
/// a confirm that references no order is unanswerable.
pub fn parse_confirm_order(message: &Value) -> Result<ConfirmOrder, String> {
    let order = message.get("order").ok_or("message.order missing")?;

    let order_id = order
        .get("id")
        .and_then(Value::as_str)
        .ok_or("order.id missing")?
        .to_string();

    let fulfillment = order
        .get("fulfillments")
        .and_then(Value::as_array)
        .and_then(|fulfillments| fulfillments.first())
        .ok_or("order.fulfillments[0] missing")?;

    let stops = fulfillment
        .get("stops")
        .and_then(Value::as_array)
        .ok_or("order.fulfillments[0].stops missing")?;
    let gps_of = |wanted: &str| -> Result<LatLon, String> {
        let stop = stops
            .iter()
            .find(|s| s.get("type").and_then(Value::as_str) == Some(wanted))
            .ok_or_else(|| format!("no `{wanted}` stop in order"))?;
        let gps = stop
            .get("location")
            .and_then(|l| l.get("gps"))
            .and_then(Value::as_str)
            .ok_or_else(|| format!("`{wanted}` stop has no location.gps"))?;
        parse_gps(gps)
    };

    let customer = fulfillment.get("customer");
    let customer_name = customer
        .and_then(|c| c.get("person"))
        .and_then(|p| p.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let customer_phone = customer
        .and_then(|c| c.get("contact"))
        .and_then(|c| c.get("phone"))
        .and_then(Value::as_str)
        .map(str::to_string);

    Ok(ConfirmOrder {
        order_id,
        pickup: gps_of("START")?,
        dropoff: gps_of("END")?,
        customer_name,
        customer_phone,
    })
}

/// Build the `on_confirm` order (dispatch-timing decision B, John
/// 2026-07-03): `status = ACTIVE`, `fulfillment.state = NEW` — "confirmed,
/// allocating a driver". The driver arrives later via an unsolicited
/// `on_status` with RIDE_ASSIGNED (Step 7). Quote carries the fare agreed at
/// init; the component breakup isn't repeated here (it was delivered on
/// `on_select`/`on_init` and optional on the wire).
#[allow(clippy::too_many_arguments)]
pub fn build_confirm_order(
    provider_id: &str,
    currency: &str,
    order_id: &str,
    tier_code: &str,
    tier_name: &str,
    vehicle_category: &str,
    fare: i32,
    pickup: LatLon,
    dropoff: LatLon,
) -> Order {
    let fare = fare.to_string();
    let price = Price {
        currency: Some(currency.to_string()),
        value: Some(fare.clone()),
        offered_value: Some(fare.clone()),
        minimum_value: None,
        maximum_value: None,
    };

    let fulfillment = Fulfillment {
        id: Some(tier_code.to_string()),
        fulfillment_type: Some("DELIVERY".to_string()),
        stops: Some(vec![
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
        ]),
        vehicle: Some(Vehicle {
            category: Some(vehicle_category.to_string()),
            ..Default::default()
        }),
        state: Some(FulfillmentState {
            descriptor: Some(Descriptor {
                code: Some("NEW".to_string()),
                name: None,
                short_desc: None,
            }),
        }),
        ..Default::default()
    };

    Order {
        id: Some(order_id.to_string()),
        status: Some("ACTIVE".to_string()),
        provider: Some(ProviderRef {
            id: Some(provider_id.to_string()),
        }),
        fulfillments: Some(vec![fulfillment]),
        items: Some(vec![Item {
            id: Some(tier_code.to_string()),
            descriptor: Some(Descriptor {
                code: Some(tier_code.to_string()),
                name: Some(tier_name.to_string()),
                short_desc: None,
            }),
            fulfillment_ids: Some(vec![tier_code.to_string()]),
            price: Some(price.clone()),
        }]),
        quote: Some(Quotation {
            breakup: None,
            price: Some(price),
            ttl: None,
        }),
        payments: Some(vec![build_payment(currency, &fare)]),
        cancellation: None,
        cancellation_terms: Some(build_cancellation_terms(currency)),
    }
}

/// Driver/vehicle/OTP snapshot captured when dispatch assigns a driver.
/// Persisted on the correlation row (`assigned_json`) and received on the
/// `/internal/dispatch/assigned` webhook — field names are the wire contract
/// with the api crate's `BecknDriverAssigned`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AssignedInfo {
    pub request_id: String,
    #[serde(default)]
    pub driver_id: String,
    #[serde(default)]
    pub driver_name: String,
    #[serde(default)]
    pub driver_phone: String,
    #[serde(default)]
    pub vehicle_plate: Option<String>,
    #[serde(default)]
    pub vehicle_model: Option<String>,
    #[serde(default)]
    pub vehicle_color: Option<String>,
    #[serde(default)]
    pub vehicle_make: Option<String>,
    #[serde(default)]
    pub otp: Option<String>,
}

/// Everything a lifecycle `order` (on_status / on_cancel) is built from —
/// the persisted correlation row plus the assigned-driver snapshot.
#[derive(Debug, Clone)]
pub struct StatusOrderArgs<'a> {
    pub provider_id: &'a str,
    pub currency: &'a str,
    pub order_id: &'a str,
    /// Network `OrderStatus`: ACTIVE | COMPLETE | CANCELLED.
    pub order_status: &'a str,
    /// Network `FulfillmentState` code: NEW | RIDE_ASSIGNED | … .
    pub state_code: &'a str,
    /// The ride id — the fulfillment id once a concrete ride exists
    /// (nammayatri uses `ride.id` there from RIDE_ASSIGNED on).
    pub ride_id: Option<&'a str>,
    pub fare: Option<i32>,
    pub tier_code: Option<&'a str>,
    /// `"lat, lon"` as persisted at confirm.
    pub pickup_gps: Option<&'a str>,
    pub dropoff_gps: Option<&'a str>,
    pub assigned: Option<&'a AssignedInfo>,
    /// `CONSUMER` | `PROVIDER` — set on cancelled orders.
    pub cancelled_by: Option<&'a str>,
}

/// Build the `order` for `on_status`/`on_cancel` from persisted state, per
/// nammayatri `tfAssignedReqToOrder`/`tfCancelReqToOrder`: `agent` +
/// `vehicle` + the START stop's OTP `authorization` appear once a driver is
/// assigned; `cancellation.cancelled_by` appears on cancelled orders.
pub fn build_status_order(args: &StatusOrderArgs<'_>) -> Order {
    let stop = |stop_type: &str,
                gps: Option<&str>,
                authorization: Option<Authorization>| {
        Stop {
            stop_type: Some(stop_type.to_string()),
            location: gps.map(|gps| Location {
                gps: Some(gps.to_string()),
                address: None,
            }),
            authorization,
        }
    };
    let otp_auth =
        args.assigned.and_then(|a| a.otp.clone()).map(|token| Authorization {
            token: Some(token),
            authorization_type: Some("OTP".to_string()),
        });

    let agent = args.assigned.map(|a| Agent {
        contact: Some(Contact {
            phone: Some(a.driver_phone.clone()),
        }),
        person: Some(Person {
            name: Some(a.driver_name.clone()),
        }),
    });
    let vehicle = args.assigned.map(|a| Vehicle {
        category: args
            .tier_code
            .and_then(crate::adapters::vehicle_category_from_tier)
            .as_ref()
            .map(|c| crate::adapters::beckn_vehicle_category(c).to_string()),
        registration: a.vehicle_plate.clone(),
        model: a.vehicle_model.clone(),
        color: a.vehicle_color.clone(),
        make: a.vehicle_make.clone(),
    });

    let fulfillment = Fulfillment {
        // The concrete ride id once one exists; the tier code otherwise
        // (matching the ids the quote phase used).
        id: args.ride_id.or(args.tier_code).map(str::to_string),
        fulfillment_type: Some("DELIVERY".to_string()),
        stops: Some(vec![
            stop("START", args.pickup_gps, otp_auth),
            stop("END", args.dropoff_gps, None),
        ]),
        vehicle,
        agent,
        customer: None,
        state: Some(FulfillmentState {
            descriptor: Some(code_descriptor(args.state_code)),
        }),
    };

    let price = args.fare.map(|fare| Price {
        currency: Some(args.currency.to_string()),
        value: Some(fare.to_string()),
        offered_value: None,
        minimum_value: None,
        maximum_value: None,
    });

    Order {
        id: Some(args.order_id.to_string()),
        status: Some(args.order_status.to_string()),
        provider: Some(ProviderRef {
            id: Some(args.provider_id.to_string()),
        }),
        fulfillments: Some(vec![fulfillment]),
        items: None,
        quote: price.clone().map(|price| Quotation {
            breakup: None,
            price: Some(price),
            ttl: None,
        }),
        payments: args
            .fare
            .map(|fare| vec![build_payment(args.currency, &fare.to_string())]),
        cancellation: args.cancelled_by.map(|by| Cancellation {
            cancelled_by: Some(by.to_string()),
        }),
        // Terms were declared at init/confirm; lifecycle orders don't repeat
        // them.
        cancellation_terms: None,
    }
}

/// Parse the `message.order_id` that `status`/`track`/`cancel` reference
/// (nammayatri `StatusReqMessage`/`TrackReqMessage`/`CancelReqMessage`).
pub fn parse_order_id(message: &Value) -> Result<String, String> {
    message
        .get("order_id")
        .and_then(Value::as_str)
        .filter(|id| !id.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| "message.order_id missing".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn swift() -> QuoteOption {
        QuoteOption {
            tier_code: "SWIFT".into(),
            tier_name: "Swift".into(),
            vehicle_category: "CAB",
            fare: 260.0,
            base_fare: 100,
            distance_fare: 120,
            time_fare: 40,
            waiting_fare: 0,
        }
    }

    fn trip() -> (LatLon, LatLon) {
        (
            LatLon {
                lat: -1.286389,
                lon: 36.817223,
            },
            LatLon {
                lat: -1.319167,
                lon: 36.925833,
            },
        )
    }

    #[test]
    fn parses_selection_from_order() {
        let message = serde_json::json!({
            "order": {
                "items": [{ "id": "SWIFT" }],
                "fulfillments": [{ "stops": [
                    { "type": "START",
                      "location": { "gps": "-1.286389, 36.817223" } },
                    { "type": "END",
                      "location": { "gps": "-1.319167, 36.925833" } }
                ]}]
            }
        });
        let selection = parse_order_selection(&message).unwrap();
        assert_eq!(selection.item_id, "SWIFT");
        assert_eq!(selection.pickup.lat, -1.286389);
        assert_eq!(selection.dropoff.lon, 36.925833);
    }

    #[test]
    fn selection_without_items_is_an_error() {
        let message = serde_json::json!({ "order": { "fulfillments": [] } });
        assert!(parse_order_selection(&message).unwrap_err().contains("items"));
    }

    #[test]
    fn cancellation_terms_appear_from_init_on() {
        let (pickup, dropoff) = trip();

        // on_select: a quote, not yet a cancellable commitment — no terms.
        let select = build_order(
            "sitwego.mobility.ke",
            "KES",
            &swift(),
            pickup,
            dropoff,
            None,
        );
        assert!(select.cancellation_terms.is_none());

        // on_init: terms declare free cancellation, no reason needed.
        let init = build_order(
            "sitwego.mobility.ke",
            "KES",
            &swift(),
            pickup,
            dropoff,
            Some("ORDER1".to_string()),
        );
        let json = serde_json::to_value(&init).unwrap();
        assert_eq!(
            json.pointer("/cancellation_terms/0/cancellation_fee/amount/value"),
            Some(&serde_json::json!("0"))
        );
        assert_eq!(
            json.pointer(
                "/cancellation_terms/0/cancellation_fee/amount/currency"
            ),
            Some(&serde_json::json!("KES"))
        );
        assert_eq!(
            json.pointer("/cancellation_terms/0/reason_required"),
            Some(&serde_json::json!(false))
        );

        // on_confirm repeats the terms on the committed order.
        let confirm = build_confirm_order(
            "sitwego.mobility.ke",
            "KES",
            "ORDER1",
            "SWIFT",
            "Swift",
            "CAB",
            260,
            pickup,
            dropoff,
        );
        assert!(confirm.cancellation_terms.is_some());
    }

    #[test]
    fn order_wire_shape() {
        let (pickup, dropoff) = trip();
        let order = build_order(
            "sitwego.mobility.ke",
            "KES",
            &swift(),
            pickup,
            dropoff,
            None,
        );
        let v = serde_json::to_value(&order).unwrap();

        // on_select has no order id yet.
        assert!(v.get("id").is_none());
        assert_eq!(v["provider"]["id"], "sitwego.mobility.ke");
        assert_eq!(v["items"][0]["id"], "SWIFT");
        assert_eq!(v["fulfillments"][0]["type"], "DELIVERY");

        // Quote: price + component breakup; zero components are dropped.
        assert_eq!(v["quote"]["price"]["value"], "260");
        assert_eq!(v["quote"]["ttl"], QUOTE_TTL);
        let breakup = v["quote"]["breakup"].as_array().unwrap();
        let titles: Vec<&str> =
            breakup.iter().map(|b| b["title"].as_str().unwrap()).collect();
        assert_eq!(
            titles,
            ["BASE_FARE", "DISTANCE_FARE", "RIDE_DURATION_FARE"]
        );
        assert_eq!(breakup[0]["price"]["value"], "100");

        // Payment terms: rider pays driver on fulfilment, zero finder fee.
        let payment = &v["payments"][0];
        assert_eq!(payment["collected_by"], "BPP");
        assert_eq!(payment["type"], "ON-FULFILLMENT");
        assert_eq!(payment["status"], "NOT-PAID");
        assert_eq!(payment["params"]["amount"], "260");
        assert_eq!(payment["params"]["currency"], "KES");
        let tags = payment["tags"].as_array().unwrap();
        assert_eq!(tags[0]["descriptor"]["code"], "BUYER_FINDER_FEES");
        assert_eq!(
            tags[0]["list"][0]["descriptor"]["code"],
            "BUYER_FINDER_FEES_PERCENTAGE"
        );
        assert_eq!(tags[0]["list"][0]["value"], "0");
    }

    #[test]
    fn parses_confirm_order_with_customer() {
        let message = serde_json::json!({
            "order": {
                "id": "01ORDER",
                "items": [{ "id": "SWIFT" }],
                "fulfillments": [{
                    "customer": {
                        "person": { "name": "Amina Wanjiru" },
                        "contact": { "phone": "+254700000001" }
                    },
                    "stops": [
                        { "type": "START",
                          "location": { "gps": "-1.286389, 36.817223" } },
                        { "type": "END",
                          "location": { "gps": "-1.319167, 36.925833" } }
                    ]
                }]
            }
        });
        let confirm = parse_confirm_order(&message).unwrap();
        assert_eq!(confirm.order_id, "01ORDER");
        assert_eq!(confirm.customer_name.as_deref(), Some("Amina Wanjiru"));
        assert_eq!(confirm.customer_phone.as_deref(), Some("+254700000001"));
        assert_eq!(confirm.pickup.lat, -1.286389);
    }

    #[test]
    fn confirm_without_order_id_is_an_error() {
        let message = serde_json::json!({
            "order": { "fulfillments": [{ "stops": [] }] }
        });
        assert!(
            parse_confirm_order(&message).unwrap_err().contains("order.id")
        );
    }

    #[test]
    fn confirm_order_wire_shape() {
        let (pickup, dropoff) = trip();
        let order = build_confirm_order(
            "sitwego.mobility.ke",
            "KES",
            "01ORDER",
            "SWIFT",
            "Swift",
            "CAB",
            260,
            pickup,
            dropoff,
        );
        let v = serde_json::to_value(&order).unwrap();

        // Decision B on the wire: confirmed + still allocating.
        assert_eq!(v["id"], "01ORDER");
        assert_eq!(v["status"], "ACTIVE");
        assert_eq!(v["fulfillments"][0]["state"]["descriptor"]["code"], "NEW");
        assert_eq!(v["quote"]["price"]["value"], "260");
        // The init-agreed fare, never re-priced.
        assert_eq!(v["payments"][0]["params"]["amount"], "260");
        assert_eq!(v["payments"][0]["collected_by"], "BPP");
    }

    #[test]
    fn status_order_with_assigned_driver_wire_shape() {
        let assigned = AssignedInfo {
            request_id: "01RIDE".into(),
            driver_id: "01DRIVER".into(),
            driver_name: "Otieno K".into(),
            driver_phone: "+254711000111".into(),
            vehicle_plate: Some("KDA 123X".into()),
            vehicle_model: Some("Vitz".into()),
            vehicle_color: Some("Silver".into()),
            vehicle_make: None,
            otp: Some("4471".into()),
        };
        let order = build_status_order(&StatusOrderArgs {
            provider_id: "sitwego.mobility.ke",
            currency: "KES",
            order_id: "01ORDER",
            order_status: "ACTIVE",
            state_code: "RIDE_ASSIGNED",
            ride_id: Some("01RIDE"),
            fare: Some(260),
            tier_code: Some("SWIFT"),
            pickup_gps: Some("-1.286389, 36.817223"),
            dropoff_gps: Some("-1.319167, 36.925833"),
            assigned: Some(&assigned),
            cancelled_by: None,
        });
        let v = serde_json::to_value(&order).unwrap();

        assert_eq!(v["id"], "01ORDER");
        assert_eq!(v["status"], "ACTIVE");
        let f = &v["fulfillments"][0];
        assert_eq!(f["id"], "01RIDE");
        assert_eq!(f["state"]["descriptor"]["code"], "RIDE_ASSIGNED");
        // The driver, per nammayatri mkFulfillmentV2.
        assert_eq!(f["agent"]["person"]["name"], "Otieno K");
        assert_eq!(f["agent"]["contact"]["phone"], "+254711000111");
        assert_eq!(f["vehicle"]["registration"], "KDA 123X");
        assert_eq!(f["vehicle"]["category"], "CAB");
        // Ride-start OTP rides on the START stop's authorization.
        assert_eq!(f["stops"][0]["type"], "START");
        assert_eq!(f["stops"][0]["authorization"]["token"], "4471");
        assert_eq!(f["stops"][0]["authorization"]["type"], "OTP");
        assert!(f["stops"][1].get("authorization").is_none());
        assert!(v.get("cancellation").is_none());
        assert_eq!(v["quote"]["price"]["value"], "260");
    }

    #[test]
    fn cancelled_status_order_carries_cancellation() {
        let order = build_status_order(&StatusOrderArgs {
            provider_id: "sitwego.mobility.ke",
            currency: "KES",
            order_id: "01ORDER",
            order_status: "CANCELLED",
            state_code: "RIDE_CANCELLED",
            ride_id: Some("01RIDE"),
            fare: Some(260),
            tier_code: Some("SWIFT"),
            pickup_gps: None,
            dropoff_gps: None,
            assigned: None,
            cancelled_by: Some("CONSUMER"),
        });
        let v = serde_json::to_value(&order).unwrap();
        assert_eq!(v["status"], "CANCELLED");
        assert_eq!(
            v["fulfillments"][0]["state"]["descriptor"]["code"],
            "RIDE_CANCELLED"
        );
        assert_eq!(v["cancellation"]["cancelled_by"], "CONSUMER");
        // No driver was assigned — no invented agent/vehicle.
        assert!(v["fulfillments"][0].get("agent").is_none());
        assert!(v["fulfillments"][0].get("vehicle").is_none());
    }

    #[test]
    fn parses_order_id_reference() {
        let ok = serde_json::json!({ "order_id": "01ORDER" });
        assert_eq!(parse_order_id(&ok).unwrap(), "01ORDER");
        let missing = serde_json::json!({});
        assert!(parse_order_id(&missing).is_err());
    }

    #[test]
    fn on_init_order_carries_the_order_id() {
        let (pickup, dropoff) = trip();
        let order = build_order(
            "sitwego.mobility.ke",
            "KES",
            &swift(),
            pickup,
            dropoff,
            Some("01ORDER".to_string()),
        );
        let v = serde_json::to_value(&order).unwrap();
        assert_eq!(v["id"], "01ORDER");
    }
}
