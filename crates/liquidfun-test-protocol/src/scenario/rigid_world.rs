#![allow(
    missing_docs,
    reason = "closed private-harness wire variants are self-describing"
)]

mod result;
mod types;
mod validation;
mod witness_registry;

/// Exact Phase 6 timestep accepted by the closed rigid-world protocol.
pub const RIGID_WORLD_TIMESTEP_BITS: u32 = 0x3c88_8889;
/// Exact Phase 6 velocity-iteration count accepted by the closed protocol.
pub const RIGID_WORLD_VELOCITY_ITERATIONS: u32 = 8;
/// Exact Phase 6 position-iteration count accepted by the closed protocol.
pub const RIGID_WORLD_POSITION_ITERATIONS: u32 = 3;
/// Maximum number of actions accepted in one rigid-world timeline.
pub const RIGID_WORLD_MAXIMUM_ACTIONS: usize = 128;

pub use result::*;
pub use types::*;
pub use validation::decode_rigid_world_request_jsonl;
pub use witness_registry::*;

#[cfg(test)]
mod tests;
