use std::error::Error;
use std::fmt;

use crate::HandleError;
use crate::collision::MassData;
use crate::math::{Sweep, Transform, Vec2};

use super::fixture::FixtureBoundsError;

mod control;

pub use control::{BodyControlError, WakePolicy};

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

#[derive(Debug, Clone, Copy)]
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

impl BodyState {
    pub(super) fn from_definition(definition: &BodyDef) -> Self {
        let position = definition.position();
        let angle = definition.angle();
        let mass = initial_body_mass(definition.body_type());
        Self {
            snapshot: definition.snapshot(),
            transform: definition.transform(),
            sweep: initial_sweep(position, angle),
            linear_velocity: definition.linear_velocity(),
            angular_velocity: definition.angular_velocity(),
            inverse_mass: mass,
            inverse_inertia: 0.0,
            force: Vec2::ZERO,
            torque: 0.0,
            sleep_time: 0.0,
        }
    }

    pub(super) const fn snapshot(self) -> BodySnapshot {
        self.snapshot
    }

    pub(super) const fn transform(self) -> Transform {
        self.transform
    }

    pub(super) const fn sweep(self) -> Sweep {
        self.sweep
    }

    pub(super) const fn solver_linear(self) -> Vec2 {
        self.linear_velocity
    }

    pub(super) const fn solver_angular(self) -> f32 {
        self.angular_velocity
    }

    pub(super) const fn inverse_mass(self) -> f32 {
        self.inverse_mass
    }

    pub(super) const fn inverse_inertia(self) -> f32 {
        self.inverse_inertia
    }

    pub(super) const fn accumulated_force(self) -> Vec2 {
        self.force
    }

    pub(super) const fn accumulated_torque(self) -> f32 {
        self.torque
    }

    pub(super) const fn sleep_time(self) -> f32 {
        self.sleep_time
    }

    pub(super) fn candidate_set_sleep_time(mut self, sleep_time: f32) -> Self {
        self.sleep_time = sleep_time;
        self
    }

    #[cfg(test)]
    pub(super) fn set_solver_motion(&mut self, linear_velocity: Vec2, angular_velocity: f32) {
        self.linear_velocity = linear_velocity;
        self.angular_velocity = angular_velocity;
        self.snapshot.linear_velocity = linear_velocity;
        self.snapshot.angular_velocity = angular_velocity;
    }

    pub(super) fn candidate_set_solver_state(
        self,
        position: Vec2,
        angle: f32,
        linear_velocity: Vec2,
        angular_velocity: f32,
    ) -> Result<Self, BodyDefError> {
        validate_body_transform(position, angle)?;
        if !linear_velocity.is_valid() || !angular_velocity.is_finite() {
            return Err(BodyDefError::NonFiniteDerivedCenter);
        }
        let transform = Transform::from_position_angle(position, angle);
        let mut snapshot = self.snapshot;
        snapshot.position = position;
        snapshot.angle = angle;
        snapshot.linear_velocity = linear_velocity;
        snapshot.angular_velocity = angular_velocity;
        Ok(Self {
            snapshot,
            transform,
            sweep: Sweep::new(
                self.sweep.local_center(),
                self.sweep.center(),
                transform.apply(self.sweep.local_center()),
                self.sweep.angle(),
                angle,
                0.0,
            )
            .map_err(|_error| BodyDefError::NonFiniteDerivedCenter)?,
            linear_velocity,
            angular_velocity,
            inverse_mass: self.inverse_mass,
            inverse_inertia: self.inverse_inertia,
            force: self.force,
            torque: self.torque,
            sleep_time: self.sleep_time,
        })
    }

