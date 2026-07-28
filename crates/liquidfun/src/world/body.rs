use std::error::Error;
use std::fmt;

use crate::HandleError;
use crate::collision::MassData;
use crate::math::{Sweep, SweepError, Transform, Vec2};

use super::fixture::FixtureBoundsError;

mod control;
mod mass;
mod state;

pub use control::{BodyControlError, WakePolicy};
pub use mass::{
    AggregateMassError, BodyMassData, BodyMassDataError, BodyMassMutationError, BodyMassResetError,
};
use mass::{
    MassState, aggregate_mass_state, checked_finite, initial_body_mass, initial_sweep,
    validate_angular_damping, validate_angular_velocity, validate_body_transform,
    validate_gravity_scale, validate_linear_damping, validate_linear_velocity,
};

/// The closed set of rigid-body motion types supported by `LiquidFun`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum BodyType {
    /// A zero-mass body moved only through explicit transform changes.
    #[default]
    Static,
    /// A zero-mass body whose solver-driven motion is introduced in Phase 7.
    Kinematic,
    /// A positive-mass body affected by rigid contact solving.
    Dynamic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BodyFlags(u8);

impl BodyFlags {
    const ACTIVE: u8 = 1 << 0;
    const SLEEPING_ALLOWED: u8 = 1 << 1;
    const AWAKE: u8 = 1 << 2;
    const FIXED_ROTATION: u8 = 1 << 3;
    const BULLET: u8 = 1 << 4;

    const fn contains(self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    fn set(&mut self, flag: u8, enabled: bool) {
        if enabled {
            self.0 |= flag;
        } else {
            self.0 &= !flag;
        }
    }
}

const INITIAL_BODY_FLAGS: BodyFlags =
    BodyFlags(BodyFlags::ACTIVE | BodyFlags::SLEEPING_ALLOWED | BodyFlags::AWAKE);

fn configured_initial_flags(active: bool) -> BodyFlags {
    let mut flags = INITIAL_BODY_FLAGS;
    flags.set(BodyFlags::ACTIVE, active);
    flags
}

/// A failure while constructing a checked [`BodyDef`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BodyDefError {
    /// The x-coordinate of the body origin is not finite.
    NonFinitePositionX,
    /// The y-coordinate of the body origin is not finite.
    NonFinitePositionY,
    /// The body angle is not finite.
    NonFiniteAngle,
    /// Applying the body transform to its current local center is not finite.
    NonFiniteDerivedCenter,
    /// The x-coordinate of linear velocity is not finite.
    NonFiniteLinearVelocityX,
    /// The y-coordinate of linear velocity is not finite.
    NonFiniteLinearVelocityY,
    /// Angular velocity is not finite.
    NonFiniteAngularVelocity,
    /// Linear damping is not finite.
    NonFiniteLinearDamping,
    /// Linear damping is negative.
    NegativeLinearDamping,
    /// Angular damping is not finite.
    NonFiniteAngularDamping,
    /// Angular damping is negative.
    NegativeAngularDamping,
    /// Gravity scale is not finite.
    NonFiniteGravityScale,
}

impl fmt::Display for BodyDefError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field = match self {
            Self::NonFinitePositionX => "position.x",
            Self::NonFinitePositionY => "position.y",
            Self::NonFiniteAngle => "angle",
            Self::NonFiniteDerivedCenter => "derived center",
            Self::NonFiniteLinearVelocityX => "linear_velocity.x",
            Self::NonFiniteLinearVelocityY => "linear_velocity.y",
            Self::NonFiniteAngularVelocity => "angular_velocity",
            Self::NonFiniteLinearDamping => "linear_damping",
            Self::NegativeLinearDamping => {
                return formatter.write_str("body definition linear_damping must be non-negative");
            }
            Self::NonFiniteAngularDamping => "angular_damping",
            Self::NegativeAngularDamping => {
                return formatter.write_str("body definition angular_damping must be non-negative");
            }
            Self::NonFiniteGravityScale => "gravity_scale",
        };
        write!(formatter, "body definition {field} must be finite")
    }
}

impl Error for BodyDefError {}

/// A failure while changing a body's checked transform through its owning world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BodyTransformError {
    /// The body identity does not resolve in this world.
    InvalidHandle(HandleError),
    /// The requested position or angle is not finite.
    InvalidTransform(BodyDefError),
    /// Fixture child bounds cannot be represented at the requested transform.
    InvalidFixtureBounds(FixtureBoundsError),
}

