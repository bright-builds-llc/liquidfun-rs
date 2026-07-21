//! Exhaustive fail-closed comparison for Phase 10 semantic observations.

use std::collections::BTreeSet;

use liquidfun_test_protocol::{
    Phase10Observation, Phase10StateObservation, Phase10ValidationKind, Sha256Hex,
};

mod numeric;
mod records;
mod registry;

use numeric::{mismatch_if, numeric, numeric_transform, numeric_vec};
pub use registry::{
    PHASE10_POLICY_REGISTRY, PHASE10_REQUIRED_POLICY_PATHS, Phase10Policy, Phase10PolicyKind,
};

/// Comparison authority selected by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase10ComparisonMode {
    /// Same-build deterministic replay: canonical semantic bytes must be identical.
    D0ByteIdentity,
    /// Cross-engine semantic comparison through the closed policy registry.
    D1Semantic,
}

/// Fail-closed error that is not parity mismatch evidence.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Phase10ComparatorError {
    /// Registry contents were missing, duplicated, reordered, wildcarded, or unknown.
    #[error("invalid Phase 10 policy registry: {reason}")]
    PolicyRegistry {
        /// Bounded reason the closed registry was rejected.
        reason: Box<str>,
    },
    /// One side failed strict semantic result validation.
    #[error("{side} Phase 10 result validation failed: {kind:?}")]
    ResultValidation {
        /// Compared role whose observation was invalid.
        side: &'static str,
        /// Closed protocol validation category.
        kind: Phase10ValidationKind,
    },
    /// Canonical semantic serialization failed.
    #[error("Phase 10 canonical semantic encoding failed")]
    CanonicalEncoding,
}

/// Stable first contextual Phase 10 divergence.
#[derive(Debug, Clone, PartialEq)]
pub struct Phase10Mismatch {
    signature_sha256: Sha256Hex,
    semantic_path: &'static str,
    policy: Phase10PolicyKind,
    scenario: Box<str>,
    operation: &'static str,
    entity: Box<str>,
    index: usize,
    expected: Box<str>,
    actual: Box<str>,
}

impl Phase10Mismatch {
    /// Returns the deterministic first-divergence identity.
    #[must_use]
    pub const fn signature_sha256(&self) -> &Sha256Hex {
        &self.signature_sha256
    }
    /// Returns the closed field path.
    #[must_use]
    pub const fn semantic_path(&self) -> &'static str {
        self.semantic_path
    }
    /// Returns the reviewed path policy.
    #[must_use]
    pub const fn policy(&self) -> Phase10PolicyKind {
        self.policy
    }
    /// Returns the scenario identity used by the semantic result.
    #[must_use]
    pub fn scenario(&self) -> &str {
        &self.scenario
    }
    /// Returns the semantic operation owning the observation.
    #[must_use]
    pub const fn operation(&self) -> &'static str {
        self.operation
    }
    /// Returns the nearest stable entity identity or collection name.
    #[must_use]
    pub fn entity(&self) -> &str {
        &self.entity
    }
    /// Returns the source-ordered record or vector-component index.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }
    /// Returns the bounded expected diagnostic.
    #[must_use]
    pub fn expected(&self) -> &str {
        &self.expected
    }
    /// Returns the bounded actual diagnostic.
    #[must_use]
    pub fn actual(&self) -> &str {
        &self.actual
    }
}

