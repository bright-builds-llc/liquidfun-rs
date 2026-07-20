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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParticleCapacityMode {
    Growable,
    Fixed,
}

/// The declared particle-lane capacity policy, independent of allocator capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticleCapacity {
    mode: ParticleCapacityMode,
    count: usize,
}

impl ParticleCapacity {
    pub(crate) const fn from_buffer_mode(mode: super::buffer::ParticleBufferMode) -> Self {
        match mode {
            super::buffer::ParticleBufferMode::Fixed { capacity } => Self {
                mode: ParticleCapacityMode::Fixed,
                count: capacity,
            },
            super::buffer::ParticleBufferMode::Growable { initial_capacity } => Self {
                mode: ParticleCapacityMode::Growable,
                count: initial_capacity,
            },
        }
    }

    /// Creates a growable policy with a checked initial allocation target.
    ///
    /// # Errors
    ///
    /// Returns [`ParticleSystemDefError::CapacityOutOfRange`] when the target
    /// cannot be represented by the pinned upstream `int32` count.
    pub fn growable(initial_capacity: usize) -> Result<Self, ParticleSystemDefError> {
        validate_capacity_range(initial_capacity)?;
        Ok(Self {
            mode: ParticleCapacityMode::Growable,
            count: initial_capacity,
        })
    }

    /// Creates a fixed policy whose owned lanes may never grow past `capacity`.
    ///
    /// # Errors
    ///
    /// Returns a typed error when `capacity` is zero or cannot be represented
    /// by the pinned upstream `int32` count.
    pub fn fixed(capacity: usize) -> Result<Self, ParticleSystemDefError> {
        if capacity == 0 {
            return Err(ParticleSystemDefError::ZeroFixedCapacity);
        }
        validate_capacity_range(capacity)?;
        Ok(Self {
            mode: ParticleCapacityMode::Fixed,
            count: capacity,
        })
    }

    /// Returns whether the policy forbids lane growth.
    #[must_use]
    pub const fn is_fixed(self) -> bool {
        matches!(self.mode, ParticleCapacityMode::Fixed)
    }

    /// Returns the fixed limit or the growable policy's initial allocation target.
    #[must_use]
    pub const fn count(self) -> usize {
        self.count
    }

    const fn maybe_fixed_limit(self) -> Option<usize> {
        if self.is_fixed() {
            Some(self.count)
        } else {
            None
        }
    }
}

/// A failure while constructing a checked [`ParticleSystemDef`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParticleSystemDefError {
    /// Density is not finite.
    NonFiniteDensity,
    /// Density is zero or negative.
    NonPositiveDensity,
    /// Gravity scale is not finite.
    NonFiniteGravityScale,
    /// Radius is not finite.
    NonFiniteRadius,
    /// Radius is zero or negative.
    NonPositiveRadius,
    /// Damping is not finite.
    NonFiniteDamping,
    /// Damping is negative.
    NegativeDamping,
    /// Pressure strength is not finite.
    NonFinitePressureStrength,
    /// Pressure strength is negative.
    NegativePressureStrength,
    /// Elastic strength is not finite.
    NonFiniteElasticStrength,
    /// Elastic strength is negative.
    NegativeElasticStrength,
    /// Spring strength is not finite.
    NonFiniteSpringStrength,
    /// Spring strength is negative.
    NegativeSpringStrength,
    /// Viscous strength is not finite.
    NonFiniteViscousStrength,
    /// Viscous strength is negative.
    NegativeViscousStrength,
    /// Surface-tension pressure strength is not finite.
    NonFiniteSurfaceTensionPressureStrength,
    /// Surface-tension pressure strength is negative.
    NegativeSurfaceTensionPressureStrength,
    /// Surface-tension normal strength is not finite.
    NonFiniteSurfaceTensionNormalStrength,
    /// Surface-tension normal strength is negative.
    NegativeSurfaceTensionNormalStrength,
    /// Repulsive strength is not finite.
    NonFiniteRepulsiveStrength,
    /// Repulsive strength is negative.
    NegativeRepulsiveStrength,
    /// Powder strength is not finite.
    NonFinitePowderStrength,
    /// Powder strength is negative.
    NegativePowderStrength,
    /// Ejection strength is not finite.
    NonFiniteEjectionStrength,
    /// Ejection strength is negative.
    NegativeEjectionStrength,
    /// Static-pressure strength is not finite.
    NonFiniteStaticPressureStrength,
    /// Static-pressure strength is negative.
    NegativeStaticPressureStrength,
    /// Static-pressure relaxation is not finite.
    NonFiniteStaticPressureRelaxation,
    /// Static-pressure relaxation is negative.
    NegativeStaticPressureRelaxation,
    /// Color-mixing strength is not finite.
    NonFiniteColorMixingStrength,
    /// Color-mixing strength is negative.
    NegativeColorMixingStrength,
    /// Lifetime granularity is not finite.
    NonFiniteLifetimeGranularity,
    /// Lifetime granularity is zero or negative.
    NonPositiveLifetimeGranularity,
    /// Static-pressure iteration count is zero.
    ZeroIterations,
    /// Static-pressure iteration count exceeds the reviewed solver bound.
    StaticPressureIterationsOutOfRange {
        /// Requested iteration count.
        requested: usize,
        /// Maximum reviewed iteration count.
        maximum: usize,
    },
    /// A capacity or iteration count exceeds the pinned `int32` range.
    CapacityOutOfRange,
    /// Fixed capacity is zero.
    ZeroFixedCapacity,
    /// A configured maximum is larger than the fixed lane capacity.
    MaximumExceedsFixedCapacity {
        /// Requested particle maximum.
        maximum: usize,
        /// Declared fixed lane capacity.
        capacity: usize,
    },
}

