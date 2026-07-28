use super::{Error, fmt, validate_capacity_range};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParticleCapacityMode {
    Growable,
    Fixed,
}

/// The declared particle-lane capacity policy, independent of allocator capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticleCapacity {
    pub(super) mode: ParticleCapacityMode,
    pub(super) count: usize,
}

impl ParticleCapacity {
    pub(crate) const fn from_buffer_mode(mode: super::super::buffer::ParticleBufferMode) -> Self {
        match mode {
            super::super::buffer::ParticleBufferMode::Fixed { capacity } => Self {
                mode: ParticleCapacityMode::Fixed,
                count: capacity,
            },
            super::super::buffer::ParticleBufferMode::Growable { initial_capacity } => Self {
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

    pub(super) const fn maybe_fixed_limit(self) -> Option<usize> {
        if self.is_fixed() {
            Some(self.count)
        } else {
            None
        }
    }
}

/// A failure while constructing a checked [`crate::ParticleSystemDef`].
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
