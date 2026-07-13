//! Source-ordered revolute-joint state and checked world authority.

use crate::math::settings::ANGULAR_SLOP;
use crate::math::{Mat33, Vec2, Vec3};
use crate::{
    JointDef, JointId, JointKind, JointLimitState, JointMutationError, JointQueryError,
    JointSnapshot, JointSpecificSnapshot, RevoluteJointDef, RevoluteJointSnapshot,
};

use super::{JointRecord, JointRuntime};
use crate::world::object::World;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RevoluteRuntime {
    impulse: Vec3,
    motor_impulse: f32,
    limit_state: JointLimitState,
}

impl RevoluteRuntime {
    pub(super) const fn new(_definition: RevoluteJointDef) -> Self {
        Self {
            impulse: Vec3::ZERO,
            motor_impulse: 0.0,
            limit_state: JointLimitState::Inactive,
        }
    }

    pub(super) fn reaction_force(self, inverse_timestep: f32) -> Vec2 {
        inverse_timestep * Vec2::new(self.impulse.x, self.impulse.y)
    }

    pub(super) fn reaction_torque(self, inverse_timestep: f32) -> f32 {
        inverse_timestep * self.impulse.z
    }

    #[allow(
        dead_code,
        reason = "used by the Phase 8 mixed-island integration plan"
    )]
    pub(super) fn initialize(
        &mut self,
        definition: RevoluteJointDef,
        angle: f32,
        warm_start_ratio: Option<f32>,
        fixed_rotation: bool,
    ) -> Result<(), JointMutationError> {
        let previous = *self;
        let candidate_state = classify_limit(definition, angle, fixed_rotation);
        if candidate_state != self.limit_state
            && matches!(
                candidate_state,
                JointLimitState::AtLower | JointLimitState::AtUpper
            )
        {
            self.impulse.z = 0.0;
        }
        self.limit_state = candidate_state;
        if !definition.is_motor_enabled() || fixed_rotation {
            self.motor_impulse = 0.0;
        }
        if let Some(ratio) = warm_start_ratio {
            if !ratio.is_finite() || ratio < 0.0 {
                return Err(JointMutationError::InvalidValue);
            }
            self.impulse *= ratio;
            self.motor_impulse *= ratio;
        } else {
            self.impulse = Vec3::ZERO;
            self.motor_impulse = 0.0;
        }
        if !self.impulse.is_valid() || !self.motor_impulse.is_finite() {
            *self = previous;
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok(())
    }

    #[allow(
        dead_code,
        reason = "used by the Phase 8 mixed-island integration plan"
    )]
    pub(super) fn solve_motor(
        &mut self,
        definition: RevoluteJointDef,
        timestep: f32,
        relative_speed: f32,
        motor_mass: f32,
    ) -> Result<f32, JointMutationError> {
        if !timestep.is_finite()
            || timestep < 0.0
            || !relative_speed.is_finite()
            || !motor_mass.is_finite()
            || motor_mass < 0.0
        {
            return Err(JointMutationError::InvalidValue);
        }
        if !definition.is_motor_enabled() || self.limit_state == JointLimitState::Equal {
            return Ok(0.0);
        }
        let impulse = -motor_mass * (relative_speed - definition.motor_speed());
        let max_impulse = timestep * definition.max_motor_torque();
        let candidate = (self.motor_impulse + impulse).clamp(-max_impulse, max_impulse);
        if !candidate.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        let applied = candidate - self.motor_impulse;
        self.motor_impulse = candidate;
        Ok(applied)
    }

    #[allow(
        dead_code,
        reason = "used by the Phase 8 mixed-island integration plan"
    )]
    pub(super) fn solve_constraint_velocity(
        &mut self,
        mass: Mat33,
        linear_error: Vec2,
        angular_error: f32,
        limit_enabled: bool,
        fixed_rotation: bool,
    ) -> Result<Vec3, JointMutationError> {
        let previous_runtime = *self;
        if !linear_error.is_valid() || !angular_error.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        let mut impulse;
        if limit_enabled && self.limit_state != JointLimitState::Inactive && !fixed_rotation {
            impulse = -mass.solve33(Vec3::new(linear_error.x, linear_error.y, angular_error));
            match self.limit_state {
                JointLimitState::AtLower if self.impulse.z + impulse.z < 0.0 => {
                    let column = mass.third_column();
                    let right = -linear_error + self.impulse.z * Vec2::new(column.x, column.y);
                    let reduced = mass.solve22(right);
                    impulse = Vec3::new(reduced.x, reduced.y, -self.impulse.z);
                    self.impulse.x += reduced.x;
                    self.impulse.y += reduced.y;
                    self.impulse.z = 0.0;
                }
                JointLimitState::AtUpper if self.impulse.z + impulse.z > 0.0 => {
                    let column = mass.third_column();
                    let right = -linear_error + self.impulse.z * Vec2::new(column.x, column.y);
                    let reduced = mass.solve22(right);
                    impulse = Vec3::new(reduced.x, reduced.y, -self.impulse.z);
                    self.impulse.x += reduced.x;
                    self.impulse.y += reduced.y;
                    self.impulse.z = 0.0;
                }
                JointLimitState::Equal | JointLimitState::AtLower | JointLimitState::AtUpper => {
                    self.impulse += impulse;
                }
                JointLimitState::Inactive => {}
            }
        } else {
            let reduced = mass.solve22(-linear_error);
            impulse = Vec3::new(reduced.x, reduced.y, 0.0);
            self.impulse.x += reduced.x;
            self.impulse.y += reduced.y;
        }
        if !impulse.is_valid() || !self.impulse.is_valid() {
            *self = previous_runtime;
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok(impulse)
    }
}