    pub(super) fn with_transform(self, position: Vec2, angle: f32) -> Result<Self, BodyDefError> {
        validate_body_transform(position, angle)?;
        let transform = Transform::from_position_angle(position, angle);
        let mut snapshot = self.snapshot;
        snapshot.position = position;
        snapshot.angle = angle;
        Ok(Self {
            snapshot,
            transform,
            sweep: Sweep::new(
                self.snapshot.local_center,
                transform.apply(self.snapshot.local_center),
                transform.apply(self.snapshot.local_center),
                angle,
                angle,
                0.0,
            )
            .map_err(|_error| BodyDefError::NonFiniteDerivedCenter)?,
            linear_velocity: self.linear_velocity,
            angular_velocity: self.angular_velocity,
            inverse_mass: self.inverse_mass,
            inverse_inertia: self.inverse_inertia,
            force: self.force,
            torque: self.torque,
            sleep_time: self.sleep_time,
        })
    }

    pub(super) fn with_body_type_and_reset_mass_data(
        mut self,
        body_type: BodyType,
        fixture_mass_data: &[MassData],
    ) -> Result<Self, AggregateMassError> {
        self.snapshot.body_type = body_type;
        if body_type == BodyType::Static {
            self.linear_velocity = Vec2::ZERO;
            self.angular_velocity = 0.0;
            self.snapshot.linear_velocity = Vec2::ZERO;
            self.snapshot.angular_velocity = 0.0;
        }
        let mass_state = aggregate_mass_state(
            body_type,
            self.snapshot.is_fixed_rotation(),
            fixture_mass_data,
        )?;
        self.with_mass_state(mass_state)
    }

    pub(super) fn set_active(&mut self, active: bool) {
        self.snapshot.flags.set(BodyFlags::ACTIVE, active);
    }

    pub(super) fn with_reset_mass_data(
        self,
        fixture_mass_data: &[MassData],
    ) -> Result<Self, AggregateMassError> {
        let mass_state = aggregate_mass_state(
            self.snapshot.body_type,
            self.snapshot.is_fixed_rotation(),
            fixture_mass_data,
        )?;
        self.with_mass_state(mass_state)
    }

    pub(super) fn set_mass_data(&mut self, data: BodyMassData) {
        if self.snapshot.body_type != BodyType::Dynamic {
            return;
        }
        let mass = if data.mass() > 0.0 { data.mass() } else { 1.0 };
        let rotational_inertia =
            if !self.snapshot.is_fixed_rotation() && data.rotational_inertia() > 0.0 {
                data.centered_rotational_inertia()
            } else {
                0.0
            };
        self.apply_mass_state(mass, data.center(), rotational_inertia);
    }

    fn apply_mass_state(&mut self, mass: f32, local_center: Vec2, rotational_inertia: f32) {
        let old_center = self.sweep.center();
        let current_center = self.transform.apply(local_center);
        self.snapshot.mass = mass;
        self.snapshot.local_center = local_center;
        self.snapshot.rotational_inertia = rotational_inertia;
        self.inverse_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        self.inverse_inertia = if rotational_inertia > 0.0 {
            1.0 / rotational_inertia
        } else {
            0.0
        };
        self.sweep = Sweep::new(
            local_center,
            current_center,
            current_center,
            self.snapshot.angle,
            self.snapshot.angle,
            0.0,
        )
        .expect("checked mass state and body transform produce a valid sweep");
        self.linear_velocity +=
            Vec2::scalar_cross(self.angular_velocity, current_center - old_center);
        self.snapshot.linear_velocity = self.linear_velocity;
    }