impl fmt::Display for ParticleSystemDefError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::NonFiniteDensity => "particle-system density must be finite",
            Self::NonPositiveDensity => "particle-system density must be positive",
            Self::NonFiniteGravityScale => "particle-system gravity scale must be finite",
            Self::NonFiniteRadius => "particle-system radius must be finite",
            Self::NonPositiveRadius => "particle-system radius must be positive",
            Self::NonFiniteDamping => "particle-system damping must be finite",
            Self::NegativeDamping => "particle-system damping must be non-negative",
            Self::NonFinitePressureStrength => "particle-system pressure strength must be finite",
            Self::NegativePressureStrength => {
                "particle-system pressure strength must be non-negative"
            }
            Self::NonFiniteElasticStrength => "particle-system elastic strength must be finite",
            Self::NegativeElasticStrength => {
                "particle-system elastic strength must be non-negative"
            }
            Self::NonFiniteSpringStrength => "particle-system spring strength must be finite",
            Self::NegativeSpringStrength => "particle-system spring strength must be non-negative",
            Self::NonFiniteViscousStrength => "particle-system viscous strength must be finite",
            Self::NegativeViscousStrength => {
                "particle-system viscous strength must be non-negative"
            }
            Self::NonFiniteSurfaceTensionPressureStrength => {
                "particle-system surface-tension pressure strength must be finite"
            }
            Self::NegativeSurfaceTensionPressureStrength => {
                "particle-system surface-tension pressure strength must be non-negative"
            }
            Self::NonFiniteSurfaceTensionNormalStrength => {
                "particle-system surface-tension normal strength must be finite"
            }
            Self::NegativeSurfaceTensionNormalStrength => {
                "particle-system surface-tension normal strength must be non-negative"
            }
            Self::NonFiniteRepulsiveStrength => "particle-system repulsive strength must be finite",
            Self::NegativeRepulsiveStrength => {
                "particle-system repulsive strength must be non-negative"
            }
            Self::NonFinitePowderStrength => "particle-system powder strength must be finite",
            Self::NegativePowderStrength => "particle-system powder strength must be non-negative",
            Self::NonFiniteEjectionStrength => "particle-system ejection strength must be finite",
            Self::NegativeEjectionStrength => {
                "particle-system ejection strength must be non-negative"
            }
            Self::NonFiniteStaticPressureStrength => {
                "particle-system static-pressure strength must be finite"
            }
            Self::NegativeStaticPressureStrength => {
                "particle-system static-pressure strength must be non-negative"
            }
            Self::NonFiniteStaticPressureRelaxation => {
                "particle-system static-pressure relaxation must be finite"
            }
            Self::NegativeStaticPressureRelaxation => {
                "particle-system static-pressure relaxation must be non-negative"
            }
            Self::NonFiniteColorMixingStrength => {
                "particle-system color-mixing strength must be finite"
            }
            Self::NegativeColorMixingStrength => {
                "particle-system color-mixing strength must be non-negative"
            }
            Self::NonFiniteLifetimeGranularity => {
                "particle-system lifetime granularity must be finite"
            }
            Self::NonPositiveLifetimeGranularity => {
                "particle-system lifetime granularity must be positive"
            }
            Self::ZeroIterations => "particle-system iteration count must be positive",
            Self::StaticPressureIterationsOutOfRange { .. } => {
                "particle-system static-pressure iterations exceed the reviewed solver bound"
            }
            Self::CapacityOutOfRange => {
                "particle-system count must fit the pinned signed 32-bit range"
            }
            Self::ZeroFixedCapacity => "fixed particle capacity must be positive",
            Self::MaximumExceedsFixedCapacity { .. } => {
                "particle maximum cannot exceed fixed lane capacity"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for ParticleSystemDefError {}

/// A reusable checked particle-system definition.
///
/// Density is kilograms per square meter, radius is meters, lifetime
/// granularity is seconds, and damping and gravity scale are dimensionless.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleSystemDef {
    paused: bool,
    strict_contact_check: bool,
    stuck_threshold: u32,
    density: f32,
    gravity_scale: f32,
    radius: f32,
    damping: f32,
    pressure_strength: f32,
    elastic_strength: f32,
    spring_strength: f32,
    viscous_strength: f32,
    surface_tension_pressure_strength: f32,
    surface_tension_normal_strength: f32,
    repulsive_strength: f32,
    powder_strength: f32,
    ejection_strength: f32,
    static_pressure_strength: f32,
    static_pressure_relaxation: f32,
    color_mixing_strength: f32,
    static_pressure_iterations: usize,
    destroy_by_age: bool,
    lifetime_granularity: f32,
    capacity: ParticleCapacity,
    maybe_maximum_count: Option<usize>,
}

