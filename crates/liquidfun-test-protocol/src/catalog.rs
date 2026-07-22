//! Bounded scenario catalog definitions and deterministic resolved plans.

mod mapping;
mod model;
mod resolve;
/// Reviewed native rigid-body, joint, and standalone-rope definitions.
pub mod scenarios;
mod wire;

pub use mapping::*;
pub use model::*;
pub use resolve::{decode_resolved_scenario, resolve_catalog};
pub use wire::*;
