//! Source-ordered capped motor-joint state and checked world authority.

use crate::math::{Mat22, Vec2};
use crate::{
    JointDef, JointId, JointKind, JointMutationError, JointQueryError, JointSnapshot,
    JointSpecificSnapshot, MotorJointDef, MotorJointSnapshot,
};

use super::JointRecord;
use crate::world::object::World;

#[derive(Debug, Clone, Copy)]
pub(crate) struct MotorRuntime {
    linear_impulse: Vec2,
    angular_impulse: f32,
    linear_mass: Mat22,
    angular_mass: f32,
    linear_error: Vec2,
    angular_error: f32,
}

impl MotorRuntime {
    pub(super) const fn new(_definition: MotorJointDef) -> Self {
        Self {
            linear_impulse: Vec2::ZERO,
            angular_impulse: 0.0,
            linear_mass: Mat22::ZERO,
            angular_mass: 0.0,
            linear_error: Vec2::ZERO,
            angular_error: 0.0,
        }
    }
    pub(super) fn reaction_force(self, inverse_timestep: f32) -> Vec2 {
        inverse_timestep * self.linear_impulse
    }
    pub(super) fn reaction_torque(self, inverse_timestep: f32) -> f32 {
        inverse_timestep * self.angular_impulse
    }
    pub(super) const fn solver_impulses(self) -> (Vec2, f32) {
        (self.linear_impulse, self.angular_impulse)
    }
    #[allow(dead_code, reason = "used by the mixed-island integration plan")]
    pub(super) fn initialize(
        &mut self,
        linear_mass: Mat22,
        inverse_inertia_sum: f32,
        linear_error: Vec2,
        angular_error: f32,
        maybe_warm_start_ratio: Option<f32>,
    ) -> Result<(), JointMutationError> {
        let previous = *self;
        if !matrix_is_valid(linear_mass)
            || !inverse_inertia_sum.is_finite()
            || inverse_inertia_sum < 0.0
            || !linear_error.is_valid()
            || !angular_error.is_finite()
        {
            return Err(JointMutationError::InvalidValue);
        }
        self.linear_mass = linear_mass.inverse();
        self.angular_mass = if inverse_inertia_sum > 0.0 {
            1.0 / inverse_inertia_sum
        } else {
            0.0
        };
        self.linear_error = linear_error;
        self.angular_error = angular_error;
        match maybe_warm_start_ratio {
            Some(ratio) if ratio.is_finite() && ratio >= 0.0 => {
                self.linear_impulse *= ratio;
                self.angular_impulse *= ratio;
            }
            Some(_) => {
                *self = previous;
                return Err(JointMutationError::InvalidValue);
            }
            None => {
                self.linear_impulse = Vec2::ZERO;
                self.angular_impulse = 0.0;
            }
        }
        if !self.linear_impulse.is_valid()
            || !self.angular_impulse.is_finite()
            || !matrix_is_valid(self.linear_mass)
            || !self.angular_mass.is_finite()
        {
            *self = previous;
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok(())
    }
    #[allow(dead_code, reason = "used by the mixed-island integration plan")]
    pub(super) fn solve_angular(
        &mut self,
        definition: MotorJointDef,
        timestep: f32,
        inverse_timestep: f32,
        relative_speed: f32,
    ) -> Result<f32, JointMutationError> {
        if !timestep.is_finite()
            || timestep < 0.0
            || !inverse_timestep.is_finite()
            || inverse_timestep < 0.0
            || !relative_speed.is_finite()
        {
            return Err(JointMutationError::InvalidValue);
        }
        let speed =
            relative_speed + inverse_timestep * definition.correction_factor() * self.angular_error;
        let impulse = -self.angular_mass * speed;
        let max_impulse = timestep * definition.max_torque();
        let candidate = (self.angular_impulse + impulse).clamp(-max_impulse, max_impulse);
        if !candidate.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        let applied = candidate - self.angular_impulse;
        self.angular_impulse = candidate;
        Ok(applied)
    }
    #[allow(dead_code, reason = "used by the mixed-island integration plan")]
    pub(super) fn solve_linear(
        &mut self,
        definition: MotorJointDef,
        timestep: f32,
        inverse_timestep: f32,
        relative_velocity: Vec2,
    ) -> Result<Vec2, JointMutationError> {
        if !timestep.is_finite()
            || timestep < 0.0
            || !inverse_timestep.is_finite()
            || inverse_timestep < 0.0
            || !relative_velocity.is_valid()
        {
            return Err(JointMutationError::InvalidValue);
        }
        let speed = relative_velocity
            + inverse_timestep * definition.correction_factor() * self.linear_error;
        let old = self.linear_impulse;
        let mut candidate = old - self.linear_mass.apply(speed);
        let max_impulse = timestep * definition.max_force();
        if candidate.length_squared() > max_impulse * max_impulse {
            let length = candidate.normalize();
            if length > 0.0 {
                candidate *= max_impulse;
            }
        }
        let applied = candidate - old;
        if !candidate.is_valid() || !applied.is_valid() || !max_impulse.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        self.linear_impulse = candidate;
        Ok(applied)
    }
}

pub(super) fn snapshot(
    world: &World,
    record: &JointRecord,
    runtime: MotorRuntime,
) -> Result<JointSnapshot, JointQueryError> {
    let JointDef::Motor(definition) = record.definition else {
        return Err(wrong_kind(record.definition));
    };
    let body_a = world.bodies.get(record.bodies[0])?.state.snapshot();
    let body_b = world.bodies.get(record.bodies[1])?.state.snapshot();
    let anchor_a = body_a.position();
    let anchor_b = body_b.position();
    let linear_error = anchor_b
        - anchor_a
        - body_a
            .transform()
            .rotation()
            .apply(definition.linear_offset());
    let angular_error = body_b.angle() - body_a.angle() - definition.angular_offset();
    if !anchor_a.is_valid()
        || !anchor_b.is_valid()
        || !linear_error.is_valid()
        || !angular_error.is_finite()
    {
        return Err(JointQueryError::NonFiniteDerivedState);
    }
    let selected_linear_error = if runtime.linear_error == Vec2::ZERO {
        linear_error
    } else {
        runtime.linear_error
    };
    let selected_angular_error = if runtime.angular_error == 0.0 {
        angular_error
    } else {
        runtime.angular_error
    };
    Ok(
        JointSnapshot::from_definition(record.definition, record.bodies).with_runtime(
            anchor_a,
            anchor_b,
            JointSpecificSnapshot::Motor(MotorJointSnapshot::new(
                definition.linear_offset(),
                definition.angular_offset(),
                definition.max_force(),
                definition.max_torque(),
                definition.correction_factor(),
                selected_linear_error,
                selected_angular_error,
            )),
        ),
    )
}

impl World {
    /// Sets the linear offset, waking both bodies only when changed exactly.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid value, identity, kind, lock, or poison.
    pub fn set_motor_linear_offset(
        &mut self,
        joint: JointId,
        value: Vec2,
    ) -> Result<(), JointMutationError> {
        self.mutate_motor_offset(
            joint,
            |definition| definition.with_offsets(value, definition.angular_offset()),
            |definition| definition.linear_offset() != value,
        )
    }
    /// Sets the angular offset, waking both bodies only when changed exactly.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid value, identity, kind, lock, or poison.
    #[allow(
        clippy::float_cmp,
        reason = "the pinned setter uses exact changed-only comparison"
    )]
    pub fn set_motor_angular_offset(
        &mut self,
        joint: JointId,
        value: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_motor_offset(
            joint,
            |definition| definition.with_offsets(definition.linear_offset(), value),
            |definition| definition.angular_offset() != value,
        )
    }
    /// Sets maximum force without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid value, identity, kind, lock, or poison.
    pub fn set_motor_max_force(
        &mut self,
        joint: JointId,
        value: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_motor(joint, |definition| {
            definition.with_caps(value, definition.max_torque())
        })
    }
    /// Sets maximum torque without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid value, identity, kind, lock, or poison.
    pub fn set_motor_max_torque(
        &mut self,
        joint: JointId,
        value: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_motor(joint, |definition| {
            definition.with_caps(definition.max_force(), value)
        })
    }
    /// Sets correction factor without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid value, identity, kind, lock, or poison.
    pub fn set_motor_correction_factor(
        &mut self,
        joint: JointId,
        value: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_motor(joint, |definition| definition.with_correction_factor(value))
    }
    fn mutate_motor_offset(
        &mut self,
        joint: JointId,
        mutate: impl FnOnce(MotorJointDef) -> Result<MotorJointDef, crate::JointDefError>,
        changed: impl FnOnce(MotorJointDef) -> bool,
    ) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = motor_record(self, joint).map_err(as_mutation_error)?;
        let JointDef::Motor(definition) = record.definition else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        let candidate = mutate(definition).map_err(|_| JointMutationError::InvalidValue)?;
        if !changed(definition) {
            return Ok(());
        }
        let bodies = record.bodies;
        self.wake_joint_bodies(bodies);
        self.joint_mut_after_validation(joint).definition = candidate.into();
        Ok(())
    }
    fn mutate_motor(
        &mut self,
        joint: JointId,
        mutate: impl FnOnce(MotorJointDef) -> Result<MotorJointDef, crate::JointDefError>,
    ) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = motor_record(self, joint).map_err(as_mutation_error)?;
        let JointDef::Motor(definition) = record.definition else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        let candidate = mutate(definition).map_err(|_| JointMutationError::InvalidValue)?;
        self.joint_mut_after_validation(joint).definition = candidate.into();
        Ok(())
    }
}

