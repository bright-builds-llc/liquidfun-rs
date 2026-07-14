//! Source-ordered prismatic-joint state and checked world authority.

use crate::math::settings::LINEAR_SLOP;
use crate::math::{Mat33, Vec2, Vec3};
use crate::{
    JointDef, JointId, JointKind, JointLimitState, JointMutationError, JointQueryError,
    JointSnapshot, JointSpecificSnapshot, PrismaticJointDef, PrismaticJointSnapshot,
};

use super::{JointRecord, JointRuntime};
use crate::world::object::World;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PrismaticRuntime {
    impulse: Vec3,
    motor_impulse: f32,
    limit_state: JointLimitState,
    axis: Vec2,
    perpendicular: Vec2,
}

impl PrismaticRuntime {
    pub(super) fn new(definition: PrismaticJointDef) -> Self {
        let axis = definition.local_axis_a();
        Self {
            impulse: Vec3::ZERO,
            motor_impulse: 0.0,
            limit_state: JointLimitState::Inactive,
            axis,
            perpendicular: Vec2::scalar_cross(1.0, axis),
        }
    }

    pub(super) fn reaction_force(self, inverse_timestep: f32) -> Vec2 {
        inverse_timestep
            * (self.impulse.x * self.perpendicular
                + (self.motor_impulse + self.impulse.z) * self.axis)
    }

    pub(super) fn reaction_torque(self, inverse_timestep: f32) -> f32 {
        inverse_timestep * self.impulse.y
    }

    pub(super) const fn solver_impulses(self) -> (Vec3, f32) {
        (self.impulse, self.motor_impulse)
    }

    pub(super) const fn solver_limit_state(self) -> JointLimitState {
        self.limit_state
    }

