//! Source-ordered mouse-joint state and checked world authority.

use crate::math::settings::TAU;
use crate::math::{Mat22, Transform, Vec2};
use crate::{
    JointDef, JointId, JointKind, JointMutationError, JointQueryError, JointSnapshot,
    JointSpecificSnapshot, MouseJointDef, MouseJointSnapshot,
};

use super::JointRecord;
use crate::world::object::World;

#[derive(Debug, Clone, Copy)]
pub(crate) struct MouseRuntime {
    local_anchor_b: Vec2,
    impulse: Vec2,
    gamma: f32,
    beta: f32,
    mass: Mat22,
    bias: Vec2,
}

impl MouseRuntime {
    pub(super) fn new(definition: MouseJointDef, body_b_transform: Transform) -> Self {
        Self {
            local_anchor_b: body_b_transform.inverse_apply(definition.target()),
            impulse: Vec2::ZERO,
            gamma: 0.0,
            beta: 0.0,
            mass: Mat22::ZERO,
            bias: Vec2::ZERO,
        }
    }

    pub(super) fn reaction_force(self, inverse_timestep: f32) -> Vec2 {
        inverse_timestep * self.impulse
    }

    #[allow(
        clippy::too_many_arguments,
        dead_code,
        reason = "the arguments are the closed source solver inputs used by island integration"
    )]
    pub(super) fn initialize(
        &mut self,
        definition: MouseJointDef,
        timestep: f32,
        body_mass: f32,
        inverse_mass: f32,
        inverse_inertia: f32,
        radius: Vec2,
        body_center: Vec2,
        maybe_warm_start_ratio: Option<f32>,
        angular_velocity: f32,
    ) -> Result<f32, JointMutationError> {
        if !timestep.is_finite()
            || timestep < 0.0
            || !body_mass.is_finite()
            || body_mass < 0.0
            || !inverse_mass.is_finite()
            || inverse_mass < 0.0
            || !inverse_inertia.is_finite()
            || inverse_inertia < 0.0
            || !radius.is_valid()
            || !body_center.is_valid()
            || !angular_velocity.is_finite()
        {
            return Err(JointMutationError::InvalidValue);
        }
        let omega = TAU * definition.frequency();
        let damping = 2.0 * body_mass * definition.damping_ratio() * omega;
        let stiffness = body_mass * (omega * omega);
        let mut gamma = timestep * (damping + timestep * stiffness);
        gamma = if gamma == 0.0 { 0.0 } else { 1.0 / gamma };
        let beta = timestep * stiffness * gamma;
        let matrix = Mat22::from_columns(
            Vec2::new(
                inverse_mass + inverse_inertia * radius.y * radius.y + gamma,
                -inverse_inertia * radius.x * radius.y,
            ),
            Vec2::new(
                -inverse_inertia * radius.x * radius.y,
                inverse_mass + inverse_inertia * radius.x * radius.x + gamma,
            ),
        )
        .inverse();
        let bias = beta * (body_center + radius - definition.target());
        let impulse = match maybe_warm_start_ratio {
            Some(ratio) if ratio.is_finite() && ratio >= 0.0 => ratio * self.impulse,
            Some(_) => return Err(JointMutationError::InvalidValue),
            None => Vec2::ZERO,
        };
        let candidate = Self {
            local_anchor_b: self.local_anchor_b,
            impulse,
            gamma,
            beta,
            mass: matrix,
            bias,
        };
        let damped_angular_velocity = 0.98 * angular_velocity;
        if !candidate.is_valid() || !damped_angular_velocity.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        *self = candidate;
        Ok(damped_angular_velocity)
    }

    #[allow(
        dead_code,
        reason = "used by the Phase 8 mixed-island integration plan"
    )]
    pub(super) fn solve_velocity(
        &mut self,
        definition: MouseJointDef,
        timestep: f32,
        point_velocity: Vec2,
    ) -> Result<Vec2, JointMutationError> {
        if !timestep.is_finite() || timestep < 0.0 || !point_velocity.is_valid() {
            return Err(JointMutationError::InvalidValue);
        }
        let candidate_delta = self
            .mass
            .apply(-(point_velocity + self.bias + self.gamma * self.impulse));
        let old_impulse = self.impulse;
        let mut candidate = old_impulse + candidate_delta;
        let max_impulse = timestep * definition.max_force();
        let candidate_length_squared = candidate.length_squared();
        if candidate_length_squared > max_impulse * max_impulse {
            let length = candidate.length();
            if length > 0.0 {
                candidate *= max_impulse / length;
            }
        }
        let applied = candidate - old_impulse;
        if !candidate.is_valid() || !applied.is_valid() || !max_impulse.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        self.impulse = candidate;
        Ok(applied)
    }

    #[allow(
        dead_code,
        reason = "consumed by the Phase 8 origin-shift integration plan"
    )]
    pub(super) fn shifted_target(
        definition: MouseJointDef,
        shift: Vec2,
    ) -> Result<Vec2, JointMutationError> {
        let target = definition.target() - shift;
        if !shift.is_valid() || !target.is_valid() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok(target)
    }

    fn is_valid(self) -> bool {
        self.local_anchor_b.is_valid()
            && self.impulse.is_valid()
            && self.gamma.is_finite()
            && self.beta.is_finite()
            && self.mass.first_column().is_valid()
            && self.mass.second_column().is_valid()
            && self.bias.is_valid()
    }
}

