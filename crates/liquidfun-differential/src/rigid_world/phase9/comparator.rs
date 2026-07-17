//! Exhaustive fail-closed comparison for Phase 9 particle observations.

use std::collections::BTreeSet;

use liquidfun_test_protocol::{
    FloatBits, Phase9BodyContactObservation, Phase9Occurrence, Phase9ParticleContactObservation,
    Phase9ParticleObservation, Phase9ParticleSnapshot, Phase9StatisticsObservation,
    RigidWorldObservation, RigidWorldRequestRecord, RigidWorldResultRecord, Sha256Hex, Vec2Bits,
    validate_rigid_world_result_against_request,
};
use sha2::{Digest, Sha256};

use crate::rigid_evidence::{RigidComparisonFailure, RigidMismatchReport};

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

fn compare_particle(
    location: Location,
    expected: &Phase9ParticleSnapshot,
    actual: &Phase9ParticleSnapshot,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    if let Some(found) = exact(
        location,
        "particle.storage.identity",
        &expected.particle_id,
        &actual.particle_id,
    )
    .or_else(|| {
        exact(
            location,
            "particle.storage.identity",
            &expected.system_id,
            &actual.system_id,
        )
    }) {
        return Ok(Some(found));
    }
    if let Some(found) = compare_vec_ulps(
        location,
        "particle.position",
        expected.position,
        actual.position,
    )?
    .or(compare_vec_ulps(
        location,
        "particle.velocity",
        expected.velocity,
        actual.velocity,
    )?) {
        return Ok(Some(found));
    }
    if let Some(found) = exact(
        location,
        "particle.configuration.bits",
        &expected.flags_bits,
        &actual.flags_bits,
    )
    .or_else(|| {
        exact(
            location,
            "particle.configuration.bits",
            &expected.color,
            &actual.color,
        )
    }) {
        return Ok(Some(found));
    }
    if let Some(found) = compare_absolute_relative(
        location,
        "particle.contact.weight",
        expected.weight_bits,
        actual.weight_bits,
    )? {
        return Ok(Some(found));
    }
    Ok(compare_vec_exact(
        location,
        "particle.force.range",
        expected.force,
        actual.force,
    )
    .or_else(|| {
        exact(
            location,
            "particle.zombie.lifecycle",
            &expected.pending_destruction,
            &actual.pending_destruction,
        )
    }))
}

fn compare_occurrence(
    location: Location,
    expected: &Phase9Occurrence,
    actual: &Phase9Occurrence,
) -> Option<Phase9Mismatch> {
    exact(
        location,
        "particle.listener.occurrence",
        &expected.ordinal,
        &actual.ordinal,
    )
    .or_else(|| {
        exact(
            location,
            "particle.filter.decision",
            &expected.kind,
            &actual.kind,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.storage.identity",
            &expected.system_id,
            &actual.system_id,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.contact.identity",
            &expected.maybe_particle_id,
            &actual.maybe_particle_id,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.contact.identity",
            &expected.maybe_other_particle_id,
            &actual.maybe_other_particle_id,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.coupling.identity",
            &expected.maybe_fixture_id,
            &actual.maybe_fixture_id,
        )
    })
}

fn compare_particle_contact(
    location: Location,
    expected: &Phase9ParticleContactObservation,
    actual: &Phase9ParticleContactObservation,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    if let Some(found) = exact(
        location,
        "particle.contact.identity",
        &expected.system_id,
        &actual.system_id,
    )
    .or_else(|| {
        exact(
            location,
            "particle.contact.identity",
            &expected.particle_a_id,
            &actual.particle_a_id,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.contact.identity",
            &expected.particle_b_id,
            &actual.particle_b_id,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.strict_contact.branch",
            &expected.flags_bits,
            &actual.flags_bits,
        )
    }) {
        return Ok(Some(found));
    }
    if let Some(found) = compare_absolute_relative(
        location,
        "particle.contact.weight",
        expected.weight_bits,
        actual.weight_bits,
    )? {
        return Ok(Some(found));
    }
    compare_vec_ulps(
        location,
        "particle.contact.normal",
        expected.normal,
        actual.normal,
    )
}

