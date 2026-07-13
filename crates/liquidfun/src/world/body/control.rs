use std::error::Error;
use std::fmt;

use crate::HandleError;
use crate::collision::MassData;
use crate::math::Vec2;

use super::{AggregateMassError, BodyFlags, BodyState, BodyType};

/// Controls whether a force or impulse wakes an asleep body before application.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum WakePolicy {
    /// Wake an asleep dynamic body before applying the requested control.
    #[default]
    Wake,
    /// Preserve sleep; an asleep body accepts the call without applying it.
    PreserveSleep,
}

/// A failure while applying a checked body control through its owning world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BodyControlError {
    /// The body identity does not resolve in this world.
    InvalidHandle(HandleError),
    /// The x-coordinate of linear velocity is not finite.
    NonFiniteLinearVelocityX,
    /// The y-coordinate of linear velocity is not finite.
    NonFiniteLinearVelocityY,
    /// Angular velocity is not finite.
    NonFiniteAngularVelocity,
    /// The x-coordinate of force is not finite.
    NonFiniteForceX,
    /// The y-coordinate of force is not finite.
    NonFiniteForceY,
    /// The x-coordinate of an application point is not finite.
    NonFinitePointX,
    /// The y-coordinate of an application point is not finite.
    NonFinitePointY,
    /// Torque is not finite.
    NonFiniteTorque,
    /// The x-coordinate of linear impulse is not finite.
    NonFiniteLinearImpulseX,
    /// The y-coordinate of linear impulse is not finite.
    NonFiniteLinearImpulseY,
    /// Angular impulse is not finite.
    NonFiniteAngularImpulse,
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
    /// Force accumulation produced a non-finite x-coordinate.
    NonFiniteDerivedForceX,
    /// Force accumulation produced a non-finite y-coordinate.
    NonFiniteDerivedForceY,
    /// Force or torque application produced a non-finite torque.
    NonFiniteDerivedTorque,
    /// An impulse produced a non-finite linear velocity x-coordinate.
    NonFiniteDerivedLinearVelocityX,
    /// An impulse produced a non-finite linear velocity y-coordinate.
    NonFiniteDerivedLinearVelocityY,
    /// An impulse produced a non-finite angular velocity.
    NonFiniteDerivedAngularVelocity,
    /// Fixed-rotation mass recomputation failed.
    InvalidAggregateMass(AggregateMassError),
}

impl fmt::Display for BodyControlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidHandle(error) => return write!(formatter, "invalid body handle: {error}"),
            Self::NonFiniteLinearVelocityX => "linear velocity.x must be finite",
            Self::NonFiniteLinearVelocityY => "linear velocity.y must be finite",
            Self::NonFiniteAngularVelocity => "angular velocity must be finite",
            Self::NonFiniteForceX => "force.x must be finite",
            Self::NonFiniteForceY => "force.y must be finite",
            Self::NonFinitePointX => "application point.x must be finite",
            Self::NonFinitePointY => "application point.y must be finite",
            Self::NonFiniteTorque => "torque must be finite",
            Self::NonFiniteLinearImpulseX => "linear impulse.x must be finite",
            Self::NonFiniteLinearImpulseY => "linear impulse.y must be finite",
            Self::NonFiniteAngularImpulse => "angular impulse must be finite",
            Self::NonFiniteLinearDamping => "linear damping must be finite",
            Self::NegativeLinearDamping => "linear damping must be non-negative",
            Self::NonFiniteAngularDamping => "angular damping must be finite",
            Self::NegativeAngularDamping => "angular damping must be non-negative",
            Self::NonFiniteGravityScale => "gravity scale must be finite",
            Self::NonFiniteDerivedForceX => "accumulated force.x must remain finite",
            Self::NonFiniteDerivedForceY => "accumulated force.y must remain finite",
            Self::NonFiniteDerivedTorque => "accumulated torque must remain finite",
            Self::NonFiniteDerivedLinearVelocityX => {
                "impulse-derived linear velocity.x must remain finite"
            }
            Self::NonFiniteDerivedLinearVelocityY => {
                "impulse-derived linear velocity.y must remain finite"
            }
            Self::NonFiniteDerivedAngularVelocity => {
                "impulse-derived angular velocity must remain finite"
            }
            Self::InvalidAggregateMass(error) => {
                return write!(formatter, "fixed-rotation mass reset failed: {error}");
            }
        };
        formatter.write_str(message)
    }
}

