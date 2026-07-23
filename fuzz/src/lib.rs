//! Bounded typed cores shared by the five private cargo-fuzz targets.

#![forbid(unsafe_code)]

use arbitrary::{Arbitrary, Unstructured};
use liquidfun::collision::narrow::collide_shapes;
use liquidfun::collision::shape::{CircleShape, PolygonShape, Shape};
use liquidfun::collision::{CollisionOutcome, FilterData};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource, ParticleSystemDef,
};
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, FixtureId, NoDecisionHook, ParticleBufferBundle,
    ParticleBufferLanes, ParticleDef, ParticleGroupId, ParticleSystemId, StepConfiguration,
    StepLimits, World,
};
use liquidfun_test_protocol::{
    HarnessLimits, Sha256Hex, decode_canonical_checkpoint_jsonl, decode_catalog_run_request_jsonl,
    decode_collision_probe_request_jsonl, decode_rigid_world_request_jsonl,
    decode_scenario_request_jsonl, decode_trace_record_jsonl,
};
use sha2::{Digest, Sha256};

/// Maximum raw protocol record accepted by the protocol target.
pub const MAX_PROTOCOL_BYTES: usize = 1024 * 1024;
/// Maximum typed operations accepted by any mutation program.
pub const MAX_MUTATION_OPERATIONS: usize = 256;
/// Maximum live-body and fixture creation attempts accepted by one world program.
pub const MAX_WORLD_ENTITIES: usize = 128;
/// Maximum particle creation attempts accepted by one particle program.
pub const MAX_PARTICLES: usize = 4_096;
/// Maximum group creation attempts accepted by one ownership program.
pub const MAX_GROUPS: usize = 64;

const RAW_OPERATION_BYTES: usize = 17;

/// Stable accepted-finding classifications for minimized regressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureClassification {
    /// Protocol, runner, or invariant harness failure.
    Harness,
    /// Semantic Rust/oracle disagreement.
    PhysicsMismatch,
    /// Memory, undefined-behavior, or other sanitizer finding.
    Sanitizer,
    /// Per-input or whole-target timeout.
    Timeout,
    /// Closed-schema or version contract failure.
    Schema,
}

impl FailureClassification {
    /// Returns the exact machine-facing classification spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Harness => "Harness",
            Self::PhysicsMismatch => "PhysicsMismatch",
            Self::Sanitizer => "Sanitizer",
            Self::Timeout => "Timeout",
            Self::Schema => "Schema",
        }
    }
}

/// Provenance required before a minimized input can become a named regression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimizedRegressionMetadata {
    /// One of the five registered fuzz target names.
    pub target: Box<str>,
    /// SHA-256 of the exact minimized bytes.
    pub input_sha256: Sha256Hex,
    /// Repository-relative path containing those exact bytes.
    pub exact_bytes_path: Box<str>,
    /// Exact generator and cargo-fuzz identity.
    pub generator: Box<str>,
    /// Exact dated Rust toolchain.
    pub toolchain: Box<str>,
    /// Candidate commit tested by the finding.
    pub candidate_commit: Box<str>,
    /// Oracle identity when the finding crosses the oracle boundary.
    pub maybe_oracle_identity: Option<Box<str>>,
    /// Comparison policy identity when applicable.
    pub maybe_policy_identity: Option<Box<str>>,
    /// Stable failure class, kept distinct from physics mismatch.
    pub classification: FailureClassification,
    /// Commit, issue, or pull request fixing the finding.
    pub fix_reference: Box<str>,
}