pub(super) fn snapshot(
    world: &World,
    record: &JointRecord,
    runtime: RevoluteRuntime,
) -> Result<JointSnapshot, JointQueryError> {
    let JointDef::Revolute(definition) = record.definition else {
        return Err(JointQueryError::WrongKind {
            expected: JointKind::Revolute,
            actual: JointKind::from_definition(record.definition),
        });
    };
    let body_a = world.bodies.get(record.bodies[0])?.state.snapshot();
    let body_b = world.bodies.get(record.bodies[1])?.state.snapshot();
    let anchor_a = body_a.transform().apply(definition.local_anchor_a());
    let anchor_b = body_b.transform().apply(definition.local_anchor_b());
    let angle = body_b.angle() - body_a.angle() - definition.reference_angle();
    let speed = body_b.angular_velocity() - body_a.angular_velocity();
    if !anchor_a.is_valid() || !anchor_b.is_valid() || !angle.is_finite() || !speed.is_finite() {
        return Err(JointQueryError::NonFiniteDerivedState);
    }
    let limit_state = classify_limit(definition, angle, false);
    Ok(
        JointSnapshot::from_definition(record.definition).with_runtime(
            anchor_a,
            anchor_b,
            JointSpecificSnapshot::Revolute(RevoluteJointSnapshot::new(
                angle,
                speed,
                limit_state,
                runtime.motor_impulse,
            )),
        ),
    )
}

impl World {
    /// Returns the current revolute coordinate in radians.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid, poisoned, or wrong-kind joint.
    pub fn revolute_joint_angle(&self, joint: JointId) -> Result<f32, JointQueryError> {
        let snapshot = self.joint_snapshot_of_kind(joint, JointKind::Revolute)?;
        let JointSpecificSnapshot::Revolute(state) = snapshot.specific() else {
            return Err(JointQueryError::WrongKind {
                expected: JointKind::Revolute,
                actual: snapshot.kind(),
            });
        };
        Ok(state.angle())
    }