    #[allow(
        dead_code,
        reason = "used by the Phase 8 mixed-island integration plan"
    )]
    pub(super) fn initialize(
        &mut self,
        definition: PrismaticJointDef,
        translation: f32,
        world_axis: Vec2,
        warm_start_ratio: Option<f32>,
    ) -> Result<(), JointMutationError> {
        let previous = *self;
        if !translation.is_finite() || !world_axis.is_valid() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        let candidate_state = classify_limit(definition, translation);
        match candidate_state {
            JointLimitState::AtLower | JointLimitState::AtUpper
                if candidate_state != self.limit_state =>
            {
                self.impulse.z = 0.0;
            }
            JointLimitState::Inactive => self.impulse.z = 0.0,
            JointLimitState::Equal | JointLimitState::AtLower | JointLimitState::AtUpper => {}
        }
        self.limit_state = candidate_state;
        self.axis = world_axis;
        self.perpendicular = Vec2::scalar_cross(1.0, world_axis);
        if !definition.is_motor_enabled() {
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
        definition: PrismaticJointDef,
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
        let impulse = motor_mass * (definition.motor_speed() - relative_speed);
        let max_impulse = timestep * definition.max_motor_force();
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
        effective_mass: Mat33,
        perpendicular_error: f32,
        angular_error: f32,
        axial_error: f32,
        limit_enabled: bool,
    ) -> Result<Vec3, JointMutationError> {
        let previous_runtime = *self;
        let errors = Vec3::new(perpendicular_error, angular_error, axial_error);
        if !errors.is_valid() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        let previous = self.impulse;
        if limit_enabled && self.limit_state != JointLimitState::Inactive {
            self.impulse += -effective_mass.solve33(errors);
            match self.limit_state {
                JointLimitState::AtLower => self.impulse.z = self.impulse.z.max(0.0),
                JointLimitState::AtUpper => self.impulse.z = self.impulse.z.min(0.0),
                JointLimitState::Equal | JointLimitState::Inactive => {}
            }
            let column = effective_mass.third_column();
            let right = -Vec2::new(perpendicular_error, angular_error)
                - (self.impulse.z - previous.z) * Vec2::new(column.x, column.y);
            let reduced = effective_mass.solve22(right) + Vec2::new(previous.x, previous.y);
            self.impulse.x = reduced.x;
            self.impulse.y = reduced.y;
        } else {
            let reduced = effective_mass.solve22(-Vec2::new(perpendicular_error, angular_error));
            self.impulse.x += reduced.x;
            self.impulse.y += reduced.y;
        }
        let applied = self.impulse - previous;
        if !applied.is_valid() || !self.impulse.is_valid() {
            *self = previous_runtime;
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok(applied)
    }
}

pub(super) fn snapshot(
    world: &World,
    record: &JointRecord,
    runtime: PrismaticRuntime,
) -> Result<JointSnapshot, JointQueryError> {
    let JointDef::Prismatic(definition) = record.definition else {
        return Err(JointQueryError::WrongKind {
            expected: JointKind::Prismatic,
            actual: JointKind::from_definition(record.definition),
        });
    };
    let body_a = world.bodies.get(record.bodies[0])?.state.snapshot();
    let body_b = world.bodies.get(record.bodies[1])?.state.snapshot();
    let transform_a = body_a.transform();
    let transform_b = body_b.transform();
    let anchor_a = transform_a.apply(definition.local_anchor_a());
    let anchor_b = transform_b.apply(definition.local_anchor_b());
    let displacement = anchor_b - anchor_a;
    let axis = transform_a.rotation().apply(definition.local_axis_a());
    let translation = displacement.dot(axis);
    let r_a = transform_a
        .rotation()
        .apply(definition.local_anchor_a() - body_a.local_center());
    let r_b = transform_b
        .rotation()
        .apply(definition.local_anchor_b() - body_b.local_center());
    let speed = displacement.dot(Vec2::scalar_cross(body_a.angular_velocity(), axis))
        + axis.dot(
            body_b.linear_velocity() + Vec2::scalar_cross(body_b.angular_velocity(), r_b)
                - body_a.linear_velocity()
                - Vec2::scalar_cross(body_a.angular_velocity(), r_a),
        );
    if !anchor_a.is_valid()
        || !anchor_b.is_valid()
        || !axis.is_valid()
        || !translation.is_finite()
        || !speed.is_finite()
    {
        return Err(JointQueryError::NonFiniteDerivedState);
    }
    Ok(
        JointSnapshot::from_definition(record.definition, record.bodies).with_runtime(
            anchor_a,
            anchor_b,
            JointSpecificSnapshot::Prismatic(PrismaticJointSnapshot::new(
                translation,
                speed,
                classify_limit(definition, translation),
                runtime.motor_impulse,
            )),
        ),
    )
}

impl World {
    /// Returns the current prismatic translation for gear coordinates.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid, poisoned, or wrong-kind joint.
    pub fn prismatic_joint_translation(&self, joint: JointId) -> Result<f32, JointQueryError> {
        let snapshot = self.joint_snapshot_of_kind(joint, JointKind::Prismatic)?;
        let JointSpecificSnapshot::Prismatic(state) = snapshot.specific() else {
            return Err(JointQueryError::WrongKind {
                expected: JointKind::Prismatic,
                actual: snapshot.kind(),
            });
        };
        Ok(state.translation())
    }

    /// Returns current translation speed along the body-A axis.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid, poisoned, or wrong-kind joint.
    pub fn prismatic_joint_speed(&self, joint: JointId) -> Result<f32, JointQueryError> {
        let snapshot = self.joint_snapshot_of_kind(joint, JointKind::Prismatic)?;
        let JointSpecificSnapshot::Prismatic(state) = snapshot.specific() else {
            return Err(JointQueryError::WrongKind {
                expected: JointKind::Prismatic,
                actual: snapshot.kind(),
            });
        };
        Ok(state.speed())
    }

    /// Returns cached motor force for an explicit inverse timestep.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid joint or inverse timestep.
    pub fn prismatic_motor_force(
        &self,
        joint: JointId,
        inverse_timestep: f32,
    ) -> Result<f32, JointQueryError> {
        self.validate_reaction_query(joint, inverse_timestep)?;
        let record = prismatic_record(self, joint)?;
        let JointRuntime::Prismatic(runtime) = record.runtime else {
            return Err(wrong_kind(record.definition));
        };
        Ok(inverse_timestep * runtime.motor_impulse)
    }

    /// Enables or disables limits, waking only when changed.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid identity, kind, lock, or poison.
    pub fn set_prismatic_limit_enabled(
        &mut self,
        joint: JointId,
        enabled: bool,
    ) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = prismatic_record(self, joint).map_err(as_mutation_error)?;
        let JointDef::Prismatic(definition) = record.definition else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        if definition.is_limit_enabled() == enabled {
            return Ok(());
        }
        let candidate = definition
            .with_limits(
                enabled,
                definition.lower_translation(),
                definition.upper_translation(),
            )
            .map_err(|_| JointMutationError::InvalidValue)?;
        let bodies = record.bodies;
        self.wake_joint_bodies(bodies);
        let record = self.joint_mut_after_validation(joint);
        record.definition = candidate.into();
        let JointRuntime::Prismatic(runtime) = &mut record.runtime else {
            return Err(JointMutationError::InvalidValue);
        };
        runtime.impulse.z = 0.0;
        runtime.limit_state = JointLimitState::Inactive;
        Ok(())
    }

    /// Sets checked translation limits, waking only when changed.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid limits, identity, kind, lock, or poison.
    #[allow(
        clippy::float_cmp,
        reason = "pinned setters use exact changed-only branches"
    )]
    pub fn set_prismatic_limits(
        &mut self,
        joint: JointId,
        lower: f32,
        upper: f32,
    ) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = prismatic_record(self, joint).map_err(as_mutation_error)?;
        let JointDef::Prismatic(definition) = record.definition else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        let candidate = definition
            .with_limits(definition.is_limit_enabled(), lower, upper)
            .map_err(|_| JointMutationError::InvalidValue)?;
        if definition.lower_translation() == lower && definition.upper_translation() == upper {
            return Ok(());
        }
        let bodies = record.bodies;
        self.wake_joint_bodies(bodies);
        let record = self.joint_mut_after_validation(joint);
        record.definition = candidate.into();
        let JointRuntime::Prismatic(runtime) = &mut record.runtime else {
            return Err(JointMutationError::InvalidValue);
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
    pub fn set_prismatic_motor_enabled(
        &mut self,
        joint: JointId,
        enabled: bool,
    ) -> Result<(), JointMutationError> {
        self.mutate_prismatic_motor(joint, |definition| {
            definition.with_motor(
                enabled,
                definition.motor_speed(),
                definition.max_motor_force(),
            )
        })
    }

    /// Sets motor speed and unconditionally wakes both bodies.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid speed, identity, kind, lock, or poison.
    pub fn set_prismatic_motor_speed(
        &mut self,
        joint: JointId,
        speed: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_prismatic_motor(joint, |definition| {
            definition.with_motor(
                definition.is_motor_enabled(),
                speed,
                definition.max_motor_force(),
            )
        })
    }

    /// Sets maximum motor force and unconditionally wakes both bodies.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid force, identity, kind, lock, or poison.
    pub fn set_prismatic_max_motor_force(
        &mut self,
        joint: JointId,
        force: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_prismatic_motor(joint, |definition| {
            definition.with_motor(
                definition.is_motor_enabled(),
                definition.motor_speed(),
                force,
            )
        })
    }

    fn mutate_prismatic_motor(
        &mut self,
        joint: JointId,
        mutate: impl FnOnce(PrismaticJointDef) -> Result<PrismaticJointDef, crate::JointDefError>,
    ) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = prismatic_record(self, joint).map_err(as_mutation_error)?;
        let JointDef::Prismatic(definition) = record.definition else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        let candidate = mutate(definition).map_err(|_| JointMutationError::InvalidValue)?;
        let bodies = record.bodies;
        self.wake_joint_bodies(bodies);
        self.joint_mut_after_validation(joint).definition = candidate.into();
        Ok(())
    }
}