impl fmt::Display for BodyTransformError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid body handle: {error}"),
            Self::InvalidTransform(error) => write!(formatter, "invalid body transform: {error}"),
            Self::InvalidFixtureBounds(error) => {
                write!(formatter, "invalid transformed fixture bounds: {error}")
            }
        }
    }
}

impl Error for BodyTransformError {}

impl From<HandleError> for BodyTransformError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

impl From<BodyDefError> for BodyTransformError {
    fn from(error: BodyDefError) -> Self {
        Self::InvalidTransform(error)
    }
}

impl From<FixtureBoundsError> for BodyTransformError {
    fn from(error: FixtureBoundsError) -> Self {
        Self::InvalidFixtureBounds(error)
    }
}

/// A failure while changing whether a body participates in simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BodyActivationError {
    /// The body identity does not resolve in this world.
    InvalidHandle(HandleError),
    /// Fixture child bounds cannot be represented during activation.
    InvalidFixtureBounds(FixtureBoundsError),
}

impl fmt::Display for BodyActivationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid body handle: {error}"),
            Self::InvalidFixtureBounds(error) => {
                write!(formatter, "invalid activated fixture bounds: {error}")
            }
        }
    }
}

impl Error for BodyActivationError {}

impl From<HandleError> for BodyActivationError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

impl From<FixtureBoundsError> for BodyActivationError {
    fn from(error: FixtureBoundsError) -> Self {
        Self::InvalidFixtureBounds(error)
    }
}

/// A failure while changing a body's motion type through its owning world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BodyTypeChangeError {
    /// The body identity does not resolve in this world.
    InvalidHandle(HandleError),
    /// The target type's complete source-ordered fixture aggregate is invalid.
    InvalidAggregateMass(AggregateMassError),
}

impl fmt::Display for BodyTypeChangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid body handle: {error}"),
            Self::InvalidAggregateMass(error) => {
                write!(formatter, "invalid aggregate body mass: {error}")
            }
        }
    }
}

impl Error for BodyTypeChangeError {}

impl From<HandleError> for BodyTypeChangeError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

impl From<AggregateMassError> for BodyTypeChangeError {
    fn from(error: AggregateMassError) -> Self {
        Self::InvalidAggregateMass(error)
    }
}

/// A reusable checked rigid-body definition.
///
/// Position coordinates are meters, `angle` is radians, linear velocity is
/// meters per second, and angular velocity is radians per second.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyDef {
    body_type: BodyType,
    position: Vec2,
    angle: f32,
    linear_velocity: Vec2,
    angular_velocity: f32,
    linear_damping: f32,
    angular_damping: f32,
    gravity_scale: f32,
    flags: BodyFlags,
}

impl BodyDef {
    /// Creates a reusable body definition after checking every transform lane.
    ///
    /// Accepted `f32` values retain their exact bit patterns and are never
    /// clamped or normalized.
    ///
    /// # Errors
    ///
    /// Returns a field-specific error when either position coordinate or the
    /// angle is non-finite.
    #[must_use = "body-definition construction can fail for a non-finite transform"]
    pub fn new(
        body_type: BodyType,
        position: Vec2,
        angle: f32,
        active: bool,
    ) -> Result<Self, BodyDefError> {
        validate_body_transform(position, angle)?;
        Ok(Self {
            body_type,
            position,
            angle,
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 1.0,
            flags: configured_initial_flags(active),
        })
    }

    /// Returns a copy configured with checked initial linear velocity.
    ///
    /// # Errors
    ///
    /// Returns a coordinate-specific error when the velocity is non-finite.
    pub fn with_linear_velocity(mut self, velocity: Vec2) -> Result<Self, BodyDefError> {
        validate_linear_velocity(velocity)?;
        self.linear_velocity = velocity;
        Ok(self)
    }

    /// Returns a copy configured with checked initial angular velocity.
    ///
    /// # Errors
    ///
    /// Returns an error when `angular_velocity` is non-finite.
    pub fn with_angular_velocity(mut self, angular_velocity: f32) -> Result<Self, BodyDefError> {
        validate_angular_velocity(angular_velocity)?;
        self.angular_velocity = angular_velocity;
        Ok(self)
    }

