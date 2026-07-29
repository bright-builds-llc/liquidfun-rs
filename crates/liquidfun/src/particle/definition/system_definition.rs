use super::{
    MAXIMUM_STATIC_PRESSURE_ITERATIONS, ParticleCapacity, ParticleCapacityMode,
    ParticleSystemDefError, validate_capacity_range, validate_maximum_capacity,
    validate_non_negative, validate_positive,
};

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