fn matrix_is_valid(matrix: Mat22) -> bool {
    matrix.first_column().is_valid() && matrix.second_column().is_valid()
}
fn motor_record(world: &World, joint: JointId) -> Result<&JointRecord, JointQueryError> {
    let record = world.joints.get(joint)?;
    if !matches!(record.definition, JointDef::Motor(_)) {
        return Err(wrong_kind(record.definition));
    }
    Ok(record)
}
fn wrong_kind(definition: JointDef) -> JointQueryError {
    JointQueryError::WrongKind {
        expected: JointKind::Motor,
        actual: JointKind::from_definition(definition),
    }
}
fn as_mutation_error(error: JointQueryError) -> JointMutationError {
    match error {
        JointQueryError::InvalidHandle(error) => JointMutationError::InvalidHandle(error),
        JointQueryError::WrongKind { expected, actual } => {
            JointMutationError::WrongKind { expected, actual }
        }
        JointQueryError::Poisoned => JointMutationError::Poisoned,
        JointQueryError::InvalidInverseTimestep | JointQueryError::NonFiniteDerivedState => {
            JointMutationError::InvalidValue
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BodyDef, World};
    fn definition() -> MotorJointDef {
        let mut world = World::new().expect("world");
        let a = world.create_body(&BodyDef::default()).expect("A");
        let b = world.create_body(&BodyDef::default()).expect("B");
        MotorJointDef::new(a, b)
            .expect("definition")
            .with_caps(4.0, 2.0)
            .expect("caps")
            .with_correction_factor(0.5)
            .expect("correction")
    }
    #[test]
    fn correction_terms_and_caps_follow_source_order() {
        // Arrange
        let definition = definition();
        let mut runtime = MotorRuntime::new(definition);
        runtime
            .initialize(Mat22::IDENTITY, 1.0, Vec2::new(2.0, 0.0), 2.0, None)
            .expect("initialize");
        // Act
        let angular = runtime
            .solve_angular(definition, 0.25, 4.0, 0.0)
            .expect("angular");
        let linear = runtime
            .solve_linear(definition, 0.25, 4.0, Vec2::ZERO)
            .expect("linear");
        // Assert
        assert_eq!(angular.to_bits(), (-0.5_f32).to_bits());
        assert_eq!(linear.length().to_bits(), 1.0_f32.to_bits());
    }
}
