use std::error::Error;
use std::fmt;

use crate::HandleError;
use crate::math::{Sweep, Transform, Vec2};

use super::fixture::FixtureBoundsError;

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
}

impl fmt::Display for BodyDefError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field = match self {
            Self::NonFinitePositionX => "position.x",
            Self::NonFinitePositionY => "position.y",
            Self::NonFiniteAngle => "angle",
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

/// A reusable checked body definition for the Phase 6 rigid-world slice.
///
/// Position coordinates are meters and `angle` is radians. This definition
/// intentionally contains only body type, transform, and active state. All
/// broader motion and integration controls remain outside the Phase 6 consumer
/// contract.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyDef {
    body_type: BodyType,
    position: Vec2,
    angle: f32,
    active: bool,
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
            active,
        })
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
        self.active
    }

    /// Returns an owned semantic snapshot of this definition.
    #[must_use]
    pub const fn snapshot(&self) -> BodySnapshot {
        BodySnapshot {
            body_type: self.body_type,
            position: self.position,
            angle: self.angle,
            active: self.active,
        }
    }
}

impl Default for BodyDef {
    fn default() -> Self {
        Self {
            body_type: BodyType::Static,
            position: Vec2::ZERO,
            angle: 0.0,
            active: true,
        }
    }
}

/// Owned semantic body state with no storage or mutable-world authority.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodySnapshot {
    body_type: BodyType,
    position: Vec2,
    angle: f32,
    active: bool,
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
        self.active
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(
    dead_code,
    reason = "the private transform, sweep, and velocity lanes are consumed by Plan 06-05"
)]
pub(super) struct BodyState {
    snapshot: BodySnapshot,
    transform: Transform,
    sweep: Sweep,
    linear_velocity: Vec2,
    angular_velocity: f32,
}

impl BodyState {
    pub(super) fn from_definition(definition: &BodyDef) -> Self {
        let position = definition.position();
        let angle = definition.angle();
        Self {
            snapshot: definition.snapshot(),
            transform: definition.transform(),
            sweep: initial_sweep(position, angle),
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
        }
    }

    pub(super) const fn snapshot(self) -> BodySnapshot {
        self.snapshot
    }

    pub(super) const fn transform(self) -> Transform {
        self.transform
    }

    pub(super) fn with_transform(self, position: Vec2, angle: f32) -> Result<Self, BodyDefError> {
        let definition = BodyDef::new(
            self.snapshot.body_type,
            position,
            angle,
            self.snapshot.active,
        )?;
        Ok(Self {
            snapshot: definition.snapshot(),
            transform: definition.transform(),
            sweep: initial_sweep(position, angle),
            linear_velocity: self.linear_velocity,
            angular_velocity: self.angular_velocity,
        })
    }

    pub(super) fn set_body_type(&mut self, body_type: BodyType) {
        self.snapshot.body_type = body_type;
        if body_type == BodyType::Static {
            self.linear_velocity = Vec2::ZERO;
            self.angular_velocity = 0.0;
        }
    }

    pub(super) fn set_active(&mut self, active: bool) {
        self.snapshot.active = active;
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
    /// Inertia about the center of mass is negative.
    NegativeCenteredRotationalInertia,
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
            Self::NegativeCenteredRotationalInertia => {
                "body centered rotational inertia must be non-negative"
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
/// Construction also proves that the parallel-axis adjustment produces a
/// finite, non-negative inertia about the center of mass.
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
    /// non-finite or negative source-ordered centered inertia.
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

        let centered_rotational_inertia = rotational_inertia - mass * center.dot(center);
        if !centered_rotational_inertia.is_finite() {
            return Err(BodyMassDataError::NonFiniteCenteredRotationalInertia);
        }
        if centered_rotational_inertia < 0.0 {
            return Err(BodyMassDataError::NegativeCenteredRotationalInertia);
        }

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