impl ParticleSystemDef {
    /// Maximum reviewed static-pressure iterations accepted per particle step.
    pub const MAX_STATIC_PRESSURE_ITERATIONS: usize = MAXIMUM_STATIC_PRESSURE_ITERATIONS;

    /// Returns a copy configured as initially paused or active.
    #[must_use]
    pub const fn with_paused(mut self, paused: bool) -> Self {
        self.paused = paused;
        self
    }

    /// Returns a copy with strict particle/body contact checking configured.
    #[must_use]
    pub const fn with_strict_contact_check(mut self, enabled: bool) -> Self {
        self.strict_contact_check = enabled;
        self
    }

    /// Returns a copy with the consecutive multi-fixture contact threshold.
    ///
    /// Zero disables stuck-particle tracking. A particle becomes a candidate
    /// only after its consecutive count is strictly greater than this value.
    #[must_use]
    pub const fn with_stuck_threshold(mut self, threshold: u32) -> Self {
        self.stuck_threshold = threshold;
        self
    }

    /// Returns a copy with checked positive density.
    ///
    /// # Errors
    ///
    /// Returns a typed error when `density` is non-finite or not positive.
    pub fn with_density(mut self, density: f32) -> Result<Self, ParticleSystemDefError> {
        validate_positive(
            density,
            ParticleSystemDefError::NonFiniteDensity,
            ParticleSystemDefError::NonPositiveDensity,
        )?;
        self.density = density;
        Ok(self)
    }

    /// Returns a copy with checked finite gravity scale.
    ///
    /// # Errors
    ///
    /// Returns [`ParticleSystemDefError::NonFiniteGravityScale`] for a
    /// non-finite value.
    pub fn with_gravity_scale(
        mut self,
        gravity_scale: f32,
    ) -> Result<Self, ParticleSystemDefError> {
        if !gravity_scale.is_finite() {
            return Err(ParticleSystemDefError::NonFiniteGravityScale);
        }
        self.gravity_scale = gravity_scale;
        Ok(self)
    }

    /// Returns a copy with checked positive radius in meters.
    ///
    /// # Errors
    ///
    /// Returns a typed error when `radius` is non-finite or not positive.
    pub fn with_radius(mut self, radius: f32) -> Result<Self, ParticleSystemDefError> {
        validate_positive(
            radius,
            ParticleSystemDefError::NonFiniteRadius,
            ParticleSystemDefError::NonPositiveRadius,
        )?;
        self.radius = radius;
        Ok(self)
    }

    /// Returns a copy with checked non-negative damping.
    ///
    /// # Errors
    ///
    /// Returns a typed error when `damping` is non-finite or negative.
    pub fn with_damping(mut self, damping: f32) -> Result<Self, ParticleSystemDefError> {
        validate_non_negative(
            damping,
            ParticleSystemDefError::NonFiniteDamping,
            ParticleSystemDefError::NegativeDamping,
        )?;
        self.damping = damping;
        Ok(self)
    }

    /// Returns a copy with checked non-negative pressure strength.
    ///
    /// # Errors
    ///
    /// Returns a coefficient-specific error when `strength` is non-finite or negative.
    pub fn with_pressure_strength(mut self, strength: f32) -> Result<Self, ParticleSystemDefError> {
        validate_non_negative(
            strength,
            ParticleSystemDefError::NonFinitePressureStrength,
            ParticleSystemDefError::NegativePressureStrength,
        )?;
        self.pressure_strength = strength;
        Ok(self)
    }

    /// Returns a copy with checked non-negative elastic strength.
    ///
    /// # Errors
    ///
    /// Returns a coefficient-specific error when `strength` is non-finite or negative.
    pub fn with_elastic_strength(mut self, strength: f32) -> Result<Self, ParticleSystemDefError> {
        validate_non_negative(
            strength,
            ParticleSystemDefError::NonFiniteElasticStrength,
            ParticleSystemDefError::NegativeElasticStrength,
        )?;
        self.elastic_strength = strength;
        Ok(self)
    }

    /// Returns a copy with checked non-negative spring strength.
    ///
    /// # Errors
    ///
    /// Returns a coefficient-specific error when `strength` is non-finite or negative.
    pub fn with_spring_strength(mut self, strength: f32) -> Result<Self, ParticleSystemDefError> {
        validate_non_negative(
            strength,
            ParticleSystemDefError::NonFiniteSpringStrength,
            ParticleSystemDefError::NegativeSpringStrength,
        )?;
        self.spring_strength = strength;
        Ok(self)
    }

    /// Returns a copy with checked non-negative viscous strength.
    ///
    /// # Errors
    ///
    /// Returns a coefficient-specific error when `strength` is non-finite or negative.
    pub fn with_viscous_strength(mut self, strength: f32) -> Result<Self, ParticleSystemDefError> {
        validate_non_negative(
            strength,
            ParticleSystemDefError::NonFiniteViscousStrength,
            ParticleSystemDefError::NegativeViscousStrength,
        )?;
        self.viscous_strength = strength;
        Ok(self)
    }

