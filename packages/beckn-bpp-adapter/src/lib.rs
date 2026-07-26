//! Beckn BPP adapter for Sitwego.
//!
//! A protocol-translation layer that exposes Sitwego over an open Beckn
//! mobility network. The module layout mirrors the responsibilities of a BPP:
//!
//! - [`context`]     — the Beckn `context` envelope: parse, validate, build.
//! - [`schemas`]     — wire types shared across actions (ACK/NACK, errors).
//! - [`controllers`] — inbound action handlers (`search` … `cancel`); ACK/NACK only.
//! - [`callbacks`]   — typed outbound `on_*` client (POST to `bap_uri`).
//! - [`validators`]  — the gate every inbound request passes through.
//! - [`correlation`] — protocol↔internal mapping + idempotency (Step 2).
//! - [`crypto`]      — native Ed25519 signing/verification + registry lookup (Step 3).
//! - [`adapters`]    — bridges into Sitwego services: pricing, dispatch (Step 4+).
//!
//! Beckn is ASYNCHRONOUS: every inbound action returns ACK/NACK immediately and
//! the real payload is delivered later as an `on_<action>` callback. Both halves
//! must exist for every action.

pub mod adapters;
pub mod app;
pub mod callbacks;
pub mod config;
pub mod context;
pub mod controllers;
pub mod correlation;
pub mod crypto;
pub mod events;
pub mod internal;
pub mod metrics;
pub mod schemas;
pub mod validators;

use std::sync::Arc;

use adapters::{DiscoveryAdapter, DispatchAdapter};
use callbacks::CallbackClient;
use config::Config;
use correlation::CorrelationStore;
use crypto::RegistryClient;

/// Shared application state handed to every handler.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub callbacks: Arc<dyn CallbackClient>,
    /// Resolves network subscribers' signing keys (Step 3).
    pub registry: Arc<dyn RegistryClient>,
    /// Idempotency + protocol↔ride mapping (Step 2).
    pub correlation: Arc<dyn CorrelationStore>,
    /// Read-only quote pipeline: route + availability + fares (Step 4).
    pub discovery: Arc<dyn DiscoveryAdapter>,
    /// Hands confirmed bookings to the dispatch core (Step 6).
    pub dispatch: Arc<dyn DispatchAdapter>,
    /// Operational counters, served at `GET /metrics` (Step 9).
    pub metrics: Arc<metrics::Metrics>,
}

impl AppState {
    pub fn new(
        config: Config,
        callbacks: Arc<dyn CallbackClient>,
        registry: Arc<dyn RegistryClient>,
        correlation: Arc<dyn CorrelationStore>,
        discovery: Arc<dyn DiscoveryAdapter>,
        dispatch: Arc<dyn DispatchAdapter>,
    ) -> Self {
        let metrics = Arc::new(metrics::Metrics::default());
        // Every on_* delivery outcome is counted, whatever client is behind.
        let callbacks = Arc::new(metrics::CountingCallbacks {
            inner: callbacks,
            metrics: metrics.clone(),
        });
        Self {
            config: Arc::new(config),
            callbacks,
            registry,
            correlation,
            discovery,
            dispatch,
            metrics,
        }
    }
}