    /// Returns a copy configured with checked linear damping.
    ///
    /// # Errors
    ///
    /// Returns an error when `linear_damping` is non-finite or negative.
    pub fn with_linear_damping(mut self, linear_damping: f32) -> Result<Self, BodyDefError> {
        validate_linear_damping(linear_damping)?;
        self.linear_damping = linear_damping;
        Ok(self)
    }

    /// Returns a copy configured with checked angular damping.
    ///
    /// # Errors
    ///
    /// Returns an error when `angular_damping` is non-finite or negative.
    pub fn with_angular_damping(mut self, angular_damping: f32) -> Result<Self, BodyDefError> {
        validate_angular_damping(angular_damping)?;
        self.angular_damping = angular_damping;
        Ok(self)
    }

    /// Returns a copy configured with checked gravity scale.
    ///
    /// # Errors
    ///
    /// Returns an error when `gravity_scale` is non-finite.
    pub fn with_gravity_scale(mut self, gravity_scale: f32) -> Result<Self, BodyDefError> {
        validate_gravity_scale(gravity_scale)?;
        self.gravity_scale = gravity_scale;
        Ok(self)
    }

    /// Returns a copy configured to allow or disallow automatic sleep.
    #[must_use]
    pub fn with_sleeping_allowed(mut self, sleeping_allowed: bool) -> Self {
        self.flags
            .set(BodyFlags::SLEEPING_ALLOWED, sleeping_allowed);
        self
    }

    /// Returns a copy configured as initially awake or asleep.
    #[must_use]
    pub fn with_awake(mut self, awake: bool) -> Self {
        self.flags.set(BodyFlags::AWAKE, awake);
        self
    }

    /// Returns a copy configured with fixed or free rotation.
    #[must_use]
    pub fn with_fixed_rotation(mut self, fixed_rotation: bool) -> Self {
        self.flags.set(BodyFlags::FIXED_ROTATION, fixed_rotation);
        self
    }

    /// Returns a copy configured for continuous collision treatment as a bullet.
    #[must_use]
    pub fn with_bullet(mut self, bullet: bool) -> Self {
        self.flags.set(BodyFlags::BULLET, bullet);
        self
    }

    /// Returns the configured motion type.
    #[must_use]
    pub const fn body_type(&self) -> BodyType {
        self.body_type
    }

    /// Returns the body origin in meters with its accepted bits unchanged.
    #[must_use]
    pub const fn position(&self) -> Vec2 {
        self.position
    }

    /// Returns the body angle in radians with its accepted bits unchanged.
    #[must_use]
    pub const fn angle(&self) -> f32 {
        self.angle
    }

    /// Returns an initialized transform derived from the checked definition.
    #[must_use]
    pub fn transform(&self) -> Transform {
        Transform::from_position_angle(self.position, self.angle)
    }

    /// Returns whether the body initially participates in simulation.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.flags.contains(BodyFlags::ACTIVE)
    }

    /// Returns initial linear velocity in meters per second.
    #[must_use]
    pub const fn linear_velocity(&self) -> Vec2 {
        self.linear_velocity
    }

    /// Returns initial angular velocity in radians per second.
    #[must_use]
    pub const fn angular_velocity(&self) -> f32 {
        self.angular_velocity
    }

    /// Returns linear damping.
    #[must_use]
    pub const fn linear_damping(&self) -> f32 {
        self.linear_damping
    }

    /// Returns angular damping.
    #[must_use]
    pub const fn angular_damping(&self) -> f32 {
        self.angular_damping
    }

    /// Returns the body's gravity multiplier.
    #[must_use]
    pub const fn gravity_scale(&self) -> f32 {
        self.gravity_scale
    }

    /// Returns whether automatic sleep is allowed.
    #[must_use]
    pub const fn is_sleeping_allowed(&self) -> bool {
        self.flags.contains(BodyFlags::SLEEPING_ALLOWED)
    }

    /// Returns whether the body starts awake.
    #[must_use]
    pub const fn is_awake(&self) -> bool {
        self.flags.contains(BodyFlags::AWAKE)
    }

    /// Returns whether rotation is fixed.
    #[must_use]
    pub const fn is_fixed_rotation(&self) -> bool {
        self.flags.contains(BodyFlags::FIXED_ROTATION)
    }

    /// Returns whether continuous collision treatment is requested.
    #[must_use]
    pub const fn is_bullet(&self) -> bool {
        self.flags.contains(BodyFlags::BULLET)
    }

    /// Returns an owned semantic snapshot of this definition.
    #[must_use]
    pub const fn snapshot(&self) -> BodySnapshot {
        let mass = initial_body_mass(self.body_type);
        BodySnapshot {
            body_type: self.body_type,
            position: self.position,
            angle: self.angle,
            mass,
            local_center: Vec2::ZERO,
            rotational_inertia: 0.0,
            linear_velocity: self.linear_velocity,
            angular_velocity: self.angular_velocity,
            linear_damping: self.linear_damping,
            angular_damping: self.angular_damping,
            gravity_scale: self.gravity_scale,
            flags: self.flags,
        }
    }
}

