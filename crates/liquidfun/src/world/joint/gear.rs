//! Source-ordered gear ownership, semantic state, and four-body solver core.

use crate::math::{Rotation, Vec2};
use crate::{
    BodyId, GearJointDef, GearJointSnapshot, JointDef, JointId, JointKind, JointMutationError,
    JointQueryError, JointSnapshot, JointSpecificSnapshot, PrismaticJointDef, RevoluteJointDef,
};

use super::{JointRecord, JointRuntime};
use crate::world::object::World;

mod solver;

pub(crate) use solver::GearSolverBody;
use solver::{GearJacobian, apply_impulse, apply_position_impulse, build_jacobian};

#[derive(Debug, Clone, Copy)]
enum GearSource {
    Revolute {
        local_anchor_moving: Vec2,
        reference_angle: f32,
    },
    Prismatic {
        local_anchor_base: Vec2,
        local_anchor_moving: Vec2,
        local_axis_base: Vec2,
    },
}

impl GearSource {
    fn from_definition(definition: JointDef) -> Option<Self> {
        match definition {
            JointDef::Revolute(definition) => Some(Self::from_revolute(definition)),
            JointDef::Prismatic(definition) => Some(Self::from_prismatic(definition)),
            _ => None,
        }
    }

    const fn from_revolute(definition: RevoluteJointDef) -> Self {
        Self::Revolute {
            local_anchor_moving: definition.local_anchor_b(),
            reference_angle: definition.reference_angle(),
        }
    }

    const fn from_prismatic(definition: PrismaticJointDef) -> Self {
        Self::Prismatic {
            local_anchor_base: definition.local_anchor_a(),
            local_anchor_moving: definition.local_anchor_b(),
            local_axis_base: definition.local_axis_a(),
        }
    }

