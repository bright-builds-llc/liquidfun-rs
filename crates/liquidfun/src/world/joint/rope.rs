//! Source-ordered unilateral rope-joint state and checked world authority.

use crate::math::{
    Vec2,
    settings::{LINEAR_SLOP, MAX_LINEAR_CORRECTION},
};
use crate::{
    JointDef, JointId, JointKind, JointLimitState, JointMutationError, JointQueryError,
    JointSnapshot, JointSpecificSnapshot, RopeJointDef, RopeJointSnapshot,
};

use super::JointRecord;
use crate::world::object::World;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RopeJointRuntime {
    direction: Vec2,
    length: f32,
    impulse: f32,
    mass: f32,
    limit_state: JointLimitState,
}

impl RopeJointRuntime {
    pub(super) const fn new(_definition: RopeJointDef) -> Self {
        Self {
            direction: Vec2::ZERO,
            length: 0.0,
            impulse: 0.0,
            mass: 0.0,
            limit_state: JointLimitState::Inactive,
        }
    }
    pub(super) fn reaction_force(self, inverse_timestep: f32) -> Vec2 {
        (inverse_timestep * self.impulse) * self.direction
    }
    #[allow(dead_code, reason = "used by the mixed-island integration plan")]
    pub(super) fn initialize(
        &mut self,
        definition: RopeJointDef,
        separation: Vec2,
        inverse_effective_mass: f32,
        maybe_warm_start_ratio: Option<f32>,
    ) -> Result<(), JointMutationError> {
        let previous = *self;
        if !separation.is_valid()
            || !inverse_effective_mass.is_finite()
            || inverse_effective_mass < 0.0
        {
            return Err(JointMutationError::InvalidValue);
        }
        self.direction = separation;
        self.length = self.direction.normalize();
        self.limit_state = if self.length - definition.max_length() > 0.0 {
            JointLimitState::AtUpper
        } else {
            JointLimitState::Inactive
        };
        if self.length <= LINEAR_SLOP {
            self.direction = Vec2::ZERO;
            self.mass = 0.0;
            self.impulse = 0.0;
            return Ok(());
        }
        self.mass = if inverse_effective_mass == 0.0 {
            0.0
        } else {
            1.0 / inverse_effective_mass
        };
        match maybe_warm_start_ratio {
            Some(ratio) if ratio.is_finite() && ratio >= 0.0 => self.impulse *= ratio,
            Some(_) => {
                *self = previous;
                return Err(JointMutationError::InvalidValue);
            }
            None => self.impulse = 0.0,
        }
        if !self.direction.is_valid()
            || !self.length.is_finite()
            || !self.impulse.is_finite()
            || !self.mass.is_finite()
        {
            *self = previous;
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok(())
    }
    #[allow(dead_code, reason = "used by the mixed-island integration plan")]
    pub(super) fn solve_velocity(
        &mut self,
        definition: RopeJointDef,
        inverse_timestep: f32,
        relative_speed: f32,
    ) -> Result<f32, JointMutationError> {
        if !inverse_timestep.is_finite() || inverse_timestep < 0.0 || !relative_speed.is_finite() {
            return Err(JointMutationError::InvalidValue);
        }
        let constraint = self.length - definition.max_length();
        let speed = if constraint < 0.0 {
            relative_speed + inverse_timestep * constraint
        } else {
            relative_speed
        };
        let raw = -self.mass * speed;
        let candidate = (self.impulse + raw).min(0.0);
        if !candidate.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        let applied = candidate - self.impulse;
        self.impulse = candidate;
        Ok(applied)
    }
    #[allow(dead_code, reason = "used by the mixed-island integration plan")]
    pub(super) fn solve_position(
        definition: RopeJointDef,
        length: f32,
        mass: f32,
    ) -> Result<(f32, bool), JointMutationError> {
        if !length.is_finite() || length < 0.0 || !mass.is_finite() || mass < 0.0 {
            return Err(JointMutationError::InvalidValue);
        }
        let correction = (length - definition.max_length()).clamp(0.0, MAX_LINEAR_CORRECTION);
        let impulse = -mass * correction;
        if !impulse.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok((impulse, length - definition.max_length() < LINEAR_SLOP))
    }
}

pub(super) fn snapshot(
    world: &World,
    record: &JointRecord,
    _runtime: RopeJointRuntime,
) -> Result<JointSnapshot, JointQueryError> {
    let JointDef::Rope(definition) = record.definition else {
        return Err(wrong_kind(record.definition));
    };
    let body_a = world.bodies.get(record.bodies[0])?.state.snapshot();
    let body_b = world.bodies.get(record.bodies[1])?.state.snapshot();
    let anchor_a = body_a.transform().apply(definition.local_anchor_a());
    let anchor_b = body_b.transform().apply(definition.local_anchor_b());
    let length = (anchor_b - anchor_a).length();
    if !anchor_a.is_valid() || !anchor_b.is_valid() || !length.is_finite() {
        return Err(JointQueryError::NonFiniteDerivedState);
    }
    let state = if length - definition.max_length() > 0.0 {
        JointLimitState::AtUpper
    } else {
        JointLimitState::Inactive
    };
    Ok(
        JointSnapshot::from_definition(record.definition).with_runtime(
            anchor_a,
            anchor_b,
            JointSpecificSnapshot::Rope(RopeJointSnapshot::new(
                definition.max_length(),
                length,
                state,
            )),
        ),
    )
}

impl World {
    /// Sets the strictly positive `RopeJoint` maximum length without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid value, identity, kind, lock, or poison.
    pub fn set_rope_joint_max_length(
        &mut self,
        joint: JointId,
        value: f32,
    ) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = rope_record(self, joint).map_err(as_mutation_error)?;
        let JointDef::Rope(definition) = record.definition else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        let candidate = definition
            .with_max_length(value)
            .map_err(|_| JointMutationError::InvalidValue)?;
        self.joint_mut_after_validation(joint).definition = candidate.into();
        Ok(())
    }
}

fn rope_record(world: &World, joint: JointId) -> Result<&JointRecord, JointQueryError> {
    let record = world.joints.get(joint)?;
    if !matches!(record.definition, JointDef::Rope(_)) {
        return Err(wrong_kind(record.definition));
    }
    Ok(record)
}
fn wrong_kind(definition: JointDef) -> JointQueryError {
    JointQueryError::WrongKind {
        expected: JointKind::Rope,
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
    fn definition() -> RopeJointDef {
        let mut world = World::new().expect("world");
        let a = world.create_body(&BodyDef::default()).expect("A");
        let b = world.create_body(&BodyDef::default()).expect("B");
        RopeJointDef::new(a, b)
            .expect("definition")
            .with_max_length(2.0)
            .expect("length")
    }
    #[test]
    fn inactive_predictive_solve_and_unilateral_clamp_match_source() {
        // Arrange
        let definition = definition();
        let mut runtime = RopeJointRuntime::new(definition);
        runtime
            .initialize(definition, Vec2::new(1.0, 0.0), 1.0, None)
            .expect("initialize");
        // Act
        let predictive = runtime
            .solve_velocity(definition, 10.0, 0.0)
            .expect("predictive");
        runtime
            .initialize(definition, Vec2::new(3.0, 0.0), 1.0, None)
            .expect("upper-limit initialize");
        let constrained = runtime
            .solve_velocity(definition, 0.0, 2.0)
            .expect("upper-limit solve");
        // Assert
        assert_eq!(predictive.to_bits(), 0.0_f32.to_bits());
        assert_eq!(constrained.to_bits(), (-2.0_f32).to_bits());
        assert!(runtime.impulse <= 0.0);
    }
}
