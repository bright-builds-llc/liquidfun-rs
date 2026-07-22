use crate::{RigidJointKind, RigidWorldWitness};

use super::{CatalogError, CatalogErrorKind, CatalogSlug, RunSettings};

const MAXIMUM_EVIDENCE_ID_BYTES: usize = 160;
const PHASE10_NATIVE_LEAVES: &[&str] = &[
    "phase10/group_create",
    "phase10/group_append",
    "phase10/group_join",
    "phase10/group_split",
    "phase10/group_flags",
    "phase10/group_destroy",
    "phase10/spring",
    "phase10/elastic",
    "phase10/reactive",
    "phase10/water",
    "phase10/zombie",
    "phase10/viscous",
    "phase10/powder",
    "phase10/tensile",
    "phase10/color_mixing",
    "phase10/static_pressure",
    "phase10/repulsive",
    "phase10/barrier",
    "phase10/solid_group",
    "phase10/rigid_group",
    "phase10/body_interaction",
    "phase10/wall",
];

/// A validated Phase 9 branch or Phase 10 compatibility-leaf identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CatalogEvidenceId(Box<str>);

impl CatalogEvidenceId {
    /// Parses a bounded evidence identity without exposing an unvalidated string downstream.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError`] when the identity is empty, oversized, non-ASCII, or contains
    /// characters outside the reviewed evidence-path alphabet.
    pub fn new(value: impl Into<String>) -> Result<Self, CatalogError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > MAXIMUM_EVIDENCE_ID_BYTES
            || !value.is_ascii()
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'-' | b'_' | b'.' | b'/')
            })
            || value.contains("//")
            || value.starts_with('/')
            || value.ends_with('/')
        {
            return Err(CatalogError::new(CatalogErrorKind::InvalidMetadata));
        }
        Ok(Self(value.into_boxed_str()))
    }

    /// Returns the validated evidence identity.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Closed evidence authority carried by one native scenario mapping.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CatalogEvidence {
    /// A retained rigid-world semantic witness.
    Rigid(RigidWorldWitness),
    /// A sealed Phase 9 branch identity.
    Phase9(CatalogEvidenceId),
    /// A sealed Phase 10 compatibility-leaf identity.
    Phase10(CatalogEvidenceId),
}

/// A downstream use that a native catalog definition explicitly permits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScenarioConsumer {
    /// Deterministic regression replay.
    Regression,
    /// Repeatable benchmark execution.
    Benchmark,
    /// Interactive testbed visualization.
    Visual,
}

impl ScenarioConsumer {
    /// Every downstream consumer controlled by catalog metadata.
    pub const ALL: [Self; 3] = [Self::Regression, Self::Benchmark, Self::Visual];
}

/// Typed test/evidence links and downstream eligibility for one native scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogCoverage {
    test_ids: Box<[CatalogSlug]>,
    evidence_leaves: Box<[CatalogEvidence]>,
    regression: bool,
    benchmark: bool,
    visual: bool,
}

impl CatalogCoverage {
    /// Creates a complete non-empty coverage declaration.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError`] when no public test or evidence leaf is mapped.
    pub fn new(
        test_ids: Vec<CatalogSlug>,
        evidence_leaves: Vec<CatalogEvidence>,
        regression: bool,
        benchmark: bool,
        visual: bool,
    ) -> Result<Self, CatalogError> {
        if test_ids.is_empty()
            || evidence_leaves.is_empty()
            || evidence_leaves.iter().enumerate().any(|(index, evidence)| {
                !valid_evidence(evidence) || evidence_leaves[index + 1..].contains(evidence)
            })
        {
            return Err(CatalogError::new(CatalogErrorKind::InvalidMetadata));
        }
        Ok(Self {
            test_ids: test_ids.into_boxed_slice(),
            evidence_leaves: evidence_leaves.into_boxed_slice(),
            regression,
            benchmark,
            visual,
        })
    }

    /// Returns stable IDs for the public integration tests protecting this definition.
    #[must_use]
    pub fn test_ids(&self) -> &[CatalogSlug] {
        &self.test_ids
    }

    /// Returns typed sealed evidence leaves covered by this definition.
    #[must_use]
    pub fn evidence_leaves(&self) -> &[CatalogEvidence] {
        &self.evidence_leaves
    }

    /// Returns whether the definition may feed the selected downstream consumer.
    #[must_use]
    pub const fn is_eligible(&self, consumer: ScenarioConsumer) -> bool {
        match consumer {
            ScenarioConsumer::Regression => self.regression,
            ScenarioConsumer::Benchmark => self.benchmark,
            ScenarioConsumer::Visual => self.visual,
        }
    }
}

fn valid_evidence(evidence: &CatalogEvidence) -> bool {
    match evidence {
        CatalogEvidence::Rigid(_) => true,
        CatalogEvidence::Phase9(id) => crate::PHASE9_REQUIRED_BRANCH_IDS
            .lines()
            .any(|branch_id| branch_id == id.as_str()),
        CatalogEvidence::Phase10(id) => {
            PHASE10_NATIVE_LEAVES.contains(&id.as_str())
                || id
                    .as_str()
                    .strip_prefix("inherited/")
                    .is_some_and(|branch_id| {
                        crate::PHASE9_REQUIRED_BRANCH_IDS
                            .lines()
                            .any(|required| required == branch_id)
                    })
        }
    }
}

/// Stable discovery and execution metadata attached to one native catalog definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogMetadata {
    tags: Box<[CatalogSlug]>,
    default_settings: RunSettings,
    coverage: CatalogCoverage,
    maybe_joint_kind: Option<RigidJointKind>,
}

impl CatalogMetadata {
    /// Creates validated scenario metadata.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError`] when the definition has no stable discovery tags.
    pub fn new(
        tags: Vec<CatalogSlug>,
        default_settings: RunSettings,
        coverage: CatalogCoverage,
        maybe_joint_kind: Option<RigidJointKind>,
    ) -> Result<Self, CatalogError> {
        if tags.is_empty() {
            return Err(CatalogError::new(CatalogErrorKind::InvalidMetadata));
        }
        Ok(Self {
            tags: tags.into_boxed_slice(),
            default_settings,
            coverage,
            maybe_joint_kind,
        })
    }

    /// Returns stable lower-kebab discovery tags.
    #[must_use]
    pub fn tags(&self) -> &[CatalogSlug] {
        &self.tags
    }

    /// Returns the exact default timestep and solver settings.
    #[must_use]
    pub const fn default_settings(&self) -> RunSettings {
        self.default_settings
    }

    /// Returns typed coverage and downstream eligibility.
    #[must_use]
    pub const fn coverage(&self) -> &CatalogCoverage {
        &self.coverage
    }

    /// Returns the exact joint kind for a joint-specific definition.
    #[must_use]
    pub const fn joint_kind(&self) -> Option<RigidJointKind> {
        self.maybe_joint_kind
    }
}
