//! Source-ordered distance-joint state and checked world authority.

use crate::math::Vec2;
use crate::math::settings::{LINEAR_SLOP, MAX_LINEAR_CORRECTION, TAU};
use crate::{
    DistanceJointDef, DistanceJointSnapshot, JointDef, JointId, JointKind, JointMutationError,
    JointQueryError, JointSnapshot, JointSpecificSnapshot,
};

use super::JointRecord;
use crate::world::object::World;

#[derive(Debug, Clone, Copy)]
pub(crate) struct DistanceRuntime {
    impulse: f32,
    direction: Vec2,
    mass: f32,
    gamma: f32,
    bias: f32,
}

impl DistanceRuntime {
    pub(super) const fn new(_definition: DistanceJointDef) -> Self {
        Self {
            impulse: 0.0,
            direction: Vec2::ZERO,
            mass: 0.0,
            gamma: 0.0,
            bias: 0.0,
        }
    }

    pub(super) fn reaction_force(self, inverse_timestep: f32) -> Vec2 {
        (inverse_timestep * self.impulse) * self.direction
    }

    #[allow(
        dead_code,
        reason = "used by the Phase 8 mixed-island integration plan"
    )]
    pub(super) fn initialize(
        &mut self,
        definition: DistanceJointDef,
        mut displacement: Vec2,
        inverse_mass: f32,
        timestep: f32,
        maybe_warm_start_ratio: Option<f32>,
    ) -> Result<(), JointMutationError> {
        if !displacement.is_valid()
            || !inverse_mass.is_finite()
            || inverse_mass < 0.0
            || !timestep.is_finite()
            || timestep < 0.0
        {
            return Err(JointMutationError::InvalidValue);
        }
        let length = displacement.length();
        if length > LINEAR_SLOP {
            displacement *= 1.0 / length;
        } else {
            displacement = Vec2::ZERO;
        }
        let mut candidate_inverse_mass = inverse_mass;
        let mut mass = if candidate_inverse_mass == 0.0 {
            0.0
        } else {
            1.0 / candidate_inverse_mass
        };
        let (gamma, bias) = if definition.frequency() > 0.0 {
            let constraint = length - definition.length();
            let omega = TAU * definition.frequency();
            let damping = 2.0 * mass * definition.damping_ratio() * omega;
            let stiffness = mass * omega * omega;
            let mut gamma = timestep * (damping + timestep * stiffness);
            gamma = if gamma == 0.0 { 0.0 } else { 1.0 / gamma };
            let bias = constraint * timestep * stiffness * gamma;
            candidate_inverse_mass += gamma;
            mass = if candidate_inverse_mass == 0.0 {
                0.0
            } else {
                1.0 / candidate_inverse_mass
            };
            (gamma, bias)
        } else {
            (0.0, 0.0)
        };
        let impulse = match maybe_warm_start_ratio {
            Some(ratio) if ratio.is_finite() && ratio >= 0.0 => self.impulse * ratio,
            Some(_) => return Err(JointMutationError::InvalidValue),
            None => 0.0,
        };
        let candidate = Self {
            impulse,
            direction: displacement,
            mass,
            gamma,
            bias,
        };
        if !candidate.is_valid() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        *self = candidate;
        Ok(())
    }

    #[allow(
        dead_code,
        reason = "used by the Phase 8 mixed-island integration plan"
    )]
    pub(super) fn solve_velocity(
        &mut self,
        relative_velocity: Vec2,
    ) -> Result<f32, JointMutationError> {
        if !relative_velocity.is_valid() {
            return Err(JointMutationError::InvalidValue);
        }
        let applied = -self.mass
            * (self.direction.dot(relative_velocity) + self.bias + self.gamma * self.impulse);
        let candidate = self.impulse + applied;
        if !applied.is_finite() || !candidate.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        self.impulse = candidate;
        Ok(applied)
    }

    #[allow(
        dead_code,
        reason = "used by the Phase 8 mixed-island integration plan"
    )]
    pub(super) fn position_impulse(
        self,
        definition: DistanceJointDef,
        current_length: f32,
    ) -> Result<Option<f32>, JointMutationError> {
        if !current_length.is_finite() || current_length < 0.0 {
            return Err(JointMutationError::InvalidValue);
        }
        if definition.frequency() > 0.0 {
            return Ok(None);
        }
        let correction = (current_length - definition.length())
            .clamp(-MAX_LINEAR_CORRECTION, MAX_LINEAR_CORRECTION);
        let impulse = -self.mass * correction;
        if !impulse.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok(Some(impulse))
    }

    fn is_valid(self) -> bool {
        self.impulse.is_finite()
            && self.direction.is_valid()
            && self.mass.is_finite()
            && self.gamma.is_finite()
            && self.bias.is_finite()
    }
}