/// Complete comparison outcome.
#[derive(Debug, Clone, PartialEq)]
pub enum Phase10ComparisonOutcome {
    /// All fields matched and the complete registry was consumed.
    Match {
        /// Complete registry in reviewed source order.
        consumed_paths: Box<[&'static str]>,
    },
    /// First source-ordered semantic disagreement.
    PhysicsMismatch(Box<Phase10Mismatch>),
}

/// Validates an exact candidate registry without wildcard or private-pass fallback.
///
/// # Errors
///
/// Returns [`Phase10ComparatorError::PolicyRegistry`] for every open,
/// incomplete, duplicated, unknown, reordered, or private solver binding.
pub fn validate_phase10_policy_registry(
    paths: &[&str],
) -> Result<Box<[&'static str]>, Phase10ComparatorError> {
    let known = PHASE10_POLICY_REGISTRY
        .iter()
        .map(|policy| policy.path)
        .collect::<BTreeSet<_>>();
    let mut consumed = BTreeSet::new();
    for path in paths {
        if path.contains('*') || path.contains('?') {
            return Err(policy_error(format!("wildcard path `{path}`")));
        }
        if path.contains("pass_id")
            || path.contains("pass_trace")
            || path.contains("pass_inventory")
        {
            return Err(policy_error(format!("private solver field `{path}`")));
        }
        if !known.contains(path) {
            return Err(policy_error(format!("unknown path `{path}`")));
        }
        if !consumed.insert(*path) {
            return Err(policy_error(format!("duplicate path `{path}`")));
        }
    }
    if paths != PHASE10_REQUIRED_POLICY_PATHS {
        let reason = PHASE10_REQUIRED_POLICY_PATHS
            .iter()
            .find(|path| !consumed.contains(**path))
            .map_or_else(
                || "policy paths are not in reviewed source order".to_owned(),
                |path| format!("missing path `{path}`"),
            );
        return Err(policy_error(reason));
    }
    Ok(PHASE10_REQUIRED_POLICY_PATHS.into())
}

/// Compares two strict Phase 10 semantic observations under D0 or D1 authority.
///
/// # Errors
///
/// Returns a harness error when the registry is invalid, either observation
/// fails strict validation, or canonical D0 encoding fails.
pub fn compare_phase10_observations(
    mode: Phase10ComparisonMode,
    expected: &Phase10Observation,
    actual: &Phase10Observation,
) -> Result<Phase10ComparisonOutcome, Phase10ComparatorError> {
    let consumed_paths = validate_phase10_policy_registry(PHASE10_REQUIRED_POLICY_PATHS)?;
    validate(expected, "expected")?;
    validate(actual, "actual")?;
    let Phase10Observation::State { state: expected } = expected;
    let Phase10Observation::State { state: actual } = actual;
    let scenario = expected.provenance.generator_id.as_str();
    let maybe_mismatch = match mode {
        Phase10ComparisonMode::D0ByteIdentity => {
            let expected_bytes = serde_json::to_vec(expected)
                .map_err(|_| Phase10ComparatorError::CanonicalEncoding)?;
            let actual_bytes = serde_json::to_vec(actual)
                .map_err(|_| Phase10ComparatorError::CanonicalEncoding)?;
            mismatch_if(
                scenario,
                "state",
                "phase10",
                0,
                "phase10.d0.bytes",
                &expected_bytes,
                &actual_bytes,
            )
        }
        Phase10ComparisonMode::D1Semantic => compare_state(scenario, expected, actual)?,
    };
    Ok(maybe_mismatch.map_or(
        Phase10ComparisonOutcome::Match { consumed_paths },
        |found| Phase10ComparisonOutcome::PhysicsMismatch(Box::new(found)),
    ))
}

fn validate(
    observation: &Phase10Observation,
    side: &'static str,
) -> Result<(), Phase10ComparatorError> {
    observation
        .validate_semantics()
        .map_err(|error| Phase10ComparatorError::ResultValidation {
            side,
            kind: error.kind(),
        })
}

macro_rules! check {
    ($scenario:expr, $operation:expr, $entity:expr, $index:expr, $path:expr, $left:expr, $right:expr) => {
        if let Some(found) = mismatch_if(
            $scenario, $operation, $entity, $index, $path, &$left, &$right,
        ) {
            return Ok(Some(found));
        }
    };
}

macro_rules! check_len {
    ($scenario:expr, $entity:expr, $path:expr, $left:expr, $right:expr) => {
        if let Some(found) = mismatch_if(
            $scenario,
            "state",
            $entity,
            $left.len().min($right.len()),
            $path,
            &$left.len(),
            &$right.len(),
        ) {
            return Ok(Some(found));
        }
    };
}

#[allow(
    clippy::too_many_lines,
    reason = "one source-ordered walker proves every top-level schema field"
)]
fn compare_state(
    scenario: &str,
    expected: &Phase10StateObservation,
    actual: &Phase10StateObservation,
) -> Result<Option<Phase10Mismatch>, Phase10ComparatorError> {
    check!(
        scenario,
        "state",
        "phase10",
        0,
        "phase10.provenance",
        expected.provenance,
        actual.provenance
    );
    check!(
        scenario,
        "state",
        "phase10",
        0,
        "phase10.outcome",
        expected.outcome,
        actual.outcome
    );
    check_len!(
        scenario,
        "groups",
        "phase10.group.membership",
        expected.groups,
        actual.groups
    );
    for (index, (left, right)) in expected.groups.iter().zip(&actual.groups).enumerate() {
        if let Some(found) = records::compare_group(scenario, index, left, right)? {
            return Ok(Some(found));
        }
    }
    check_len!(
        scenario,
        "particles",
        "phase10.particle.identity",
        expected.particles,
        actual.particles
    );
    for (index, (left, right)) in expected.particles.iter().zip(&actual.particles).enumerate() {
        if let Some(found) = records::compare_particle(scenario, index, left, right)? {
            return Ok(Some(found));
        }
    }
    check_len!(
        scenario,
        "pairs",
        "phase10.pair.identity",
        expected.pairs,
        actual.pairs
    );
    for (index, (left, right)) in expected.pairs.iter().zip(&actual.pairs).enumerate() {
        if let Some(found) = records::compare_pair(scenario, index, left, right)? {
            return Ok(Some(found));
        }
    }
    check_len!(
        scenario,
        "triads",
        "phase10.triad.identity",
        expected.triads,
        actual.triads
    );
    for (index, (left, right)) in expected.triads.iter().zip(&actual.triads).enumerate() {
        if let Some(found) = records::compare_triad(scenario, index, left, right)? {
            return Ok(Some(found));
        }
    }
    check_len!(
        scenario,
        "particle_contacts",
        "phase10.contact.identity",
        expected.particle_contacts,
        actual.particle_contacts
    );
    for (index, (left, right)) in expected
        .particle_contacts
        .iter()
        .zip(&actual.particle_contacts)
        .enumerate()
    {
        if let Some(found) = records::compare_particle_contact(scenario, index, left, right)? {
            return Ok(Some(found));
        }
    }
    check_len!(
        scenario,
        "body_contacts",
        "phase10.contact.identity",
        expected.body_contacts,
        actual.body_contacts
    );
    for (index, (left, right)) in expected
        .body_contacts
        .iter()
        .zip(&actual.body_contacts)
        .enumerate()
    {
        if let Some(found) = records::compare_body_contact(scenario, index, left, right)? {
            return Ok(Some(found));
        }
    }
    check_len!(
        scenario,
        "events",
        "phase10.event.ordinal",
        expected.events,
        actual.events
    );
    for (index, (left, right)) in expected.events.iter().zip(&actual.events).enumerate() {
        let entity = format!("event:{index}");
        check!(
            scenario,
            "event",
            &entity,
            index,
            "phase10.event.ordinal",
            left.ordinal,
            right.ordinal
        );
        check!(
            scenario,
            "event",
            &entity,
            index,
            "phase10.event.kind",
            left.kind,
            right.kind
        );
        check!(
            scenario,
            "event",
            &entity,
            index,
            "phase10.event.identity",
            (
                &left.system_id,
                &left.maybe_group_id,
                &left.maybe_particle_id,
                &left.maybe_other_particle_id,
                &left.maybe_body_id
            ),
            (
                &right.system_id,
                &right.maybe_group_id,
                &right.maybe_particle_id,
                &right.maybe_other_particle_id,
                &right.maybe_body_id
            )
        );
    }
    check_len!(
        scenario,
        "witnesses",
        "phase10.witness.ordinal",
        expected.witnesses,
        actual.witnesses
    );
    for (index, (left, right)) in expected.witnesses.iter().zip(&actual.witnesses).enumerate() {
        check!(
            scenario,
            "witness",
            &format!("witness:{index}"),
            index,
            "phase10.witness.ordinal",
            left.ordinal,
            right.ordinal
        );
        check!(
            scenario,
            "witness",
            &format!("witness:{index}"),
            index,
            "phase10.witness.leaf",
            left.behavior_leaf,
            right.behavior_leaf
        );
        check!(
            scenario,
            "witness",
            &format!("witness:{index}"),
            index,
            "phase10.witness.role",
            left.role,
            right.role
        );
        if let Some(found) =
            records::compare_witness(scenario, index, &left.observation, &right.observation)?
        {
            return Ok(Some(found));
        }
    }
    Ok(None)
}

fn policy_error(reason: String) -> Phase10ComparatorError {
    Phase10ComparatorError::PolicyRegistry {
        reason: reason.into(),
    }
}