fn prismatic_record(world: &World, joint: JointId) -> Result<&JointRecord, JointQueryError> {
    let record = world.joints.get(joint)?;
    if !matches!(record.definition, JointDef::Prismatic(_)) {
        return Err(wrong_kind(record.definition));
    }
    Ok(record)
}

fn wrong_kind(definition: JointDef) -> JointQueryError {
    JointQueryError::WrongKind {
        expected: JointKind::Prismatic,
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

fn classify_limit(definition: PrismaticJointDef, translation: f32) -> JointLimitState {
    if !definition.is_limit_enabled() {
        return JointLimitState::Inactive;
    }
    if (definition.upper_translation() - definition.lower_translation()).abs() < 2.0 * LINEAR_SLOP {
        JointLimitState::Equal
    } else if translation <= definition.lower_translation() {
        JointLimitState::AtLower
    } else if translation >= definition.upper_translation() {
        JointLimitState::AtUpper
    } else {
        JointLimitState::Inactive
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BodyDef, World};

    fn definition() -> PrismaticJointDef {
        let mut world = World::new().expect("world");
        let a = world.create_body(&BodyDef::default()).expect("body a");
        let b = world.create_body(&BodyDef::default()).expect("body b");
        PrismaticJointDef::new(a, b).expect("joint")
    }

    #[test]
    #[allow(clippy::float_cmp, reason = "exact source arithmetic is under test")]
    fn motor_impulse_is_capped_by_timestep_force() {
        // Arrange
        let definition = definition().with_motor(true, 10.0, 4.0).expect("motor");
        let mut runtime = PrismaticRuntime::new(definition);

        // Act
        let applied = runtime
            .solve_motor(definition, 0.25, 0.0, 1.0)
            .expect("solve");

        // Assert
        assert_eq!(applied, 1.0);
    }

    #[test]
    fn equal_limit_uses_linear_slop_boundary() {
        // Arrange
        let definition = definition().with_limits(true, 1.0, 1.0).expect("limits");
        let mut runtime = PrismaticRuntime::new(definition);

        // Act
        runtime
            .initialize(definition, 2.0, Vec2::new(1.0, 0.0), None)
            .expect("initialize");

        // Assert
        assert_eq!(runtime.limit_state, JointLimitState::Equal);
    }

    #[test]
    #[allow(
        clippy::float_cmp,
        reason = "transactional exact cache state is under test"
    )]
    fn derived_motor_overflow_is_transactional() {
        // Arrange
        let definition = definition()
            .with_motor(true, f32::MAX, f32::MAX)
            .expect("finite motor");
        let mut runtime = PrismaticRuntime::new(definition);

        // Act
        let result = runtime.solve_motor(definition, 2.0, 0.0, 2.0);

        // Assert
        assert_eq!(result, Err(JointMutationError::NonFiniteDerivedState));
        assert_eq!(runtime.motor_impulse, 0.0);
    }

    #[test]
    fn upper_limit_block_solver_clamps_positive_accumulation() {
        // Arrange
        let definition = definition();
        let mut runtime = PrismaticRuntime::new(definition);
        runtime.limit_state = JointLimitState::AtUpper;

        // Act
        let applied = runtime
            .solve_constraint_velocity(Mat33::IDENTITY, 0.0, 0.0, -1.0, true)
            .expect("block solve");

        // Assert
        assert_eq!(applied, Vec3::ZERO);
        assert_eq!(runtime.impulse, Vec3::ZERO);
    }

    #[test]
    fn entering_equal_limit_retains_source_axial_cache() {
        // Arrange
        let definition = definition().with_limits(true, 0.0, 0.0).expect("limits");
        let mut runtime = PrismaticRuntime {
            impulse: Vec3::new(0.0, 0.0, 3.0),
            motor_impulse: 0.0,
            limit_state: JointLimitState::AtLower,
            axis: Vec2::new(1.0, 0.0),
            perpendicular: Vec2::new(0.0, 1.0),
        };

        // Act
        runtime
            .initialize(definition, 0.0, Vec2::new(1.0, 0.0), Some(1.0))
            .expect("initialize");

        // Assert
        assert_eq!(runtime.impulse.z.to_bits(), 3.0_f32.to_bits());
        assert_eq!(runtime.limit_state, JointLimitState::Equal);
    }
}
