//! In-process operational counters (Step 9).
//!
//! Deliberately not a metrics *stack*: the backend has no Prometheus yet, so
//! these are plain atomics exposed as JSON at `GET /metrics` — enough to see
//! whether the adapter is receiving, answering and delivering, and cheap to
//! re-export through a real exporter when the workspace grows one.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;

use crate::callbacks::{Callback, CallbackClient, CallbackError};

#[derive(Default)]
pub struct Metrics {
    /// Inbound Beckn actions that passed the gate (signature + schema) and
    /// were ACKed into the async half.
    pub inbound_accepted: AtomicU64,
    /// Inbound actions rejected at the gate (bad signature, schema, version,
    /// impersonation).
    pub inbound_rejected: AtomicU64,
    /// Outbound `on_*` callbacks delivered to the BAP (after retries).
    pub callbacks_delivered: AtomicU64,
    /// Callbacks that exhausted retries or were rejected — protocol messages
    /// the network did not get. The number that must stay at zero.
    pub callbacks_failed: AtomicU64,
    /// Ride lifecycle events belonging to a Beckn order, fully handled.
    pub ride_events_handled: AtomicU64,
    /// Beckn-owned ride events that hit a transient failure and were left in
    /// the PEL for the recovery pass.
    pub ride_events_requeued: AtomicU64,
    /// Stale PEL messages reclaimed and replayed by the recovery pass.
    pub ride_events_recovered: AtomicU64,
}

/// Relaxed is enough: counters, no ordering dependencies.
pub fn bump(counter: &AtomicU64) {
    counter.fetch_add(1, Ordering::Relaxed);
}

impl Metrics {
    pub fn snapshot(&self) -> serde_json::Value {
        let read = |c: &AtomicU64| c.load(Ordering::Relaxed);
        serde_json::json!({
            "inbound": {
                "accepted": read(&self.inbound_accepted),
                "rejected": read(&self.inbound_rejected),
            },
            "callbacks": {
                "delivered": read(&self.callbacks_delivered),
                "failed": read(&self.callbacks_failed),
            },
            "ride_events": {
                "handled": read(&self.ride_events_handled),
                "requeued": read(&self.ride_events_requeued),
                "recovered": read(&self.ride_events_recovered),
            },
        })
    }
}

/// Decorator counting every callback outcome; wraps whatever client the app
/// was built with, so handlers and tests all count the same way.
pub struct CountingCallbacks {
    pub inner: Arc<dyn CallbackClient>,
    pub metrics: Arc<Metrics>,
}

#[async_trait]
impl CallbackClient for CountingCallbacks {
    async fn send(&self, callback: Callback) -> Result<(), CallbackError> {
        match self.inner.send(callback).await {
            Ok(()) => {
                bump(&self.metrics.callbacks_delivered);
                Ok(())
            }
            Err(e) => {
                bump(&self.metrics.callbacks_failed);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_reflects_bumps() {
        let m = Metrics::default();
        bump(&m.inbound_accepted);
        bump(&m.inbound_accepted);
        bump(&m.callbacks_failed);
        let snap = m.snapshot();
        assert_eq!(snap["inbound"]["accepted"], 2);
        assert_eq!(snap["inbound"]["rejected"], 0);
        assert_eq!(snap["callbacks"]["failed"], 1);
    }
}