impl MinimizedRegressionMetadata {
    /// Creates complete metadata and hashes the exact supplied bytes.
    #[must_use]
    #[allow(
        clippy::too_many_arguments,
        reason = "the provenance contract is intentionally flat"
    )]
    pub fn from_exact_bytes(
        target: impl Into<Box<str>>,
        exact_bytes: &[u8],
        exact_bytes_path: impl Into<Box<str>>,
        generator: impl Into<Box<str>>,
        toolchain: impl Into<Box<str>>,
        candidate_commit: impl Into<Box<str>>,
        maybe_oracle_identity: Option<Box<str>>,
        maybe_policy_identity: Option<Box<str>>,
        classification: FailureClassification,
        fix_reference: impl Into<Box<str>>,
    ) -> Self {
        Self {
            target: target.into(),
            input_sha256: Sha256Hex::from_digest(Sha256::digest(exact_bytes).into()),
            exact_bytes_path: exact_bytes_path.into(),
            generator: generator.into(),
            toolchain: toolchain.into(),
            candidate_commit: candidate_commit.into(),
            maybe_oracle_identity,
            maybe_policy_identity,
            classification,
            fix_reference: fix_reference.into(),
        }
    }
}

/// Result of pre-validating and executing one fuzz input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuzzDisposition {
    /// The bounded typed input reached the reviewed target boundary.
    Executed,
    /// The input exceeded a reviewed bound and produced no effects.
    Rejected,
}

#[derive(Debug, Clone, Copy, Arbitrary)]
struct RawOperation {
    kind: u8,
    first: u32,
    second: u32,
    third: u32,
    fourth: u32,
}

/// Exercises production strict decoders under the exact one-mebibyte input cap.
#[must_use]
pub fn fuzz_protocol(bytes: &[u8]) -> FuzzDisposition {
    if bytes.len() > MAX_PROTOCOL_BYTES {
        return FuzzDisposition::Rejected;
    }

    let limits = HarnessLimits::phase2_default_v1();
    let _scenario = decode_scenario_request_jsonl(bytes, &limits);
    let _collision = decode_collision_probe_request_jsonl(bytes, &limits);
    let _rigid = decode_rigid_world_request_jsonl(bytes, &limits);
    let _catalog = decode_catalog_run_request_jsonl(bytes, &limits);
    let _trace = decode_trace_record_jsonl(bytes, &limits);
    let _checkpoint = decode_canonical_checkpoint_jsonl(bytes, &limits);
    FuzzDisposition::Executed
}

/// Exercises checked shape construction and collision dispatch with at most 256 operations.
#[must_use]
pub fn fuzz_shapes_collision(bytes: &[u8]) -> FuzzDisposition {
    let Some(operations) = decode_operations(bytes, MAX_MUTATION_OPERATIONS) else {
        return FuzzDisposition::Rejected;
    };

    for operation in operations {
        exercise_shape_operation(operation);
    }
    FuzzDisposition::Executed
}

/// Exercises checked body/fixture mutation with 256 operations and 128 creations at most.
///
/// # Panics
///
/// Panics only when a committed engine invariant fails, which is a fuzz finding.
#[must_use]
pub fn fuzz_world_mutation(bytes: &[u8]) -> FuzzDisposition {
    let Some(operations) = decode_operations(bytes, MAX_MUTATION_OPERATIONS) else {
        return FuzzDisposition::Rejected;
    };
    let creation_count = operations
        .iter()
        .filter(|operation| matches!(operation.kind % 6, 0 | 1))
        .count();
    if creation_count > MAX_WORLD_ENTITIES {
        return FuzzDisposition::Rejected;
    }

    let Ok(mut world) = World::new() else {
        panic!("world identity exhausted during bounded fuzz execution");
    };
    let mut bodies = Vec::<BodyId>::new();
    let mut fixtures = Vec::<FixtureId>::new();
    for operation in operations {
        execute_world_operation(&mut world, &mut bodies, &mut fixtures, operation);
        assert_live_bodies_are_finite(&world, &bodies);
    }
    FuzzDisposition::Executed
}

