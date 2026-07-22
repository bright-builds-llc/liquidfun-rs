//! Private canonical-catalog benchmark support.
//!
//! The package resolves and validates representative catalog cases before Criterion sees them.
//! Each returned duration covers only the declared logical actions; restart, checkpoint capture,
//! and semantic validation remain outside that measured interval.

#![forbid(unsafe_code)]

use std::time::{Duration, Instant};

use liquidfun_differential::{
    NativeCatalogBackend, SessionCommand, SessionController, SessionControllerError,
};
use liquidfun_test_protocol::{
    CanonicalCheckpoint, CatalogSlug, FloatBits, ResolveRequest, ResolvedScenario, RunSettings,
    ScenarioConsumer, ScenarioVersion, Sha256Hex, resolve_catalog, reviewed_scenario_catalog,
};

type RestartFactory =
    fn(&ResolvedScenario) -> Result<SessionController<NativeCatalogBackend>, BenchmarkCaseError>;
type CheckpointValidator = fn(&CanonicalCheckpoint, &CanonicalCheckpoint) -> bool;

const CASE_SPECS: [CatalogBenchmarkSpec; 7] = [
    CatalogBenchmarkSpec::new(
        "rigid-runtime-mutation",
        "38acf7adfcfeb510cd3254614934d44604bcbd32c12cf15cd5861ae466252dae",
        1,
        1,
        1,
    ),
    CatalogBenchmarkSpec::new(
        "joint-distance-behavior",
        "2eaf8f031603887a6807d185404c839d366905a88c75f4076140c5ece6cf1af4",
        1,
        2,
        1,
    ),
    CatalogBenchmarkSpec::new(
        "particle-system-pause-action",
        "1a1f8e68f0a05f8cc16c589a7db6c4fb05e93b165fa5da13fb8cf241aaba3826",
        1,
        4,
        2,
    ),
    CatalogBenchmarkSpec::new(
        "particle-group-construction-append",
        "93a0f8c793f213b3dda6911e62830d0f02a3b14324193244d0cd6e1693512cda",
        1,
        4,
        2,
    ),
    CatalogBenchmarkSpec::new(
        "particle-contacts-and-coupling",
        "4f0c0f2279f0360c24c4ea10504b36ce8f506fd8b5a4415dcfd00819ea5de122",
        1,
        4,
        2,
    ),
    CatalogBenchmarkSpec::new(
        "particle-aabb-query-controls",
        "1ac03e065afbec5e90770d28578acfcc7b94909704298b2f9d03c50bba561467",
        1,
        3,
        2,
    ),
    CatalogBenchmarkSpec::new(
        "particle-ray-callback-controls",
        "d1e22656861f3906fbccd3ba033f952467758cd0620ab29a18e4a638b872279e",
        1,
        5,
        2,
    ),
];

#[derive(Debug, Clone, Copy)]
struct CatalogBenchmarkSpec {
    slug: &'static str,
    resolved_sha256: &'static str,
    warmup_runs: u32,
    measured_horizon: u32,
    particle_iterations: u32,
}

impl CatalogBenchmarkSpec {
    const fn new(
        slug: &'static str,
        resolved_sha256: &'static str,
        warmup_runs: u32,
        measured_horizon: u32,
        particle_iterations: u32,
    ) -> Self {
        Self {
            slug,
            resolved_sha256,
            warmup_runs,
            measured_horizon,
            particle_iterations,
        }
    }
}

/// Stable failure categories for benchmark preparation and sample validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkCaseErrorKind {
    /// The reviewed catalog or stable case identity did not validate.
    Catalog,
    /// The selected catalog mapping is not benchmark-eligible.
    Ineligible,
    /// Resolved bytes or settings disagree with the fixed case identity.
    IdentityMismatch,
    /// The resolved logical schedule disagrees with the fixed measured horizon.
    HorizonMismatch,
    /// Native session construction or logical execution failed.
    NativeExecution,
    /// The final semantic checkpoint disagrees with the validated expected checkpoint.
    CheckpointMismatch,
    /// Accumulating measured sample durations exceeded [`Duration`].
    DurationOverflow,
}

/// Bounded benchmark-case failure without raw protocol records or private engine details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("catalog benchmark failure: {kind:?}")]
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

/// One fully resolved, semantically validated benchmark case.
///
/// The case fixes every execution identity and carries the only restart and checkpoint-validation
/// functions used by the benchmark bridge.
pub struct CatalogBenchmarkCase {
    slug: CatalogSlug,
    scenario_version: ScenarioVersion,
    resolved_sha256: Sha256Hex,
    settings: RunSettings,
    warmup_runs: u32,
    measured_horizon: u32,
    resolved: ResolvedScenario,
    expected_checkpoint: CanonicalCheckpoint,
    restart_factory: RestartFactory,
    checkpoint_validator: CheckpointValidator,
}

impl CatalogBenchmarkCase {
    /// Returns the fixed stable catalog slug.
    #[must_use]
    pub const fn slug(&self) -> &CatalogSlug {
        &self.slug
    }

