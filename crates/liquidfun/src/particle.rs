//! Checked particle contracts and private identity-preserving storage.
//!
//! The public definitions preserve the pinned `LiquidFun` flag values, defaults,
//! and physical units while rejecting invalid candidates before they reach a
//! world. Dense particle rows and allocation details remain private.

pub(crate) mod body_contact;
mod buffer;
mod contact;
mod definition;
mod editor;
pub(crate) mod force;
mod group;
pub(crate) mod lifetime;
mod proxy;
pub(crate) mod query;
#[allow(
    dead_code,
    reason = "later Phase 10 plans dispatch kernels through this private manifest"
)]
mod solver;
mod statistics;
#[allow(
    dead_code,
    reason = "later Phase 10 plans consume the private topology planners"
)]
mod topology;
mod view;

pub use crate::world::particle_object::{
    ParticleCreationReceipt, ParticleSnapshot, ParticleSystemSnapshot,
};
pub use body_contact::{ParticleBodyContact, ParticleBodyContactEffect, ParticleBodyContactUpdate};
pub use buffer::{
    ParticleBufferAdoptionError, ParticleBufferAdoptionErrorKind, ParticleBufferBundle,
    ParticleBufferError, ParticleBufferErrorKind, ParticleBufferLanes, ParticleBufferMode,
    ParticleBufferTeardown,
};
pub use contact::{
    ParticleContact, ParticleContactEffect, ParticleContactError, ParticleContactUpdate,
};
pub use definition::{
    ParticleCapacity, ParticleColor, ParticleDef, ParticleDefError, ParticleFlags,
    ParticleSystemDef, ParticleSystemDefError,
};
pub use editor::{ParticleEditError, ParticleEditor};
pub use force::ParticleForceError;
pub use group::{
    FilledParticleGroupShapes, ParticleGroupDestination, ParticleGroupFlags,
    ParticleGroupPositions, ParticleGroupRecipe, ParticleGroupRecipeError, ParticleGroupSource,
    ParticleGroupStrokeShape, ParticleGroupView,
};
pub use lifetime::{
    ParticleDestructionOccurrence, ParticleLifetimeClock, ParticleLifetimeError,
    ParticleLifetimeOrder,
};
pub use proxy::{ParticleNeighborPair, ParticleNeighborhood, ParticleProxyError};
pub use query::{
    ParticleQueryError, ParticleQueryOccurrence, ParticleRayCastError, ParticleRayHit,
};
pub use statistics::{ParticleSystemStatistics, ParticleWorldStatistics};
pub use view::{
    ParticleBodyContactView, ParticleContactView, ParticlePairView, ParticleSystemView,
    ParticleTriadView,
};

pub(crate) mod storage;
