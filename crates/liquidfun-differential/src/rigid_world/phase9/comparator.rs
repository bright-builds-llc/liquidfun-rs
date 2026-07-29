//! Exhaustive fail-closed comparison for Phase 9 particle observations.

mod numeric;
mod observations;

use std::collections::BTreeSet;

use liquidfun_test_protocol::{
    FloatBits, Phase9BodyContactObservation, Phase9Occurrence, Phase9ParticleContactObservation,
    Phase9ParticleObservation, Phase9ParticleSnapshot, Phase9StatisticsObservation,
    RigidWorldObservation, RigidWorldRequestRecord, RigidWorldResultRecord, Sha256Hex, Vec2Bits,
    validate_rigid_world_result_against_request,
};
use sha2::{Digest, Sha256};

use crate::rigid_evidence::{RigidComparisonFailure, RigidMismatchReport};

use numeric::{
    compare_absolute_relative, compare_dimensioned, compare_vec_exact, compare_vec_ulps, exact,
    mismatch, observation_kind, policy_error,
};
use observations::{
    compare_body_contact, compare_occurrence, compare_particle, compare_particle_contact,
    compare_ray, compare_statistics,
};

use super::{
    PHASE9_REGISTRY_ID, PHASE9_REQUIRED_POLICY_PATHS, Phase9PolicyKind, phase9_policy_for_path,
};

const PHASE9_MAX_ULPS: u32 = 4;
const PHASE9_ABSOLUTE_RELATIVE_ABSOLUTE: f32 = 1.0e-6;
const PHASE9_ABSOLUTE_RELATIVE_RELATIVE: f32 = 1.0e-5;
const PHASE9_RAY_FRACTION_ABSOLUTE: f32 = 1.0e-6;
const PHASE9_BODY_MASS_ABSOLUTE: f32 = 1.0e-5;

/// Fail-closed comparator error that is not physics mismatch evidence.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Phase9ComparatorError {
    /// The inherited Phase 6 through Phase 8 boundary rejected the comparison.
    #[error("retained rigid comparison failed: {0:?}")]
    RetainedRigid(RigidComparisonFailure),
    /// The exact policy registry was missing, duplicated, wildcarded, or unknown.
    #[error("invalid Phase 9 policy registry: {reason}")]
    PolicyRegistry {
        /// Exact reason the candidate registry failed closed.
        reason: Box<str>,
    },
    /// A result failed request-bound action and lifecycle validation.
    #[error("{side} Phase 9 result validation failed: {message}")]
    ResultValidation {
        /// Engine role whose output failed validation.
        side: &'static str,
        /// Bounded protocol diagnostic.
        message: Box<str>,
    },
    /// Corresponding observation variants did not expose the same typed structure.
    #[error("Phase 9 observation structure mismatch at `{path}`: {expected} != {actual}")]
    Structure {
        /// Closed semantic path owning the structure.
        path: &'static str,
        /// Expected typed structure.
        expected: Box<str>,
        /// Actual typed structure.
        actual: Box<str>,
    },
    /// A numeric observation contained a non-finite value.
    #[error("non-finite Phase 9 value at `{path}`")]
    NonFinite {
        /// Closed semantic path owning the invalid value.
        path: &'static str,
    },
}

/// Stable first Phase 9 semantic divergence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase9Mismatch {
    signature_sha256: Sha256Hex,
    timeline_index: usize,
    checkpoint_index: usize,
    observation_index: usize,
    semantic_path: &'static str,
    kind: Phase9PolicyKind,
    expected: Box<str>,
    actual: Box<str>,
    maybe_expected_bits: Option<FloatBits>,
    maybe_actual_bits: Option<FloatBits>,
}

impl Phase9Mismatch {
    /// Returns the deterministic first-divergence identity.
    #[must_use]
    pub const fn signature_sha256(&self) -> &Sha256Hex {
        &self.signature_sha256
    }

    /// Returns the source-ordered timeline index.
    #[must_use]
    pub const fn timeline_index(&self) -> usize {
        self.timeline_index
    }

    /// Returns the source-ordered checkpoint index.
    #[must_use]
    pub const fn checkpoint_index(&self) -> usize {
        self.checkpoint_index
    }

    /// Returns the source-ordered particle-observation index.
    #[must_use]
    pub const fn observation_index(&self) -> usize {
        self.observation_index
    }

    /// Returns the exact closed semantic path.
    #[must_use]
    pub const fn semantic_path(&self) -> &'static str {
        self.semantic_path
    }

    /// Returns the reviewed policy class that rejected the values.
    #[must_use]
    pub const fn kind(&self) -> Phase9PolicyKind {
        self.kind
    }

    /// Returns the expected diagnostic value.
    #[must_use]
    pub fn expected(&self) -> &str {
        &self.expected
    }

    /// Returns the actual diagnostic value.
    #[must_use]
    pub fn actual(&self) -> &str {
        &self.actual
    }
}