/// Exercises particle creation, invalidation, compaction, and stepping under reviewed caps.
///
/// # Panics
///
/// Panics only when a committed engine invariant fails, which is a fuzz finding.
#[must_use]
pub fn fuzz_particles(bytes: &[u8]) -> FuzzDisposition {
    let Some(operations) = decode_operations(bytes, MAX_MUTATION_OPERATIONS) else {
        return FuzzDisposition::Rejected;
    };
    let Some(particle_budget) = checked_particle_budget(&operations) else {
        return FuzzDisposition::Rejected;
    };
    if particle_budget > MAX_PARTICLES {
        return FuzzDisposition::Rejected;
    }

    let Ok(mut world) = World::new() else {
        panic!("world identity exhausted during bounded fuzz execution");
    };
    let Ok(definition) = ParticleSystemDef::default().with_maximum_count(MAX_PARTICLES) else {
        panic!("reviewed particle maximum became invalid");
    };
    let Ok(system) = world.create_particle_system_with_def(&definition) else {
        panic!("fresh world rejected the reviewed particle system");
    };
    let mut particles = Vec::with_capacity(particle_budget);
    for operation in operations {
        execute_particle_operation(&mut world, system, &mut particles, operation);
        assert_live_particles_are_finite(&world, &particles);
    }
    FuzzDisposition::Executed
}

/// Exercises owned particle buffers, group creation, teardown, and stale-handle rejection.
///
/// # Panics
///
/// Panics only when a committed engine invariant fails, which is a fuzz finding.
#[must_use]
pub fn fuzz_groups_ownership(bytes: &[u8]) -> FuzzDisposition {
    let Some(operations) = decode_operations(bytes, MAX_MUTATION_OPERATIONS) else {
        return FuzzDisposition::Rejected;
    };
    let group_budget = operations
        .iter()
        .filter(|operation| operation.kind % 3 == 0)
        .count();
    if group_budget > MAX_GROUPS {
        return FuzzDisposition::Rejected;
    }

    let Ok(mut world) = World::new() else {
        panic!("world identity exhausted during bounded fuzz execution");
    };
    let Some(system) = create_owned_buffer_system(&mut world) else {
        panic!("fresh world rejected reviewed owned particle buffers");
    };
    let mut groups = Vec::<ParticleGroupId>::with_capacity(group_budget);
    for operation in operations {
        execute_group_operation(&mut world, system, &mut groups, operation);
    }
    let Ok(teardown) = world.destroy_particle_system_with_buffers(system) else {
        panic!("live owned-buffer system failed to return its lanes");
    };
    assert!(teardown.into_lanes().positions().len() <= MAX_GROUPS);
    assert!(
        groups
            .iter()
            .all(|group| world.particle_group_view(*group).is_err())
    );
    assert!(world.particle_system_view(system).is_err());
    FuzzDisposition::Executed
}

fn decode_operations(bytes: &[u8], maximum: usize) -> Option<Vec<RawOperation>> {
    let operation_count = bytes.len().div_ceil(RAW_OPERATION_BYTES);
    if operation_count > maximum {
        return None;
    }

    let mut operations = Vec::with_capacity(operation_count);
    for chunk in bytes.chunks(RAW_OPERATION_BYTES) {
        let mut padded = [0_u8; RAW_OPERATION_BYTES];
        padded[..chunk.len()].copy_from_slice(chunk);
        let mut input = Unstructured::new(&padded);
        let Ok(operation) = RawOperation::arbitrary(&mut input) else {
            return None;
        };
        operations.push(operation);
    }
    Some(operations)
}

fn finite_scalar(raw: u32) -> f32 {
    let bounded = u16::try_from(raw % 20_001).unwrap_or_default();
    let signed = i16::try_from(bounded).unwrap_or_default() - 10_000;
    f32::from(signed) / 1_000.0
}

fn positive_scalar(raw: u32) -> f32 {
    let bounded = u16::try_from(raw % 2_000 + 1).unwrap_or_default();
    f32::from(bounded) / 1_000.0
}