    /// Returns the current relative angular speed.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid, poisoned, or wrong-kind joint.
    pub fn revolute_joint_speed(&self, joint: JointId) -> Result<f32, JointQueryError> {
        let snapshot = self.joint_snapshot_of_kind(joint, JointKind::Revolute)?;
        let JointSpecificSnapshot::Revolute(state) = snapshot.specific() else {
            return Err(JointQueryError::WrongKind {
                expected: JointKind::Revolute,
                actual: snapshot.kind(),
            });
        };
        Ok(state.speed())
    }

    /// Returns the cached motor torque for an explicit inverse timestep.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid joint or inverse timestep.
    pub fn revolute_motor_torque(
        &self,
        joint: JointId,
        inverse_timestep: f32,
    ) -> Result<f32, JointQueryError> {
        self.validate_reaction_query(joint, inverse_timestep)?;
        let record = revolute_record(self, joint)?;
        let JointRuntime::Revolute(runtime) = record.runtime else {
            return Err(wrong_kind(record.definition));
        };
        Ok(inverse_timestep * runtime.motor_impulse)
    }

    /// Enables or disables the revolute limit, waking only when changed.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid identity, kind, lock, or poison.
    pub fn set_revolute_limit_enabled(
        &mut self,
        joint: JointId,
        enabled: bool,
    ) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = revolute_record(self, joint).map_err(as_mutation_error)?;
        let JointDef::Revolute(definition) = record.definition else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        if definition.is_limit_enabled() == enabled {
            return Ok(());
        }
        let bodies = record.bodies;
        let candidate = definition
            .with_limits(enabled, definition.lower_angle(), definition.upper_angle())
            .map_err(|_| JointMutationError::InvalidValue)?;
        self.wake_joint_bodies(bodies);
        let record = self.joint_mut_after_validation(joint);
        record.definition = candidate.into();
        let JointRuntime::Revolute(runtime) = &mut record.runtime else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        runtime.impulse.z = 0.0;
        runtime.limit_state = JointLimitState::Inactive;
        Ok(())
    }

    /// Sets checked revolute angular limits, waking only when changed.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid limits, identity, kind, lock, or poison.
    #[allow(
        clippy::float_cmp,
        reason = "pinned setters use exact changed-only branches"
    )]
    pub fn set_revolute_limits(
        &mut self,
        joint: JointId,
        lower: f32,
        upper: f32,
    ) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = revolute_record(self, joint).map_err(as_mutation_error)?;
        let JointDef::Revolute(definition) = record.definition else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        let candidate = definition
            .with_limits(definition.is_limit_enabled(), lower, upper)
            .map_err(|_| JointMutationError::InvalidValue)?;
        if definition.lower_angle() == lower && definition.upper_angle() == upper {
            return Ok(());
        }
        let bodies = record.bodies;
        self.wake_joint_bodies(bodies);
        let record = self.joint_mut_after_validation(joint);
        record.definition = candidate.into();
        let JointRuntime::Revolute(runtime) = &mut record.runtime else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        runtime.impulse.z = 0.0;
        runtime.limit_state = JointLimitState::Inactive;
        Ok(())
    }

    /// Enables or disables the motor and unconditionally wakes both bodies.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid identity, kind, lock, or poison.
    pub fn set_revolute_motor_enabled(
        &mut self,
        joint: JointId,
        enabled: bool,
    ) -> Result<(), JointMutationError> {
        self.mutate_revolute_motor(joint, |definition| {
            definition.with_motor(
                enabled,
                definition.motor_speed(),
                definition.max_motor_torque(),
            )
        })
    }

    /// Sets motor speed and unconditionally wakes both bodies.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid speed, identity, kind, lock, or poison.
    pub fn set_revolute_motor_speed(
        &mut self,
        joint: JointId,
        speed: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_revolute_motor(joint, |definition| {
            definition.with_motor(
                definition.is_motor_enabled(),
                speed,
                definition.max_motor_torque(),
            )
        })
    }

    /// Sets maximum motor torque and unconditionally wakes both bodies.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid torque, identity, kind, lock, or poison.
    pub fn set_revolute_max_motor_torque(
        &mut self,
        joint: JointId,
        torque: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_revolute_motor(joint, |definition| {
            definition.with_motor(
                definition.is_motor_enabled(),
                definition.motor_speed(),
                torque,
            )
        })
    }

    fn mutate_revolute_motor(
        &mut self,
        joint: JointId,
        mutate: impl FnOnce(RevoluteJointDef) -> Result<RevoluteJointDef, crate::JointDefError>,
    ) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = revolute_record(self, joint).map_err(as_mutation_error)?;
        let JointDef::Revolute(definition) = record.definition else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        let candidate = mutate(definition).map_err(|_| JointMutationError::InvalidValue)?;
        let bodies = record.bodies;
        self.wake_joint_bodies(bodies);
        self.joint_mut_after_validation(joint).definition = candidate.into();
        Ok(())
    }
}

