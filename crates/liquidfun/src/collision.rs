//! Safe collision-domain foundations for `LiquidFun` compatibility.
//!
//! This deep module owns initialized geometry values, immutable shapes,
//! source-ordered narrow-phase kernels, spatial acceleration, broad-phase
//! pairing, and continuous-collision primitives. Its implementation is being
//! added incrementally during Phase 5; the presence of a child module does not
//! by itself claim parity with the pinned C++ oracle.
//!
//! Shared domain values and errors are curated at this module root. Concrete
//! kernel types remain reachable through their documented child modules while
//! those implementations are developed. Plan 05-07 will finalize the curated
//! root re-export list after every kernel exists.

mod error;
mod types;

/// Immutable broad-phase proxy management and pair generation.
pub mod broad_phase;
/// Shape-child distance, overlap, and cache operations.
pub mod distance;
/// Supported shape-pair manifold generation.
pub mod narrow;
/// Owned circle, edge, polygon, chain, and exhaustive shape values.
pub mod shape;
/// Checked time-of-impact operations over shape sweeps.
pub mod toi;
/// Dynamic AABB-tree storage, queries, ray casts, and metrics.
pub mod tree;

pub use error::CollisionError;

// Shared initialized values are re-exported here as Plan 05-01 implements
// them. Concrete kernel re-exports remain reserved for serialized Plan 05-07
// integration so parallel implementation plans do not edit this entrypoint.