fn exercise_shape_operation(operation: RawOperation) {
    let circle_a = CircleShape::new(
        Vec2::new(
            finite_scalar(operation.first),
            finite_scalar(operation.second),
        ),
        positive_scalar(operation.third),
    );
    let circle_b = CircleShape::new(Vec2::ZERO, positive_scalar(operation.fourth));
    let polygon = PolygonShape::box_shape(
        positive_scalar(operation.first),
        positive_scalar(operation.second),
    );
    let (Ok(circle_a), Ok(circle_b), Ok(polygon)) = (circle_a, circle_b, polygon) else {
        return;
    };
    let shape_a = if operation.kind & 1 == 0 {
        Shape::from(circle_a)
    } else {
        Shape::from(polygon)
    };
    let shape_b = Shape::from(circle_b);
    let (Ok(child_a), Ok(child_b)) = (shape_a.child_index(0), shape_b.child_index(0)) else {
        return;
    };
    let transform = Transform::from_position_angle(
        Vec2::new(
            finite_scalar(operation.third),
            finite_scalar(operation.fourth),
        ),
        finite_scalar(operation.first),
    );
    if let Ok(CollisionOutcome::Touching(pair)) = collide_shapes(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        transform,
    ) {
        assert!(
            pair.manifold()
                .points()
                .iter()
                .all(|point| point.local_point().is_valid())
        );
    }
}

fn execute_world_operation(
    world: &mut World,
    bodies: &mut Vec<BodyId>,
    fixtures: &mut Vec<FixtureId>,
    operation: RawOperation,
) {
    match operation.kind % 6 {
        0 => {
            let body_type = match operation.first % 3 {
                0 => BodyType::Static,
                1 => BodyType::Kinematic,
                _ => BodyType::Dynamic,
            };
            let definition = BodyDef::new(
                body_type,
                Vec2::new(
                    finite_scalar(operation.second),
                    finite_scalar(operation.third),
                ),
                finite_scalar(operation.fourth),
                operation.kind & 1 == 0,
            );
            if let Ok(definition) = definition
                && let Ok(body) = world.create_body(&definition)
            {
                bodies.push(body);
            }
        }
        1 => {
            let Some(body) = select(bodies, operation.first) else {
                return;
            };
            let circle = CircleShape::new(Vec2::ZERO, positive_scalar(operation.second));
            let Ok(circle) = circle else {
                return;
            };
            let definition = FixtureDef::new(
                Shape::from(circle),
                positive_scalar(operation.third),
                positive_scalar(operation.fourth),
                positive_scalar(operation.first),
                operation.kind & 1 == 0,
                FilterData::default(),
            );
            if let Ok(definition) = definition
                && let Ok(fixture) = world.create_fixture(body, &definition)
            {
                fixtures.push(fixture);
            }
        }
        2 => {
            if let Some(body) = select(bodies, operation.first) {
                let _destroyed = world.destroy_body(body);
            }
        }
        3 => {
            if let Some(fixture) = select(fixtures, operation.first) {
                let _destroyed = world.destroy_fixture(fixture);
            }
        }
        4 => {
            if let Some(body) = select(bodies, operation.first) {
                let _transformed = world.set_body_transform(
                    body,
                    Vec2::new(
                        finite_scalar(operation.second),
                        finite_scalar(operation.third),
                    ),
                    finite_scalar(operation.fourth),
                );
            }
        }
        _ => {
            if let Some(body) = select(bodies, operation.first) {
                let _activated = world.set_body_active(body, operation.second & 1 == 0);
            }
        }
    }
}

fn assert_live_bodies_are_finite(world: &World, bodies: &[BodyId]) {
    for body in bodies {
        if let Ok(snapshot) = world.body_snapshot(*body) {
            assert!(snapshot.position().is_valid());
            assert!(snapshot.angle().is_finite());
        }
    }
}

fn checked_particle_budget(operations: &[RawOperation]) -> Option<usize> {
    operations
        .iter()
        .filter(|operation| operation.kind % 4 == 0)
        .try_fold(0_usize, |total, operation| {
            total.checked_add(usize::try_from(operation.first % 32 + 1).ok()?)
        })
}

