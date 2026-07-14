//! Source-ordered pulley-joint state and checked solver scratch.

use crate::math::Vec2;
use crate::math::settings::LINEAR_SLOP;
use crate::{
    JointDef, JointKind, JointMutationError, JointQueryError, JointSnapshot, JointSpecificSnapshot,
    PulleyJointDef, PulleyJointSnapshot,
};

use super::JointRecord;
use crate::world::object::World;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PulleyRuntime {
    impulse: f32,
    direction_a: Vec2,
    direction_b: Vec2,
    mass: f32,
    constant: f32,
}

impl PulleyRuntime {
    pub(super) fn new(definition: PulleyJointDef) -> Self {
        Self {
            impulse: 0.0,
            direction_a: Vec2::ZERO,
            direction_b: Vec2::ZERO,
            mass: 0.0,
            constant: definition.constant(),
        }
    }

    pub(super) fn reaction_force(self, inverse_timestep: f32) -> Vec2 {
        inverse_timestep * (self.impulse * self.direction_b)
    }

    pub(super) const fn solver_impulse(self) -> f32 {
        self.impulse
    }

    pub(super) const fn solver_directions(self) -> [Vec2; 2] {
        [self.direction_a, self.direction_b]
    }

    #[allow(
        dead_code,
        reason = "used by the Phase 8 mixed-island integration plan"
    )]
    pub(super) fn initialize(
        &mut self,
        definition: PulleyJointDef,
        direction_a: Vec2,
        direction_b: Vec2,
        effective_mass_a: f32,
        effective_mass_b: f32,
        maybe_warm_start_ratio: Option<f32>,
    ) -> Result<(), JointMutationError> {
        if !direction_a.is_valid()
            || !direction_b.is_valid()
            || !effective_mass_a.is_finite()
            || effective_mass_a < 0.0
            || !effective_mass_b.is_finite()
            || effective_mass_b < 0.0
        {
            return Err(JointMutationError::InvalidValue);
        }
        let direction_a = normalized_segment(direction_a);
        let direction_b = normalized_segment(direction_b);
        let denominator =
            effective_mass_a + definition.ratio() * definition.ratio() * effective_mass_b;
        let mass = if denominator > 0.0 {
            1.0 / denominator
        } else {
            0.0
        };
        let impulse = match maybe_warm_start_ratio {
            Some(ratio) if ratio.is_finite() && ratio >= 0.0 => self.impulse * ratio,
            Some(_) => return Err(JointMutationError::InvalidValue),
            None => 0.0,
        };
        let candidate = Self {
            impulse,
            direction_a,
            direction_b,
            mass,
            constant: definition.constant(),
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
        definition: PulleyJointDef,
        velocity_a: Vec2,
        velocity_b: Vec2,
    ) -> Result<f32, JointMutationError> {
        if !velocity_a.is_valid() || !velocity_b.is_valid() {
            return Err(JointMutationError::InvalidValue);
        }
        let speed = -self.direction_a.dot(velocity_a)
            - definition.ratio() * self.direction_b.dot(velocity_b);
        let applied = -self.mass * speed;
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
        definition: PulleyJointDef,
        length_a: f32,
        length_b: f32,
    ) -> Result<(f32, bool), JointMutationError> {
        if !length_a.is_finite() || length_a < 0.0 || !length_b.is_finite() || length_b < 0.0 {
            return Err(JointMutationError::InvalidValue);
        }
        let constraint = self.constant - length_a - definition.ratio() * length_b;
        let impulse = -self.mass * constraint;
        if !constraint.is_finite() || !impulse.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok((impulse, constraint.abs() < LINEAR_SLOP))
    }

    #[allow(
        dead_code,
        reason = "consumed by the Phase 8 origin-shift integration plan"
    )]
    pub(super) fn shifted_ground_anchors(
        definition: PulleyJointDef,
        shift: Vec2,
    ) -> Result<[Vec2; 2], JointMutationError> {
        let anchors = [
            definition.ground_anchor_a() - shift,
            definition.ground_anchor_b() - shift,
        ];
        if !shift.is_valid() || !anchors[0].is_valid() || !anchors[1].is_valid() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok(anchors)
    }

    fn is_valid(self) -> bool {
        self.impulse.is_finite()
            && self.direction_a.is_valid()
            && self.direction_b.is_valid()
            && self.mass.is_finite()
            && self.constant.is_finite()
    }
}

