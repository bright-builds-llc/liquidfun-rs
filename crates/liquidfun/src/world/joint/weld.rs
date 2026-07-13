//! Source-ordered weld-joint state and checked world authority.

use std::f32::consts::TAU;

use crate::math::{Mat33, Vec2, Vec3};
use crate::{
    JointDef, JointId, JointKind, JointMutationError, JointQueryError, JointSnapshot,
    JointSpecificSnapshot, WeldJointDef, WeldJointSnapshot,
};

use super::JointRecord;
use crate::world::object::World;

#[derive(Debug, Clone, Copy)]
pub(crate) struct WeldRuntime {
    impulse: Vec3,
    gamma: f32,
    bias: f32,
    effective_mass: Mat33,
}

impl WeldRuntime {
    pub(super) const fn new(_definition: WeldJointDef) -> Self {
        Self {
            impulse: Vec3::ZERO,
            gamma: 0.0,
            bias: 0.0,
            effective_mass: Mat33::ZERO,
        }
    }

    pub(super) fn reaction_force(self, inverse_timestep: f32) -> Vec2 {
        inverse_timestep * Vec2::new(self.impulse.x, self.impulse.y)
    }

    pub(super) fn reaction_torque(self, inverse_timestep: f32) -> f32 {
        inverse_timestep * self.impulse.z
    }

    #[allow(dead_code, reason = "used by the mixed-island integration plan")]
    pub(super) fn initialize(
        &mut self,
        definition: WeldJointDef,
        timestep: f32,
        angular_error: f32,
        inverse_inertia_sum: f32,
        mass: Mat33,
        maybe_warm_start_ratio: Option<f32>,
    ) -> Result<(), JointMutationError> {
        let previous = *self;
        if !timestep.is_finite()
            || timestep < 0.0
            || !angular_error.is_finite()
            || !inverse_inertia_sum.is_finite()
            || inverse_inertia_sum < 0.0
            || !matrix_is_valid(mass)
        {
            return Err(JointMutationError::InvalidValue);
        }
        if definition.frequency() > 0.0 {
            self.effective_mass = mass.inverse22();
            let rotational_mass = if inverse_inertia_sum > 0.0 {
                1.0 / inverse_inertia_sum
            } else {
                0.0
            };
            let omega = TAU * definition.frequency();
            let damping = 2.0 * rotational_mass * definition.damping_ratio() * omega;
            let stiffness = rotational_mass * omega * omega;
            self.gamma = timestep * (damping + timestep * stiffness);
            self.gamma = if self.gamma == 0.0 {
                0.0
            } else {
                1.0 / self.gamma
            };
            self.bias = angular_error * timestep * stiffness * self.gamma;
            let inverse_angular_mass = inverse_inertia_sum + self.gamma;
            let angular_mass = if inverse_angular_mass == 0.0 {
                0.0
            } else {
                1.0 / inverse_angular_mass
            };
            let first = self.effective_mass.first_column();
            let second = self.effective_mass.second_column();
            self.effective_mass =
                Mat33::from_columns(first, second, Vec3::new(0.0, 0.0, angular_mass));
        } else {
            self.effective_mass = mass.symmetric_inverse33();
            self.gamma = 0.0;
            self.bias = 0.0;
        }
        match maybe_warm_start_ratio {
            Some(ratio) if ratio.is_finite() && ratio >= 0.0 => self.impulse *= ratio,
            Some(_) => {
                *self = previous;
                return Err(JointMutationError::InvalidValue);
            }
            None => self.impulse = Vec3::ZERO,
        }
        if !self.impulse.is_valid()
            || !self.gamma.is_finite()
            || !self.bias.is_finite()
            || !matrix_is_valid(self.effective_mass)
        {
            *self = previous;
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok(())
    }

    #[allow(dead_code, reason = "used by the mixed-island integration plan")]
    pub(super) fn solve_velocity(
        &mut self,
        definition: WeldJointDef,
        linear_error: Vec2,
        angular_error: f32,
    ) -> Result<Vec3, JointMutationError> {
        let previous = *self;
        if !linear_error.is_valid() || !angular_error.is_finite() {
            return Err(JointMutationError::InvalidValue);
        }
        let impulse = if definition.frequency() > 0.0 {
            let angular = -self.effective_mass.third_column().z
                * (angular_error + self.bias + self.gamma * self.impulse.z);
            self.impulse.z += angular;
            let linear = -self.effective_mass.apply22(linear_error);
            self.impulse.x += linear.x;
            self.impulse.y += linear.y;
            Vec3::new(linear.x, linear.y, angular)
        } else {
            let impulse = -self.effective_mass.apply(Vec3::new(
                linear_error.x,
                linear_error.y,
                angular_error,
            ));
            self.impulse += impulse;
            impulse
        };
        if !impulse.is_valid() || !self.impulse.is_valid() {
            *self = previous;
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok(impulse)
    }

    #[allow(dead_code, reason = "used by the mixed-island integration plan")]
    pub(super) fn solve_position(
        definition: WeldJointDef,
        mass: Mat33,
        linear_error: Vec2,
        angular_error: f32,
    ) -> Result<(Vec3, bool), JointMutationError> {
        if !linear_error.is_valid() || !angular_error.is_finite() || !matrix_is_valid(mass) {
            return Err(JointMutationError::InvalidValue);
        }
        let impulse = if definition.frequency() > 0.0 {
            let linear = -mass.solve22(linear_error);
            Vec3::new(linear.x, linear.y, 0.0)
        } else {
            -mass.solve33(Vec3::new(linear_error.x, linear_error.y, angular_error))
        };
        if !impulse.is_valid() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        let angular_residual = if definition.frequency() > 0.0 {
            0.0
        } else {
            angular_error.abs()
        };
        Ok((
            impulse,
            linear_error.length() <= crate::math::settings::LINEAR_SLOP
                && angular_residual <= crate::math::settings::ANGULAR_SLOP,
        ))
    }
}

pub(super) fn snapshot(
    world: &World,
    record: &JointRecord,
    runtime: WeldRuntime,
) -> Result<JointSnapshot, JointQueryError> {
    let JointDef::Weld(definition) = record.definition else {
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
            JointSpecificSnapshot::Weld(WeldJointSnapshot::new(
                definition.reference_angle(),
                definition.frequency(),
                definition.damping_ratio(),
                runtime.gamma,
                runtime.bias,
            )),
        ),
    )
}

impl World {
    /// Sets weld rotational frequency without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid value, identity, kind, lock, or poison.
    pub fn set_weld_frequency(
        &mut self,
        joint: JointId,
        frequency: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_weld(joint, |definition| definition.with_frequency(frequency))
    }