    fn with_mass_state(self, mass_state: MassState) -> Result<Self, AggregateMassError> {
        let old_center = self.sweep.center();
        let current_center = self.transform.apply(mass_state.local_center);
        if !current_center.x.is_finite() || !current_center.y.is_finite() {
            return Err(AggregateMassError::NonFiniteDerivedCenter);
        }
        let sweep = Sweep::new(
            mass_state.local_center,
            current_center,
            current_center,
            self.snapshot.angle,
            self.snapshot.angle,
            0.0,
        )
        .map_err(|_error| AggregateMassError::NonFiniteDerivedCenter)?;
        let linear_velocity = self.linear_velocity
            + Vec2::scalar_cross(self.angular_velocity, current_center - old_center);
        if !linear_velocity.x.is_finite() || !linear_velocity.y.is_finite() {
            return Err(AggregateMassError::NonFiniteDerivedVelocity);
        }

        let mut snapshot = self.snapshot;
        snapshot.mass = mass_state.mass;
        snapshot.local_center = mass_state.local_center;
        snapshot.rotational_inertia = mass_state.rotational_inertia;
        snapshot.linear_velocity = linear_velocity;
        Ok(Self {
            snapshot,
            transform: self.transform,
            sweep,
            linear_velocity,
            angular_velocity: self.angular_velocity,
            inverse_mass: mass_state.inverse_mass,
            inverse_inertia: mass_state.inverse_inertia,
            force: self.force,
            torque: self.torque,
            sleep_time: self.sleep_time,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct MassState {
    mass: f32,
    local_center: Vec2,
    rotational_inertia: f32,
    inverse_mass: f32,
    inverse_inertia: f32,
}

/// A failure while aggregating fixture mass properties in source order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AggregateMassError {
    /// Adding a fixture mass produced a non-finite aggregate.
    NonFiniteMass,
    /// Multiplying or adding the weighted x-coordinate produced a non-finite value.
    NonFiniteWeightedCenterX,
    /// Multiplying or adding the weighted y-coordinate produced a non-finite value.
    NonFiniteWeightedCenterY,
    /// Adding fixture inertia produced a non-finite aggregate.
    NonFiniteRotationalInertia,
    /// Inverting positive aggregate mass produced a non-finite value.
    NonFiniteInverseMass,
    /// Normalizing the aggregate center produced a non-finite x-coordinate.
    NonFiniteLocalCenterX,
    /// Normalizing the aggregate center produced a non-finite y-coordinate.
    NonFiniteLocalCenterY,
    /// Computing the squared aggregate center produced a non-finite value.
    NonFiniteCenterMagnitude,
    /// Applying the parallel-axis mass shift produced a non-finite value.
    NonFiniteCenterShift,
    /// Subtracting the parallel-axis shift produced a non-finite centered inertia.
    NonFiniteCenteredRotationalInertia,
    /// Positive origin inertia did not remain positive after centering.
    NonPositiveCenteredRotationalInertia,
    /// Inverting centered rotational inertia produced a non-finite value.
    NonFiniteInverseInertia,
    /// The aggregate center cannot be transformed into a finite world center.
    NonFiniteDerivedCenter,
    /// Moving the center of mass produced a non-finite linear velocity.
    NonFiniteDerivedVelocity,
}

impl fmt::Display for AggregateMassError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::NonFiniteMass => "aggregate fixture mass must remain finite",
            Self::NonFiniteWeightedCenterX => {
                "aggregate fixture weighted center.x must remain finite"
            }
            Self::NonFiniteWeightedCenterY => {
                "aggregate fixture weighted center.y must remain finite"
            }
            Self::NonFiniteRotationalInertia => {
                "aggregate fixture rotational inertia must remain finite"
            }
            Self::NonFiniteInverseMass => "aggregate fixture inverse mass must remain finite",
            Self::NonFiniteLocalCenterX => "aggregate fixture local center.x must remain finite",
            Self::NonFiniteLocalCenterY => "aggregate fixture local center.y must remain finite",
            Self::NonFiniteCenterMagnitude => {
                "aggregate fixture center magnitude must remain finite"
            }
            Self::NonFiniteCenterShift => {
                "aggregate fixture parallel-axis shift must remain finite"
            }
            Self::NonFiniteCenteredRotationalInertia => {
                "aggregate fixture centered inertia must remain finite"
            }
            Self::NonPositiveCenteredRotationalInertia => {
                "aggregate fixture centered inertia must remain positive"
            }
            Self::NonFiniteInverseInertia => "aggregate fixture inverse inertia must remain finite",
            Self::NonFiniteDerivedCenter => {
                "aggregate fixture center must produce a finite world center"
            }
            Self::NonFiniteDerivedVelocity => {
                "aggregate fixture center shift must produce finite velocity"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for AggregateMassError {}

/// A failure while explicitly recomputing a body's fixture-derived mass state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BodyMassResetError {
    /// The body identity does not resolve in this world.
    InvalidHandle(HandleError),
    /// The complete source-ordered fixture aggregate is invalid.
    InvalidAggregateMass(AggregateMassError),
}

impl fmt::Display for BodyMassResetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid body handle: {error}"),
            Self::InvalidAggregateMass(error) => {
                write!(formatter, "invalid aggregate body mass: {error}")
            }
        }
    }
}