pub(super) fn snapshot(
    world: &World,
    record: &JointRecord,
    runtime: MouseRuntime,
) -> Result<JointSnapshot, JointQueryError> {
    let JointDef::Mouse(definition) = record.definition else {
        return Err(wrong_kind(record.definition));
    };
    let body_b = world.bodies.get(record.bodies[1])?.state.snapshot();
    let anchor_a = definition.target();
    let anchor_b = body_b.transform().apply(runtime.local_anchor_b);
    if !anchor_a.is_valid() || !anchor_b.is_valid() {
        return Err(JointQueryError::NonFiniteDerivedState);
    }
    Ok(
        JointSnapshot::from_definition(record.definition).with_runtime(
            anchor_a,
            anchor_b,
            JointSpecificSnapshot::Mouse(MouseJointSnapshot::new(
                definition.target(),
                definition.max_force(),
                definition.frequency(),
                definition.damping_ratio(),
                runtime.gamma,
                runtime.beta,
            )),
        ),
    )
}

impl World {
    /// Sets the world-space target and wakes body B.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid identity, kind, target, lock, or poison.
    pub fn set_mouse_target(
        &mut self,
        joint: JointId,
        target: Vec2,
    ) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = self.joints.get(joint)?;
        let JointDef::Mouse(definition) = record.definition else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        let candidate = definition
            .with_target(target)
            .map_err(|_| JointMutationError::InvalidValue)?;
        let body_b = record.bodies[1];
        let body = self.body_mut_after_validation(body_b);
        body.state = body.state.candidate_set_awake(true);
        self.joint_mut_after_validation(joint).definition = candidate.into();
        Ok(())
    }

    /// Sets the non-negative force cap without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid identity, kind, value, lock, or poison.
    pub fn set_mouse_max_force(
        &mut self,
        joint: JointId,
        max_force: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_mouse(joint, |definition| definition.with_max_force(max_force))
    }

    /// Sets the non-negative response frequency without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid identity, kind, value, lock, or poison.
    pub fn set_mouse_frequency(
        &mut self,
        joint: JointId,
        frequency: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_mouse(joint, |definition| definition.with_frequency(frequency))
    }

    /// Sets the non-negative damping ratio without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid identity, kind, value, lock, or poison.
    pub fn set_mouse_damping_ratio(
        &mut self,
        joint: JointId,
        damping_ratio: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_mouse(joint, |definition| {
            definition.with_damping_ratio(damping_ratio)
        })
    }

    fn mutate_mouse(
        &mut self,
        joint: JointId,
        mutate: impl FnOnce(MouseJointDef) -> Result<MouseJointDef, crate::JointDefError>,
    ) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = self.joints.get(joint)?;
        let JointDef::Mouse(definition) = record.definition else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        let candidate = mutate(definition).map_err(|_| JointMutationError::InvalidValue)?;
        self.joint_mut_after_validation(joint).definition = candidate.into();
        Ok(())
    }
}

fn wrong_kind(definition: JointDef) -> JointQueryError {
    JointQueryError::WrongKind {
        expected: JointKind::Mouse,
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

    fn definition() -> MouseJointDef {
        let mut world = World::new().expect("world");
        let body_a = world.create_body(&BodyDef::default()).expect("A");
        let body_b = world.create_body(&BodyDef::default()).expect("B");
        MouseJointDef::new(body_a, body_b)
            .expect("definition")
            .with_max_force(10.0)
            .expect("force")
    }

    #[test]
    fn gamma_beta_warm_start_and_angular_damping_follow_source_grouping() {
        // Arrange
        let definition = definition();
        let mut runtime = MouseRuntime::new(definition, Transform::IDENTITY);
        runtime.impulse = Vec2::new(2.0, 0.0);

        // Act
        let angular = runtime
            .initialize(
                definition,
                0.5,
                2.0,
                0.5,
                1.0,
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Some(0.5),
                10.0,
            )
            .expect("initialization");

        // Assert
        assert!(runtime.gamma > 0.0);
        assert!(runtime.beta > 0.0);
        assert_eq!(runtime.impulse, Vec2::new(1.0, 0.0));
        assert_eq!(angular.to_bits(), 9.8_f32.to_bits());
    }

    #[test]
    fn velocity_impulse_is_capped_and_shift_candidate_is_checked() {
        // Arrange
        let definition = definition();
        let mut runtime = MouseRuntime::new(definition, Transform::IDENTITY);
        runtime.mass = Mat22::IDENTITY;

        // Act
        let applied = runtime
            .solve_velocity(definition, 0.5, Vec2::new(100.0, 0.0))
            .expect("solve");
        let reaction = runtime.reaction_force(2.0);
        let before_overflow = runtime.impulse;
        let overflow = runtime.solve_velocity(definition, f32::MAX, Vec2::ZERO);
        let shifted = MouseRuntime::shifted_target(definition, Vec2::new(1.0, 2.0)).expect("shift");

        // Assert
        assert_eq!(applied.length().to_bits(), 5.0_f32.to_bits());
        assert_eq!(reaction, 2.0 * runtime.impulse);
        assert_eq!(overflow, Err(JointMutationError::NonFiniteDerivedState));
        assert_eq!(runtime.impulse, before_overflow);
        assert_eq!(shifted, definition.target() - Vec2::new(1.0, 2.0));
    }
}