fn compare_body_contact(
    location: Location,
    expected: &Phase9BodyContactObservation,
    actual: &Phase9BodyContactObservation,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    if let Some(found) = exact(
        location,
        "particle.coupling.identity",
        &expected.system_id,
        &actual.system_id,
    )
    .or_else(|| {
        exact(
            location,
            "particle.coupling.identity",
            &expected.particle_id,
            &actual.particle_id,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.coupling.identity",
            &expected.body_id,
            &actual.body_id,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.coupling.identity",
            &expected.fixture_id,
            &actual.fixture_id,
        )
    }) {
        return Ok(Some(found));
    }
    if let Some(found) = compare_absolute_relative(
        location,
        "particle.contact.weight",
        expected.weight_bits,
        actual.weight_bits,
    )?
    .or(compare_vec_ulps(
        location,
        "particle.contact.normal",
        expected.normal,
        actual.normal,
    )?) {
        return Ok(Some(found));
    }
    compare_dimensioned(
        location,
        "particle.body_contact.mass",
        expected.mass_bits,
        actual.mass_bits,
        PHASE9_BODY_MASS_ABSOLUTE,
    )
}

fn compare_statistics(
    location: Location,
    expected: &Phase9StatisticsObservation,
    actual: &Phase9StatisticsObservation,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    if let Some(found) = exact(
        location,
        "particle.storage.identity",
        &expected.maybe_system_id,
        &actual.maybe_system_id,
    )
    .or_else(|| {
        exact(
            location,
            "particle.statistics.counts",
            &expected.system_count,
            &actual.system_count,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.statistics.counts",
            &expected.particle_count,
            &actual.particle_count,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.statistics.counts",
            &expected.pending_particle_count,
            &actual.pending_particle_count,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.statistics.counts",
            &expected.particle_contact_count,
            &actual.particle_contact_count,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.statistics.counts",
            &expected.body_contact_count,
            &actual.body_contact_count,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.lifetime.order",
            &expected.stuck_particle_ids,
            &actual.stuck_particle_ids,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.capacity.mode",
            &expected.declared_capacity,
            &actual.declared_capacity,
        )
    })
    .or_else(|| {
        exact(
            location,
            "particle.capacity.mode",
            &expected.effective_capacity,
            &actual.effective_capacity,
        )
    }) {
        return Ok(Some(found));
    }
    compare_absolute_relative(
        location,
        "particle.statistics.collision_energy",
        expected.collision_energy_bits,
        actual.collision_energy_bits,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "ray structure is compared as one parallel record"
)]
fn compare_ray(
    location: Location,
    expected_terminated: bool,
    expected_particles: &[liquidfun_test_protocol::ScenarioId],
    expected_fractions: &[FloatBits],
    actual_terminated: bool,
    actual_particles: &[liquidfun_test_protocol::ScenarioId],
    actual_fractions: &[FloatBits],
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    if expected_particles.len() != expected_fractions.len()
        || actual_particles.len() != actual_fractions.len()
    {
        return Err(Phase9ComparatorError::Structure {
            path: "particle.ray.fraction",
            expected: format!(
                "{} ids/{} fractions",
                expected_particles.len(),
                expected_fractions.len()
            )
            .into(),
            actual: format!(
                "{} ids/{} fractions",
                actual_particles.len(),
                actual_fractions.len()
            )
            .into(),
        });
    }
    if let Some(found) = exact(
        location,
        "particle.query.culling",
        &expected_terminated,
        &actual_terminated,
    )
    .or_else(|| {
        exact(
            location,
            "particle.query.order",
            &expected_particles,
            &actual_particles,
        )
    }) {
        return Ok(Some(found));
    }
    for (expected, actual) in expected_fractions.iter().zip(actual_fractions) {
        if let Some(found) = compare_dimensioned(
            location,
            "particle.ray.fraction",
            *expected,
            *actual,
            PHASE9_RAY_FRACTION_ABSOLUTE,
        )? {
            return Ok(Some(found));
        }
    }
    Ok(None)
}

fn compare_vec_ulps(
    location: Location,
    path: &'static str,
    expected: Vec2Bits,
    actual: Vec2Bits,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    compare_ulps(location, path, expected.x_bits, actual.x_bits).and_then(|maybe| {
        maybe.map_or_else(
            || compare_ulps(location, path, expected.y_bits, actual.y_bits),
            |found| Ok(Some(found)),
        )
    })
}

fn compare_vec_exact(
    location: Location,
    path: &'static str,
    expected: Vec2Bits,
    actual: Vec2Bits,
) -> Option<Phase9Mismatch> {
    bits_exact(location, path, expected.x_bits, actual.x_bits)
        .or_else(|| bits_exact(location, path, expected.y_bits, actual.y_bits))
}