    /// Returns a copy with checked non-negative surface-tension pressure strength.
    ///
    /// # Errors
    ///
    /// Returns a coefficient-specific error when `strength` is non-finite or negative.
    pub fn with_surface_tension_pressure_strength(
        mut self,
        strength: f32,
    ) -> Result<Self, ParticleSystemDefError> {
        validate_non_negative(
            strength,
            ParticleSystemDefError::NonFiniteSurfaceTensionPressureStrength,
            ParticleSystemDefError::NegativeSurfaceTensionPressureStrength,
        )?;
        self.surface_tension_pressure_strength = strength;
        Ok(self)
    }

    /// Returns a copy with checked non-negative surface-tension normal strength.
    ///
    /// # Errors
    ///
    /// Returns a coefficient-specific error when `strength` is non-finite or negative.
    pub fn with_surface_tension_normal_strength(
        mut self,
        strength: f32,
    ) -> Result<Self, ParticleSystemDefError> {
        validate_non_negative(
            strength,
            ParticleSystemDefError::NonFiniteSurfaceTensionNormalStrength,
            ParticleSystemDefError::NegativeSurfaceTensionNormalStrength,
        )?;
        self.surface_tension_normal_strength = strength;
        Ok(self)
    }

    /// Returns a copy with checked non-negative repulsive strength.
    ///
    /// # Errors
    ///
    /// Returns a coefficient-specific error when `strength` is non-finite or negative.
    pub fn with_repulsive_strength(
        mut self,
        strength: f32,
    ) -> Result<Self, ParticleSystemDefError> {
        validate_non_negative(
            strength,
            ParticleSystemDefError::NonFiniteRepulsiveStrength,
            ParticleSystemDefError::NegativeRepulsiveStrength,
        )?;
        self.repulsive_strength = strength;
        Ok(self)
    }

    /// Returns a copy with checked non-negative powder strength.
    ///
    /// # Errors
    ///
    /// Returns a coefficient-specific error when `strength` is non-finite or negative.
    pub fn with_powder_strength(mut self, strength: f32) -> Result<Self, ParticleSystemDefError> {
        validate_non_negative(
            strength,
            ParticleSystemDefError::NonFinitePowderStrength,
            ParticleSystemDefError::NegativePowderStrength,
        )?;
        self.powder_strength = strength;
        Ok(self)
    }

    /// Returns a copy with checked non-negative ejection strength.
    ///
    /// # Errors
    ///
    /// Returns a coefficient-specific error when `strength` is non-finite or negative.
    pub fn with_ejection_strength(mut self, strength: f32) -> Result<Self, ParticleSystemDefError> {
        validate_non_negative(
            strength,
            ParticleSystemDefError::NonFiniteEjectionStrength,
            ParticleSystemDefError::NegativeEjectionStrength,
        )?;
        self.ejection_strength = strength;
        Ok(self)
    }

    /// Returns a copy with checked non-negative static-pressure strength.
    ///
    /// # Errors
    ///
    /// Returns a coefficient-specific error when `strength` is non-finite or negative.
    pub fn with_static_pressure_strength(
        mut self,
        strength: f32,
    ) -> Result<Self, ParticleSystemDefError> {
        validate_non_negative(
            strength,
            ParticleSystemDefError::NonFiniteStaticPressureStrength,
            ParticleSystemDefError::NegativeStaticPressureStrength,
        )?;
        self.static_pressure_strength = strength;
        Ok(self)
    }

    /// Returns a copy with checked non-negative static-pressure relaxation.
    ///
    /// # Errors
    ///
    /// Returns a coefficient-specific error when `relaxation` is non-finite or negative.
    pub fn with_static_pressure_relaxation(
        mut self,
        relaxation: f32,
    ) -> Result<Self, ParticleSystemDefError> {
        validate_non_negative(
            relaxation,
            ParticleSystemDefError::NonFiniteStaticPressureRelaxation,
            ParticleSystemDefError::NegativeStaticPressureRelaxation,
        )?;
        self.static_pressure_relaxation = relaxation;
        Ok(self)
    }

    /// Returns a copy with checked non-negative color-mixing strength.
    ///
    /// # Errors
    ///
    /// Returns a coefficient-specific error when `strength` is non-finite or negative.
    pub fn with_color_mixing_strength(
        mut self,
        strength: f32,
    ) -> Result<Self, ParticleSystemDefError> {
        validate_non_negative(
            strength,
            ParticleSystemDefError::NonFiniteColorMixingStrength,
            ParticleSystemDefError::NegativeColorMixingStrength,
        )?;
        self.color_mixing_strength = strength;
        Ok(self)
    }

    /// Returns a copy with a checked positive static-pressure iteration count.
    ///
    /// This records the pinned definition input only; Phase 10 owns static
    /// pressure solver behavior.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the count is zero or exceeds the reviewed
    /// solver bound.
    pub fn with_static_pressure_iterations(
        mut self,
        iterations: usize,
    ) -> Result<Self, ParticleSystemDefError> {
        if iterations == 0 {
            return Err(ParticleSystemDefError::ZeroIterations);
        }
        if iterations > Self::MAX_STATIC_PRESSURE_ITERATIONS {
            return Err(ParticleSystemDefError::StaticPressureIterationsOutOfRange {
                requested: iterations,
                maximum: Self::MAX_STATIC_PRESSURE_ITERATIONS,
            });
        }
        self.static_pressure_iterations = iterations;
        Ok(self)
    }