impl Error for BodyControlError {}

impl From<HandleError> for BodyControlError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

impl From<AggregateMassError> for BodyControlError {
    fn from(error: AggregateMassError) -> Self {
        Self::InvalidAggregateMass(error)
    }
}

impl BodyState {
    pub(crate) fn candidate_set_linear_velocity(
        mut self,
        velocity: Vec2,
    ) -> Result<Self, BodyControlError> {
        if self.snapshot.body_type == BodyType::Static {
            return Ok(self);
        }
        validate_linear_velocity(velocity)?;
        if velocity.length_squared() > 0.0 {
            self.wake();
        }
        self.linear_velocity = velocity;
        self.snapshot.linear_velocity = velocity;
        Ok(self)
    }

    pub(crate) fn candidate_set_angular_velocity(
        mut self,
        angular_velocity: f32,
    ) -> Result<Self, BodyControlError> {
        if self.snapshot.body_type == BodyType::Static {
            return Ok(self);
        }
        validate_angular_velocity(angular_velocity)?;
        if angular_velocity * angular_velocity > 0.0 {
            self.wake();
        }
        self.angular_velocity = angular_velocity;
        self.snapshot.angular_velocity = angular_velocity;
        Ok(self)
    }

    pub(crate) fn candidate_apply_force(
        mut self,
        force: Vec2,
        point: Vec2,
        wake_policy: WakePolicy,
    ) -> Result<Self, BodyControlError> {
        if !self.prepare_dynamic_application(wake_policy) {
            return Ok(self);
        }
        validate_force(force)?;
        validate_point(point)?;
        let force_x = checked(
            self.force.x + force.x,
            BodyControlError::NonFiniteDerivedForceX,
        )?;
        let force_y = checked(
            self.force.y + force.y,
            BodyControlError::NonFiniteDerivedForceY,
        )?;
        let offset_x = checked(
            point.x - self.sweep.center().x,
            BodyControlError::NonFiniteDerivedTorque,
        )?;
        let offset_y = checked(
            point.y - self.sweep.center().y,
            BodyControlError::NonFiniteDerivedTorque,
        )?;
        let lever_torque = checked(
            offset_x * force.y - offset_y * force.x,
            BodyControlError::NonFiniteDerivedTorque,
        )?;
        let torque = checked(
            self.torque + lever_torque,
            BodyControlError::NonFiniteDerivedTorque,
        )?;
        self.force = Vec2::new(force_x, force_y);
        self.torque = torque;
        Ok(self)
    }

    pub(crate) fn candidate_apply_force_to_center(
        mut self,
        force: Vec2,
        wake_policy: WakePolicy,
    ) -> Result<Self, BodyControlError> {
        if !self.prepare_dynamic_application(wake_policy) {
            return Ok(self);
        }
        validate_force(force)?;
        self.force = Vec2::new(
            checked(
                self.force.x + force.x,
                BodyControlError::NonFiniteDerivedForceX,
            )?,
            checked(
                self.force.y + force.y,
                BodyControlError::NonFiniteDerivedForceY,
            )?,
        );
        Ok(self)
    }

    pub(crate) fn candidate_apply_torque(
        mut self,
        torque: f32,
        wake_policy: WakePolicy,
    ) -> Result<Self, BodyControlError> {
        if !self.prepare_dynamic_application(wake_policy) {
            return Ok(self);
        }
        validate_torque(torque)?;
        self.torque = checked(
            self.torque + torque,
            BodyControlError::NonFiniteDerivedTorque,
        )?;
        Ok(self)
    }