fn compare_ulps(
    location: Location,
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    finite(path, expected, actual)?;
    Ok(
        (ulp_distance(expected.bits(), actual.bits()) > PHASE9_MAX_ULPS)
            .then(|| numeric_mismatch(location, path, expected, actual)),
    )
}

fn compare_absolute_relative(
    location: Location,
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    finite(path, expected, actual)?;
    let expected_value = expected.to_f32();
    let actual_value = actual.to_f32();
    let difference = (expected_value - actual_value).abs();
    let matches = difference <= PHASE9_ABSOLUTE_RELATIVE_ABSOLUTE
        || difference
            <= PHASE9_ABSOLUTE_RELATIVE_RELATIVE * expected_value.abs().max(actual_value.abs());
    Ok((!matches).then(|| numeric_mismatch(location, path, expected, actual)))
}

fn compare_dimensioned(
    location: Location,
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
    maximum: f32,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    finite(path, expected, actual)?;
    Ok(((expected.to_f32() - actual.to_f32()).abs() > maximum)
        .then(|| numeric_mismatch(location, path, expected, actual)))
}

fn finite(
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
) -> Result<(), Phase9ComparatorError> {
    if expected.to_f32().is_finite() && actual.to_f32().is_finite() {
        return Ok(());
    }
    Err(Phase9ComparatorError::NonFinite { path })
}

fn exact<T: std::fmt::Debug + PartialEq>(
    location: Location,
    path: &'static str,
    expected: &T,
    actual: &T,
) -> Option<Phase9Mismatch> {
    (expected != actual).then(|| {
        mismatch(
            location,
            path,
            format!("{expected:?}"),
            format!("{actual:?}"),
            None,
        )
    })
}

fn bits_exact(
    location: Location,
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
) -> Option<Phase9Mismatch> {
    (expected != actual).then(|| numeric_mismatch(location, path, expected, actual))
}

fn numeric_mismatch(
    location: Location,
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
) -> Phase9Mismatch {
    mismatch(
        location,
        path,
        format!("0x{:08x}", expected.bits()),
        format!("0x{:08x}", actual.bits()),
        Some((expected, actual)),
    )
}

fn mismatch(
    location: Location,
    path: &'static str,
    expected: String,
    actual: String,
    maybe_bits: Option<(FloatBits, FloatBits)>,
) -> Phase9Mismatch {
    let kind = phase9_policy_for_path(path)
        .expect("every comparator path is a closed reviewed Phase 9 policy");
    let input = format!(
        "{PHASE9_REGISTRY_ID}\0{}\0{}\0{}\0{path}\0{kind:?}\0{expected}\0{actual}\0{:?}",
        location.timeline, location.checkpoint, location.observation, maybe_bits,
    );
    Phase9Mismatch {
        signature_sha256: Sha256Hex::from_digest(Sha256::digest(input.as_bytes()).into()),
        timeline_index: location.timeline,
        checkpoint_index: location.checkpoint,
        observation_index: location.observation,
        semantic_path: path,
        kind,
        expected: expected.into(),
        actual: actual.into(),
        maybe_expected_bits: maybe_bits.map(|bits| bits.0),
        maybe_actual_bits: maybe_bits.map(|bits| bits.1),
    }
}

fn policy_error(reason: String) -> Phase9ComparatorError {
    Phase9ComparatorError::PolicyRegistry {
        reason: reason.into(),
    }
}

const fn observation_kind(observation: &Phase9ParticleObservation) -> &'static str {
    match observation {
        Phase9ParticleObservation::System { .. } => "system",
        Phase9ParticleObservation::Particle { .. } => "particle",
        Phase9ParticleObservation::Lifecycle { .. } => "lifecycle",
        Phase9ParticleObservation::ParticleContact { .. } => "particle_contact",
        Phase9ParticleObservation::BodyContact { .. } => "body_contact",
        Phase9ParticleObservation::Statistics { .. } => "statistics",
        Phase9ParticleObservation::Query { .. } => "query",
        Phase9ParticleObservation::RayCast { .. } => "ray_cast",
        Phase9ParticleObservation::MixedState { .. } => "mixed_state",
    }
}

fn ulp_distance(left: u32, right: u32) -> u32 {
    ordered_float_bits(left).abs_diff(ordered_float_bits(right))
}

const fn ordered_float_bits(bits: u32) -> u32 {
    if bits & 0x8000_0000 == 0 {
        bits | 0x8000_0000
    } else {
        !bits
    }
}