/// Complete Phase 9 request/result comparison outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase9ComparisonOutcome {
    /// First source-ordered retained Phase 6 through Phase 8 disagreement.
    RetainedRigidMismatch(Box<RigidMismatchReport>),
    /// All particle semantics matched and the complete policy registry was consumed.
    Match {
        /// Every required path in reviewed source order.
        consumed_paths: Box<[&'static str]>,
    },
    /// First source-ordered semantic disagreement.
    PhysicsMismatch(Box<Phase9Mismatch>),
}

/// Comparison outcome for one typed observation pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase9ObservationComparison {
    /// The complete typed observation matched.
    Match,
    /// The first field differed.
    PhysicsMismatch(Box<Phase9Mismatch>),
}

impl Phase9ObservationComparison {
    /// Extracts mismatch evidence for focused comparator tests.
    ///
    /// # Panics
    ///
    /// Panics when called on a matching observation pair.
    #[must_use]
    pub fn expect_mismatch(self) -> Box<Phase9Mismatch> {
        let Self::PhysicsMismatch(mismatch) = self else {
            panic!("expected Phase 9 observation mismatch");
        };
        mismatch
    }
}

#[derive(Debug, Clone, Copy)]
struct Location {
    timeline: usize,
    checkpoint: usize,
    observation: usize,
}

/// Validates an exact candidate policy registry without wildcard fallback.
///
/// # Errors
///
/// Returns [`Phase9ComparatorError::PolicyRegistry`] for a missing, duplicate,
/// unknown, reordered, or wildcard path.
pub fn validate_phase9_policy_registry(
    paths: &[&str],
) -> Result<Box<[&'static str]>, Phase9ComparatorError> {
    let mut consumed = BTreeSet::new();
    for path in paths {
        if path.contains('*') || path.contains('?') {
            return Err(policy_error(format!("wildcard path `{path}`")));
        }
        if phase9_policy_for_path(path).is_none() {
            return Err(policy_error(format!("unknown path `{path}`")));
        }
        if !consumed.insert(*path) {
            return Err(policy_error(format!("duplicate path `{path}`")));
        }
    }
    if paths != PHASE9_REQUIRED_POLICY_PATHS {
        let maybe_missing = PHASE9_REQUIRED_POLICY_PATHS
            .iter()
            .find(|path| !consumed.contains(**path));
        let reason = maybe_missing.map_or_else(
            || "policy paths are not in reviewed source order".to_owned(),
            |path| format!("missing path `{path}`"),
        );
        return Err(policy_error(reason));
    }
    Ok(PHASE9_REQUIRED_POLICY_PATHS.into())
}

/// Compares request-valid Phase 9 observations in timeline/checkpoint/source order.
///
/// # Errors
///
/// Returns a harness error when policy identity, request-bound validation, typed
/// observation structure, or finite-number requirements fail.
pub fn compare_phase9_rigid_world_results(
    request: &RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    oracle: &RigidWorldResultRecord,
) -> Result<Phase9ComparisonOutcome, Phase9ComparatorError> {
    let consumed_paths = validate_phase9_policy_registry(PHASE9_REQUIRED_POLICY_PATHS)?;
    validate_rigid_world_result_against_request(request, native).map_err(|error| {
        Phase9ComparatorError::ResultValidation {
            side: "native",
            message: error.to_string().into(),
        }
    })?;
    validate_rigid_world_result_against_request(request, oracle).map_err(|error| {
        Phase9ComparatorError::ResultValidation {
            side: "oracle",
            message: error.to_string().into(),
        }
    })?;

    for (timeline_index, (expected_timeline, actual_timeline)) in native
        .timelines()
        .iter()
        .zip(oracle.timelines())
        .enumerate()
    {
        for (checkpoint_index, (expected_checkpoint, actual_checkpoint)) in expected_timeline
            .checkpoints
            .iter()
            .zip(actual_timeline.checkpoints.iter())
            .enumerate()
        {
            let expected = particle_observations(&expected_checkpoint.observations);
            let actual = particle_observations(&actual_checkpoint.observations);
            if expected.len() != actual.len() {
                return Ok(Phase9ComparisonOutcome::PhysicsMismatch(Box::new(
                    mismatch(
                        Location {
                            timeline: timeline_index,
                            checkpoint: checkpoint_index,
                            observation: expected.len().min(actual.len()),
                        },
                        "particle.listener.occurrence",
                        expected.len().to_string(),
                        actual.len().to_string(),
                        None,
                    ),
                )));
            }
            for (observation_index, (expected, actual)) in
                expected.into_iter().zip(actual).enumerate()
            {
                let location = Location {
                    timeline: timeline_index,
                    checkpoint: checkpoint_index,
                    observation: observation_index,
                };
                if let Some(found) = compare_observation(location, expected, actual)? {
                    return Ok(Phase9ComparisonOutcome::PhysicsMismatch(Box::new(found)));
                }
            }
        }
    }

    Ok(Phase9ComparisonOutcome::Match { consumed_paths })
}

