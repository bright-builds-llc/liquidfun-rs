//! Bounded scenario catalog definitions and deterministic resolved plans.

mod model;
mod resolve;

pub use model::*;
pub use resolve::{decode_resolved_scenario, resolve_catalog};
