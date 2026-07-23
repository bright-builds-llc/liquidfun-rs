//! Complete typed matrix preparation and paired native sample execution.

use std::time::Duration;

use liquidfun_differential::{
    MAXIMUM_BENCHMARK_ACTIONS, MAXIMUM_BENCHMARK_SAMPLES, MAXIMUM_BENCHMARK_WARMUPS,
    PerformanceExecutionError, PreparedNativeBenchmark,
};
use liquidfun_test_protocol::{
    CanonicalCheckpoint, CatalogSlug, PerformanceCase, PerformanceMatrix, PerformancePolicy,
    PerformanceSizePoint, PerformanceWorkloadKind, ResolveRequest, ResolvedScenario,
    ScenarioConsumer, Sha256Hex, resolve_catalog, reviewed_scenario_catalog,
};

/// Stable failure categories for complete-matrix benchmark preparation and execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkCaseErrorKind {
    /// The reviewed matrix or catalog could not be constructed.
    Catalog,
    /// A mapped scenario is not benchmark-eligible.
    Ineligible,
    /// Resolved bytes or hash disagree with the sealed matrix row.
    IdentityMismatch,
    /// A requested warm-up, sample, or action count exceeds its bound.
    ResourceLimit,
    /// Native execution failed.
    NativeExecution,
    /// Post-timer semantic state disagreed with prepared authority.
    CheckpointMismatch,
    /// Accumulating sample durations overflowed.
    DurationOverflow,
}

/// Redacted paired benchmark failure without raw protocol or engine-private state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("paired benchmark failure: {kind:?}")]
pub struct BenchmarkCaseError {
    kind: BenchmarkCaseErrorKind,
}

impl BenchmarkCaseError {
    const fn new(kind: BenchmarkCaseErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(self) -> BenchmarkCaseErrorKind {
        self.kind
    }
}

/// Closed resource bounds applied before repeated benchmark execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkExecutionBounds {
    warmups: u32,
    samples: u64,
    actions: u32,
}

impl BenchmarkExecutionBounds {
    /// Validates warm-up, sample, and logical-action counts.
    ///
    /// # Errors
    ///
    /// Returns a resource-limit failure for zero work or any count above its reviewed cap.
    pub const fn new(warmups: u32, samples: u64, actions: u32) -> Result<Self, BenchmarkCaseError> {
        if warmups > MAXIMUM_BENCHMARK_WARMUPS
            || samples == 0
            || samples > MAXIMUM_BENCHMARK_SAMPLES
            || actions == 0
            || actions > MAXIMUM_BENCHMARK_ACTIONS
        {
            return Err(BenchmarkCaseError::new(
                BenchmarkCaseErrorKind::ResourceLimit,
            ));
        }
        Ok(Self {
            warmups,
            samples,
            actions,
        })
    }
}

/// Caller-visible order required when the oracle half is added to a paired sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairedEngineOrder {
    /// Native Rust executes before the pinned oracle.
    NativeThenOracle,
    /// Pinned oracle executes before native Rust.
    OracleThenNative,
}

/// One complete sealed matrix row prepared for native execution.
pub struct PairedBenchmarkCase {
    matrix_case: PerformanceCase,
    prepared: PreparedNativeBenchmark,
    bounds: BenchmarkExecutionBounds,
}

impl PairedBenchmarkCase {
    /// Returns the closed workload kind.
    #[must_use]
    pub const fn workload(&self) -> PerformanceWorkloadKind {
        self.matrix_case.workload()
    }

    /// Returns the reviewed cardinality point.
    #[must_use]
    pub const fn size_point(&self) -> PerformanceSizePoint {
        self.matrix_case.size_point()
    }

    /// Returns the exact canonical resolved-input hash.
    #[must_use]
    pub const fn resolved_sha256(&self) -> &Sha256Hex {
        self.matrix_case.resolved_sha256()
    }

    /// Returns the immutable canonical resolved input.
    #[must_use]
    pub const fn resolved(&self) -> &ResolvedScenario {
        self.prepared.resolved()
    }

    /// Returns the post-setup semantic authority.
    #[must_use]
    pub const fn expected_checkpoint(&self) -> &CanonicalCheckpoint {
        self.prepared.expected_checkpoint()
    }

