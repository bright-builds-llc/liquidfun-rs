use super::{BodyDefError, BodyType, Error, HandleError, MassData, Sweep, Vec2, fmt};

#[derive(Debug, Clone, Copy)]
pub(super) struct MassState {
    pub(super) mass: f32,
    pub(super) local_center: Vec2,
    pub(super) rotational_inertia: f32,
    pub(super) inverse_mass: f32,
    pub(super) inverse_inertia: f32,
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

/// A failure while replacing a body's custom mass state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BodyMassMutationError {
    /// The body identity does not resolve in this world.
    InvalidHandle(HandleError),
    /// The checked source data produced an invalid derived body state.
    InvalidDerivedMass(AggregateMassError),
}

impl fmt::Display for BodyMassMutationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid body handle: {error}"),
            Self::InvalidDerivedMass(error) => {
                write!(formatter, "invalid custom body mass: {error}")
            }
        }
    }
}

impl Error for BodyMassMutationError {}

impl From<HandleError> for BodyMassMutationError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

impl From<AggregateMassError> for BodyMassMutationError {
    fn from(error: AggregateMassError) -> Self {
        Self::InvalidDerivedMass(error)
    }
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

pub(super) fn aggregate_mass_state(
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

pub(super) fn checked_finite(
    value: f32,
    error: AggregateMassError,
) -> Result<f32, AggregateMassError> {
    if !value.is_finite() {
        return Err(error);
    }
    Ok(value)
}

pub(super) const fn initial_body_mass(body_type: BodyType) -> f32 {
    match body_type {
        BodyType::Dynamic => 1.0,
        BodyType::Static | BodyType::Kinematic => 0.0,
    }
}

pub(super) fn initial_sweep(position: Vec2, angle: f32) -> Sweep {
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

pub(super) fn validate_body_transform(position: Vec2, angle: f32) -> Result<(), BodyDefError> {
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

pub(super) fn validate_linear_velocity(velocity: Vec2) -> Result<(), BodyDefError> {
    if !velocity.x.is_finite() {
        return Err(BodyDefError::NonFiniteLinearVelocityX);
    }
    if !velocity.y.is_finite() {
        return Err(BodyDefError::NonFiniteLinearVelocityY);
    }
    Ok(())
}

pub(super) fn validate_angular_velocity(angular_velocity: f32) -> Result<(), BodyDefError> {
    if !angular_velocity.is_finite() {
        return Err(BodyDefError::NonFiniteAngularVelocity);
    }
    Ok(())
}

pub(super) fn validate_linear_damping(damping: f32) -> Result<(), BodyDefError> {
    if !damping.is_finite() {
        return Err(BodyDefError::NonFiniteLinearDamping);
    }
    if damping < 0.0 {
        return Err(BodyDefError::NegativeLinearDamping);
    }
    Ok(())
}

pub(super) fn validate_angular_damping(damping: f32) -> Result<(), BodyDefError> {
    if !damping.is_finite() {
        return Err(BodyDefError::NonFiniteAngularDamping);
    }
    if damping < 0.0 {
        return Err(BodyDefError::NegativeAngularDamping);
    }
    Ok(())
}

pub(super) fn validate_gravity_scale(gravity_scale: f32) -> Result<(), BodyDefError> {
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