impl Error for BodyMassResetError {}

impl From<HandleError> for BodyMassResetError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

impl From<AggregateMassError> for BodyMassResetError {
    fn from(error: AggregateMassError) -> Self {
        Self::InvalidAggregateMass(error)
    }
}

fn aggregate_mass_state(
    body_type: BodyType,
    fixed_rotation: bool,
    fixture_mass_data: &[MassData],
) -> Result<MassState, AggregateMassError> {
    if body_type != BodyType::Dynamic {
        return Ok(MassState {
            mass: 0.0,
            local_center: Vec2::ZERO,
            rotational_inertia: 0.0,
            inverse_mass: 0.0,
            inverse_inertia: 0.0,
        });
    }

    let mut mass = 0.0;
    let mut weighted_center = Vec2::ZERO;
    let mut rotational_inertia = 0.0;
    for data in fixture_mass_data {
        mass = checked_finite(mass + data.mass(), AggregateMassError::NonFiniteMass)?;
        let weighted_x = checked_finite(
            data.mass() * data.center().x,
            AggregateMassError::NonFiniteWeightedCenterX,
        )?;
        weighted_center.x = checked_finite(
            weighted_center.x + weighted_x,
            AggregateMassError::NonFiniteWeightedCenterX,
        )?;
        let weighted_y = checked_finite(
            data.mass() * data.center().y,
            AggregateMassError::NonFiniteWeightedCenterY,
        )?;
        weighted_center.y = checked_finite(
            weighted_center.y + weighted_y,
            AggregateMassError::NonFiniteWeightedCenterY,
        )?;
        rotational_inertia = checked_finite(
            rotational_inertia + data.rotational_inertia(),
            AggregateMassError::NonFiniteRotationalInertia,
        )?;
    }

    let (mass, inverse_mass, local_center) = if mass > 0.0 {
        let inverse_mass = checked_finite(1.0 / mass, AggregateMassError::NonFiniteInverseMass)?;
        let local_center = Vec2::new(
            checked_finite(
                weighted_center.x * inverse_mass,
                AggregateMassError::NonFiniteLocalCenterX,
            )?,
            checked_finite(
                weighted_center.y * inverse_mass,
                AggregateMassError::NonFiniteLocalCenterY,
            )?,
        );
        (mass, inverse_mass, local_center)
    } else {
        (1.0, 1.0, Vec2::ZERO)
    };

    let (rotational_inertia, inverse_inertia) = if rotational_inertia > 0.0 && !fixed_rotation {
        let squared_center = [
            checked_finite(
                local_center.x * local_center.x,
                AggregateMassError::NonFiniteCenterMagnitude,
            )?,
            checked_finite(
                local_center.y * local_center.y,
                AggregateMassError::NonFiniteCenterMagnitude,
            )?,
        ];
        let center_magnitude = checked_finite(
            squared_center[0] + squared_center[1],
            AggregateMassError::NonFiniteCenterMagnitude,
        )?;
        let center_shift = checked_finite(
            mass * center_magnitude,
            AggregateMassError::NonFiniteCenterShift,
        )?;
        let centered = checked_finite(
            rotational_inertia - center_shift,
            AggregateMassError::NonFiniteCenteredRotationalInertia,
        )?;
        if centered <= 0.0 {
            return Err(AggregateMassError::NonPositiveCenteredRotationalInertia);
        }
        let inverse = checked_finite(1.0 / centered, AggregateMassError::NonFiniteInverseInertia)?;
        (centered, inverse)
    } else {
        (0.0, 0.0)
    };

    Ok(MassState {
        mass,
        local_center,
        rotational_inertia,
        inverse_mass,
        inverse_inertia,
    })
}