    /// Returns the fixed measured logical-action horizon.
    #[must_use]
    pub const fn logical_horizon(&self) -> u32 {
        self.bounds.actions
    }

    /// Returns the alternating caller order for one zero-based sample index.
    #[must_use]
    pub const fn sample_order(&self, sample_index: u64) -> PairedEngineOrder {
        if sample_index.is_multiple_of(2) {
            PairedEngineOrder::NativeThenOracle
        } else {
            PairedEngineOrder::OracleThenNative
        }
    }

    /// Measures one native half after checking the paired sample index.
    ///
    /// # Errors
    ///
    /// Returns a bounded failure before accepting a duration when the sample index, execution, or
    /// semantic checkpoint violates the sealed case.
    pub fn measure_native_sample(&self, sample_index: u64) -> Result<Duration, BenchmarkCaseError> {
        if sample_index >= self.bounds.samples {
            return Err(BenchmarkCaseError::new(
                BenchmarkCaseErrorKind::ResourceLimit,
            ));
        }
        self.prepared.measure_sample().map_err(map_execution_error)
    }

    /// Measures bounded Rust-only Criterion iterations over fresh validated sessions.
    ///
    /// # Errors
    ///
    /// Returns a bounded resource, execution, semantic, or duration failure.
    pub fn measure_native_iterations(
        &self,
        iterations: u64,
    ) -> Result<Duration, BenchmarkCaseError> {
        if iterations == 0 || iterations > self.bounds.samples {
            return Err(BenchmarkCaseError::new(
                BenchmarkCaseErrorKind::ResourceLimit,
            ));
        }
        let mut total = Duration::ZERO;
        for sample_index in 0..iterations {
            let elapsed = self.measure_native_sample(sample_index)?;
            total = total
                .checked_add(elapsed)
                .ok_or_else(|| BenchmarkCaseError::new(BenchmarkCaseErrorKind::DurationOverflow))?;
        }
        Ok(total)
    }

    /// Returns the stable diagnostic benchmark identifier.
    #[must_use]
    pub fn diagnostic_id(&self) -> String {
        format!(
            "diagnostic-{}-{}",
            self.workload().as_str(),
            size_point_id(self.size_point())
        )
    }
}

/// Loads and prepares every sealed Phase 12 matrix row in stable order.
///
/// # Errors
///
/// Returns a bounded failure before registration when catalog eligibility, resolved bytes/hash,
/// horizon, warm-up semantics, or resource limits drift.
pub fn paired_benchmark_cases() -> Result<Vec<PairedBenchmarkCase>, BenchmarkCaseError> {
    let matrix = PerformanceMatrix::reviewed_v1()
        .map_err(|_error| BenchmarkCaseError::new(BenchmarkCaseErrorKind::Catalog))?;
    let catalog = reviewed_scenario_catalog()
        .map_err(|_error| BenchmarkCaseError::new(BenchmarkCaseErrorKind::Catalog))?;
    let policy = PerformancePolicy::reviewed_v1();
    let mut prepared_by_hash = Vec::<(Sha256Hex, PreparedNativeBenchmark)>::new();
    let mut cases = Vec::with_capacity(matrix.cases().len());
    for matrix_case in matrix.cases().iter().cloned() {
        let bounds = BenchmarkExecutionBounds::new(
            u32::from(policy.warmup_runs()),
            MAXIMUM_BENCHMARK_SAMPLES,
            matrix_case.logical_horizon(),
        )?;
        let maybe_prepared = prepared_by_hash
            .iter()
            .find(|(sha256, _prepared)| sha256 == matrix_case.resolved_sha256())
            .map(|(_sha256, prepared)| prepared.clone());
        let prepared = if let Some(prepared) = maybe_prepared {
            prepared
        } else {
            let prepared = prepare_native_case(&catalog, &matrix_case, bounds.warmups)?;
            prepared_by_hash.push((matrix_case.resolved_sha256().clone(), prepared.clone()));
            prepared
        };
        cases.push(PairedBenchmarkCase {
            matrix_case,
            prepared,
            bounds,
        });
    }
    Ok(cases)
}