    /// Returns the fixed scenario schema version.
    #[must_use]
    pub const fn scenario_version(&self) -> ScenarioVersion {
        self.scenario_version
    }

    /// Returns the exact resolved-byte content hash.
    #[must_use]
    pub const fn resolved_sha256(&self) -> &Sha256Hex {
        &self.resolved_sha256
    }

    /// Returns the exact run settings used for validation and every sample.
    #[must_use]
    pub const fn settings(&self) -> RunSettings {
        self.settings
    }

    /// Returns the number of complete untimed warmup runs.
    #[must_use]
    pub const fn warmup_runs(&self) -> u32 {
        self.warmup_runs
    }

    /// Returns the exact number of logical actions timed per iteration.
    #[must_use]
    pub const fn measured_horizon(&self) -> u32 {
        self.measured_horizon
    }

    /// Returns the immutable canonical scenario shared by validation and every restart.
    #[must_use]
    pub const fn resolved(&self) -> &ResolvedScenario {
        &self.resolved
    }

    /// Returns the untimed semantic checkpoint accepted before benchmark registration.
    #[must_use]
    pub const fn expected_checkpoint(&self) -> &CanonicalCheckpoint {
        &self.expected_checkpoint
    }

    /// Applies this case's fixed semantic validator.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkCaseErrorKind::CheckpointMismatch`] for any semantic difference.
    pub fn validate_checkpoint(
        &self,
        candidate: &CanonicalCheckpoint,
    ) -> Result<(), BenchmarkCaseError> {
        if !(self.checkpoint_validator)(&self.expected_checkpoint, candidate) {
            return Err(BenchmarkCaseError::new(
                BenchmarkCaseErrorKind::CheckpointMismatch,
            ));
        }
        Ok(())
    }

    /// Executes `iterations` validated samples and returns only accumulated logical-tick time.
    ///
    /// A fresh session is constructed before each timer starts. After the timer stops, the method
    /// captures and validates the declared final checkpoint. A mismatch returns an error before
    /// Criterion can accept the duration as a timing sample.
    ///
    /// # Errors
    ///
    /// Returns a bounded benchmark preparation, native execution, semantic, or duration failure.
    pub fn measure_iterations(&self, iterations: u64) -> Result<Duration, BenchmarkCaseError> {
        let mut measured = Duration::ZERO;
        for _ in 0..iterations {
            let mut controller = (self.restart_factory)(&self.resolved)?;
            let started = Instant::now();
            execute_logical_horizon(&mut controller, self.measured_horizon)?;
            let elapsed = started.elapsed();
            measured = measured
                .checked_add(elapsed)
                .ok_or_else(|| BenchmarkCaseError::new(BenchmarkCaseErrorKind::DurationOverflow))?;
            let checkpoint = capture_final_checkpoint(&mut controller, &self.resolved)?;
            self.validate_checkpoint(&checkpoint)?;
        }
        Ok(measured)
    }
}

/// Resolves and pre-validates the representative rigid, joint, particle, group, mixed, query,
/// and ray smoke cases.
///
/// # Errors
///
/// Returns a bounded failure before benchmark registration when catalog authority, fixed identity,
/// native restart, or semantic checkpoint equivalence does not validate.
pub fn representative_catalog_benchmarks() -> Result<Vec<CatalogBenchmarkCase>, BenchmarkCaseError>
{
    let catalog = reviewed_scenario_catalog()
        .map_err(|_error| BenchmarkCaseError::new(BenchmarkCaseErrorKind::Catalog))?;
    CASE_SPECS
        .iter()
        .map(|spec| prepare_case(&catalog, *spec))
        .collect()
}