impl Default for BodyDef {
    fn default() -> Self {
        Self {
            body_type: BodyType::Static,
            position: Vec2::ZERO,
            angle: 0.0,
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 1.0,
            flags: INITIAL_BODY_FLAGS,
        }
    }
}

/// Owned semantic body state with no storage or mutable-world authority.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodySnapshot {
    body_type: BodyType,
    position: Vec2,
    angle: f32,
    mass: f32,
    local_center: Vec2,
    rotational_inertia: f32,
    linear_velocity: Vec2,
    angular_velocity: f32,
    linear_damping: f32,
    angular_damping: f32,
    gravity_scale: f32,
    flags: BodyFlags,
}

impl Eq for BodySnapshot {}

impl BodySnapshot {
    /// Returns the body's motion type.
    #[must_use]
    pub const fn body_type(self) -> BodyType {
        self.body_type
    }

    /// Returns the body origin in meters.
    #[must_use]
    pub const fn position(self) -> Vec2 {
        self.position
    }

    /// Returns the body angle in radians.
    #[must_use]
    pub const fn angle(self) -> f32 {
        self.angle
    }

    /// Returns an initialized transform for the captured pose.
    #[must_use]
    pub fn transform(self) -> Transform {
        Transform::from_position_angle(self.position, self.angle)
    }

    /// Returns whether the body was active when captured.
    #[must_use]
    pub const fn is_active(self) -> bool {
        self.flags.contains(BodyFlags::ACTIVE)
    }

    /// Returns body mass in kilograms.
    #[must_use]
    pub const fn mass(self) -> f32 {
        self.mass
    }

    /// Returns the local center of mass in meters.
    #[must_use]
    pub const fn local_center(self) -> Vec2 {
        self.local_center
    }

    /// Returns rotational inertia about the local center of mass.
    #[must_use]
    pub const fn rotational_inertia(self) -> f32 {
        self.rotational_inertia
    }

    /// Returns linear velocity in meters per second.
    #[must_use]
    pub const fn linear_velocity(self) -> Vec2 {
        self.linear_velocity
    }

    /// Returns angular velocity in radians per second.
    #[must_use]
    pub const fn angular_velocity(self) -> f32 {
        self.angular_velocity
    }

    /// Returns linear damping.
    #[must_use]
    pub const fn linear_damping(self) -> f32 {
        self.linear_damping
    }

    /// Returns angular damping.
    #[must_use]
    pub const fn angular_damping(self) -> f32 {
        self.angular_damping
    }

    /// Returns the body's gravity multiplier.
    #[must_use]
    pub const fn gravity_scale(self) -> f32 {
        self.gravity_scale
    }

    /// Returns whether automatic sleep is allowed.
    #[must_use]
    pub const fn is_sleeping_allowed(self) -> bool {
        self.flags.contains(BodyFlags::SLEEPING_ALLOWED)
    }

    /// Returns whether the body is awake.
    #[must_use]
    pub const fn is_awake(self) -> bool {
        self.flags.contains(BodyFlags::AWAKE)
    }

    /// Returns whether rotation is fixed.
    #[must_use]
    pub const fn is_fixed_rotation(self) -> bool {
        self.flags.contains(BodyFlags::FIXED_ROTATION)
    }

    /// Returns whether continuous collision treatment is requested.
    #[must_use]
    pub const fn is_bullet(self) -> bool {
        self.flags.contains(BodyFlags::BULLET)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct BodyState {
    snapshot: BodySnapshot,
    transform: Transform,
    sweep: Sweep,
    linear_velocity: Vec2,
    angular_velocity: f32,
    inverse_mass: f32,
    inverse_inertia: f32,
    force: Vec2,
    torque: f32,
    sleep_time: f32,
}