    /// Returns a copy configured to destroy the oldest particle when full.
    #[must_use]
    pub const fn with_destruction_by_age(mut self, enabled: bool) -> Self {
        self.destroy_by_age = enabled;
        self
    }

    /// Returns a copy with checked positive lifetime granularity in seconds.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the value is non-finite or not positive.
    pub fn with_lifetime_granularity(
        mut self,
        seconds: f32,
    ) -> Result<Self, ParticleSystemDefError> {
        validate_positive(
            seconds,
            ParticleSystemDefError::NonFiniteLifetimeGranularity,
            ParticleSystemDefError::NonPositiveLifetimeGranularity,
        )?;
        self.lifetime_granularity = seconds;
        Ok(self)
    }

    /// Returns a copy with a checked owned-lane capacity policy.
    ///
    /// # Errors
    ///
    /// Returns an error when an existing maximum exceeds a new fixed capacity.
    pub fn with_capacity(
        mut self,
        capacity: ParticleCapacity,
    ) -> Result<Self, ParticleSystemDefError> {
        validate_maximum_capacity(self.maybe_maximum_count, capacity)?;
        self.capacity = capacity;
        Ok(self)
    }

    /// Returns a copy with the pinned maximum-count convention (`0` is unlimited).
    ///
    /// # Errors
    ///
    /// Returns an error when the count is outside `int32` range or exceeds a
    /// configured fixed capacity.
    pub fn with_maximum_count(
        mut self,
        maximum_count: usize,
    ) -> Result<Self, ParticleSystemDefError> {
        validate_capacity_range(maximum_count)?;
        let maybe_maximum_count = (maximum_count != 0).then_some(maximum_count);
        validate_maximum_capacity(maybe_maximum_count, self.capacity)?;
        self.maybe_maximum_count = maybe_maximum_count;
        Ok(self)
    }

    /// Returns whether this system starts paused.
    #[must_use]
    pub const fn is_paused(self) -> bool {
        self.paused
    }

    /// Returns whether strict particle/body contact checking is enabled.
    #[must_use]
    pub const fn uses_strict_contact_check(self) -> bool {
        self.strict_contact_check
    }

    /// Returns the consecutive multi-fixture contact threshold.
    #[must_use]
    pub const fn stuck_threshold(self) -> u32 {
        self.stuck_threshold
    }

    /// Returns density in kilograms per square meter.
    #[must_use]
    pub const fn density(self) -> f32 {
        self.density
    }

    /// Returns the dimensionless gravity scale.
    #[must_use]
    pub const fn gravity_scale(self) -> f32 {
        self.gravity_scale
    }

    /// Returns particle radius in meters.
    #[must_use]
    pub const fn radius(self) -> f32 {
        self.radius
    }

    /// Returns the damping coefficient.
    #[must_use]
    pub const fn damping(self) -> f32 {
        self.damping
    }

    /// Returns the pressure strength.
    #[must_use]
    pub const fn pressure_strength(self) -> f32 {
        self.pressure_strength
    }

    /// Returns the elastic strength.
    #[must_use]
    pub const fn elastic_strength(self) -> f32 {
        self.elastic_strength
    }

    /// Returns the spring strength.
    #[must_use]
    pub const fn spring_strength(self) -> f32 {
        self.spring_strength
    }

    /// Returns the viscous strength.
    #[must_use]
    pub const fn viscous_strength(self) -> f32 {
        self.viscous_strength
    }

    /// Returns the surface-tension pressure strength.
    #[must_use]
    pub const fn surface_tension_pressure_strength(self) -> f32 {
        self.surface_tension_pressure_strength
    }

    /// Returns the surface-tension normal strength.
    #[must_use]
    pub const fn surface_tension_normal_strength(self) -> f32 {
        self.surface_tension_normal_strength
    }

    /// Returns the repulsive strength.
    #[must_use]
    pub const fn repulsive_strength(self) -> f32 {
        self.repulsive_strength
    }

    /// Returns the powder strength.
    #[must_use]
    pub const fn powder_strength(self) -> f32 {
        self.powder_strength
    }

    /// Returns the ejection strength.
    #[must_use]
    pub const fn ejection_strength(self) -> f32 {
        self.ejection_strength
    }

    /// Returns the static-pressure strength.
    #[must_use]
    pub const fn static_pressure_strength(self) -> f32 {
        self.static_pressure_strength
    }

    /// Returns the static-pressure relaxation.
    #[must_use]
    pub const fn static_pressure_relaxation(self) -> f32 {
        self.static_pressure_relaxation
    }

    /// Returns the color-mixing strength.
    #[must_use]
    pub const fn color_mixing_strength(self) -> f32 {
        self.color_mixing_strength
    }

    /// Returns the configured static-pressure iteration count.
    #[must_use]
    pub const fn static_pressure_iterations(self) -> usize {
        self.static_pressure_iterations
    }

