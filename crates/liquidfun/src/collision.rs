//! Safe collision-domain foundations for `LiquidFun` compatibility.
//!
//! This deep module owns initialized geometry values, immutable shapes,
//! source-ordered narrow-phase kernels, spatial acceleration, broad-phase
//! pairing, and continuous-collision primitives. Its implementation is being
//! added incrementally during Phase 5; the presence of a child module does not
//! by itself claim parity with the pinned C++ oracle.
//!
//! Shared domain values, immutable shapes, and completed collision kernels are
//! curated explicitly at this module root. Their child-module paths remain
//! available for source mapping and compatibility.

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

#[cfg(not(feature = "differential-internals"))]
/// The development-only diagnostic seam is unavailable to ordinary consumers.
///
/// ```compile_fail
/// use liquidfun::collision::differential::GjkTermination;
///
/// let _ = GjkTermination::Triangle;
/// ```
mod differential_feature_is_disabled {}

/// Development-only owned collision diagnostics for the private differential harness.
#[cfg(feature = "differential-internals")]
#[doc(hidden)]
pub mod differential;

pub use broad_phase::{BroadPhase, FilterData};
pub use distance::{
    DistanceCache, DistanceCacheSnapshot, DistanceResult, SupportIndexPair, distance, test_overlap,
};
pub use error::CollisionError;
pub use narrow::{
    PairManifold, PairOrientation, PointStates, WorldManifold, WorldManifoldPoint, collide_circles,
    collide_edge_circle, collide_edge_polygon, collide_polygon_circle, collide_polygons,
    collide_shapes, point_states, world_manifold,
};
pub use shape::{ChainShape, CircleShape, EdgeShape, PointDistance, PolygonShape, Shape};
pub use toi::{TimeOfImpactInput, TimeOfImpactOutput, TimeOfImpactState, time_of_impact};
pub use tree::{DynamicTree, ProxyId, QueryControl, RayCastControl, TreeError};
pub use types::{
    Aabb, ChildIndex, CollisionOutcome, ContactFeatureId, FeatureKind, Manifold, ManifoldKind,
    ManifoldPoint, MassData, PointState, RayCastHit, RayCastInput,
};
