use super::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupFlags,
    ParticleGroupRecipeError, ParticleGroupSource, Transform, Vec2,
};

/// An owned, reusable, invariant-bearing particle-group creation recipe.
///
/// Source and destination are independent: exactly one source is always
/// present, while append targets cannot be confused with sampling geometry.
/// A non-positive lifetime selects the pinned infinite-lifetime behavior.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleGroupRecipe<UserAssociation = ()> {
    source: ParticleGroupSource,
    destination: ParticleGroupDestination,
    particle_flags: ParticleFlags,
    group_flags: ParticleGroupFlags,
    transform: Transform,
    linear_velocity: Vec2,
    angular_velocity: f32,
    color: ParticleColor,
    strength: f32,
    maybe_stride: Option<f32>,
    lifetime: f32,
    maybe_user_association: Option<UserAssociation>,
}

impl ParticleGroupRecipe<()> {
    /// Creates a recipe from one checked source and an independent destination.
    #[must_use]
    pub fn new(source: ParticleGroupSource, destination: ParticleGroupDestination) -> Self {
        Self {
            source,
            destination,
            particle_flags: ParticleFlags::WATER,
            group_flags: ParticleGroupFlags::empty(),
            transform: Transform::IDENTITY,
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            color: ParticleColor::ZERO,
            strength: 1.0,
            maybe_stride: None,
            lifetime: 0.0,
            maybe_user_association: None,
        }
    }
}

impl<UserAssociation> ParticleGroupRecipe<UserAssociation> {
    /// Returns a copy with exact known and retained unknown particle flags.
    #[must_use]
    pub const fn with_particle_flags(mut self, flags: ParticleFlags) -> Self {
        self.particle_flags = flags;
        self
    }

    /// Returns a copy with exact public and retained unknown group bits.
    #[must_use]
    pub const fn with_group_flags(mut self, flags: ParticleGroupFlags) -> Self {
        self.group_flags = flags;
        self
    }

    /// Returns a copy with a checked finite transform.
    ///
    /// # Errors
    ///
    /// Returns [`ParticleGroupRecipeError::NonFiniteTransform`] when any
    /// translation or rotation component is non-finite.
    pub fn with_transform(
        mut self,
        transform: Transform,
    ) -> Result<Self, ParticleGroupRecipeError> {
        let rotation = transform.rotation();
        if !transform.position().is_valid()
            || !rotation.sine().is_finite()
            || !rotation.cosine().is_finite()
        {
            return Err(ParticleGroupRecipeError::NonFiniteTransform);
        }
        self.transform = transform;
        Ok(self)
    }

    /// Returns a copy with checked finite linear velocity.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a non-finite component.
    pub fn with_linear_velocity(
        mut self,
        velocity: Vec2,
    ) -> Result<Self, ParticleGroupRecipeError> {
        if !velocity.is_valid() {
            return Err(ParticleGroupRecipeError::NonFiniteLinearVelocity);
        }
        self.linear_velocity = velocity;
        Ok(self)
    }

    /// Returns a copy with checked finite angular velocity in radians per second.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a non-finite value.
    pub fn with_angular_velocity(
        mut self,
        angular_velocity: f32,
    ) -> Result<Self, ParticleGroupRecipeError> {
        if !angular_velocity.is_finite() {
            return Err(ParticleGroupRecipeError::NonFiniteAngularVelocity);
        }
        self.angular_velocity = angular_velocity;
        Ok(self)
    }

    /// Returns a copy with an exact particle color.
    #[must_use]
    pub const fn with_color(mut self, color: ParticleColor) -> Self {
        self.color = color;
        self
    }

    /// Returns a copy with checked finite non-negative connection strength.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a non-finite or negative value.
    pub fn with_strength(self, strength: f32) -> Result<Self, ParticleGroupRecipeError> {
        if !strength.is_finite() {
            return Err(ParticleGroupRecipeError::NonFiniteStrength);
        }
        if strength < 0.0 {
            return Err(ParticleGroupRecipeError::NegativeStrength);
        }
        let mut recipe = self;
        recipe.strength = strength;
        Ok(recipe)
    }

    /// Returns a copy with checked positive particle sampling stride in meters.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a non-finite or non-positive value.
    pub fn with_stride(self, stride: f32) -> Result<Self, ParticleGroupRecipeError> {
        if !stride.is_finite() {
            return Err(ParticleGroupRecipeError::NonFiniteStride);
        }
        if stride <= 0.0 {
            return Err(ParticleGroupRecipeError::NonPositiveStride);
        }
        let mut recipe = self;
        recipe.maybe_stride = Some(stride);
        Ok(recipe)
    }

    /// Returns a copy using the particle system's pinned default stride.
    #[must_use]
    pub const fn with_default_stride(mut self) -> Self {
        self.maybe_stride = None;
        self
    }

    /// Returns a copy with checked finite lifetime in seconds.
    ///
    /// Values at or below zero retain their exact bits and select the pinned
    /// infinite-lifetime behavior.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a non-finite value.
    pub fn with_lifetime(self, lifetime: f32) -> Result<Self, ParticleGroupRecipeError> {
        if !lifetime.is_finite() {
            return Err(ParticleGroupRecipeError::NonFiniteLifetime);
        }
        let mut recipe = self;
        recipe.lifetime = lifetime;
        Ok(recipe)
    }

    /// Carries an application-owned association input with this recipe.
    #[must_use]
    pub fn with_user_association<NewAssociation>(
        self,
        user_association: NewAssociation,
    ) -> ParticleGroupRecipe<NewAssociation> {
        ParticleGroupRecipe {
            source: self.source,
            destination: self.destination,
            particle_flags: self.particle_flags,
            group_flags: self.group_flags,
            transform: self.transform,
            linear_velocity: self.linear_velocity,
            angular_velocity: self.angular_velocity,
            color: self.color,
            strength: self.strength,
            maybe_stride: self.maybe_stride,
            lifetime: self.lifetime,
            maybe_user_association: Some(user_association),
        }
    }

    /// Returns the single checked sampling source.
    #[must_use]
    pub const fn source(&self) -> &ParticleGroupSource {
        &self.source
    }

    /// Returns whether creation starts a group or appends to a target.
    #[must_use]
    pub const fn destination(&self) -> ParticleGroupDestination {
        self.destination
    }

    /// Returns exact particle behavior flags.
    #[must_use]
    pub const fn particle_flags(&self) -> ParticleFlags {
        self.particle_flags
    }

    /// Returns exact public and retained unknown group flag bits.
    #[must_use]
    pub const fn group_flags(&self) -> ParticleGroupFlags {
        self.group_flags
    }

    /// Returns the finite sampling transform.
    #[must_use]
    pub const fn transform(&self) -> Transform {
        self.transform
    }

    /// Returns finite linear velocity in meters per second.
    #[must_use]
    pub const fn linear_velocity(&self) -> Vec2 {
        self.linear_velocity
    }

    /// Returns finite angular velocity in radians per second.
    #[must_use]
    pub const fn angular_velocity(&self) -> f32 {
        self.angular_velocity
    }

    /// Returns the exact particle color.
    #[must_use]
    pub const fn color(&self) -> ParticleColor {
        self.color
    }

    /// Returns finite non-negative connection strength.
    #[must_use]
    pub const fn strength(&self) -> f32 {
        self.strength
    }

    /// Returns a positive explicit stride, or `None` for the pinned default.
    #[must_use]
    pub const fn maybe_stride(&self) -> Option<f32> {
        self.maybe_stride
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

    pub(crate) fn into_user_association(self) -> Option<UserAssociation> {
        self.maybe_user_association
    }
}