    pub(crate) fn candidate_apply_linear_impulse(
        mut self,
        impulse: Vec2,
        point: Vec2,
        wake_policy: WakePolicy,
    ) -> Result<Self, BodyControlError> {
        if !self.prepare_dynamic_application(wake_policy) {
            return Ok(self);
        }
        validate_linear_impulse(impulse)?;
        validate_point(point)?;
        let linear_velocity = impulse_linear_velocity(self, impulse)?;
        let offset_x = checked(
            point.x - self.sweep.center().x,
            BodyControlError::NonFiniteDerivedAngularVelocity,
        )?;
        let offset_y = checked(
            point.y - self.sweep.center().y,
            BodyControlError::NonFiniteDerivedAngularVelocity,
        )?;
        let angular_delta = checked(
            self.inverse_inertia * (offset_x * impulse.y - offset_y * impulse.x),
            BodyControlError::NonFiniteDerivedAngularVelocity,
        )?;
        let angular_velocity = checked(
            self.angular_velocity + angular_delta,
            BodyControlError::NonFiniteDerivedAngularVelocity,
        )?;
        self.linear_velocity = linear_velocity;
        self.angular_velocity = angular_velocity;
        self.snapshot.linear_velocity = linear_velocity;
        self.snapshot.angular_velocity = angular_velocity;
        Ok(self)
    }

    pub(crate) fn candidate_apply_linear_impulse_to_center(
        mut self,
        impulse: Vec2,
        wake_policy: WakePolicy,
    ) -> Result<Self, BodyControlError> {
        if !self.prepare_dynamic_application(wake_policy) {
            return Ok(self);
        }
        validate_linear_impulse(impulse)?;
        let linear_velocity = impulse_linear_velocity(self, impulse)?;
        self.linear_velocity = linear_velocity;
        self.snapshot.linear_velocity = linear_velocity;
        Ok(self)
    }

    pub(crate) fn candidate_apply_angular_impulse(
        mut self,
        impulse: f32,
        wake_policy: WakePolicy,
    ) -> Result<Self, BodyControlError> {
        if !self.prepare_dynamic_application(wake_policy) {
            return Ok(self);
        }
        validate_angular_impulse(impulse)?;
        let delta = checked(
            self.inverse_inertia * impulse,
            BodyControlError::NonFiniteDerivedAngularVelocity,
        )?;
        let angular_velocity = checked(
            self.angular_velocity + delta,
            BodyControlError::NonFiniteDerivedAngularVelocity,
        )?;
        self.angular_velocity = angular_velocity;
        self.snapshot.angular_velocity = angular_velocity;
        Ok(self)
    }

    pub(crate) fn candidate_set_awake(mut self, awake: bool) -> Self {
        if awake {
            self.wake();
        } else {
            self.snapshot.flags.set(BodyFlags::AWAKE, false);
            self.sleep_time = 0.0;
            self.linear_velocity = Vec2::ZERO;
            self.angular_velocity = 0.0;
            self.snapshot.linear_velocity = Vec2::ZERO;
            self.snapshot.angular_velocity = 0.0;
            self.force = Vec2::ZERO;
            self.torque = 0.0;
        }
        self
    }

    pub(crate) fn candidate_set_sleeping_allowed(mut self, allowed: bool) -> Self {
        if allowed {
            self.snapshot.flags.set(BodyFlags::SLEEPING_ALLOWED, true);
        } else {
            self.snapshot.flags.set(BodyFlags::SLEEPING_ALLOWED, false);
            self.wake();
        }
        self
    }

    pub(crate) fn candidate_set_linear_damping(
        mut self,
        damping: f32,
    ) -> Result<Self, BodyControlError> {
        validate_linear_damping(damping)?;
        self.snapshot.linear_damping = damping;
        Ok(self)
    }

    pub(crate) fn candidate_set_angular_damping(
        mut self,
        damping: f32,
    ) -> Result<Self, BodyControlError> {
        validate_angular_damping(damping)?;
        self.snapshot.angular_damping = damping;
        Ok(self)
    }

    pub(crate) fn candidate_set_gravity_scale(
        mut self,
        gravity_scale: f32,
    ) -> Result<Self, BodyControlError> {
        validate_gravity_scale(gravity_scale)?;
        self.snapshot.gravity_scale = gravity_scale;
        Ok(self)
    }

