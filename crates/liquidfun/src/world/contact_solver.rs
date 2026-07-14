use crate::collision::{ContactFeatureId, Manifold, Shape, world_manifold};
use crate::math::settings::{
    BAUMGARTE, LINEAR_SLOP, MAX_LINEAR_CORRECTION, MAX_ROTATION, MAX_ROTATION_SQUARED,
    MAX_TRANSLATION, MAX_TRANSLATION_SQUARED, TOI_BAUMGARTE, VELOCITY_THRESHOLD,
};
use crate::math::{Transform, Vec2, clamp, max};

use super::body::BodyState;
use super::config::StepConfiguration;
use super::contact::{Contact, ContactPoint, ManagedContactSnapshot};
use super::joint::solver::{
    JointConstraintInput, JointImpulseSolution, build_constraints as build_joint_constraints,
    solve_position as solve_joint_position, solve_velocity as solve_joint_velocity,
    transient_impulses as transient_joint_impulses, warm_start as warm_start_joints,
};

const MAX_CONDITION_NUMBER: f32 = 1_000.0;

mod toi;
pub(super) use toi::solve_toi_constraints;

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
    InvalidProxyBounds,
    CapacityExceeded {
        resource: &'static str,
        limit: usize,
    },
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SolvedBodyMotion {
    pub(super) position: Vec2,
    pub(super) angle: f32,
    pub(super) linear: Vec2,
    pub(super) angular: f32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SolverBody {
    pub(super) center: Vec2,
    pub(super) local_center: Vec2,
    pub(super) angle: f32,
    pub(super) transform: Transform,
    pub(super) linear_velocity: Vec2,
    pub(super) angular_velocity: f32,
    pub(super) inverse_mass: f32,
    pub(super) inverse_inertia: f32,
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

    pub(super) fn is_finite(self) -> bool {
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
    contact_index: usize,
    first_body_index: usize,
    second_body_index: usize,
    manifold: Manifold,
    first_radius: f32,
    second_radius: f32,
    points: [ConstraintPoint; 2],
    point_count: usize,
    normal: Vec2,
    friction: f32,
    tangent_speed: f32,
    k: [[f32; 2]; 2],
    normal_mass: [[f32; 2]; 2],
}

pub(super) struct ContactConstraintInput<'a> {
    pub(super) contact_index: usize,
    pub(super) first_body_index: usize,
    pub(super) second_body_index: usize,
    pub(super) contact: &'a Contact,
    pub(super) first_shape: &'a Shape,
    pub(super) second_shape: &'a Shape,
}

#[derive(Debug)]
pub(super) struct ContactImpulseSolution {
    pub(super) contact_index: usize,
    pub(super) impulses: Vec<(ContactFeatureId, f32, f32)>,
}

#[derive(Debug)]
pub(super) struct IslandConstraintSolution {
    pub(super) motions: Vec<SolvedBodyMotion>,
    pub(super) contact_impulses: Vec<ContactImpulseSolution>,
    pub(super) joint_impulses: Vec<JointImpulseSolution>,
    pub(super) position_solved: bool,
}

fn build_constraints(
    inputs: &[ContactConstraintInput<'_>],
    bodies: &[SolverBody],
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<Vec<VelocityConstraint>, ContactSolveFailure> {
    let mut constraints = Vec::new();
    constraints.try_reserve_exact(inputs.len()).map_err(|_| {
        ContactSolveFailure::CapacityExceeded {
            resource: "island contact constraints",
            limit: inputs.len(),
        }
    })?;
    for input in inputs {
        constraints.push(build_constraint(
            input,
            bodies,
            time_step_ratio,
            warm_starting,
        )?);
    }
    Ok(constraints)
}

fn transient_impulses(
    constraints: &[VelocityConstraint],
) -> Result<Vec<ContactImpulseSolution>, ContactSolveFailure> {
    let mut contact_impulses = Vec::new();
    contact_impulses
        .try_reserve_exact(constraints.len())
        .map_err(|_| ContactSolveFailure::CapacityExceeded {
            resource: "island contact impulses",
            limit: constraints.len(),
        })?;
    for constraint in constraints {
        contact_impulses.push(ContactImpulseSolution {
            contact_index: constraint.contact_index,
            impulses: constraint.points[..constraint.point_count]
                .iter()
                .map(|point| {
                    (
                        point.feature_id,
                        point.normal_impulse,
                        point.tangent_impulse,
                    )
                })
                .collect(),
        });
    }
    Ok(contact_impulses)
}

pub(super) fn solve_island_constraints(
    body_states: &[BodyState],
    inputs: &[ContactConstraintInput<'_>],
    joint_inputs: &[JointConstraintInput<'_>],
    gravity: Vec2,
    configuration: StepConfiguration,
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<IslandConstraintSolution, ContactSolveFailure> {
    let mut bodies = Vec::new();
    bodies.try_reserve_exact(body_states.len()).map_err(|_| {
        ContactSolveFailure::CapacityExceeded {
            resource: "island solver bodies",
            limit: body_states.len(),
        }
    })?;
    for state in body_states {
        let mut body = SolverBody::from_state(*state)?;
        integrate_velocity(&mut body, *state, gravity, configuration.time_step())?;
        bodies.push(body);
    }

    let mut constraints = build_constraints(inputs, &bodies, time_step_ratio, warm_starting)?;
    let mut joint_constraints = build_joint_constraints(
        joint_inputs,
        &bodies,
        configuration.time_step(),
        time_step_ratio,
        warm_starting,
    )?;

    if warm_starting {
        for constraint in &constraints {
            warm_start(constraint, &mut bodies)?;
        }
        warm_start_joints(&joint_constraints, &mut bodies)?;
    }
    for _iteration in 0..configuration.velocity_iterations() {
        for constraint in &mut joint_constraints {
            solve_joint_velocity(constraint, &mut bodies)?;
        }
        for constraint in &mut constraints {
            solve_velocity_constraints(constraint, &mut bodies)?;
        }
    }

    let contact_impulses = transient_impulses(&constraints)?;
    let joint_impulses = transient_joint_impulses(&joint_constraints);

    for body in &mut bodies {
        integrate_position(body, configuration.time_step());
    }
    let mut position_solved = false;
    for _iteration in 0..configuration.position_iterations() {
        position_solved = true;
        for constraint in &constraints {
            position_solved =
                solve_position_constraints(constraint, &mut bodies)? && position_solved;
        }
        for constraint in &joint_constraints {
            position_solved = solve_joint_position(*constraint, &mut bodies)? && position_solved;
        }
        if position_solved {
            break;
        }
    }

    for body in &bodies {
        if !body.is_finite() {
            return Err(ContactSolveFailure::NonFinite);
        }
    }
    for constraint in &constraints {
        validate_solution(constraint, &bodies)?;
    }

    Ok(IslandConstraintSolution {
        motions: bodies
            .into_iter()
            .map(|body| SolvedBodyMotion {
                position: body.transform.position(),
                angle: body.angle,
                linear: body.linear_velocity,
                angular: body.angular_velocity,
            })
            .collect(),
        contact_impulses,
        joint_impulses,
        position_solved,
    })
}

fn integrate_velocity(
    body: &mut SolverBody,
    state: BodyState,
    gravity: Vec2,
    time_step: f32,
) -> Result<(), ContactSolveFailure> {
    if state.snapshot().body_type() != super::body::BodyType::Dynamic {
        return Ok(());
    }
    body.linear_velocity += time_step
        * (state.snapshot().gravity_scale() * gravity
            + state.inverse_mass() * state.accumulated_force());
    body.angular_velocity += time_step * state.inverse_inertia() * state.accumulated_torque();
    body.linear_velocity *= 1.0 / (1.0 + time_step * state.snapshot().linear_damping());
    body.angular_velocity *= 1.0 / (1.0 + time_step * state.snapshot().angular_damping());
    if !body.is_finite() {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(())
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
    constraint: &VelocityConstraint,
    bodies: &mut [SolverBody],
) -> Result<bool, ContactSolveFailure> {
    let (mut first, mut second) = constraint_bodies(constraint, bodies)?;
    let mut minimum_separation = 0.0_f32;
    for point_index in 0..constraint.manifold.points().len() {
        let maybe_world = world_manifold(
            &constraint.manifold,
            first.transform,
            constraint.first_radius,
            second.transform,
            constraint.second_radius,
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
    store_constraint_bodies(constraint, bodies, first, second)?;
    Ok(minimum_separation >= -3.0 * LINEAR_SLOP)
}

impl SolverBody {
    pub(super) fn synchronize_transform(&mut self) {
        let rotation = crate::math::Rotation::from_angle(self.angle);
        self.transform = Transform::from_position_angle(
            self.center - rotation.apply(self.local_center),
            self.angle,
        );
    }
}

fn build_constraint(
    input: &ContactConstraintInput<'_>,
    bodies: &[SolverBody],
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<VelocityConstraint, ContactSolveFailure> {
    if input.first_body_index == input.second_body_index {
        return Err(ContactSolveFailure::UnsupportedTopology);
    }
    let first = *bodies
        .get(input.first_body_index)
        .ok_or(ContactSolveFailure::UnsupportedTopology)?;
    let second = *bodies
        .get(input.second_body_index)
        .ok_or(ContactSolveFailure::UnsupportedTopology)?;
    let contact = input.contact;
    let Some(manifold) = contact.maybe_manifold.as_ref() else {
        return Err(ContactSolveFailure::UnsupportedTopology);
    };
    if contact.points.is_empty() || contact.points.len() > 2 {
        return Err(ContactSolveFailure::UnsupportedTopology);
    }
    let maybe_world_manifold = world_manifold(
        manifold,
        first.transform,
        shape_radius(input.first_shape),
        second.transform,
        shape_radius(input.second_shape),
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
        contact_index: input.contact_index,
        first_body_index: input.first_body_index,
        second_body_index: input.second_body_index,
        manifold: manifold.clone(),
        first_radius: shape_radius(input.first_shape),
        second_radius: shape_radius(input.second_shape),
        points: [placeholder; 2],
        point_count: contact.points.len(),
        normal: world.normal(),
        friction: contact.friction,
        tangent_speed: contact.tangent_speed,
        k: [[0.0; 2]; 2],
        normal_mass: [[0.0; 2]; 2],
    };
    let tangent = constraint.normal.cross_scalar(1.0);
    for (index, (contact_point, world_point)) in
        contact.points.iter().zip(world.points()).enumerate()
    {
        constraint.points[index] = build_constraint_point(
            *contact_point,
            world_point.point(),
            constraint.normal,
            tangent,
            first,
            second,
            contact.restitution,
            time_step_ratio,
            warm_starting,
        );
    }
    prepare_two_point_block(&mut constraint, first, second);
    if !constraint.normal.is_valid()
        || !constraint.friction.is_finite()
        || !constraint.tangent_speed.is_finite()
        || constraint.friction < 0.0
        || constraint.points[..constraint.point_count]
            .iter()
            .any(|point| !point.is_finite())
    {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(constraint)
}

#[allow(clippy::too_many_arguments)]
fn build_constraint_point(
    contact_point: ContactPoint,
    world_point: Vec2,
    normal: Vec2,
    tangent: Vec2,
    first: SolverBody,
    second: SolverBody,
    restitution: f32,
    time_step_ratio: f32,
    warm_starting: bool,
) -> ConstraintPoint {
    let r_a = world_point - first.center;
    let r_b = world_point - second.center;
    let normal_arm_a = r_a.cross(normal);
    let normal_arm_b = r_b.cross(normal);
    let normal_k = first.inverse_mass
        + second.inverse_mass
        + first.inverse_inertia * normal_arm_a * normal_arm_a
        + second.inverse_inertia * normal_arm_b * normal_arm_b;
    let tangent_arm_a = r_a.cross(tangent);
    let tangent_arm_b = r_b.cross(tangent);
    let tangent_k = first.inverse_mass
        + second.inverse_mass
        + first.inverse_inertia * tangent_arm_a * tangent_arm_a
        + second.inverse_inertia * tangent_arm_b * tangent_arm_b;
    let relative = second.linear_velocity + Vec2::scalar_cross(second.angular_velocity, r_b)
        - first.linear_velocity
        - Vec2::scalar_cross(first.angular_velocity, r_a);
    let relative_normal_velocity = normal.dot(relative);
    ConstraintPoint {
        feature_id: contact_point.feature_id(),
        r_a,
        r_b,
        normal_impulse: if warm_starting {
            time_step_ratio * contact_point.normal_impulse()
        } else {
            0.0
        },
        tangent_impulse: if warm_starting {
            time_step_ratio * contact_point.tangent_impulse()
        } else {
            0.0
        },
        normal_mass: if normal_k > 0.0 { 1.0 / normal_k } else { 0.0 },
        tangent_mass: if tangent_k > 0.0 {
            1.0 / tangent_k
        } else {
            0.0
        },
        velocity_bias: if relative_normal_velocity < -VELOCITY_THRESHOLD {
            -restitution * relative_normal_velocity
        } else {
            0.0
        },
    }
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

fn warm_start(
    constraint: &VelocityConstraint,
    bodies: &mut [SolverBody],
) -> Result<(), ContactSolveFailure> {
    let (mut first, mut second) = constraint_bodies(constraint, bodies)?;
    let tangent = constraint.normal.cross_scalar(1.0);
    for point in &constraint.points[..constraint.point_count] {
        let impulse = point.normal_impulse * constraint.normal + point.tangent_impulse * tangent;
        first.angular_velocity -= first.inverse_inertia * point.r_a.cross(impulse);
        first.linear_velocity -= first.inverse_mass * impulse;
        second.angular_velocity += second.inverse_inertia * point.r_b.cross(impulse);
        second.linear_velocity += second.inverse_mass * impulse;
    }
    store_constraint_bodies(constraint, bodies, first, second)
}

fn solve_velocity_constraints(
    constraint: &mut VelocityConstraint,
    bodies: &mut [SolverBody],
) -> Result<(), ContactSolveFailure> {
    let (mut first, mut second) = constraint_bodies(constraint, bodies)?;
    solve_tangent_constraints(constraint, &mut first, &mut second);
    if constraint.point_count == 1 {
        solve_one_normal_constraint(constraint, &mut first, &mut second);
    } else {
        solve_two_normal_constraints(constraint, &mut first, &mut second);
    }
    store_constraint_bodies(constraint, bodies, first, second)
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
        let lambda = point.tangent_mass * (constraint.tangent_speed - relative.dot(tangent));
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
    bodies: &[SolverBody],
) -> Result<(), ContactSolveFailure> {
    let (first, second) = constraint_bodies(constraint, bodies)?;
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

fn constraint_bodies(
    constraint: &VelocityConstraint,
    bodies: &[SolverBody],
) -> Result<(SolverBody, SolverBody), ContactSolveFailure> {
    let first = bodies
        .get(constraint.first_body_index)
        .copied()
        .ok_or(ContactSolveFailure::UnsupportedTopology)?;
    let second = bodies
        .get(constraint.second_body_index)
        .copied()
        .ok_or(ContactSolveFailure::UnsupportedTopology)?;
    Ok((first, second))
}

fn store_constraint_bodies(
    constraint: &VelocityConstraint,
    bodies: &mut [SolverBody],
    first: SolverBody,
    second: SolverBody,
) -> Result<(), ContactSolveFailure> {
    if constraint.first_body_index == constraint.second_body_index {
        return Err(ContactSolveFailure::UnsupportedTopology);
    }
    let Some(first_lane) = bodies.get_mut(constraint.first_body_index) else {
        return Err(ContactSolveFailure::UnsupportedTopology);
    };
    *first_lane = first;
    let Some(second_lane) = bodies.get_mut(constraint.second_body_index) else {
        return Err(ContactSolveFailure::UnsupportedTopology);
    };
    *second_lane = second;
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
    use crate::{BodyDef, BodyType, FixtureDef, StepConfiguration, StepHook, StepLimits, World};

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
    fn multi_contact_island_solves_all_manager_occurrences() {
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
        // Act
        let report = world
            .step(
                phase6_step_configuration(),
                &mut hook,
                StepLimits::default(),
            )
            .expect("multi-contact topology should solve as one island");

        // Assert
        assert_eq!(report.contact_solves().len(), 2);
        assert!(
            report
                .contact_solves()
                .iter()
                .any(|solve| solve.contact().fixtures() == [static_fixture, first_dynamic_fixture])
        );
        for body in [first_dynamic, second_dynamic] {
            let (linear, angular) = world.body_solver_velocity_for_test(body);
            assert!(linear.is_valid());
            assert!(angular.is_finite());
        }
    }
}