fn execute_particle_operation(
    world: &mut World,
    system: ParticleSystemId,
    particles: &mut Vec<liquidfun::ParticleId>,
    operation: RawOperation,
) {
    match operation.kind % 4 {
        0 => {
            let count = operation.first % 32 + 1;
            for ordinal in 0..count {
                let definition = ParticleDef::default()
                    .with_position(Vec2::new(
                        finite_scalar(operation.second.wrapping_add(ordinal)),
                        finite_scalar(operation.third),
                    ))
                    .and_then(|definition| {
                        definition.with_velocity(Vec2::new(
                            finite_scalar(operation.fourth),
                            finite_scalar(ordinal),
                        ))
                    });
                if let Ok(definition) = definition
                    && let Ok(receipt) = world.create_particle_with_def(system, None, &definition)
                {
                    particles.push(receipt.created_particle());
                }
            }
        }
        1 => {
            if let Some(particle) = select(particles, operation.first) {
                let _marked = world.mark_particle_for_destruction(particle);
            }
        }
        2 => {
            let _compacted = world.compact_pending_particles(system);
        }
        _ => {
            let Ok(configuration) = StepConfiguration::new(0.0, 1, 1) else {
                panic!("reviewed zero-duration step became invalid");
            };
            let _report = world.step(configuration, &mut NoDecisionHook, StepLimits::default());
        }
    }
}

fn assert_live_particles_are_finite(world: &World, particles: &[liquidfun::ParticleId]) {
    for particle in particles {
        if let Ok(snapshot) = world.particle_snapshot(*particle) {
            assert!(snapshot.position().is_valid());
            assert!(snapshot.velocity().is_valid());
        }
    }
}

fn create_owned_buffer_system(world: &mut World) -> Option<ParticleSystemId> {
    let lanes = ParticleBufferLanes::new(
        Vec::with_capacity(MAX_GROUPS),
        Vec::with_capacity(MAX_GROUPS),
        Vec::with_capacity(MAX_GROUPS),
        None,
    );
    let buffers = ParticleBufferBundle::fixed(MAX_GROUPS, lanes).ok()?;
    world
        .create_particle_system_with_buffers(&ParticleSystemDef::default(), buffers)
        .ok()
}

fn execute_group_operation(
    world: &mut World,
    system: ParticleSystemId,
    groups: &mut Vec<ParticleGroupId>,
    operation: RawOperation,
) {
    match operation.kind % 3 {
        0 => {
            let source = ParticleGroupSource::positions(vec![Vec2::new(
                finite_scalar(operation.first),
                finite_scalar(operation.second),
            )]);
            let Ok(source) = source else {
                return;
            };
            let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New);
            if let Ok(group) = world.create_particle_group(system, &recipe) {
                groups.push(group);
            }
        }
        1 => {
            if let Some(group) = select(groups, operation.first)
                && let Ok(view) = world.particle_group_view(group)
            {
                let members = view.member_ids().to_vec();
                for member in members {
                    let _marked = world.mark_particle_for_destruction(member);
                }
                let _compacted = world.compact_pending_particles(system);
                let _destroyed = world.destroy_particle_group(group);
            }
        }
        _ => {
            if let Some(group) = select(groups, operation.first) {
                let _view = world.particle_group_view(group);
            }
        }
    }
}

fn select<T: Copy>(values: &[T], raw: u32) -> Option<T> {
    if values.is_empty() {
        return None;
    }
    let index = usize::try_from(raw).unwrap_or_default() % values.len();
    values.get(index).copied()
}

#[cfg(test)]
mod tests {
    use super::{
        FailureClassification, FuzzDisposition, MAX_GROUPS, MAX_MUTATION_OPERATIONS, MAX_PARTICLES,
        MAX_PROTOCOL_BYTES, MAX_WORLD_ENTITIES, RAW_OPERATION_BYTES, fuzz_groups_ownership,
        fuzz_particles, fuzz_protocol, fuzz_shapes_collision, fuzz_world_mutation,
    };

