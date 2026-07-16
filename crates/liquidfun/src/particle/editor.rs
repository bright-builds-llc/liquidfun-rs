//! Closure-scoped checked particle mutation.

use std::error::Error;
use std::fmt;
use std::marker::PhantomData;

use crate::HandleError;
use crate::math::Vec2;

/// A checked particle-edit failure that occurs before storage mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParticleEditError {
    /// The stable particle or its owning system is not live in this world.
    InvalidHandle(HandleError),
    /// Position x is not finite.
    NonFinitePositionX,
    /// Position y is not finite.
    NonFinitePositionY,
    /// Velocity x is not finite.
    NonFiniteVelocityX,
    /// Velocity y is not finite.
    NonFiniteVelocityY,
}

impl fmt::Display for ParticleEditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid particle handle: {error}"),
            Self::NonFinitePositionX => formatter.write_str("particle position x must be finite"),
            Self::NonFinitePositionY => formatter.write_str("particle position y must be finite"),
            Self::NonFiniteVelocityX => formatter.write_str("particle velocity x must be finite"),
            Self::NonFiniteVelocityY => formatter.write_str("particle velocity y must be finite"),
        }
    }
}

impl Error for ParticleEditError {}

impl From<HandleError> for ParticleEditError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

/// A closure-scoped candidate for supported particle mutations.
///
/// The world commits the complete copied candidate only after the closure
/// returns successfully. The editor cannot escape its scope:
///
/// ```compile_fail
/// use liquidfun::World;
/// let mut world = World::new().expect("world key should remain available");
/// let system = world.create_particle_system().expect("system should fit");
/// let particle = world
///     .create_particle(system, None)
///     .expect("particle should fit")
///     .created_particle();
/// let _escaped = world.edit_particle(particle, |editor| Ok(editor));
/// ```
pub struct ParticleEditor<'edit> {
    position: Vec2,
    velocity: Vec2,
    _scope: PhantomData<&'edit mut ()>,
}

impl ParticleEditor<'_> {
    pub(crate) const fn new(position: Vec2, velocity: Vec2) -> Self {
        Self {
            position,
            velocity,
            _scope: PhantomData,
        }
    }

    /// Returns the candidate position in meters.
    #[must_use]
    pub const fn position(&self) -> Vec2 {
        self.position
    }

    /// Returns the candidate velocity in meters per second.
    #[must_use]
    pub const fn velocity(&self) -> Vec2 {
        self.velocity
    }

    /// Replaces the candidate position after finite validation.
    ///
    /// # Errors
    ///
    /// Returns a coordinate-specific error when either value is non-finite.
    pub fn set_position(&mut self, position: Vec2) -> Result<(), ParticleEditError> {
        validate_vector(
            position,
            ParticleEditError::NonFinitePositionX,
            ParticleEditError::NonFinitePositionY,
        )?;
        self.position = position;
        Ok(())
    }

    /// Replaces the candidate velocity after finite validation.
    ///
    /// # Errors
    ///
    /// Returns a coordinate-specific error when either value is non-finite.
    pub fn set_velocity(&mut self, velocity: Vec2) -> Result<(), ParticleEditError> {
        validate_vector(
            velocity,
            ParticleEditError::NonFiniteVelocityX,
            ParticleEditError::NonFiniteVelocityY,
        )?;
        self.velocity = velocity;
        Ok(())
    }

    pub(crate) const fn into_parts(self) -> (Vec2, Vec2) {
        (self.position, self.velocity)
    }
}

fn validate_vector(
    value: Vec2,
    invalid_x: ParticleEditError,
    invalid_y: ParticleEditError,
) -> Result<(), ParticleEditError> {
    if !value.x.is_finite() {
        return Err(invalid_x);
    }
    if !value.y.is_finite() {
        return Err(invalid_y);
    }
    Ok(())
}