fn checked_finite(value: f32, error: AggregateMassError) -> Result<f32, AggregateMassError> {
    if !value.is_finite() {
        return Err(error);
    }
    Ok(value)
}

const fn initial_body_mass(body_type: BodyType) -> f32 {
    match body_type {
        BodyType::Dynamic => 1.0,
        BodyType::Static | BodyType::Kinematic => 0.0,
    }
}

fn initial_sweep(position: Vec2, angle: f32) -> Sweep {
    Sweep::new(Vec2::ZERO, position, position, angle, angle, 0.0)
        .expect("checked body transforms always produce a valid initial sweep")
}

/// A failure while constructing checked custom body mass data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BodyMassDataError {
    /// Mass is not finite.
    NonFiniteMass,
    /// The x-coordinate of the local center is not finite.
    NonFiniteCenterX,
    /// The y-coordinate of the local center is not finite.
    NonFiniteCenterY,
    /// Rotational inertia about the local origin is not finite.
    NonFiniteRotationalInertia,
    /// The source-ordered centered inertia computation is not finite.
    NonFiniteCenteredRotationalInertia,
    /// Mass is negative.
    NegativeMass,
    /// Positive origin inertia did not produce positive inertia about the center of mass.
    NonPositiveCenteredRotationalInertia,
}

impl fmt::Display for BodyMassDataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::NonFiniteMass => "body mass must be finite",
            Self::NonFiniteCenterX => "body mass center.x must be finite",
            Self::NonFiniteCenterY => "body mass center.y must be finite",
            Self::NonFiniteRotationalInertia => "body rotational inertia must be finite",
            Self::NonFiniteCenteredRotationalInertia => {
                "body centered rotational inertia must be finite"
            }
            Self::NegativeMass => "body mass must be non-negative",
            Self::NonPositiveCenteredRotationalInertia => {
                "body centered rotational inertia must be positive"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for BodyMassDataError {}

/// Checked custom mass properties for one body.
///
/// Mass is kilograms, `center` is meters in the body's local frame, and
/// rotational inertia is kilograms-meter-squared about the local origin.
/// Origin inertia zero selects the pinned no-inertia branch. Positive origin
/// inertia must produce finite, positive inertia about the center of mass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyMassData {
    mass: f32,
    center: Vec2,
    rotational_inertia: f32,
    centered_rotational_inertia: f32,
}

impl BodyMassData {
    /// Creates checked custom body mass properties.
    ///
    /// # Errors
    ///
    /// Returns a typed error for any non-finite input, negative mass, or a
    /// non-finite or non-positive source-ordered centered inertia when origin
    /// inertia is positive.
    #[must_use = "body mass-data construction can fail for invalid values"]
    pub fn new(
        mass: f32,
        center: Vec2,
        rotational_inertia: f32,
    ) -> Result<Self, BodyMassDataError> {
        validate_body_mass_inputs(mass, center, rotational_inertia)?;
        if mass < 0.0 {
            return Err(BodyMassDataError::NegativeMass);
        }

        let centered_rotational_inertia = if rotational_inertia == 0.0 {
            0.0
        } else {
            let effective_mass = if mass > 0.0 { mass } else { 1.0 };
            let squared_center = [
                checked_body_mass_finite(center.x * center.x)?,
                checked_body_mass_finite(center.y * center.y)?,
            ];
            let center_dot = checked_body_mass_finite(squared_center[0] + squared_center[1])?;
            let parallel_axis = checked_body_mass_finite(effective_mass * center_dot)?;
            let centered = checked_body_mass_finite(rotational_inertia - parallel_axis)?;
            if centered <= 0.0 {
                return Err(BodyMassDataError::NonPositiveCenteredRotationalInertia);
            }
            centered
        };

        Ok(Self {
            mass,
            center,
            rotational_inertia,
            centered_rotational_inertia,
        })
    }