/// Compares one pair through the same exhaustive typed walker as the full runner.
///
/// # Errors
///
/// Returns a harness error for a structural or non-finite disagreement.
pub fn compare_phase9_particle_observations(
    expected: &Phase9ParticleObservation,
    actual: &Phase9ParticleObservation,
) -> Result<Phase9ObservationComparison, Phase9ComparatorError> {
    validate_phase9_policy_registry(PHASE9_REQUIRED_POLICY_PATHS)?;
    let maybe_mismatch = compare_observation(
        Location {
            timeline: 0,
            checkpoint: 0,
            observation: 0,
        },
        expected,
        actual,
    )?;
    Ok(
        maybe_mismatch.map_or(Phase9ObservationComparison::Match, |mismatch| {
            Phase9ObservationComparison::PhysicsMismatch(Box::new(mismatch))
        }),
    )
}

fn particle_observations(
    observations: &[RigidWorldObservation],
) -> Vec<&Phase9ParticleObservation> {
    observations
        .iter()
        .filter_map(|observation| {
            let RigidWorldObservation::Particle { observation } = observation else {
                return None;
            };
            Some(observation)
        })
        .collect()
}

#[allow(
    clippy::too_many_lines,
    reason = "one exhaustive match proves the closed observation schema"
)]
fn compare_observation(
    location: Location,
    expected: &Phase9ParticleObservation,
    actual: &Phase9ParticleObservation,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    match (expected, actual) {
        (
            Phase9ParticleObservation::System {
                system_id: expected_system,
                paused: expected_paused,
                particle_ids: expected_particles,
            },
            Phase9ParticleObservation::System {
                system_id: actual_system,
                paused: actual_paused,
                particle_ids: actual_particles,
            },
        ) => Ok(exact(
            location,
            "particle.storage.identity",
            expected_system,
            actual_system,
        )
        .or_else(|| {
            exact(
                location,
                "particle.strict_contact.branch",
                expected_paused,
                actual_paused,
            )
        })
        .or_else(|| {
            exact(
                location,
                "particle.permutation.order",
                expected_particles,
                actual_particles,
            )
        })),
        (
            Phase9ParticleObservation::Particle { snapshot: expected },
            Phase9ParticleObservation::Particle { snapshot: actual },
        ) => compare_particle(location, expected, actual),
        (
            Phase9ParticleObservation::Lifecycle {
                occurrence: expected,
            },
            Phase9ParticleObservation::Lifecycle { occurrence: actual },
        ) => Ok(compare_occurrence(location, expected, actual)),
        (
            Phase9ParticleObservation::ParticleContact { contact: expected },
            Phase9ParticleObservation::ParticleContact { contact: actual },
        ) => compare_particle_contact(location, expected, actual),
        (
            Phase9ParticleObservation::BodyContact { contact: expected },
            Phase9ParticleObservation::BodyContact { contact: actual },
        ) => compare_body_contact(location, expected, actual),
        (
            Phase9ParticleObservation::Statistics {
                statistics: expected,
            },
            Phase9ParticleObservation::Statistics { statistics: actual },
        ) => compare_statistics(location, expected, actual),
        (
            Phase9ParticleObservation::Query {
                terminated: expected_terminated,
                particle_ids: expected_particles,
            },
            Phase9ParticleObservation::Query {
                terminated: actual_terminated,
                particle_ids: actual_particles,
            },
        ) => Ok(exact(
            location,
            "particle.query.culling",
            expected_terminated,
            actual_terminated,
        )
        .or_else(|| {
            exact(
                location,
                "particle.query.order",
                expected_particles,
                actual_particles,
            )
        })),
        (
            Phase9ParticleObservation::RayCast {
                terminated: expected_terminated,
                particle_ids: expected_particles,
                fractions_bits: expected_fractions,
            },
            Phase9ParticleObservation::RayCast {
                terminated: actual_terminated,
                particle_ids: actual_particles,
                fractions_bits: actual_fractions,
            },
        ) => compare_ray(
            location,
            *expected_terminated,
            expected_particles,
            expected_fractions,
            *actual_terminated,
            actual_particles,
            actual_fractions,
        ),
        (
            Phase9ParticleObservation::MixedState {
                body_ids: expected_bodies,
                particle_ids: expected_particles,
            },
            Phase9ParticleObservation::MixedState {
                body_ids: actual_bodies,
                particle_ids: actual_particles,
            },
        ) => Ok(exact(
            location,
            "particle.coupling.identity",
            expected_bodies,
            actual_bodies,
        )
        .or_else(|| {
            exact(
                location,
                "particle.storage.identity",
                expected_particles,
                actual_particles,
            )
        })),
        _ => Err(Phase9ComparatorError::Structure {
            path: "particle.storage.identity",
            expected: observation_kind(expected).into(),
            actual: observation_kind(actual).into(),
        }),
    }
}