    pub(crate) fn candidate_set_bullet(mut self, bullet: bool) -> Self {
        self.snapshot.flags.set(BodyFlags::BULLET, bullet);
        self
    }

    pub(crate) fn candidate_set_fixed_rotation(
        mut self,
        fixed_rotation: bool,
        fixture_mass_data: &[MassData],
    ) -> Result<Self, BodyControlError> {
        if self.snapshot.is_fixed_rotation() == fixed_rotation {
            return Ok(self);
        }
        self.snapshot
            .flags
            .set(BodyFlags::FIXED_ROTATION, fixed_rotation);
        self.angular_velocity = 0.0;
        self.snapshot.angular_velocity = 0.0;
        self.with_reset_mass_data(fixture_mass_data)
            .map_err(BodyControlError::from)
    }

    fn prepare_dynamic_application(&mut self, wake_policy: WakePolicy) -> bool {
        if self.snapshot.body_type != BodyType::Dynamic {
            return false;
        }
        if wake_policy == WakePolicy::Wake {
            self.wake();
        }
        self.snapshot.is_awake()
    }

    fn wake(&mut self) {
        if !self.snapshot.is_awake() {
            self.snapshot.flags.set(BodyFlags::AWAKE, true);
            self.sleep_time = 0.0;
        }
    }
}

fn impulse_linear_velocity(state: BodyState, impulse: Vec2) -> Result<Vec2, BodyControlError> {
    Ok(Vec2::new(
        checked(
            state.linear_velocity.x + state.inverse_mass * impulse.x,
            BodyControlError::NonFiniteDerivedLinearVelocityX,
        )?,
        checked(
            state.linear_velocity.y + state.inverse_mass * impulse.y,
            BodyControlError::NonFiniteDerivedLinearVelocityY,
        )?,
    ))
}

fn validate_linear_velocity(velocity: Vec2) -> Result<(), BodyControlError> {
    if !velocity.x.is_finite() {
        return Err(BodyControlError::NonFiniteLinearVelocityX);
    }
    if !velocity.y.is_finite() {
        return Err(BodyControlError::NonFiniteLinearVelocityY);
    }
    Ok(())
}

fn validate_angular_velocity(angular_velocity: f32) -> Result<(), BodyControlError> {
    if !angular_velocity.is_finite() {
        return Err(BodyControlError::NonFiniteAngularVelocity);
    }
    Ok(())
}

fn validate_force(force: Vec2) -> Result<(), BodyControlError> {
    if !force.x.is_finite() {
        return Err(BodyControlError::NonFiniteForceX);
    }
    if !force.y.is_finite() {
        return Err(BodyControlError::NonFiniteForceY);
    }
    Ok(())
}

fn validate_point(point: Vec2) -> Result<(), BodyControlError> {
    if !point.x.is_finite() {
        return Err(BodyControlError::NonFinitePointX);
    }
    if !point.y.is_finite() {
        return Err(BodyControlError::NonFinitePointY);
    }
    Ok(())
}

fn validate_torque(torque: f32) -> Result<(), BodyControlError> {
    if !torque.is_finite() {
        return Err(BodyControlError::NonFiniteTorque);
    }
    Ok(())
}

fn validate_linear_impulse(impulse: Vec2) -> Result<(), BodyControlError> {
    if !impulse.x.is_finite() {
        return Err(BodyControlError::NonFiniteLinearImpulseX);
    }
    if !impulse.y.is_finite() {
        return Err(BodyControlError::NonFiniteLinearImpulseY);
    }
    Ok(())
}

fn validate_angular_impulse(impulse: f32) -> Result<(), BodyControlError> {
    if !impulse.is_finite() {
        return Err(BodyControlError::NonFiniteAngularImpulse);
    }
    Ok(())
}

fn validate_linear_damping(damping: f32) -> Result<(), BodyControlError> {
    if !damping.is_finite() {
        return Err(BodyControlError::NonFiniteLinearDamping);
    }
    if damping < 0.0 {
        return Err(BodyControlError::NegativeLinearDamping);
    }
    Ok(())
}