fn normalized_segment(mut direction: Vec2) -> Vec2 {
    let length = direction.length();
    if length > 10.0 * LINEAR_SLOP {
        direction *= 1.0 / length;
        direction
    } else {
        Vec2::ZERO
    }
}

pub(super) fn snapshot(
    world: &World,
    record: &JointRecord,
    runtime: PulleyRuntime,
) -> Result<JointSnapshot, JointQueryError> {
    let JointDef::Pulley(definition) = record.definition else {
        return Err(JointQueryError::WrongKind {
            expected: JointKind::Pulley,
            actual: JointKind::from_definition(record.definition),
        });
    };
    let body_a = world.bodies.get(record.bodies[0])?.state.snapshot();
    let body_b = world.bodies.get(record.bodies[1])?.state.snapshot();
    let anchor_a = body_a.transform().apply(definition.local_anchor_a());
    let anchor_b = body_b.transform().apply(definition.local_anchor_b());
    let current_length_a = (anchor_a - definition.ground_anchor_a()).length();
    let current_length_b = (anchor_b - definition.ground_anchor_b()).length();
    if !anchor_a.is_valid()
        || !anchor_b.is_valid()
        || !current_length_a.is_finite()
        || !current_length_b.is_finite()
    {
        return Err(JointQueryError::NonFiniteDerivedState);
    }
    Ok(
        JointSnapshot::from_definition(record.definition, record.bodies).with_runtime(
            anchor_a,
            anchor_b,
            JointSpecificSnapshot::Pulley(PulleyJointSnapshot::new(
                definition.ground_anchor_a(),
                definition.ground_anchor_b(),
                definition.length_a(),
                definition.length_b(),
                current_length_a,
                current_length_b,
                definition.ratio(),
                runtime.constant,
            )),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BodyDef, World};

    fn definition() -> PulleyJointDef {
        let mut world = World::new().expect("world");
        let body_a = world.create_body(&BodyDef::default()).expect("A");
        let body_b = world.create_body(&BodyDef::default()).expect("B");
        PulleyJointDef::new(body_a, body_b).expect("definition")
    }

    #[test]
    fn degenerate_segments_are_zero_and_warm_impulse_scales() {
        // Arrange
        let definition = definition();
        let mut runtime = PulleyRuntime::new(definition);
        runtime.impulse = 4.0;

        // Act
        runtime
            .initialize(
                definition,
                Vec2::new(0.01, 0.0),
                Vec2::new(2.0, 0.0),
                1.0,
                1.0,
                Some(0.5),
            )
            .expect("initialization");

        // Assert
        assert_eq!(runtime.direction_a, Vec2::ZERO);
        assert_eq!(runtime.direction_b, Vec2::new(1.0, 0.0));
        assert_eq!(runtime.impulse.to_bits(), 2.0_f32.to_bits());
    }

    #[test]
    fn cold_solve_and_shift_candidate_preserve_source_geometry() {
        // Arrange
        let definition = definition();
        let mut runtime = PulleyRuntime::new(definition);
        runtime
            .initialize(
                definition,
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
                1.0,
                1.0,
                None,
            )
            .expect("initialization");

        // Act
        let impulse = runtime
            .solve_velocity(definition, Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0))
            .expect("solve");
        let position = runtime
            .position_impulse(definition, 2.0, 2.0)
            .expect("position solve");
        let before_overflow = runtime.impulse;
        let overflow = runtime.solve_velocity(
            definition,
            Vec2::new(f32::MAX, 0.0),
            Vec2::new(0.0, f32::MAX),
        );
        let reaction = runtime.reaction_force(2.0);
        let shifted =
            PulleyRuntime::shifted_ground_anchors(definition, Vec2::new(1.0, 2.0)).expect("shift");

        // Assert
        assert!(impulse > 0.0);
        assert!(position.0.is_finite());
        assert_eq!(overflow, Err(JointMutationError::NonFiniteDerivedState));
        assert_eq!(runtime.impulse.to_bits(), before_overflow.to_bits());
        assert!(reaction.is_valid());
        assert_eq!(
            shifted[0],
            definition.ground_anchor_a() - Vec2::new(1.0, 2.0)
        );
        assert_eq!(
            shifted[1],
            definition.ground_anchor_b() - Vec2::new(1.0, 2.0)
        );
    }
}
