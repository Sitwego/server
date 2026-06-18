//! Admin-plane database queries.
//!
//! Everything under here is reached only through the private admin listener
//! (see `api::admin`) — never a public route. Domain truth (driver activation,
//! document review state, billing adjustments) lives in the core Postgres; the
//! admin BFF's Supabase only owns the audit trail.

pub mod docs;
pub mod drivers;
