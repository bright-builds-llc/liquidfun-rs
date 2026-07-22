use crate::{RigidJointKind, RigidWorldWitness};

use super::{CatalogError, CatalogErrorKind, CatalogSlug, RunSettings};

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
    evidence_leaves: Box<[RigidWorldWitness]>,
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
        evidence_leaves: Vec<RigidWorldWitness>,
        regression: bool,
        benchmark: bool,
        visual: bool,
    ) -> Result<Self, CatalogError> {
        if test_ids.is_empty() || evidence_leaves.is_empty() {
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

    /// Returns typed rigid-world evidence leaves covered by this definition.
    #[must_use]
    pub fn evidence_leaves(&self) -> &[RigidWorldWitness] {
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