fn validate_angular_damping(damping: f32) -> Result<(), BodyControlError> {
    if !damping.is_finite() {
        return Err(BodyControlError::NonFiniteAngularDamping);
    }
    if damping < 0.0 {
        return Err(BodyControlError::NegativeAngularDamping);
    }
    Ok(())
}

fn validate_gravity_scale(gravity_scale: f32) -> Result<(), BodyControlError> {
    if !gravity_scale.is_finite() {
        return Err(BodyControlError::NonFiniteGravityScale);
    }
    Ok(())
}

fn checked(value: f32, error: BodyControlError) -> Result<f32, BodyControlError> {
    if !value.is_finite() {
        return Err(error);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use crate::collision::MassData;
    use crate::math::Vec2;

    use super::{BodyState, BodyType, WakePolicy};
    use crate::world::BodyDef;

    fn body_state(body_type: BodyType, awake: bool) -> BodyState {
        let definition = BodyDef::new(body_type, Vec2::ZERO, 0.0, true)
            .expect("finite definition should be accepted")
            .with_awake(awake);
        BodyState::from_definition(&definition)
    }

    #[test]
    fn non_dynamic_force_and_impulse_calls_are_successful_no_effects() {
        // Arrange
        let static_body = body_state(BodyType::Static, false);
        let kinematic_body = body_state(BodyType::Kinematic, false);

        // Act
        let static_candidate = static_body
            .candidate_apply_force(
                Vec2::new(f32::NAN, 0.0),
                Vec2::new(f32::INFINITY, 0.0),
                WakePolicy::Wake,
            )
            .expect("static force application should be ignored");
        let kinematic_candidate = kinematic_body
            .candidate_apply_linear_impulse_to_center(Vec2::new(f32::NAN, 0.0), WakePolicy::Wake)
            .expect("kinematic impulse application should be ignored");

        // Assert
        assert_eq!(static_candidate.snapshot(), static_body.snapshot());
        assert_eq!(kinematic_candidate.snapshot(), kinematic_body.snapshot());
    }

    #[test]
    fn preserve_sleep_application_is_a_successful_no_effect() {
        // Arrange
        let body = body_state(BodyType::Dynamic, false);

        // Act
        let candidate = body
            .candidate_apply_torque(f32::NAN, WakePolicy::PreserveSleep)
            .expect("preserved sleeping application should be ignored");

        // Assert
        assert_eq!(candidate.snapshot(), body.snapshot());
        assert_eq!(candidate.force, body.force);
        assert_eq!(candidate.torque.to_bits(), body.torque.to_bits());
    }

    #[test]
    fn wake_policy_wakes_before_a_valid_application() {
        // Arrange
        let body = body_state(BodyType::Dynamic, false);

        // Act
        let candidate = body
            .candidate_apply_force_to_center(Vec2::new(1.0, 0.0), WakePolicy::Wake)
            .expect("finite force should be accepted");

        // Assert
        assert!(candidate.snapshot().is_awake());
        assert_eq!(candidate.force, Vec2::new(1.0, 0.0));
    }

    #[test]
    fn nonzero_velocity_wakes_while_zero_velocity_preserves_sleep() {
        // Arrange
        let body = body_state(BodyType::Dynamic, false);

        // Act
        let zero = body
            .candidate_set_linear_velocity(Vec2::ZERO)
            .expect("zero velocity should be accepted");
        let nonzero = body
            .candidate_set_angular_velocity(-1.0)
            .expect("finite angular velocity should be accepted");

        // Assert
        assert!(!zero.snapshot().is_awake());
        assert!(nonzero.snapshot().is_awake());
    }

    #[test]
    fn sleeping_clears_motion_force_and_torque() {
        // Arrange
        let definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
            .expect("finite definition should be accepted")
            .with_linear_velocity(Vec2::new(2.0, -3.0))
            .expect("finite velocity should be accepted")
            .with_angular_velocity(4.0)
            .expect("finite angular velocity should be accepted");
        let body = BodyState::from_definition(&definition)
            .candidate_apply_force_to_center(Vec2::new(5.0, 6.0), WakePolicy::Wake)
            .expect("finite force should be accepted")
            .candidate_apply_torque(7.0, WakePolicy::Wake)
            .expect("finite torque should be accepted");

        // Act
        let sleeping = body.candidate_set_awake(false);

        // Assert
        assert!(!sleeping.snapshot().is_awake());
        assert_eq!(sleeping.snapshot().linear_velocity(), Vec2::ZERO);
        assert_eq!(
            sleeping.snapshot().angular_velocity().to_bits(),
            0.0_f32.to_bits()
        );
        assert_eq!(sleeping.force, Vec2::ZERO);
        assert_eq!(sleeping.torque.to_bits(), 0.0_f32.to_bits());
    }

    #[test]
    fn disabling_sleep_wakes_an_asleep_body() {
        // Arrange
        let body = body_state(BodyType::Dynamic, false);

        // Act
        let candidate = body.candidate_set_sleeping_allowed(false);

        // Assert
        assert!(!candidate.snapshot().is_sleeping_allowed());
        assert!(candidate.snapshot().is_awake());
    }

    #[test]
    fn passive_controls_preserve_sleep() {
        // Arrange
        let body = body_state(BodyType::Dynamic, false);

        // Act
        let candidate = body
            .candidate_set_linear_damping(0.25)
            .expect("finite damping should be accepted")
            .candidate_set_angular_damping(0.5)
            .expect("finite damping should be accepted")
            .candidate_set_gravity_scale(-1.0)
            .expect("finite gravity scale should be accepted")
            .candidate_set_bullet(true);

        // Assert
        assert!(!candidate.snapshot().is_awake());
        assert_eq!(
            candidate.snapshot().linear_damping().to_bits(),
            0.25_f32.to_bits()
        );
        assert_eq!(
            candidate.snapshot().angular_damping().to_bits(),
            0.5_f32.to_bits()
        );
        assert_eq!(
            candidate.snapshot().gravity_scale().to_bits(),
            (-1.0_f32).to_bits()
        );
        assert!(candidate.snapshot().is_bullet());
    }

    #[test]
    fn fixed_rotation_clears_angular_velocity_and_recomputes_inertia() {
        // Arrange
        let definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
            .expect("finite definition should be accepted")
            .with_angular_velocity(3.0)
            .expect("finite angular velocity should be accepted");
        let mass_data = [MassData::new(2.0, Vec2::ZERO, 4.0)
            .expect("finite positive mass data should be accepted")];
        let body = BodyState::from_definition(&definition)
            .with_reset_mass_data(&mass_data)
            .expect("valid mass data should aggregate");

        // Act
        let fixed = body
            .candidate_set_fixed_rotation(true, &mass_data)
            .expect("fixed rotation should recompute valid mass data");
        let free = fixed
            .candidate_set_fixed_rotation(false, &mass_data)
            .expect("free rotation should recompute valid mass data");

        // Assert
        assert!(fixed.snapshot().is_fixed_rotation());
        assert_eq!(
            fixed.snapshot().angular_velocity().to_bits(),
            0.0_f32.to_bits()
        );
        assert_eq!(
            fixed.snapshot().rotational_inertia().to_bits(),
            0.0_f32.to_bits()
        );
        assert_eq!(fixed.inverse_inertia().to_bits(), 0.0_f32.to_bits());
        assert!(!free.snapshot().is_fixed_rotation());
        assert_eq!(
            free.snapshot().rotational_inertia().to_bits(),
            4.0_f32.to_bits()
        );
        assert_eq!(free.inverse_inertia().to_bits(), 0.25_f32.to_bits());
    }

    #[test]
    fn derived_overflow_returns_error_without_a_candidate() {
        // Arrange
        let body = body_state(BodyType::Dynamic, true)
            .candidate_set_linear_velocity(Vec2::new(f32::MAX, 0.0))
            .expect("finite velocity should be accepted");

        // Act
        let maybe_candidate = body
            .candidate_apply_linear_impulse_to_center(Vec2::new(f32::MAX, 0.0), WakePolicy::Wake);

        // Assert
        assert!(maybe_candidate.is_err());
        assert_eq!(
            body.snapshot().linear_velocity().x.to_bits(),
            f32::MAX.to_bits()
        );
    }
}