    /// Sets weld rotational damping ratio without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid value, identity, kind, lock, or poison.
    pub fn set_weld_damping_ratio(
        &mut self,
        joint: JointId,
        damping_ratio: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_weld(joint, |definition| {
            definition.with_damping_ratio(damping_ratio)
        })
    }

    fn mutate_weld(
        &mut self,
        joint: JointId,
        mutate: impl FnOnce(WeldJointDef) -> Result<WeldJointDef, crate::JointDefError>,
    ) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = weld_record(self, joint).map_err(as_mutation_error)?;
        let JointDef::Weld(definition) = record.definition else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        let candidate = mutate(definition).map_err(|_| JointMutationError::InvalidValue)?;
        self.joint_mut_after_validation(joint).definition = candidate.into();
        Ok(())
    }
}

fn matrix_is_valid(matrix: Mat33) -> bool {
    matrix.first_column().is_valid()
        && matrix.second_column().is_valid()
        && matrix.third_column().is_valid()
}

fn weld_record(world: &World, joint: JointId) -> Result<&JointRecord, JointQueryError> {
    let record = world.joints.get(joint)?;
    if !matches!(record.definition, JointDef::Weld(_)) {
        return Err(wrong_kind(record.definition));
    }
    Ok(record)
}

fn wrong_kind(definition: JointDef) -> JointQueryError {
    JointQueryError::WrongKind {
        expected: JointKind::Weld,
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

    fn definition() -> WeldJointDef {
        let mut world = World::new().expect("world");
        let body_a = world.create_body(&BodyDef::default()).expect("A");
        let body_b = world.create_body(&BodyDef::default()).expect("B");
        WeldJointDef::new(body_a, body_b).expect("definition")
    }

    #[test]
    fn rigid_and_soft_initialization_choose_distinct_mass_branches() {
        // Arrange
        let rigid = definition();
        let soft = rigid
            .with_frequency(2.0)
            .expect("frequency")
            .with_damping_ratio(0.7)
            .expect("damping");
        let mass = Mat33::IDENTITY;
        let mut rigid_runtime = WeldRuntime::new(rigid);
        let mut soft_runtime = WeldRuntime::new(soft);

        // Act
        rigid_runtime
            .initialize(rigid, 0.5, 1.0, 2.0, mass, None)
            .expect("rigid");
        soft_runtime
            .initialize(soft, 0.5, 1.0, 2.0, mass, None)
            .expect("soft");

        // Assert
        assert_eq!(rigid_runtime.gamma.to_bits(), 0.0_f32.to_bits());
        assert_eq!(rigid_runtime.bias.to_bits(), 0.0_f32.to_bits());
        assert!(soft_runtime.gamma > 0.0);
        assert!(soft_runtime.bias > 0.0);
    }

    #[test]
    fn warm_start_and_velocity_solve_preserve_complete_impulse() {
        // Arrange
        let definition = definition();
        let mut runtime = WeldRuntime::new(definition);
        runtime.impulse = Vec3::new(2.0, 4.0, 6.0);
        runtime
            .initialize(definition, 0.5, 0.0, 2.0, Mat33::IDENTITY, Some(0.5))
            .expect("initialize");

        // Act
        let applied = runtime
            .solve_velocity(definition, Vec2::new(1.0, 2.0), 3.0)
            .expect("solve");

        // Assert
        assert_eq!(applied, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(runtime.impulse, Vec3::ZERO);
    }
}
