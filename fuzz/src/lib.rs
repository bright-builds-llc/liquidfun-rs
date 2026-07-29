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

include!("lib/operations.rs");

#[cfg(test)]
mod tests;