    fn coordinate(self, base: GearBodyGeometry, moving: GearBodyGeometry) -> f32 {
        match self {
            Self::Revolute {
                reference_angle, ..
            } => moving.angle - base.angle - reference_angle,
            Self::Prismatic {
                local_anchor_base,
                local_anchor_moving,
                local_axis_base,
            } => {
                let base_rotation = Rotation::from_angle(base.angle);
                let moving_rotation = Rotation::from_angle(moving.angle);
                let moving_in_base = base_rotation.inverse_apply(
                    moving_rotation.apply(local_anchor_moving) + (moving.position - base.position),
                );
                (moving_in_base - local_anchor_base).dot(local_axis_base)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct GearBodyGeometry {
    position: Vec2,
    angle: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GearRuntime {
    source_joints: [JointId; 2],
    source_a: GearSource,
    source_b: GearSource,
    ratio: f32,
    constant: f32,
    impulse: f32,
    jacobian: GearJacobian,
}

impl GearRuntime {
    pub(super) fn new(
        definition: GearJointDef,
        source_definitions: &[JointDef; 2],
        geometries: [GearBodyGeometry; 4],
    ) -> Result<Self, JointMutationError> {
        let Some(source_a) = GearSource::from_definition(source_definitions[0]) else {
            return Err(JointMutationError::WrongKind {
                expected: JointKind::Revolute,
                actual: JointKind::from_definition(source_definitions[0]),
            });
        };
        let Some(source_b) = GearSource::from_definition(source_definitions[1]) else {
            return Err(JointMutationError::WrongKind {
                expected: JointKind::Revolute,
                actual: JointKind::from_definition(source_definitions[1]),
            });
        };
        let coordinate1 = source_a.coordinate(geometries[2], geometries[0]);
        let coordinate2 = source_b.coordinate(geometries[3], geometries[1]);
        let constant = coordinate1 + definition.ratio() * coordinate2;
        if !coordinate1.is_finite() || !coordinate2.is_finite() || !constant.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok(Self {
            source_joints: definition.source_joints(),
            source_a,
            source_b,
            ratio: definition.ratio(),
            constant,
            impulse: 0.0,
            jacobian: GearJacobian::ZERO,
        })
    }

    pub(super) fn reaction_force(self, inverse_timestep: f32) -> Vec2 {
        inverse_timestep * self.impulse * self.jacobian.linear_ac
    }

    pub(super) fn reaction_torque(self, inverse_timestep: f32) -> f32 {
        inverse_timestep * self.impulse * self.jacobian.angular_a
    }

    pub(super) fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio;
    }

    pub(super) fn initialize_velocity(
        &mut self,
        bodies: &[GearSolverBody; 4],
        warm_starting: bool,
    ) -> Result<(), JointMutationError> {
        if !bodies.iter().all(|body| body.is_valid()) {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        self.jacobian = build_jacobian(self.source_a, self.source_b, self.ratio, bodies)?;
        if !warm_starting {
            self.impulse = 0.0;
        }
        Ok(())
    }

    pub(super) fn warm_start(
        &self,
        bodies: &mut [GearSolverBody; 4],
    ) -> Result<(), JointMutationError> {
        let previous = *bodies;
        apply_impulse(bodies, self.jacobian, self.impulse);
        if !bodies.iter().all(|body| body.is_valid()) {
            *bodies = previous;
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok(())
    }

    pub(super) fn solve_velocity(
        &mut self,
        bodies: &mut [GearSolverBody; 4],
    ) -> Result<(), JointMutationError> {
        let previous_bodies = *bodies;
        let previous_impulse = self.impulse;
        let [body_a, body_b, body_c, body_d] = *bodies;
        let mut velocity_error = self
            .jacobian
            .linear_ac
            .dot(body_a.linear_velocity - body_c.linear_velocity)
            + self
                .jacobian
                .linear_bd
                .dot(body_b.linear_velocity - body_d.linear_velocity);
        velocity_error += self.jacobian.angular_a * body_a.angular_velocity
            - self.jacobian.angular_c * body_c.angular_velocity
            + self.jacobian.angular_b * body_b.angular_velocity
            - self.jacobian.angular_d * body_d.angular_velocity;
        let impulse = -self.jacobian.mass * velocity_error;
        self.impulse += impulse;
        apply_impulse(bodies, self.jacobian, impulse);
        if !self.impulse.is_finite() || !bodies.iter().all(|body| body.is_valid()) {
            *bodies = previous_bodies;
            self.impulse = previous_impulse;
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok(())
    }

    pub(super) fn solve_position(
        &self,
        bodies: &mut [GearSolverBody; 4],
    ) -> Result<bool, JointMutationError> {
        let previous = *bodies;
        let jacobian = build_jacobian(self.source_a, self.source_b, self.ratio, bodies)?;
        let coordinate1 = self
            .source_a
            .coordinate(bodies[2].geometry(), bodies[0].geometry());
        let coordinate2 = self
            .source_b
            .coordinate(bodies[3].geometry(), bodies[1].geometry());
        let position_error = (coordinate1 + self.ratio * coordinate2) - self.constant;
        let impulse = -position_error * jacobian.mass;
        apply_position_impulse(bodies, jacobian, impulse);
        if !position_error.is_finite()
            || !impulse.is_finite()
            || !bodies.iter().all(|body| body.is_valid())
        {
            *bodies = previous;
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        // The pinned implementation leaves linearError at zero and therefore reports success.
        Ok(true)
    }
}

pub(super) fn snapshot(
    world: &World,
    record: &JointRecord,
    runtime: GearRuntime,
) -> Result<JointSnapshot, JointQueryError> {
    let source1 = world.joints.get(runtime.source_joints[0])?;
    let source2 = world.joints.get(runtime.source_joints[1])?;
    let [body_a, body_b, body_c, body_d] = [
        source1.bodies[1],
        source2.bodies[1],
        source1.bodies[0],
        source2.bodies[0],
    ];
    let geometries = [body_a, body_b, body_c, body_d].map(|body| {
        world.bodies.get(body).map(|record| GearBodyGeometry {
            position: record.state.snapshot().position(),
            angle: record.state.snapshot().angle(),
        })
    });
    let [geometry_a, geometry_b, geometry_c, geometry_d] = geometries;
    let coordinate1 = runtime.source_a.coordinate(geometry_c?, geometry_a?);
    let coordinate2 = runtime.source_b.coordinate(geometry_d?, geometry_b?);
    let anchor_a = world
        .bodies
        .get(body_a)?
        .state
        .snapshot()
        .transform()
        .apply(match runtime.source_a {
            GearSource::Revolute {
                local_anchor_moving,
                ..
            }
            | GearSource::Prismatic {
                local_anchor_moving,
                ..
            } => local_anchor_moving,
        });
    let anchor_b = world
        .bodies
        .get(body_b)?
        .state
        .snapshot()
        .transform()
        .apply(match runtime.source_b {
            GearSource::Revolute {
                local_anchor_moving,
                ..
            }
            | GearSource::Prismatic {
                local_anchor_moving,
                ..
            } => local_anchor_moving,
        });
    if !coordinate1.is_finite()
        || !coordinate2.is_finite()
        || !anchor_a.is_valid()
        || !anchor_b.is_valid()
    {
        return Err(JointQueryError::NonFiniteDerivedState);
    }
    Ok(
        JointSnapshot::from_definition(record.definition, record.bodies).with_runtime(
            anchor_a,
            anchor_b,
            JointSpecificSnapshot::Gear(GearJointSnapshot::new(
                runtime.source_joints,
                [body_a, body_b, body_c, body_d],
                runtime.ratio,
                runtime.constant,
                coordinate1,
                coordinate2,
            )),
        ),
    )
}

impl World {
    /// Returns the current gear ratio.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid, poisoned, or wrong-kind joint.
    pub fn gear_joint_ratio(&self, joint: JointId) -> Result<f32, JointQueryError> {
        self.ensure_joint_queryable()?;
        let record = self.joints.get(joint)?;
        let JointRuntime::Gear(runtime) = record.runtime else {
            return Err(wrong_kind(record.definition));
        };
        Ok(runtime.ratio)
    }

    /// Replaces the ratio without waking bodies or recomputing the creation constant.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for a non-finite ratio, invalid identity, kind,
    /// locked world, or poisoned world.
    pub fn set_gear_ratio(&mut self, joint: JointId, ratio: f32) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        if !ratio.is_finite() {
            return Err(JointMutationError::InvalidValue);
        }
        let record = self.joints.get(joint)?;
        let JointDef::Gear(definition) = record.definition else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        let candidate = definition
            .with_ratio(ratio)
            .map_err(|_| JointMutationError::InvalidValue)?;
        let record = self.joint_mut_after_validation(joint);
        let JointRuntime::Gear(runtime) = &mut record.runtime else {
            return Err(JointMutationError::InvalidValue);
        };
        runtime.set_ratio(ratio);
        record.definition = candidate.into();
        Ok(())
    }
}

fn wrong_kind(definition: JointDef) -> JointQueryError {
    JointQueryError::WrongKind {
        expected: JointKind::Gear,
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

pub(super) fn body_geometry(
    world: &World,
    body: BodyId,
) -> Result<GearBodyGeometry, crate::HandleError> {
    let snapshot = world.bodies.get(body)?.state.snapshot();
    Ok(GearBodyGeometry {
        position: snapshot.position(),
        angle: snapshot.angle(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solver_body(linear_velocity: Vec2, angular_velocity: f32) -> GearSolverBody {
        GearSolverBody {
            center: Vec2::ZERO,
            angle: 0.0,
            linear_velocity,
            angular_velocity,
            local_center: Vec2::ZERO,
            inverse_mass: 1.0,
            inverse_inertia: 1.0,
        }
    }

    #[test]
    fn all_four_source_combinations_build_finite_four_body_jacobians() {
        // Arrange
        let revolute = GearSource::Revolute {
            local_anchor_moving: Vec2::ZERO,
            reference_angle: 0.0,
        };
        let prismatic = GearSource::Prismatic {
            local_anchor_base: Vec2::ZERO,
            local_anchor_moving: Vec2::new(0.0, 1.0),
            local_axis_base: Vec2::new(1.0, 0.0),
        };
        let bodies = [
            solver_body(Vec2::new(1.0, 0.0), 1.0),
            solver_body(Vec2::new(0.0, 1.0), -1.0),
            solver_body(Vec2::ZERO, 0.0),
            solver_body(Vec2::ZERO, 0.0),
        ];

        for (source_a, source_b, ratio) in [
            (revolute, revolute, 2.0),
            (revolute, prismatic, -0.5),
            (prismatic, revolute, 0.0),
            (prismatic, prismatic, 3.0),
        ] {
            // Act
            let jacobian =
                build_jacobian(source_a, source_b, ratio, &bodies).expect("finite Jacobian");

            // Assert
            assert!(jacobian.is_valid());
        }
    }

    #[test]
    fn all_four_source_combinations_solve_and_produce_reactions() {
        // Arrange
        let revolute = GearSource::Revolute {
            local_anchor_moving: Vec2::ZERO,
            reference_angle: 0.0,
        };
        let prismatic = GearSource::Prismatic {
            local_anchor_base: Vec2::ZERO,
            local_anchor_moving: Vec2::new(0.0, 1.0),
            local_axis_base: Vec2::new(1.0, 0.0),
        };
        let mut world = crate::World::new().expect("world");
        let ids = [
            world.create_body(&crate::BodyDef::default()).expect("A"),
            world.create_body(&crate::BodyDef::default()).expect("B"),
            world.create_body(&crate::BodyDef::default()).expect("C"),
            world.create_body(&crate::BodyDef::default()).expect("D"),
        ];
        let joint1 = world
            .create_joint(
                crate::RevoluteJointDef::new(ids[2], ids[0])
                    .expect("source 1")
                    .into(),
            )
            .expect("joint 1");
        let joint2 = world
            .create_joint(
                crate::RevoluteJointDef::new(ids[3], ids[1])
                    .expect("source 2")
                    .into(),
            )
            .expect("joint 2");
        let gear = world
            .create_joint(
                crate::GearJointDef::new(joint1, joint2)
                    .expect("gear")
                    .into(),
            )
            .expect("gear joint");
        let JointRuntime::Gear(base_runtime) = world.joints.get(gear).expect("gear").runtime else {
            panic!("expected gear runtime");
        };

        for (source_a, source_b, ratio) in [
            (revolute, revolute, 2.0),
            (revolute, prismatic, -0.5),
            (prismatic, revolute, 0.0),
            (prismatic, prismatic, 3.0),
        ] {
            let mut runtime = base_runtime;
            runtime.source_a = source_a;
            runtime.source_b = source_b;
            runtime.ratio = ratio;
            let mut bodies = [
                solver_body(Vec2::new(1.0, 0.0), 1.0),
                solver_body(Vec2::new(1.0, 0.0), 0.0),
                solver_body(Vec2::ZERO, 0.0),
                solver_body(Vec2::ZERO, 0.0),
            ];

            // Act
            runtime
                .initialize_velocity(&bodies, false)
                .expect("initialize");
            runtime.solve_velocity(&mut bodies).expect("velocity solve");
            bodies[0].center.x += 0.25;
            bodies[0].angle += 0.25;
            let solved_position = runtime.solve_position(&mut bodies).expect("position solve");
            let force = runtime.reaction_force(2.0);
            let torque = runtime.reaction_torque(2.0);

            // Assert
            assert!(solved_position);
            assert!(bodies.iter().all(|body| body.is_valid()));
            assert!(force.is_valid());
            assert!(torque.is_finite());
            assert!(
                force != Vec2::ZERO || torque != 0.0 || ratio == 0.0,
                "ratio {ratio} produced no reaction for {source_a:?}/{source_b:?}"
            );
        }
    }
}