    /// Returns whether creation at the maximum evicts the oldest particle.
    #[must_use]
    pub const fn destroys_by_age(self) -> bool {
        self.destroy_by_age
    }

    /// Returns lifetime granularity in seconds.
    #[must_use]
    pub const fn lifetime_granularity(self) -> f32 {
        self.lifetime_granularity
    }

    /// Returns the declared lane capacity policy.
    #[must_use]
    pub const fn capacity(self) -> ParticleCapacity {
        self.capacity
    }

    /// Returns the particle maximum, or `None` for the pinned unlimited value.
    #[must_use]
    pub const fn maximum_count(self) -> Option<usize> {
        self.maybe_maximum_count
    }
}

impl Default for ParticleSystemDef {
    fn default() -> Self {
        Self {
            paused: false,
            strict_contact_check: false,
            stuck_threshold: 0,
            density: 1.0,
            gravity_scale: 1.0,
            radius: 1.0,
            damping: 1.0,
            pressure_strength: 0.05,
            elastic_strength: 0.25,
            spring_strength: 0.25,
            viscous_strength: 0.25,
            surface_tension_pressure_strength: 0.2,
            surface_tension_normal_strength: 0.2,
            repulsive_strength: 1.0,
            powder_strength: 0.5,
            ejection_strength: 0.5,
            static_pressure_strength: 0.2,
            static_pressure_relaxation: 0.2,
            color_mixing_strength: 0.5,
            static_pressure_iterations: 8,
            destroy_by_age: true,
            lifetime_granularity: 1.0 / 60.0,
            capacity: ParticleCapacity {
                mode: ParticleCapacityMode::Growable,
                count: 0,
            },
            maybe_maximum_count: None,
        }
    }
}

/// A failure while constructing a checked [`ParticleDef`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParticleDefError {
    /// Position x-coordinate is not finite.
    NonFinitePositionX,
    /// Position y-coordinate is not finite.
    NonFinitePositionY,
    /// Velocity x-coordinate is not finite.
    NonFiniteVelocityX,
    /// Velocity y-coordinate is not finite.
    NonFiniteVelocityY,
    /// Lifetime is not finite.
    NonFiniteLifetime,
}

impl fmt::Display for ParticleDefError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field = match self {
            Self::NonFinitePositionX => "position.x",
            Self::NonFinitePositionY => "position.y",
            Self::NonFiniteVelocityX => "velocity.x",
            Self::NonFiniteVelocityY => "velocity.y",
            Self::NonFiniteLifetime => "lifetime",
        };
        write!(formatter, "particle definition {field} must be finite")
    }
}

impl Error for ParticleDefError {}

/// A reusable checked particle definition with an application-owned association input.
///
/// The generic association value is carried by the definition but is never stored
/// as `Any` or a raw pointer in [`crate::World`]. Later creation APIs can pair the
/// value with the returned stable [`crate::ParticleId`] in an application-owned
/// [`crate::AssociationMap`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleDef<UserAssociation = ()> {
    flags: ParticleFlags,
    position: Vec2,
    velocity: Vec2,
    color: ParticleColor,
    lifetime: f32,
    maybe_user_association: Option<UserAssociation>,
}

impl<UserAssociation> ParticleDef<UserAssociation> {
    /// Returns a copy with checked finite position in meters.
    ///
    /// # Errors
    ///
    /// Returns a coordinate-specific error for a non-finite value.
    pub fn with_position(mut self, position: Vec2) -> Result<Self, ParticleDefError> {
        validate_vector(
            position,
            ParticleDefError::NonFinitePositionX,
            ParticleDefError::NonFinitePositionY,
        )?;
        self.position = position;
        Ok(self)
    }

    /// Returns a copy with checked finite velocity in meters per second.
    ///
    /// # Errors
    ///
    /// Returns a coordinate-specific error for a non-finite value.
    pub fn with_velocity(mut self, velocity: Vec2) -> Result<Self, ParticleDefError> {
        validate_vector(
            velocity,
            ParticleDefError::NonFiniteVelocityX,
            ParticleDefError::NonFiniteVelocityY,
        )?;
        self.velocity = velocity;
        Ok(self)
    }

    /// Returns a copy with exact color components.
    #[must_use]
    pub const fn with_color(mut self, color: ParticleColor) -> Self {
        self.color = color;
        self
    }

