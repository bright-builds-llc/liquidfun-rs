//! Source-ordered capped friction-joint state and checked world authority.

use crate::math::{Mat22, Vec2};
use crate::{
    FrictionJointDef, FrictionJointSnapshot, JointDef, JointId, JointKind, JointMutationError,
    JointQueryError, JointSnapshot, JointSpecificSnapshot,
};

use super::JointRecord;
use crate::world::object::World;

#[derive(Debug, Clone, Copy)]
pub(crate) struct FrictionRuntime {
    linear_impulse: Vec2,
    angular_impulse: f32,
    linear_mass: Mat22,
    angular_mass: f32,
}

impl FrictionRuntime {
    pub(super) const fn new(_definition: FrictionJointDef) -> Self {
        Self {
            linear_impulse: Vec2::ZERO,
            angular_impulse: 0.0,
            linear_mass: Mat22::ZERO,
            angular_mass: 0.0,
        }
    }
    pub(super) fn reaction_force(self, inverse_timestep: f32) -> Vec2 {
        inverse_timestep * self.linear_impulse
    }
    pub(super) fn reaction_torque(self, inverse_timestep: f32) -> f32 {
        inverse_timestep * self.angular_impulse
    }

    #[allow(dead_code, reason = "used by the mixed-island integration plan")]
    pub(super) fn initialize(
        &mut self,
        linear_mass: Mat22,
        inverse_inertia_sum: f32,
        maybe_warm_start_ratio: Option<f32>,
    ) -> Result<(), JointMutationError> {
        let previous = *self;
        if !matrix_is_valid(linear_mass)
            || !inverse_inertia_sum.is_finite()
            || inverse_inertia_sum < 0.0
        {
            return Err(JointMutationError::InvalidValue);
        }
        self.linear_mass = linear_mass.inverse();
        self.angular_mass = if inverse_inertia_sum > 0.0 {
            1.0 / inverse_inertia_sum
        } else {
            0.0
        };
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
        definition: FrictionJointDef,
        timestep: f32,
        relative_speed: f32,
    ) -> Result<f32, JointMutationError> {
        if !timestep.is_finite() || timestep < 0.0 || !relative_speed.is_finite() {
            return Err(JointMutationError::InvalidValue);
        }
        let impulse = -self.angular_mass * relative_speed;
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
        definition: FrictionJointDef,
        timestep: f32,
        relative_velocity: Vec2,
    ) -> Result<Vec2, JointMutationError> {
        if !timestep.is_finite() || timestep < 0.0 || !relative_velocity.is_valid() {
            return Err(JointMutationError::InvalidValue);
        }
        let old = self.linear_impulse;
        let mut candidate = old - self.linear_mass.apply(relative_velocity);
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
    _runtime: FrictionRuntime,
) -> Result<JointSnapshot, JointQueryError> {
    let JointDef::Friction(definition) = record.definition else {
        return Err(wrong_kind(record.definition));
    };
    let body_a = world.bodies.get(record.bodies[0])?.state.snapshot();
    let body_b = world.bodies.get(record.bodies[1])?.state.snapshot();
    let anchor_a = body_a.transform().apply(definition.local_anchor_a());
    let anchor_b = body_b.transform().apply(definition.local_anchor_b());
    if !anchor_a.is_valid() || !anchor_b.is_valid() {
        return Err(JointQueryError::NonFiniteDerivedState);
    }
    Ok(
        JointSnapshot::from_definition(record.definition).with_runtime(
            anchor_a,
            anchor_b,
            JointSpecificSnapshot::Friction(FrictionJointSnapshot::new(
                definition.max_force(),
                definition.max_torque(),
            )),
        ),
    )
}

impl World {
    /// Sets the friction force cap without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid value, identity, kind, lock, or poison.
    pub fn set_friction_max_force(
        &mut self,
        joint: JointId,
        value: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_friction(joint, |definition| definition.with_max_force(value))
    }
    /// Sets the friction torque cap without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid value, identity, kind, lock, or poison.
    pub fn set_friction_max_torque(
        &mut self,
        joint: JointId,
        value: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_friction(joint, |definition| definition.with_max_torque(value))
    }
    fn mutate_friction(
        &mut self,
        joint: JointId,
        mutate: impl FnOnce(FrictionJointDef) -> Result<FrictionJointDef, crate::JointDefError>,
    ) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = friction_record(self, joint).map_err(as_mutation_error)?;
        let JointDef::Friction(definition) = record.definition else {
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
fn friction_record(world: &World, joint: JointId) -> Result<&JointRecord, JointQueryError> {
    let record = world.joints.get(joint)?;
    if !matches!(record.definition, JointDef::Friction(_)) {
        return Err(wrong_kind(record.definition));
    }
    Ok(record)
}
fn wrong_kind(definition: JointDef) -> JointQueryError {
    JointQueryError::WrongKind {
        expected: JointKind::Friction,
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
    fn definition() -> FrictionJointDef {
        let mut world = World::new().expect("world");
        let a = world.create_body(&BodyDef::default()).expect("A");
        let b = world.create_body(&BodyDef::default()).expect("B");
        FrictionJointDef::new(a, b)
            .expect("definition")
            .with_max_force(4.0)
            .expect("force")
            .with_max_torque(2.0)
            .expect("torque")
    }
    #[test]
    fn linear_and_angular_impulses_are_capped() {
        // Arrange
        let definition = definition();
        let mut runtime = FrictionRuntime::new(definition);
        runtime
            .initialize(Mat22::IDENTITY, 1.0, None)
            .expect("initialize");
        // Act
        let linear = runtime
            .solve_linear(definition, 0.25, Vec2::new(100.0, 0.0))
            .expect("linear");
        let angular = runtime
            .solve_angular(definition, 0.25, 100.0)
            .expect("angular");
        // Assert
        assert_eq!(linear.length().to_bits(), 1.0_f32.to_bits());
        assert_eq!(angular.to_bits(), (-0.5_f32).to_bits());
    }
}