fn prepare_native_case(
    catalog: &liquidfun_test_protocol::ScenarioCatalog,
    matrix_case: &PerformanceCase,
    warmup_runs: u32,
) -> Result<PreparedNativeBenchmark, BenchmarkCaseError> {
    let slug = CatalogSlug::new(scenario_slug(matrix_case.workload()))
        .map_err(|_error| BenchmarkCaseError::new(BenchmarkCaseErrorKind::Catalog))?;
    let mapping = catalog
        .mapping(&slug, liquidfun_test_protocol::ScenarioVersion::CURRENT)
        .ok_or_else(|| BenchmarkCaseError::new(BenchmarkCaseErrorKind::Catalog))?;
    if !mapping.is_eligible(ScenarioConsumer::Benchmark) {
        return Err(BenchmarkCaseError::new(BenchmarkCaseErrorKind::Ineligible));
    }
    let definition = catalog
        .definitions()
        .iter()
        .find(|definition| definition.slug() == &slug)
        .ok_or_else(|| BenchmarkCaseError::new(BenchmarkCaseErrorKind::Catalog))?;
    let settings = definition
        .metadata()
        .ok_or_else(|| BenchmarkCaseError::new(BenchmarkCaseErrorKind::Catalog))?
        .default_settings();
    let resolved = resolve_catalog(
        catalog.definitions(),
        &ResolveRequest::new(slug, None, settings),
    )
    .map_err(|_error| BenchmarkCaseError::new(BenchmarkCaseErrorKind::Catalog))?;
    if resolved.identity().content_sha256() != matrix_case.resolved_sha256() {
        return Err(BenchmarkCaseError::new(
            BenchmarkCaseErrorKind::IdentityMismatch,
        ));
    }
    PreparedNativeBenchmark::new(
        resolved,
        matrix_case.resolved_sha256(),
        matrix_case.logical_horizon(),
        warmup_runs,
    )
    .map_err(map_execution_error)
}

const fn scenario_slug(workload: PerformanceWorkloadKind) -> &'static str {
    match workload {
        PerformanceWorkloadKind::Joints => "joint-distance-behavior",
        PerformanceWorkloadKind::ParticleContacts => "particle-contacts-and-coupling",
        PerformanceWorkloadKind::ParticleSort
        | PerformanceWorkloadKind::ParticlePressure
        | PerformanceWorkloadKind::LargeParticleSystem => "particle-group-construction-append",
        PerformanceWorkloadKind::ParticleLifecycle => "particle-system-pause-action",
        PerformanceWorkloadKind::AabbQuery => "particle-aabb-query-controls",
        PerformanceWorkloadKind::RayCast => "particle-ray-callback-controls",
        PerformanceWorkloadKind::WorldStep
        | PerformanceWorkloadKind::BroadPhase
        | PerformanceWorkloadKind::NarrowPhase
        | PerformanceWorkloadKind::ContactSolve
        | PerformanceWorkloadKind::Ccd
        | PerformanceWorkloadKind::MixedWorld => "rigid-runtime-mutation",
    }
}

const fn size_point_id(size_point: PerformanceSizePoint) -> &'static str {
    match size_point {
        PerformanceSizePoint::Fixed => "fixed",
        PerformanceSizePoint::Entities128 => "128",
        PerformanceSizePoint::Entities1024 => "1024",
        PerformanceSizePoint::Entities8192 => "8192",
    }
}

const fn map_execution_error(error: PerformanceExecutionError) -> BenchmarkCaseError {
    let kind = match error.kind() {
        liquidfun_differential::PerformanceExecutionErrorKind::ResolvedIdentity => {
            BenchmarkCaseErrorKind::IdentityMismatch
        }
        liquidfun_differential::PerformanceExecutionErrorKind::ResourceLimit
        | liquidfun_differential::PerformanceExecutionErrorKind::HorizonMismatch => {
            BenchmarkCaseErrorKind::ResourceLimit
        }
        liquidfun_differential::PerformanceExecutionErrorKind::NativeExecution => {
            BenchmarkCaseErrorKind::NativeExecution
        }
        liquidfun_differential::PerformanceExecutionErrorKind::CheckpointMismatch => {
            BenchmarkCaseErrorKind::CheckpointMismatch
        }
        liquidfun_differential::PerformanceExecutionErrorKind::DurationOverflow => {
            BenchmarkCaseErrorKind::DurationOverflow
        }
    };
    BenchmarkCaseError::new(kind)
}