pub(super) fn snapshot(
    world: &World,
    record: &JointRecord,
    runtime: DistanceRuntime,
) -> Result<JointSnapshot, JointQueryError> {
    let JointDef::Distance(definition) = record.definition else {
        return Err(wrong_kind(record.definition));
    };
    let body_a = world.bodies.get(record.bodies[0])?.state.snapshot();
    let body_b = world.bodies.get(record.bodies[1])?.state.snapshot();
    let anchor_a = body_a.transform().apply(definition.local_anchor_a());
    let anchor_b = body_b.transform().apply(definition.local_anchor_b());
    let current_length = (anchor_b - anchor_a).length();
    if !anchor_a.is_valid() || !anchor_b.is_valid() || !current_length.is_finite() {
        return Err(JointQueryError::NonFiniteDerivedState);
    }
    Ok(
        JointSnapshot::from_definition(record.definition).with_runtime(
            anchor_a,
            anchor_b,
            JointSpecificSnapshot::Distance(DistanceJointSnapshot::new(
                definition.length(),
                current_length,
                definition.frequency(),
                definition.damping_ratio(),
                runtime.gamma,
                runtime.bias,
            )),
        ),
    )
}

impl World {
    /// Sets the positive natural distance without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid identity, kind, value, lock, or poison.
    pub fn set_distance_length(
        &mut self,
        joint: JointId,
        length: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_distance(joint, |definition| definition.with_length(length))
    }

    /// Sets the non-negative softness frequency without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid identity, kind, value, lock, or poison.
    pub fn set_distance_frequency(
        &mut self,
        joint: JointId,
        frequency: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_distance(joint, |definition| definition.with_frequency(frequency))
    }

    /// Sets the non-negative damping ratio without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid identity, kind, value, lock, or poison.
    pub fn set_distance_damping_ratio(
        &mut self,
        joint: JointId,
        damping_ratio: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_distance(joint, |definition| {
            definition.with_damping_ratio(damping_ratio)
        })
    }

    fn mutate_distance(
        &mut self,
        joint: JointId,
        mutate: impl FnOnce(DistanceJointDef) -> Result<DistanceJointDef, crate::JointDefError>,
    ) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = self.joints.get(joint)?;
        let JointDef::Distance(definition) = record.definition else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        let candidate = mutate(definition).map_err(|_| JointMutationError::InvalidValue)?;
        self.joint_mut_after_validation(joint).definition = candidate.into();
        Ok(())
    }
}

fn wrong_kind(definition: JointDef) -> JointQueryError {
    JointQueryError::WrongKind {
        expected: JointKind::Distance,
        actual: JointKind::from_definition(definition),
    }
}

fn as_mutation_error(error: JointQueryError) -> JointMutationError {
    match error {
        JointQueryError::InvalidHandle(error) => JointMutationError::InvalidHandle(error),
        JointQueryError::WrongKind { expected, actual } => {
            JointMutationError::WrongKind { expected, actual }
        }
        JointQueryError::NonFiniteDerivedState => JointMutationError::NonFiniteDerivedState,
        JointQueryError::Poisoned => JointMutationError::Poisoned,
        JointQueryError::InvalidInverseTimestep => JointMutationError::InvalidValue,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BodyDef, World};

    fn definition() -> DistanceJointDef {
        let mut world = World::new().expect("world");
        let body_a = world.create_body(&BodyDef::default()).expect("A");
        let body_b = world.create_body(&BodyDef::default()).expect("B");
        DistanceJointDef::new(body_a, body_b).expect("definition")
    }

    #[test]
    fn rigid_and_soft_initialization_preserve_source_branches() {
        // Arrange
        let rigid = definition();
        let soft = rigid.with_frequency(2.0).expect("frequency");
        let mut rigid_runtime = DistanceRuntime::new(rigid);
        let mut soft_runtime = DistanceRuntime::new(soft);

        // Act
        rigid_runtime
            .initialize(rigid, Vec2::new(2.0, 0.0), 2.0, 0.5, None)
            .expect("rigid initialization");
        soft_runtime
            .initialize(soft, Vec2::new(2.0, 0.0), 2.0, 0.5, None)
            .expect("soft initialization");

        // Assert
        assert_eq!(rigid_runtime.gamma.to_bits(), 0.0_f32.to_bits());
        assert_eq!(rigid_runtime.bias.to_bits(), 0.0_f32.to_bits());
        assert!(soft_runtime.gamma > 0.0);
        assert!(soft_runtime.bias > 0.0);
        assert_eq!(soft_runtime.position_impulse(soft, 4.0), Ok(None));
        assert!(
            rigid_runtime
                .position_impulse(rigid, 4.0)
                .expect("position")
                .is_some()
        );
    }

    #[test]
    fn zero_separation_warm_start_and_overflow_are_transactional() {
        // Arrange
        let definition = definition();
        let mut runtime = DistanceRuntime::new(definition);
        runtime.impulse = 2.0;

        // Act
        runtime
            .initialize(definition, Vec2::ZERO, 1.0, 0.5, Some(0.5))
            .expect("warm initialization");
        let zero_direction = runtime.direction;
        let warm_impulse = runtime.impulse;
        runtime.direction = Vec2::new(1.0, 0.0);
        runtime.mass = f32::MAX;
        let before = runtime;
        let result = runtime.solve_velocity(Vec2::new(f32::MAX, 0.0));
        let reaction = before.reaction_force(2.0);

        // Assert
        assert_eq!(zero_direction, Vec2::ZERO);
        assert_eq!(warm_impulse.to_bits(), 1.0_f32.to_bits());
        assert_eq!(result, Err(JointMutationError::NonFiniteDerivedState));
        assert_eq!(runtime.impulse.to_bits(), before.impulse.to_bits());
        assert_eq!(reaction, Vec2::new(2.0, 0.0));
    }
}
