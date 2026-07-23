//! Closed performance evidence vocabulary for Phase 12.

mod matrix;
mod policy;
mod report;

pub use matrix::*;
pub use policy::*;
pub use report::*;

/// Stable validation categories for the performance evidence contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceErrorKind {
    /// Fewer than five independent baseline runs were requested.
    BaselineRunsBelowMinimum,
    /// More runs than the reviewed resource bound were requested.
    BaselineRunsAboveMaximum,
    /// Confidence was below 95 percent.
    ConfidenceBelowMinimum,
    /// Confidence exceeded 100 percent.
    ConfidenceAboveMaximum,
    /// The practical regression floor was below three percent.
    PracticalFloorBelowMinimum,
    /// A percentage-like value exceeded 100 percent.
    PercentageAboveMaximum,
    /// A matrix case identity appeared more than once.
    DuplicateCaseIdentity,
    /// The exact reviewed workload and size-point matrix was not present.
    IncompleteWorkloadMatrix,
    /// A matrix case did not bind a valid catalog scenario.
    InvalidCaseBinding,
    /// A stable identity field was empty or oversized.
    InvalidIdentityField,
    /// A timing sample or interval violated reviewed bounds.
    InvalidMeasurement,
    /// Performance data attempted to promote a D1 physics fixture.
    FixturePromotionForbidden,
    /// Deterministic JSON rendering failed.
    CanonicalEncoding,
    /// Catalog projection could not be produced.
    CatalogProjection,
}

/// Redacted performance-contract validation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("performance contract validation failure: {kind:?}")]
pub struct PerformanceError {
    kind: PerformanceErrorKind,
}

impl PerformanceError {
    pub(super) const fn new(kind: PerformanceErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(self) -> PerformanceErrorKind {
        self.kind
    }
}
