//! Checked particle contracts and private identity-preserving storage.
//!
//! The public definitions preserve the pinned `LiquidFun` flag values, defaults,
//! and physical units while rejecting invalid candidates before they reach a
//! world. Dense particle rows and allocation details remain private.

mod buffer;
mod definition;

pub use crate::world::particle_object::{ParticleSnapshot, ParticleSystemSnapshot};
pub use buffer::{
    ParticleBufferAdoptionError, ParticleBufferAdoptionErrorKind, ParticleBufferBundle,
    ParticleBufferError, ParticleBufferErrorKind, ParticleBufferLanes, ParticleBufferMode,
    ParticleBufferTeardown,
};
pub use definition::{
    ParticleCapacity, ParticleColor, ParticleDef, ParticleDefError, ParticleFlags,
    ParticleSystemDef, ParticleSystemDefError,
};

pub(crate) mod storage;
