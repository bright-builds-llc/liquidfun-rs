use crate::collision::{ContactFeatureId, Shape, world_manifold};
use crate::math::settings::{
    BAUMGARTE, LINEAR_SLOP, MAX_LINEAR_CORRECTION, MAX_ROTATION, MAX_ROTATION_SQUARED,
    MAX_TRANSLATION, MAX_TRANSLATION_SQUARED, VELOCITY_THRESHOLD,
};
use crate::math::{Transform, Vec2, clamp, max};

use super::body::BodyState;
use super::contact::{Contact, ManagedContactSnapshot};

const WITNESS_TIME_STEP_BITS: u32 = 1_015_580_809;
const VELOCITY_ITERATIONS: usize = 8;
const POSITION_ITERATIONS: usize = 3;
const WARM_START_RATIO: f32 = 1.0;
const MAX_CONDITION_NUMBER: f32 = 1_000.0;

/// Owned post-solve evidence for one private manager contact occurrence.
#[derive(Debug, Clone, PartialEq)]
pub struct ContactSolve {
    contact: ManagedContactSnapshot,
}

impl ContactSolve {
    pub(super) const fn new(contact: ManagedContactSnapshot) -> Self {
        Self { contact }
    }

    /// Returns owned post-solve manifold, material, and warm-start state.
    #[must_use]
    pub const fn contact(&self) -> &ManagedContactSnapshot {
        &self.contact
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ContactSolveFailure {
    UnsupportedTopology,
    NonFinite,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SolvedBodyMotion {
    pub(super) position: Vec2,
    pub(super) angle: f32,
    pub(super) linear: Vec2,
    pub(super) angular: f32,
}

#[derive(Debug)]
pub(super) struct ContactSolveCommit {
    pub(super) first_motion: SolvedBodyMotion,
    pub(super) second_motion: SolvedBodyMotion,
    pub(super) impulses: Vec<(ContactFeatureId, f32, f32)>,
}

#[derive(Debug, Clone, Copy)]
struct SolverBody {
    center: Vec2,
    local_center: Vec2,
    angle: f32,
    transform: Transform,
    linear_velocity: Vec2,
    angular_velocity: f32,
    inverse_mass: f32,
    inverse_inertia: f32,
}

impl SolverBody {
    fn from_state(state: BodyState) -> Result<Self, ContactSolveFailure> {
        let body = Self {
            center: state.sweep().center(),
            local_center: state.sweep().local_center(),
            angle: state.snapshot().angle(),
            transform: state.transform(),
            linear_velocity: state.solver_linear(),
            angular_velocity: state.solver_angular(),
            inverse_mass: state.inverse_mass(),
            inverse_inertia: state.inverse_inertia(),
        };
        if !body.is_finite() {
            return Err(ContactSolveFailure::NonFinite);
        }
        Ok(body)
    }

    fn is_finite(self) -> bool {
        self.center.is_valid()
            && self.transform.position().is_valid()
            && self.transform.rotation().sine().is_finite()
            && self.transform.rotation().cosine().is_finite()
            && self.angle.is_finite()
            && self.linear_velocity.is_valid()
            && self.angular_velocity.is_finite()
            && self.inverse_mass.is_finite()
            && self.inverse_inertia.is_finite()
    }
}

#[derive(Debug, Clone, Copy)]
struct ConstraintPoint {
    feature_id: ContactFeatureId,
    r_a: Vec2,
    r_b: Vec2,
    normal_impulse: f32,
    tangent_impulse: f32,
    normal_mass: f32,
    tangent_mass: f32,
    velocity_bias: f32,
}

impl ConstraintPoint {
    const fn cold(feature_id: ContactFeatureId) -> Self {
        Self {
            feature_id,
            r_a: Vec2::ZERO,
            r_b: Vec2::ZERO,
            normal_impulse: 0.0,
            tangent_impulse: 0.0,
            normal_mass: 0.0,
            tangent_mass: 0.0,
            velocity_bias: 0.0,
        }
    }

    fn is_finite(self) -> bool {
        self.r_a.is_valid()
            && self.r_b.is_valid()
            && self.normal_impulse.is_finite()
            && self.tangent_impulse.is_finite()
            && self.normal_mass.is_finite()
            && self.tangent_mass.is_finite()
            && self.velocity_bias.is_finite()
    }
}

#[derive(Debug)]
struct VelocityConstraint {
    points: [ConstraintPoint; 2],
    point_count: usize,
    normal: Vec2,
    friction: f32,
    k: [[f32; 2]; 2],
    normal_mass: [[f32; 2]; 2],
}

#[allow(clippy::too_many_arguments)]
pub(super) fn solve_contact(
    contact: &Contact,
    first_state: BodyState,
    second_state: BodyState,
    first_shape: &Shape,
    second_shape: &Shape,
) -> Result<ContactSolveCommit, ContactSolveFailure> {
    let witness_time_step = f32::from_bits(WITNESS_TIME_STEP_BITS);
    if !witness_time_step.is_finite() || witness_time_step <= 0.0 {
        return Err(ContactSolveFailure::NonFinite);
    }

    let mut first = SolverBody::from_state(first_state)?;
    let mut second = SolverBody::from_state(second_state)?;
    let mut constraint = build_constraint(contact, first, second, first_shape, second_shape)?;
    warm_start(&constraint, &mut first, &mut second);
    for _iteration in 0..VELOCITY_ITERATIONS {
        solve_velocity_constraints(&mut constraint, &mut first, &mut second);
    }
    integrate_position(&mut first, witness_time_step);
    integrate_position(&mut second, witness_time_step);
    for _iteration in 0..POSITION_ITERATIONS {
        if solve_position_constraints(contact, &mut first, &mut second, first_shape, second_shape)?
        {
            break;
        }
    }
    validate_solution(&constraint, first, second)?;

    let impulses = constraint.points[..constraint.point_count]
        .iter()
        .map(|point| {
            (
                point.feature_id,
                point.normal_impulse,
                point.tangent_impulse,
            )
        })
        .collect();
    Ok(ContactSolveCommit {
        first_motion: SolvedBodyMotion {
            position: first.transform.position(),
            angle: first.angle,
            linear: first.linear_velocity,
            angular: first.angular_velocity,
        },
        second_motion: SolvedBodyMotion {
            position: second.transform.position(),
            angle: second.angle,
            linear: second.linear_velocity,
            angular: second.angular_velocity,
        },
        impulses,
    })
}

fn integrate_position(body: &mut SolverBody, time_step: f32) {
    let mut translation = time_step * body.linear_velocity;
    if translation.length_squared() > MAX_TRANSLATION_SQUARED {
        body.linear_velocity *= MAX_TRANSLATION / translation.length();
        translation = time_step * body.linear_velocity;
    }
    let mut rotation = time_step * body.angular_velocity;
    if rotation * rotation > MAX_ROTATION_SQUARED {
        body.angular_velocity *= MAX_ROTATION / rotation.abs();
        rotation = time_step * body.angular_velocity;
    }
    body.center += translation;
    body.angle += rotation;
    body.synchronize_transform();
}

fn solve_position_constraints(
    contact: &Contact,
    first: &mut SolverBody,
    second: &mut SolverBody,
    first_shape: &Shape,
    second_shape: &Shape,
) -> Result<bool, ContactSolveFailure> {
    let Some(manifold) = contact.maybe_manifold.as_ref() else {
        return Err(ContactSolveFailure::UnsupportedTopology);
    };
    let mut minimum_separation = 0.0_f32;
    for point_index in 0..manifold.points().len() {
        let maybe_world = world_manifold(
            manifold,
            first.transform,
            shape_radius(first_shape),
            second.transform,
            shape_radius(second_shape),
        )
        .map_err(|_error| ContactSolveFailure::NonFinite)?;
        let Some(world) = maybe_world else {
            return Err(ContactSolveFailure::UnsupportedTopology);
        };
        let Some(point) = world.points().get(point_index).copied() else {
            return Err(ContactSolveFailure::UnsupportedTopology);
        };
        let normal = world.normal();
        let r_a = point.point() - first.center;
        let r_b = point.point() - second.center;
        minimum_separation = minimum_separation.min(point.separation());
        let correction = clamp(
            BAUMGARTE * (point.separation() + LINEAR_SLOP),
            -MAX_LINEAR_CORRECTION,
            0.0,
        );
        let normal_arm_a = r_a.cross(normal);
        let normal_arm_b = r_b.cross(normal);
        let effective_mass = first.inverse_mass
            + second.inverse_mass
            + first.inverse_inertia * normal_arm_a * normal_arm_a
            + second.inverse_inertia * normal_arm_b * normal_arm_b;
        let impulse = if effective_mass > 0.0 {
            -correction / effective_mass
        } else {
            0.0
        };
        let position_impulse = impulse * normal;
        first.center -= first.inverse_mass * position_impulse;
        first.angle -= first.inverse_inertia * r_a.cross(position_impulse);
        second.center += second.inverse_mass * position_impulse;
        second.angle += second.inverse_inertia * r_b.cross(position_impulse);
        first.synchronize_transform();
        second.synchronize_transform();
    }
    Ok(minimum_separation >= -3.0 * LINEAR_SLOP)
}

impl SolverBody {
    fn synchronize_transform(&mut self) {
        let rotation = crate::math::Rotation::from_angle(self.angle);
        self.transform = Transform::from_position_angle(
            self.center - rotation.apply(self.local_center),
            self.angle,
        );
    }
}

fn build_constraint(
    contact: &Contact,
    first: SolverBody,
    second: SolverBody,
    first_shape: &Shape,
    second_shape: &Shape,
) -> Result<VelocityConstraint, ContactSolveFailure> {
    let Some(manifold) = contact.maybe_manifold.as_ref() else {
        return Err(ContactSolveFailure::UnsupportedTopology);
    };
    if contact.points.is_empty() || contact.points.len() > 2 {
        return Err(ContactSolveFailure::UnsupportedTopology);
    }
    let maybe_world_manifold = world_manifold(
        manifold,
        first.transform,
        shape_radius(first_shape),
        second.transform,
        shape_radius(second_shape),
    )
    .map_err(|_error| ContactSolveFailure::NonFinite)?;
    let Some(world) = maybe_world_manifold else {
        return Err(ContactSolveFailure::UnsupportedTopology);
    };
    if world.points().len() != contact.points.len() {
        return Err(ContactSolveFailure::UnsupportedTopology);
    }

    let placeholder = ConstraintPoint::cold(contact.points[0].feature_id());
    let mut constraint = VelocityConstraint {
        points: [placeholder; 2],
        point_count: contact.points.len(),
        normal: world.normal(),
        friction: contact.friction,
        k: [[0.0; 2]; 2],
        normal_mass: [[0.0; 2]; 2],
    };
    let tangent = constraint.normal.cross_scalar(1.0);
    for (index, (contact_point, world_point)) in
        contact.points.iter().zip(world.points()).enumerate()
    {
        let r_a = world_point.point() - first.center;
        let r_b = world_point.point() - second.center;
        let normal_arms = [r_a.cross(constraint.normal), r_b.cross(constraint.normal)];
        let normal_k = first.inverse_mass
            + second.inverse_mass
            + first.inverse_inertia * normal_arms[0] * normal_arms[0]
            + second.inverse_inertia * normal_arms[1] * normal_arms[1];
        let tangent_arms = [r_a.cross(tangent), r_b.cross(tangent)];
        let tangent_k = first.inverse_mass
            + second.inverse_mass
            + first.inverse_inertia * tangent_arms[0] * tangent_arms[0]
            + second.inverse_inertia * tangent_arms[1] * tangent_arms[1];
        let relative = second.linear_velocity + Vec2::scalar_cross(second.angular_velocity, r_b)
            - first.linear_velocity
            - Vec2::scalar_cross(first.angular_velocity, r_a);
        let relative_normal_velocity = constraint.normal.dot(relative);
        let velocity_bias = if relative_normal_velocity < -VELOCITY_THRESHOLD {
            -contact.restitution * relative_normal_velocity
        } else {
            0.0
        };
        constraint.points[index] = ConstraintPoint {
            feature_id: contact_point.feature_id(),
            r_a,
            r_b,
            normal_impulse: WARM_START_RATIO * contact_point.normal_impulse(),
            tangent_impulse: WARM_START_RATIO * contact_point.tangent_impulse(),
            normal_mass: if normal_k > 0.0 { 1.0 / normal_k } else { 0.0 },
            tangent_mass: if tangent_k > 0.0 {
                1.0 / tangent_k
            } else {
                0.0
            },
            velocity_bias,
        };
    }
    prepare_two_point_block(&mut constraint, first, second);
    if !constraint.normal.is_valid()
        || !constraint.friction.is_finite()
        || constraint.friction < 0.0
        || constraint.points[..constraint.point_count]
            .iter()
            .any(|point| !point.is_finite())
    {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(constraint)
}

fn prepare_two_point_block(
    constraint: &mut VelocityConstraint,
    first: SolverBody,
    second: SolverBody,
) {
    if constraint.point_count != 2 {
        return;
    }
    let first_point = constraint.points[0];
    let second_point = constraint.points[1];
    let normal_arms = [
        [
            first_point.r_a.cross(constraint.normal),
            first_point.r_b.cross(constraint.normal),
        ],
        [
            second_point.r_a.cross(constraint.normal),
            second_point.r_b.cross(constraint.normal),
        ],
    ];
    let k11 = first.inverse_mass
        + second.inverse_mass
        + first.inverse_inertia * normal_arms[0][0] * normal_arms[0][0]
        + second.inverse_inertia * normal_arms[0][1] * normal_arms[0][1];
    let k22 = first.inverse_mass
        + second.inverse_mass
        + first.inverse_inertia * normal_arms[1][0] * normal_arms[1][0]
        + second.inverse_inertia * normal_arms[1][1] * normal_arms[1][1];
    let k12 = first.inverse_mass
        + second.inverse_mass
        + first.inverse_inertia * normal_arms[0][0] * normal_arms[1][0]
        + second.inverse_inertia * normal_arms[0][1] * normal_arms[1][1];
    let determinant = k11 * k22 - k12 * k12;
    if k11 * k11 >= MAX_CONDITION_NUMBER * determinant || determinant == 0.0 {
        constraint.point_count = 1;
        return;
    }
    constraint.k = [[k11, k12], [k12, k22]];
    let inverse = 1.0 / determinant;
    constraint.normal_mass = [
        [inverse * k22, -inverse * k12],
        [-inverse * k12, inverse * k11],
    ];
}

fn warm_start(constraint: &VelocityConstraint, first: &mut SolverBody, second: &mut SolverBody) {
    let tangent = constraint.normal.cross_scalar(1.0);
    for point in &constraint.points[..constraint.point_count] {
        let impulse = point.normal_impulse * constraint.normal + point.tangent_impulse * tangent;
        first.angular_velocity -= first.inverse_inertia * point.r_a.cross(impulse);
        first.linear_velocity -= first.inverse_mass * impulse;
        second.angular_velocity += second.inverse_inertia * point.r_b.cross(impulse);
        second.linear_velocity += second.inverse_mass * impulse;
    }
}

fn solve_velocity_constraints(
    constraint: &mut VelocityConstraint,
    first: &mut SolverBody,
    second: &mut SolverBody,
) {
    solve_tangent_constraints(constraint, first, second);
    if constraint.point_count == 1 {
        solve_one_normal_constraint(constraint, first, second);
    } else {
        solve_two_normal_constraints(constraint, first, second);
    }
}

fn solve_tangent_constraints(
    constraint: &mut VelocityConstraint,
    first: &mut SolverBody,
    second: &mut SolverBody,
) {
    let tangent = constraint.normal.cross_scalar(1.0);
    for point in &mut constraint.points[..constraint.point_count] {
        let relative = second.linear_velocity
            + Vec2::scalar_cross(second.angular_velocity, point.r_b)
            - first.linear_velocity
            - Vec2::scalar_cross(first.angular_velocity, point.r_a);
        let lambda = point.tangent_mass * -relative.dot(tangent);
        let maximum_friction = constraint.friction * point.normal_impulse;
        let new_impulse =
            (point.tangent_impulse + lambda).clamp(-maximum_friction, maximum_friction);
        let applied = new_impulse - point.tangent_impulse;
        point.tangent_impulse = new_impulse;
        apply_impulse(applied * tangent, *point, first, second);
    }
}

fn solve_one_normal_constraint(
    constraint: &mut VelocityConstraint,
    first: &mut SolverBody,
    second: &mut SolverBody,
) {
    let point = &mut constraint.points[0];
    let relative = second.linear_velocity + Vec2::scalar_cross(second.angular_velocity, point.r_b)
        - first.linear_velocity
        - Vec2::scalar_cross(first.angular_velocity, point.r_a);
    let lambda = -point.normal_mass * (relative.dot(constraint.normal) - point.velocity_bias);
    let new_impulse = max(point.normal_impulse + lambda, 0.0);
    let applied = new_impulse - point.normal_impulse;
    point.normal_impulse = new_impulse;
    apply_impulse(applied * constraint.normal, *point, first, second);
}

fn solve_two_normal_constraints(
    constraint: &mut VelocityConstraint,
    first: &mut SolverBody,
    second: &mut SolverBody,
) {
    let first_point = constraint.points[0];
    let second_point = constraint.points[1];
    let accumulated = [first_point.normal_impulse, second_point.normal_impulse];
    let relative_first = second.linear_velocity
        + Vec2::scalar_cross(second.angular_velocity, first_point.r_b)
        - first.linear_velocity
        - Vec2::scalar_cross(first.angular_velocity, first_point.r_a);
    let relative_second = second.linear_velocity
        + Vec2::scalar_cross(second.angular_velocity, second_point.r_b)
        - first.linear_velocity
        - Vec2::scalar_cross(first.angular_velocity, second_point.r_a);
    let mut b = [
        relative_first.dot(constraint.normal) - first_point.velocity_bias,
        relative_second.dot(constraint.normal) - second_point.velocity_bias,
    ];
    b[0] -= constraint.k[0][0] * accumulated[0] + constraint.k[0][1] * accumulated[1];
    b[1] -= constraint.k[1][0] * accumulated[0] + constraint.k[1][1] * accumulated[1];

    let both = [
        -(constraint.normal_mass[0][0] * b[0] + constraint.normal_mass[0][1] * b[1]),
        -(constraint.normal_mass[1][0] * b[0] + constraint.normal_mass[1][1] * b[1]),
    ];
    let maybe_solution = if both[0] >= 0.0 && both[1] >= 0.0 {
        Some(both)
    } else {
        let first_only = [-first_point.normal_mass * b[0], 0.0];
        let second_velocity = constraint.k[1][0] * first_only[0] + b[1];
        if first_only[0] >= 0.0 && second_velocity >= 0.0 {
            Some(first_only)
        } else {
            let second_only = [0.0, -second_point.normal_mass * b[1]];
            let first_velocity = constraint.k[0][1] * second_only[1] + b[0];
            if second_only[1] >= 0.0 && first_velocity >= 0.0 {
                Some(second_only)
            } else if b[0] >= 0.0 && b[1] >= 0.0 {
                Some([0.0, 0.0])
            } else {
                None
            }
        }
    };
    let Some(solution) = maybe_solution else {
        return;
    };
    let delta = [solution[0] - accumulated[0], solution[1] - accumulated[1]];
    apply_two_impulses(
        delta,
        [first_point, second_point],
        constraint.normal,
        first,
        second,
    );
    constraint.points[0].normal_impulse = solution[0];
    constraint.points[1].normal_impulse = solution[1];
}

fn apply_impulse(
    impulse: Vec2,
    point: ConstraintPoint,
    first: &mut SolverBody,
    second: &mut SolverBody,
) {
    first.linear_velocity -= first.inverse_mass * impulse;
    first.angular_velocity -= first.inverse_inertia * point.r_a.cross(impulse);
    second.linear_velocity += second.inverse_mass * impulse;
    second.angular_velocity += second.inverse_inertia * point.r_b.cross(impulse);
}

fn apply_two_impulses(
    delta: [f32; 2],
    points: [ConstraintPoint; 2],
    normal: Vec2,
    first: &mut SolverBody,
    second: &mut SolverBody,
) {
    let first_impulse = delta[0] * normal;
    let second_impulse = delta[1] * normal;
    let total = first_impulse + second_impulse;
    first.linear_velocity -= first.inverse_mass * total;
    first.angular_velocity -= first.inverse_inertia
        * (points[0].r_a.cross(first_impulse) + points[1].r_a.cross(second_impulse));
    second.linear_velocity += second.inverse_mass * total;
    second.angular_velocity += second.inverse_inertia
        * (points[0].r_b.cross(first_impulse) + points[1].r_b.cross(second_impulse));
}

fn validate_solution(
    constraint: &VelocityConstraint,
    first: SolverBody,
    second: SolverBody,
) -> Result<(), ContactSolveFailure> {
    if !first.is_finite()
        || !second.is_finite()
        || constraint.points[..constraint.point_count]
            .iter()
            .any(|point| !point.is_finite())
    {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(())
}

const fn shape_radius(shape: &Shape) -> f32 {
    match shape {
        Shape::Circle(circle) => circle.radius(),
        Shape::Edge(edge) => edge.radius(),
        Shape::Polygon(polygon) => polygon.radius(),
        Shape::Chain(chain) => chain.radius(),
    }
}

#[cfg(test)]
mod tests {
    use crate::collision::{CircleShape, FilterData};
    use crate::{
        BodyDef, BodyType, FixtureDef, StepConfiguration, StepError, StepHook, StepLimits, World,
    };

    use super::*;

    struct NoopHook;

    impl StepHook for NoopHook {}

    fn phase6_step_configuration() -> StepConfiguration {
        StepConfiguration::new(1.0 / 60.0, 8, 3).expect("fixed test configuration should be valid")
    }

    fn body_definition(body_type: BodyType, position: Vec2) -> BodyDef {
        BodyDef::new(body_type, position, 0.0, true).expect("test body should be valid")
    }

    fn fixture_definition() -> FixtureDef {
        let shape =
            Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test circle should be valid"));
        FixtureDef::new(shape, 1.0, 0.5, 0.25, false, FilterData::default())
            .expect("test fixture should be valid")
    }

    #[test]
    fn unsupported_preflight_preserves_seeded_velocities_and_all_impulses() {
        // Arrange
        let mut world = World::new().expect("test world should be available");
        let static_body = world
            .create_body(&body_definition(BodyType::Static, Vec2::ZERO))
            .expect("static body should fit");
        let first_dynamic = world
            .create_body(&body_definition(BodyType::Dynamic, Vec2::new(1.5, 0.0)))
            .expect("first dynamic body should fit");
        let static_fixture = world
            .create_fixture(static_body, &fixture_definition())
            .expect("static fixture should fit");
        let first_dynamic_fixture = world
            .create_fixture(first_dynamic, &fixture_definition())
            .expect("first dynamic fixture should fit");
        let mut hook = NoopHook;
        world
            .step(
                phase6_step_configuration(),
                &mut hook,
                StepLimits::default(),
            )
            .expect("one supported contact should solve");
        world.seed_first_contact_impulses_for_test(2.0, -0.5);
        world.set_body_solver_velocity_for_test(first_dynamic, Vec2::new(3.0, -4.0), 0.75);
        let second_dynamic = world
            .create_body(&body_definition(BodyType::Dynamic, Vec2::new(-1.5, 0.0)))
            .expect("second dynamic body should fit");
        world
            .create_fixture(second_dynamic, &fixture_definition())
            .expect("second dynamic fixture should fit");
        world.set_body_solver_velocity_for_test(second_dynamic, Vec2::new(-2.0, 5.0), -0.25);
        let first_velocity_before = world.body_solver_velocity_for_test(first_dynamic);
        let second_velocity_before = world.body_solver_velocity_for_test(second_dynamic);
        let contacts_before = world.contact_snapshots_for_test();

        // Act
        let error = world
            .step(
                phase6_step_configuration(),
                &mut hook,
                StepLimits::default(),
            )
            .expect_err("multi-contact topology should fail closed");

        // Assert
        assert!(matches!(error, StepError::UnsupportedSolverTopology { .. }));
        assert_eq!(
            world.body_solver_velocity_for_test(first_dynamic),
            first_velocity_before
        );
        assert_eq!(
            world.body_solver_velocity_for_test(second_dynamic),
            second_velocity_before
        );
        let contacts_after = world.contact_snapshots_for_test();
        let existing_before = contacts_before
            .iter()
            .find(|contact| {
                contact.fixtures() == [static_fixture, first_dynamic_fixture]
                    || contact.fixtures() == [first_dynamic_fixture, static_fixture]
            })
            .expect("existing contact should be captured before preflight");
        let existing_after = contacts_after
            .iter()
            .find(|contact| {
                contact.fixtures() == [static_fixture, first_dynamic_fixture]
                    || contact.fixtures() == [first_dynamic_fixture, static_fixture]
            })
            .expect("existing contact should remain after preflight");
        assert_eq!(existing_before.points(), existing_after.points());
        assert_eq!(
            existing_after.points()[0].normal_impulse().to_bits(),
            2.0_f32.to_bits()
        );
        assert_eq!(
            existing_after.points()[0].tangent_impulse().to_bits(),
            (-0.5_f32).to_bits()
        );
        for contact in contacts_after
            .iter()
            .filter(|contact| contact.fixtures() != existing_after.fixtures())
        {
            assert!(contact.points().iter().all(|point| {
                point.normal_impulse().to_bits() == 0 && point.tangent_impulse().to_bits() == 0
            }));
        }
    }
}
