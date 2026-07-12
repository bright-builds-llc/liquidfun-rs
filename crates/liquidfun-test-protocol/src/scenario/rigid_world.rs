#![allow(
    missing_docs,
    reason = "closed private-harness wire variants are self-describing"
)]

mod result;
mod types;
mod validation;
mod witness_registry;

pub use result::*;
pub use types::*;
pub use validation::decode_rigid_world_request_jsonl;
pub use witness_registry::*;

#[cfg(test)]
mod tests;