fn revolute_record(world: &World, joint: JointId) -> Result<&JointRecord, JointQueryError> {
    let record = world.joints.get(joint)?;
    if !matches!(record.definition, JointDef::Revolute(_)) {
        return Err(wrong_kind(record.definition));
    }
    Ok(record)
}

fn wrong_kind(definition: JointDef) -> JointQueryError {
    JointQueryError::WrongKind {
        expected: JointKind::Revolute,
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

fn classify_limit(
    definition: RevoluteJointDef,
    angle: f32,
    fixed_rotation: bool,
) -> JointLimitState {
    if !definition.is_limit_enabled() || fixed_rotation {
        return JointLimitState::Inactive;
    }
    if (definition.upper_angle() - definition.lower_angle()).abs() < 2.0 * ANGULAR_SLOP {
        JointLimitState::Equal
    } else if angle <= definition.lower_angle() {
        JointLimitState::AtLower
    } else if angle >= definition.upper_angle() {
        JointLimitState::AtUpper
    } else {
        JointLimitState::Inactive
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BodyDef, World};

    fn definition() -> RevoluteJointDef {
        let mut world = World::new().expect("world");
        let a = world.create_body(&BodyDef::default()).expect("body a");
        let b = world.create_body(&BodyDef::default()).expect("body b");
        RevoluteJointDef::new(a, b).expect("joint")
    }

    #[test]
    #[allow(clippy::float_cmp, reason = "exact source arithmetic is under test")]
    fn motor_impulse_is_capped_by_timestep_torque() {
        // Arrange
        let definition = definition().with_motor(true, 10.0, 2.0).expect("motor");
        let mut runtime = RevoluteRuntime::new(definition);

        // Act
        let applied = runtime
            .solve_motor(definition, 0.25, 0.0, 1.0)
            .expect("solve");

        // Assert
        assert_eq!(applied, 0.5);
    }

    #[test]
    #[allow(clippy::float_cmp, reason = "exact source arithmetic is under test")]
    fn warm_start_scales_source_impulses() {
        // Arrange
        let definition = definition();
        let mut runtime = RevoluteRuntime {
            impulse: Vec3::new(2.0, 4.0, 6.0),
            motor_impulse: 8.0,
            limit_state: JointLimitState::Inactive,
        };

        // Act
        runtime
            .initialize(definition, 0.0, Some(0.5), false)
            .expect("initialize");

        // Assert
        assert_eq!(runtime.impulse, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(runtime.motor_impulse, 0.0);
    }

    #[test]
    fn lower_limit_block_solver_clamps_negative_accumulation() {
        // Arrange
        let mut runtime = RevoluteRuntime {
            impulse: Vec3::ZERO,
            motor_impulse: 0.0,
            limit_state: JointLimitState::AtLower,
        };

        // Act
        let applied = runtime
            .solve_constraint_velocity(Mat33::IDENTITY, Vec2::ZERO, 1.0, true, false)
            .expect("block solve");

        // Assert
        assert_eq!(applied, Vec3::ZERO);
        assert_eq!(runtime.impulse, Vec3::ZERO);
    }
}
