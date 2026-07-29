use std::error::Error;
use std::fmt;

use bitflags::bitflags;

use crate::math::Vec2;

const MAX_UPSTREAM_COUNT: usize = i32::MAX as usize;
const MAXIMUM_STATIC_PRESSURE_ITERATIONS: usize = 1_024;

bitflags! {
    /// Particle behavior and callback flags with the pinned upstream bit values.
    ///
    /// LiquidFun stores the flags as an unrestricted `uint32`. Accordingly,
    /// [`Self::from_bits_retain`] preserves unknown bits for forward-compatible
    /// round trips instead of discarding them at the Rust boundary.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct ParticleFlags: u32 {
        /// Water behavior, represented upstream by the absence of other bits.
        const WATER = 0;
        /// Mark the particle for removal during the next lifecycle pass.
        const ZOMBIE = 1 << 1;
        /// Keep the particle at zero velocity.
        const WALL = 1 << 2;
        /// Enable spring behavior.
        const SPRING = 1 << 3;
        /// Enable elastic behavior.
        const ELASTIC = 1 << 4;
        /// Enable viscous behavior.
        const VISCOUS = 1 << 5;
        /// Disable isotropic pressure and enable powder behavior.
        const POWDER = 1 << 6;
        /// Enable tensile surface tension.
        const TENSILE = 1 << 7;
        /// Mix colors between contacting particles.
        const COLOR_MIXING = 1 << 8;
        /// Request a destruction-listener occurrence when the particle is removed.
        const DESTRUCTION_LISTENER = 1 << 9;
        /// Prevent other particles from leaking through the particle.
        const BARRIER = 1 << 10;
        /// Enable the static-pressure behavior.
        const STATIC_PRESSURE = 1 << 11;
        /// Mark the particle for pair or triad regeneration.
        const REACTIVE = 1 << 12;
        /// Enable the high-repulsion behavior.
        const REPULSIVE = 1 << 13;
        /// Request fixture-contact listener occurrences.
        const FIXTURE_CONTACT_LISTENER = 1 << 14;
        /// Request particle-contact listener occurrences.
        const PARTICLE_CONTACT_LISTENER = 1 << 15;
        /// Request fixture-contact filtering.
        const FIXTURE_CONTACT_FILTER = 1 << 16;
        /// Request particle-contact filtering.
        const PARTICLE_CONTACT_FILTER = 1 << 17;
    }
}

/// An exact four-channel particle color corresponding to `b2ParticleColor`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ParticleColor {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl ParticleColor {
    /// The zero color, which does not require allocating the optional color lane.
    pub const ZERO: Self = Self::new(0, 0, 0, 0);

    /// Creates a color from exact red, green, blue, and alpha components.
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Returns whether all four components are zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.red == 0 && self.green == 0 && self.blue == 0 && self.alpha == 0
    }

    /// Returns the exact `[red, green, blue, alpha]` components.
    #[must_use]
    pub const fn components(self) -> [u8; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }
}

mod particle;
mod system;
mod system_definition;
mod validation;

pub use particle::*;
pub use system::*;
pub use system_definition::*;
use validation::{
    validate_capacity_range, validate_maximum_capacity, validate_non_negative, validate_positive,
    validate_vector,
};

#[cfg(test)]
mod tests;