    #[test]
    fn protocol_accepts_exact_one_mebibyte_and_rejects_n_plus_one() {
        // Arrange
        let at_limit = vec![0_u8; MAX_PROTOCOL_BYTES];
        let over_limit = vec![0_u8; MAX_PROTOCOL_BYTES + 1];

        // Act
        let accepted = fuzz_protocol(&at_limit);
        let rejected = fuzz_protocol(&over_limit);

        // Assert
        assert_eq!(accepted, FuzzDisposition::Executed);
        assert_eq!(rejected, FuzzDisposition::Rejected);
    }

    #[test]
    fn shapes_accept_256_operations_and_reject_n_plus_one() {
        // Arrange
        let at_limit = vec![0_u8; MAX_MUTATION_OPERATIONS * RAW_OPERATION_BYTES];
        let over_limit = vec![0_u8; (MAX_MUTATION_OPERATIONS + 1) * RAW_OPERATION_BYTES];

        // Act
        let accepted = fuzz_shapes_collision(&at_limit);
        let rejected = fuzz_shapes_collision(&over_limit);

        // Assert
        assert_eq!(accepted, FuzzDisposition::Executed);
        assert_eq!(rejected, FuzzDisposition::Rejected);
    }

    #[test]
    fn world_accepts_128_creations_and_rejects_n_plus_one() {
        // Arrange
        let at_limit = vec![0_u8; MAX_WORLD_ENTITIES * RAW_OPERATION_BYTES];
        let over_limit = vec![0_u8; (MAX_WORLD_ENTITIES + 1) * RAW_OPERATION_BYTES];

        // Act
        let accepted = fuzz_world_mutation(&at_limit);
        let rejected = fuzz_world_mutation(&over_limit);

        // Assert
        assert_eq!(accepted, FuzzDisposition::Executed);
        assert_eq!(rejected, FuzzDisposition::Rejected);
    }

    #[test]
    fn particles_accept_4096_creation_attempts_and_reject_n_plus_one() {
        // Arrange
        let at_limit = particle_program(MAX_PARTICLES);
        let over_limit = particle_program(MAX_PARTICLES + 1);

        // Act
        let accepted = fuzz_particles(&at_limit);
        let rejected = fuzz_particles(&over_limit);

        // Assert
        assert_eq!(accepted, FuzzDisposition::Executed);
        assert_eq!(rejected, FuzzDisposition::Rejected);
    }

    #[test]
    fn groups_accept_64_creations_and_reject_n_plus_one() {
        // Arrange
        let at_limit = vec![0_u8; MAX_GROUPS * RAW_OPERATION_BYTES];
        let over_limit = vec![0_u8; (MAX_GROUPS + 1) * RAW_OPERATION_BYTES];

        // Act
        let accepted = fuzz_groups_ownership(&at_limit);
        let rejected = fuzz_groups_ownership(&over_limit);

        // Assert
        assert_eq!(accepted, FuzzDisposition::Executed);
        assert_eq!(rejected, FuzzDisposition::Rejected);
    }

    #[test]
    fn regression_failure_classification_is_closed_and_exact() {
        // Arrange / Act
        let spellings = [
            FailureClassification::Harness,
            FailureClassification::PhysicsMismatch,
            FailureClassification::Sanitizer,
            FailureClassification::Timeout,
            FailureClassification::Schema,
        ]
        .map(FailureClassification::as_str);

        // Assert
        assert_eq!(
            spellings,
            [
                "Harness",
                "PhysicsMismatch",
                "Sanitizer",
                "Timeout",
                "Schema",
            ]
        );
    }

    fn particle_program(total: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        let full_operations = total / 32;
        let remainder = total % 32;
        for _ in 0..full_operations {
            bytes.extend(operation_with_first(31));
        }
        if remainder != 0 {
            bytes.extend(operation_with_first(
                u32::try_from(remainder - 1).unwrap_or_default(),
            ));
        }
        bytes
    }

    fn operation_with_first(first: u32) -> [u8; RAW_OPERATION_BYTES] {
        let mut operation = [0_u8; RAW_OPERATION_BYTES];
        operation[1..5].copy_from_slice(&first.to_le_bytes());
        operation
    }
}
