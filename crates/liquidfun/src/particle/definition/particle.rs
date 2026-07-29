use super::{Error, ParticleColor, ParticleFlags, Vec2, fmt, validate_vector};

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