fn prepare_case(
    catalog: &liquidfun_test_protocol::ScenarioCatalog,
    spec: CatalogBenchmarkSpec,
) -> Result<CatalogBenchmarkCase, BenchmarkCaseError> {
    let slug = CatalogSlug::new(spec.slug)
        .map_err(|_error| BenchmarkCaseError::new(BenchmarkCaseErrorKind::Catalog))?;
    let scenario_version = ScenarioVersion::CURRENT;
    let mapping = catalog
        .mapping(&slug, scenario_version)
        .ok_or_else(|| BenchmarkCaseError::new(BenchmarkCaseErrorKind::Catalog))?;
    if !mapping.is_eligible(ScenarioConsumer::Benchmark) {
        return Err(BenchmarkCaseError::new(BenchmarkCaseErrorKind::Ineligible));
    }
    let definition = catalog
        .definitions()
        .iter()
        .find(|definition| {
            definition.slug() == &slug && definition.scenario_version() == scenario_version
        })
        .ok_or_else(|| BenchmarkCaseError::new(BenchmarkCaseErrorKind::Catalog))?;
    let settings = definition
        .metadata()
        .ok_or_else(|| BenchmarkCaseError::new(BenchmarkCaseErrorKind::Catalog))?
        .default_settings();
    let fixed_settings = RunSettings::new(
        FloatBits::from_f32(1.0 / 60.0),
        8,
        3,
        spec.particle_iterations,
    )
    .map_err(|_error| BenchmarkCaseError::new(BenchmarkCaseErrorKind::IdentityMismatch))?;
    if settings != fixed_settings {
        return Err(BenchmarkCaseError::new(
            BenchmarkCaseErrorKind::IdentityMismatch,
        ));
    }
    let resolved = resolve_catalog(
        catalog.definitions(),
        &ResolveRequest::new(slug.clone(), None, settings),
    )
    .map_err(|_error| BenchmarkCaseError::new(BenchmarkCaseErrorKind::Catalog))?;
    let resolved_sha256 = resolved.identity().content_sha256().clone();
    if resolved_sha256.as_str() != spec.resolved_sha256 {
        return Err(BenchmarkCaseError::new(
            BenchmarkCaseErrorKind::IdentityMismatch,
        ));
    }
    let measured_horizon = u32::try_from(resolved.checkpoints().len())
        .map_err(|_| BenchmarkCaseError::new(BenchmarkCaseErrorKind::HorizonMismatch))?;
    if measured_horizon != spec.measured_horizon {
        return Err(BenchmarkCaseError::new(
            BenchmarkCaseErrorKind::HorizonMismatch,
        ));
    }
    let expected_checkpoint = run_untimed(&resolved, measured_horizon)?;
    let checkpoint_validator = exact_checkpoint as CheckpointValidator;
    for _ in 0..spec.warmup_runs {
        let candidate = run_untimed(&resolved, measured_horizon)?;
        if !checkpoint_validator(&expected_checkpoint, &candidate) {
            return Err(BenchmarkCaseError::new(
                BenchmarkCaseErrorKind::CheckpointMismatch,
            ));
        }
    }
    Ok(CatalogBenchmarkCase {
        slug,
        scenario_version,
        resolved_sha256,
        settings,
        warmup_runs: spec.warmup_runs,
        measured_horizon,
        resolved,
        expected_checkpoint,
        restart_factory: restart_native,
        checkpoint_validator,
    })
}

fn run_untimed(
    resolved: &ResolvedScenario,
    measured_horizon: u32,
) -> Result<CanonicalCheckpoint, BenchmarkCaseError> {
    let mut controller = restart_native(resolved)?;
    execute_logical_horizon(&mut controller, measured_horizon)?;
    capture_final_checkpoint(&mut controller, resolved)
}

fn restart_native(
    resolved: &ResolvedScenario,
) -> Result<SessionController<NativeCatalogBackend>, BenchmarkCaseError> {
    let mut controller = SessionController::new(NativeCatalogBackend::new());
    submit(
        &mut controller,
        SessionCommand::Select {
            resolved: resolved.clone(),
        },
    )?;
    Ok(controller)
}

fn execute_logical_horizon(
    controller: &mut SessionController<NativeCatalogBackend>,
    measured_horizon: u32,
) -> Result<(), BenchmarkCaseError> {
    for _ in 0..measured_horizon {
        submit(controller, SessionCommand::StepOnce)?;
    }
    if controller.completed_logical_steps() != measured_horizon {
        return Err(BenchmarkCaseError::new(
            BenchmarkCaseErrorKind::HorizonMismatch,
        ));
    }
    Ok(())
}

fn capture_final_checkpoint(
    controller: &mut SessionController<NativeCatalogBackend>,
    resolved: &ResolvedScenario,
) -> Result<CanonicalCheckpoint, BenchmarkCaseError> {
    let checkpoint_id = resolved
        .checkpoints()
        .last()
        .ok_or_else(|| BenchmarkCaseError::new(BenchmarkCaseErrorKind::HorizonMismatch))?
        .checkpoint_id()
        .clone();
    submit(
        controller,
        SessionCommand::CaptureCheckpoint { checkpoint_id },
    )?;
    controller
        .captures()
        .last()
        .map(|capture| capture.value().clone())
        .ok_or_else(|| BenchmarkCaseError::new(BenchmarkCaseErrorKind::CheckpointMismatch))
}

fn submit(
    controller: &mut SessionController<NativeCatalogBackend>,
    command: SessionCommand,
) -> Result<(), BenchmarkCaseError> {
    let command_id = controller
        .next_command_id()
        .ok_or_else(|| BenchmarkCaseError::new(BenchmarkCaseErrorKind::NativeExecution))?;
    controller
        .submit(command_id, command)
        .map(|_outcome| ())
        .map_err(map_controller_error)
}

const fn map_controller_error(_error: SessionControllerError) -> BenchmarkCaseError {
    BenchmarkCaseError::new(BenchmarkCaseErrorKind::NativeExecution)
}

fn exact_checkpoint(expected: &CanonicalCheckpoint, candidate: &CanonicalCheckpoint) -> bool {
    expected == candidate
}