    /// Returns mass in kilograms.
    #[must_use]
    pub const fn mass(self) -> f32 {
        self.mass
    }

    /// Returns the local center of mass in meters.
    #[must_use]
    pub const fn center(self) -> Vec2 {
        self.center
    }

    /// Returns rotational inertia about the local origin in kilograms-meter-squared.
    #[must_use]
    pub const fn rotational_inertia(self) -> f32 {
        self.rotational_inertia
    }

    /// Returns rotational inertia about the center of mass in kilograms-meter-squared.
    #[must_use]
    pub const fn centered_rotational_inertia(self) -> f32 {
        self.centered_rotational_inertia
    }
}

fn checked_body_mass_finite(value: f32) -> Result<f32, BodyMassDataError> {
    if !value.is_finite() {
        return Err(BodyMassDataError::NonFiniteCenteredRotationalInertia);
    }
    Ok(value)
}

fn validate_body_transform(position: Vec2, angle: f32) -> Result<(), BodyDefError> {
    if !position.x.is_finite() {
        return Err(BodyDefError::NonFinitePositionX);
    }
    if !position.y.is_finite() {
        return Err(BodyDefError::NonFinitePositionY);
    }
    if !angle.is_finite() {
        return Err(BodyDefError::NonFiniteAngle);
    }
    Ok(())
}

fn validate_linear_velocity(velocity: Vec2) -> Result<(), BodyDefError> {
    if !velocity.x.is_finite() {
        return Err(BodyDefError::NonFiniteLinearVelocityX);
    }
    if !velocity.y.is_finite() {
        return Err(BodyDefError::NonFiniteLinearVelocityY);
    }
    Ok(())
}

fn validate_angular_velocity(angular_velocity: f32) -> Result<(), BodyDefError> {
    if !angular_velocity.is_finite() {
        return Err(BodyDefError::NonFiniteAngularVelocity);
    }
    Ok(())
}

fn validate_linear_damping(damping: f32) -> Result<(), BodyDefError> {
    if !damping.is_finite() {
        return Err(BodyDefError::NonFiniteLinearDamping);
    }
    if damping < 0.0 {
        return Err(BodyDefError::NegativeLinearDamping);
    }
    Ok(())
}

fn validate_angular_damping(damping: f32) -> Result<(), BodyDefError> {
    if !damping.is_finite() {
        return Err(BodyDefError::NonFiniteAngularDamping);
    }
    if damping < 0.0 {
        return Err(BodyDefError::NegativeAngularDamping);
    }
    Ok(())
}

fn validate_gravity_scale(gravity_scale: f32) -> Result<(), BodyDefError> {
    if !gravity_scale.is_finite() {
        return Err(BodyDefError::NonFiniteGravityScale);
    }
    Ok(())
}

fn validate_body_mass_inputs(
    mass: f32,
    center: Vec2,
    rotational_inertia: f32,
) -> Result<(), BodyMassDataError> {
    if !mass.is_finite() {
        return Err(BodyMassDataError::NonFiniteMass);
    }
    if !center.x.is_finite() {
        return Err(BodyMassDataError::NonFiniteCenterX);
    }
    if !center.y.is_finite() {
        return Err(BodyMassDataError::NonFiniteCenterY);
    }
    if !rotational_inertia.is_finite() {
        return Err(BodyMassDataError::NonFiniteRotationalInertia);
    }
    Ok(())
}