    /// Returns a copy with exact known and retained unknown flag bits.
    #[must_use]
    pub const fn with_flags(mut self, flags: ParticleFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Returns a copy with checked finite lifetime in seconds.
    ///
    /// Values at or below zero retain their exact bits and select the pinned
    /// infinite-lifetime behavior.
    ///
    /// # Errors
    ///
    /// Returns [`ParticleDefError::NonFiniteLifetime`] for a non-finite value.
    pub fn with_lifetime(mut self, lifetime: f32) -> Result<Self, ParticleDefError> {
        if !lifetime.is_finite() {
            return Err(ParticleDefError::NonFiniteLifetime);
        }
        self.lifetime = lifetime;
        Ok(self)
    }

    /// Carries a typed application-owned association input with this definition.
    #[must_use]
    pub fn with_user_association<NewAssociation>(
        self,
        user_association: NewAssociation,
    ) -> ParticleDef<NewAssociation> {
        ParticleDef {
            flags: self.flags,
            position: self.position,
            velocity: self.velocity,
            color: self.color,
            lifetime: self.lifetime,
            maybe_user_association: Some(user_association),
        }
    }

    /// Returns exact particle flags.
    #[must_use]
    pub const fn flags(&self) -> ParticleFlags {
        self.flags
    }

    /// Returns position in meters.
    #[must_use]
    pub const fn position(&self) -> Vec2 {
        self.position
    }

    /// Returns velocity in meters per second.
    #[must_use]
    pub const fn velocity(&self) -> Vec2 {
        self.velocity
    }

    /// Returns the exact particle color.
    #[must_use]
    pub const fn color(&self) -> ParticleColor {
        self.color
    }

    /// Returns lifetime in seconds; values at or below zero mean infinite.
    #[must_use]
    pub const fn lifetime(&self) -> f32 {
        self.lifetime
    }

    /// Returns the typed application association input, when present.
    #[must_use]
    pub const fn maybe_user_association(&self) -> Option<&UserAssociation> {
        self.maybe_user_association.as_ref()
    }
}

impl Default for ParticleDef<()> {
    fn default() -> Self {
        Self {
            flags: ParticleFlags::WATER,
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            color: ParticleColor::ZERO,
            lifetime: 0.0,
            maybe_user_association: None,
        }
    }
}

fn validate_positive(
    value: f32,
    non_finite: ParticleSystemDefError,
    non_positive: ParticleSystemDefError,
) -> Result<(), ParticleSystemDefError> {
    if !value.is_finite() {
        return Err(non_finite);
    }
    if value <= 0.0 {
        return Err(non_positive);
    }
    Ok(())
}

fn validate_non_negative(
    value: f32,
    non_finite: ParticleSystemDefError,
    negative: ParticleSystemDefError,
) -> Result<(), ParticleSystemDefError> {
    if !value.is_finite() {
        return Err(non_finite);
    }
    if value < 0.0 {
        return Err(negative);
    }
    Ok(())
}

fn validate_capacity_range(count: usize) -> Result<(), ParticleSystemDefError> {
    if count > MAX_UPSTREAM_COUNT {
        return Err(ParticleSystemDefError::CapacityOutOfRange);
    }
    Ok(())
}

fn validate_maximum_capacity(
    maybe_maximum: Option<usize>,
    capacity: ParticleCapacity,
) -> Result<(), ParticleSystemDefError> {
    let (Some(maximum), Some(fixed_capacity)) = (maybe_maximum, capacity.maybe_fixed_limit())
    else {
        return Ok(());
    };
    if maximum > fixed_capacity {
        return Err(ParticleSystemDefError::MaximumExceedsFixedCapacity {
            maximum,
            capacity: fixed_capacity,
        });
    }
    Ok(())
}

fn validate_vector(
    value: Vec2,
    invalid_x: ParticleDefError,
    invalid_y: ParticleDefError,
) -> Result<(), ParticleDefError> {
    if !value.x.is_finite() {
        return Err(invalid_x);
    }
    if !value.y.is_finite() {
        return Err(invalid_y);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ParticleSystemDef, ParticleSystemDefError};

    type Builder = fn(ParticleSystemDef, f32) -> Result<ParticleSystemDef, ParticleSystemDefError>;
    type Accessor = fn(ParticleSystemDef) -> f32;

    struct CoefficientCase {
        name: &'static str,
        default: f32,
        builder: Builder,
        accessor: Accessor,
        non_finite_error: ParticleSystemDefError,
        negative_error: ParticleSystemDefError,
    }

    fn coefficient_cases() -> [CoefficientCase; 12] {
        [
            CoefficientCase {
                name: "pressure strength",
                default: 0.05,
                builder: ParticleSystemDef::with_pressure_strength,
                accessor: ParticleSystemDef::pressure_strength,
                non_finite_error: ParticleSystemDefError::NonFinitePressureStrength,
                negative_error: ParticleSystemDefError::NegativePressureStrength,
            },
            CoefficientCase {
                name: "elastic strength",
                default: 0.25,
                builder: ParticleSystemDef::with_elastic_strength,
                accessor: ParticleSystemDef::elastic_strength,
                non_finite_error: ParticleSystemDefError::NonFiniteElasticStrength,
                negative_error: ParticleSystemDefError::NegativeElasticStrength,
            },
            CoefficientCase {
                name: "spring strength",
                default: 0.25,
                builder: ParticleSystemDef::with_spring_strength,
                accessor: ParticleSystemDef::spring_strength,
                non_finite_error: ParticleSystemDefError::NonFiniteSpringStrength,
                negative_error: ParticleSystemDefError::NegativeSpringStrength,
            },
            CoefficientCase {
                name: "viscous strength",
                default: 0.25,
                builder: ParticleSystemDef::with_viscous_strength,
                accessor: ParticleSystemDef::viscous_strength,
                non_finite_error: ParticleSystemDefError::NonFiniteViscousStrength,
                negative_error: ParticleSystemDefError::NegativeViscousStrength,
            },
            CoefficientCase {
                name: "surface-tension pressure strength",
                default: 0.2,
                builder: ParticleSystemDef::with_surface_tension_pressure_strength,
                accessor: ParticleSystemDef::surface_tension_pressure_strength,
                non_finite_error: ParticleSystemDefError::NonFiniteSurfaceTensionPressureStrength,
                negative_error: ParticleSystemDefError::NegativeSurfaceTensionPressureStrength,
            },
            CoefficientCase {
                name: "surface-tension normal strength",
                default: 0.2,
                builder: ParticleSystemDef::with_surface_tension_normal_strength,
                accessor: ParticleSystemDef::surface_tension_normal_strength,
                non_finite_error: ParticleSystemDefError::NonFiniteSurfaceTensionNormalStrength,
                negative_error: ParticleSystemDefError::NegativeSurfaceTensionNormalStrength,
            },
            CoefficientCase {
                name: "repulsive strength",
                default: 1.0,
                builder: ParticleSystemDef::with_repulsive_strength,
                accessor: ParticleSystemDef::repulsive_strength,
                non_finite_error: ParticleSystemDefError::NonFiniteRepulsiveStrength,
                negative_error: ParticleSystemDefError::NegativeRepulsiveStrength,
            },
            CoefficientCase {
                name: "powder strength",
                default: 0.5,
                builder: ParticleSystemDef::with_powder_strength,
                accessor: ParticleSystemDef::powder_strength,
                non_finite_error: ParticleSystemDefError::NonFinitePowderStrength,
                negative_error: ParticleSystemDefError::NegativePowderStrength,
            },
            CoefficientCase {
                name: "ejection strength",
                default: 0.5,
                builder: ParticleSystemDef::with_ejection_strength,
                accessor: ParticleSystemDef::ejection_strength,
                non_finite_error: ParticleSystemDefError::NonFiniteEjectionStrength,
                negative_error: ParticleSystemDefError::NegativeEjectionStrength,
            },
            CoefficientCase {
                name: "static-pressure strength",
                default: 0.2,
                builder: ParticleSystemDef::with_static_pressure_strength,
                accessor: ParticleSystemDef::static_pressure_strength,
                non_finite_error: ParticleSystemDefError::NonFiniteStaticPressureStrength,
                negative_error: ParticleSystemDefError::NegativeStaticPressureStrength,
            },
            CoefficientCase {
                name: "static-pressure relaxation",
                default: 0.2,
                builder: ParticleSystemDef::with_static_pressure_relaxation,
                accessor: ParticleSystemDef::static_pressure_relaxation,
                non_finite_error: ParticleSystemDefError::NonFiniteStaticPressureRelaxation,
                negative_error: ParticleSystemDefError::NegativeStaticPressureRelaxation,
            },
            CoefficientCase {
                name: "color-mixing strength",
                default: 0.5,
                builder: ParticleSystemDef::with_color_mixing_strength,
                accessor: ParticleSystemDef::color_mixing_strength,
                non_finite_error: ParticleSystemDefError::NonFiniteColorMixingStrength,
                negative_error: ParticleSystemDefError::NegativeColorMixingStrength,
            },
        ]
    }

    #[test]
    fn solver_coefficient_defaults_match_pinned_bits() {
        // Arrange / Act
        let definition = ParticleSystemDef::default();

        // Assert
        for case in coefficient_cases() {
            assert_eq!(
                (case.accessor)(definition).to_bits(),
                case.default.to_bits(),
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn solver_coefficient_builders_preserve_configured_bits() {
        // Arrange
        let definition = ParticleSystemDef::default();
        let configured = 0.375_f32;

        for case in coefficient_cases() {
            // Act
            let result = (case.builder)(definition, configured)
                .expect("finite non-negative solver coefficient should be valid");

            // Assert
            assert_eq!(
                (case.accessor)(result).to_bits(),
                configured.to_bits(),
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn solver_coefficient_builders_reject_non_finite_values_without_mutation() {
        // Arrange
        let definition = ParticleSystemDef::default();
        let non_finite_values = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY];

        for case in coefficient_cases() {
            for value in non_finite_values {
                // Act
                let result = (case.builder)(definition, value);

                // Assert
                assert_eq!(result, Err(case.non_finite_error), "{}", case.name);
                assert_eq!(
                    (case.accessor)(definition).to_bits(),
                    case.default.to_bits(),
                    "{}",
                    case.name
                );
            }
        }
    }

    #[test]
    fn solver_coefficient_builders_reject_negative_values_without_mutation() {
        // Arrange
        let definition = ParticleSystemDef::default();

        for case in coefficient_cases() {
            // Act
            let result = (case.builder)(definition, -0.25);

            // Assert
            assert_eq!(result, Err(case.negative_error), "{}", case.name);
            assert_eq!(
                (case.accessor)(definition).to_bits(),
                case.default.to_bits(),
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn solver_coefficient_builders_accept_exact_positive_zero() {
        // Arrange
        let definition = ParticleSystemDef::default();

        for case in coefficient_cases() {
            // Act
            let result = (case.builder)(definition, 0.0)
                .expect("zero disables or suppresses the corresponding solver effect");

            // Assert
            assert_eq!((case.accessor)(result).to_bits(), 0.0_f32.to_bits());
        }
    }
}
