//! Closed Phase 12 performance evidence commands.

#[path = "performance/analysis.rs"]
pub(crate) mod analysis;
#[path = "performance/evidence.rs"]
mod evidence;
#[path = "performance/paths.rs"]
mod paths;
#[path = "performance/runner.rs"]
mod runner;

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use liquidfun_differential::{OracleExecutable, OraclePreset};
use liquidfun_test_protocol::{
    ResolveRequest, ResolvedScenario, Sha256Hex,
    performance::{
        PerformanceMatrix, PerformancePolicy, PerformanceSizePoint, PerformanceWorkloadKind,
        benchmark_policy_sha256, render_performance_matrix,
    },
    resolve_catalog, reviewed_scenario_catalog,
};
use paths::{PerformancePaths, read_bounded_regular_file, write_json_atomically};
use runner::ConcretePairedRunProvider;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const PERFORMANCE_USAGE: &str = "Usage: cargo xtask performance \
    <paired [--check] | calibrate | validate [--emit-identity] | optimization-check>";

/// Closed error returned by performance orchestration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PerformanceCommandError {
    kind: &'static str,
    message: String,
}

impl PerformanceCommandError {
    fn new(kind: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    #[cfg(test)]
    #[allow(dead_code, reason = "used by the focused integration test crate")]
    pub(crate) const fn kind(&self) -> &'static str {
        self.kind
    }
}

impl Display for PerformanceCommandError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.kind, self.message)
    }
}

impl Error for PerformanceCommandError {}

/// Repository and confined output roots used by the closed command.
#[derive(Debug, Clone)]
pub(crate) struct PerformanceEnvironment {
    paths: PerformancePaths,
}

impl PerformanceEnvironment {
    pub(crate) fn production() -> Result<Self, PerformanceCommandError> {
        let root = repository_root()?;
        let paths = PerformancePaths::production(&root)
            .map_err(|message| PerformanceCommandError::new("path", message))?;
        Ok(Self { paths })
    }

    #[cfg(test)]
    #[allow(dead_code, reason = "used by the focused integration test crate")]
    pub(crate) fn for_test(
        repository_root: &Path,
        output_root: &Path,
    ) -> Result<Self, PerformanceCommandError> {
        let paths = PerformancePaths::new(repository_root, output_root)
            .map_err(|message| PerformanceCommandError::new("path", message))?;
        Ok(Self { paths })
    }

    #[cfg(test)]
    #[allow(dead_code, reason = "used by the focused integration test crate")]
    pub(crate) fn output_root(&self) -> &Path {
        self.paths.output_root()
    }
}

/// One sealed case supplied to the injected paired-run provider.
pub(crate) struct PairedCaseRequest<'a> {
    case: &'a PreparedCase,
    policy: &'a PerformancePolicy,
    matrix_sha256: &'a Sha256Hex,
}

impl PairedCaseRequest<'_> {
    pub(crate) const fn workload(&self) -> PerformanceWorkloadKind {
        self.case.workload
    }

    pub(crate) const fn size_point(&self) -> PerformanceSizePoint {
        self.case.size_point
    }

    pub(crate) fn scenario_id(&self) -> &str {
        self.case.resolved.identity().slug().as_str()
    }

    pub(crate) const fn resolved_sha256(&self) -> &Sha256Hex {
        &self.case.resolved_sha256
    }

    pub(crate) const fn policy(&self) -> &PerformancePolicy {
        self.policy
    }
}

/// Closed terminal failures from the production Plan 12-20 runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PairedRunFailure {
    Harness,
    PhysicsMismatch,
    Construction,
}

/// Injection seam around the existing Plan 12-20 paired runner.
pub(crate) trait PairedRunProvider {
    fn run_case(&mut self, request: PairedCaseRequest<'_>) -> Result<Value, PairedRunFailure>;
}

/// Runs the production closed performance command.
pub(crate) fn run(args: &[String]) -> Result<(), PerformanceCommandError> {
    let environment = PerformanceEnvironment::production()?;
    if let ClosedCommand::Paired { check: false } = parse_command(args)? {
        let mut provider = ConcretePairedRunProvider::new(&environment)?;
        run_with_provider(args, &environment, &mut provider)
    } else {
        let mut provider = UnavailablePairedRunProvider;
        run_with_provider(args, &environment, &mut provider)
    }
}

/// Runs a closed mode with an injected provider for deterministic orchestration tests.
pub(crate) fn run_with_provider(
    args: &[String],
    environment: &PerformanceEnvironment,
    provider: &mut impl PairedRunProvider,
) -> Result<(), PerformanceCommandError> {
    let command = parse_command(args)?;
    let contract = load_contract(&environment.paths)?;
    match command {
        ClosedCommand::Paired { check } => paired(environment, provider, &contract, check),
        ClosedCommand::Calibrate => evidence::calibrate(environment, &contract),
        ClosedCommand::Validate { emit_identity } => {
            evidence::validate(environment, &contract, emit_identity)
        }
        ClosedCommand::OptimizationCheck => evidence::optimization_check(environment),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClosedCommand {
    Paired { check: bool },
    Calibrate,
    Validate { emit_identity: bool },
    OptimizationCheck,
}

fn parse_command(args: &[String]) -> Result<ClosedCommand, PerformanceCommandError> {
    match args {
        [command] if command == "paired" => Ok(ClosedCommand::Paired { check: false }),
        [command, option] if command == "paired" && option == "--check" => {
            Ok(ClosedCommand::Paired { check: true })
        }
        [command] if command == "calibrate" => Ok(ClosedCommand::Calibrate),
        [command] if command == "validate" => Ok(ClosedCommand::Validate {
            emit_identity: false,
        }),
        [command, option] if command == "validate" && option == "--emit-identity" => {
            Ok(ClosedCommand::Validate {
                emit_identity: true,
            })
        }
        [command] if command == "optimization-check" => Ok(ClosedCommand::OptimizationCheck),
        _ => Err(PerformanceCommandError::new("usage", PERFORMANCE_USAGE)),
    }
}

#[derive(Debug)]
struct PreparedCase {
    case_id: String,
    workload: PerformanceWorkloadKind,
    size_point: PerformanceSizePoint,
    catalog_sha256: Sha256Hex,
    resolved_sha256: Sha256Hex,
    logical_horizon: u32,
    resolved: ResolvedScenario,
}

fn paired(
    environment: &PerformanceEnvironment,
    provider: &mut impl PairedRunProvider,
    contract: &PerformanceContract,
    check: bool,
) -> Result<(), PerformanceCommandError> {
    let cases = prepare_cases(&contract.matrix)?;
    OracleExecutable::resolve(environment.paths.repository_root(), OraclePreset::Release)
        .map_err(|error| PerformanceCommandError::new("oracle_release", error.to_string()))?;
    if check {
        println!(
            "performance paired check: {} sealed cases, oracle-release, policy {}",
            cases.len(),
            contract.policy_sha256.as_str()
        );
        return Ok(());
    }

    let raw_directory = environment.paths.raw_directory();
    if raw_directory.exists() {
        fs::remove_dir_all(&raw_directory).map_err(|error| {
            PerformanceCommandError::new(
                "output",
                format!("failed to replace {}: {error}", raw_directory.display()),
            )
        })?;
    }
    fs::create_dir_all(&raw_directory).map_err(|error| {
        PerformanceCommandError::new(
            "output",
            format!("failed to create {}: {error}", raw_directory.display()),
        )
    })?;

    let mut completed = Vec::with_capacity(cases.len());
    for case in &cases {
        let request = PairedCaseRequest {
            case,
            policy: &contract.policy,
            matrix_sha256: &contract.matrix_sha256,
        };
        let report = provider
            .run_case(request)
            .map_err(|failure| match failure {
                PairedRunFailure::Harness => {
                    PerformanceCommandError::new("paired_harness", case.case_id.clone())
                }
                PairedRunFailure::PhysicsMismatch => {
                    PerformanceCommandError::new("paired_mismatch", case.case_id.clone())
                }
                PairedRunFailure::Construction => {
                    PerformanceCommandError::new("paired_construction", case.case_id.clone())
                }
            })?;
        let report_path = raw_directory.join(format!("{}.json", case.case_id));
        write_json_atomically(&report_path, &report)
            .map_err(|message| PerformanceCommandError::new("output", message))?;
        completed.push(case.case_id.clone());
    }
    let summary = json!({
        "schema_version": 1,
        "claim_status": "raw_unreviewed_measurements_only",
        "policy_sha256": contract.policy_sha256,
        "matrix_sha256": contract.matrix_sha256,
        "completed_cases": completed,
    });
    write_json_atomically(
        &environment.paths.output_root().join("paired-summary.json"),
        &summary,
    )
    .map_err(|message| PerformanceCommandError::new("output", message))?;
    println!("performance paired: persisted {} raw reports", cases.len());
    Ok(())
}

struct UnavailablePairedRunProvider;

impl PairedRunProvider for UnavailablePairedRunProvider {
    fn run_case(&mut self, _request: PairedCaseRequest<'_>) -> Result<Value, PairedRunFailure> {
        Err(PairedRunFailure::Construction)
    }
}

fn prepare_cases(matrix: &PerformanceMatrix) -> Result<Vec<PreparedCase>, PerformanceCommandError> {
    let catalog = reviewed_scenario_catalog()
        .map_err(|error| PerformanceCommandError::new("catalog", error.to_string()))?;
    let mut resolved_by_hash = BTreeMap::new();
    for definition in catalog.definitions() {
        let Some(metadata) = definition.metadata() else {
            continue;
        };
        let request =
            ResolveRequest::new(definition.slug().clone(), None, metadata.default_settings());
        let Ok(resolved) = resolve_catalog(catalog.definitions(), &request) else {
            continue;
        };
        resolved_by_hash.insert(
            resolved.identity().content_sha256().as_str().to_owned(),
            resolved,
        );
    }
    matrix
        .cases()
        .iter()
        .map(|case| {
            let resolved = resolved_by_hash
                .get(case.resolved_sha256().as_str())
                .cloned()
                .ok_or_else(|| {
                    PerformanceCommandError::new(
                        "matrix",
                        format!("sealed case {} did not resolve", case.workload().as_str()),
                    )
                })?;
            Ok(PreparedCase {
                case_id: format!(
                    "{}-{}",
                    case.workload().as_str(),
                    size_point_id(case.size_point())
                ),
                workload: case.workload(),
                size_point: case.size_point(),
                catalog_sha256: case.catalog_sha256().clone(),
                resolved_sha256: case.resolved_sha256().clone(),
                logical_horizon: case.logical_horizon(),
                resolved,
            })
        })
        .collect()
}

fn size_point_id(size: PerformanceSizePoint) -> &'static str {
    match size {
        PerformanceSizePoint::Fixed => "fixed",
        PerformanceSizePoint::WorkUnits128 => "128",
        PerformanceSizePoint::WorkUnits1024 => "1024",
        PerformanceSizePoint::WorkUnits8192 => "8192",
    }
}

#[derive(Debug)]
struct PerformanceContract {
    policy: PerformancePolicy,
    policy_sha256: Sha256Hex,
    matrix: PerformanceMatrix,
    matrix_sha256: Sha256Hex,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrackedPolicy {
    schema_version: u8,
    analysis_version: String,
    interval_method: IntervalMethod,
    measurement_policy: PerformancePolicy,
    minimum_profile_basis_points: u16,
    bottleneck_kinds: Vec<String>,
    required_correctness_gates: Vec<String>,
    allowed_optimization_mode: String,
    timing_authority: String,
    claim_scope: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum IntervalMethod {
    StudentT95,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewedManifest {
    schema_version: u8,
    policy_sha256: String,
    matrix_sha256: String,
    reviewed_reports: Vec<String>,
}

fn load_contract(paths: &PerformancePaths) -> Result<PerformanceContract, PerformanceCommandError> {
    let policy_bytes = read_bounded_regular_file(&paths.policy())
        .map_err(|message| PerformanceCommandError::new("policy", message))?;
    let tracked: TrackedPolicy = serde_json::from_slice(&policy_bytes)
        .map_err(|error| PerformanceCommandError::new("policy", error.to_string()))?;
    validate_tracked_policy(&tracked)?;
    let policy_sha256 = benchmark_policy_sha256()
        .map_err(|error| PerformanceCommandError::new("policy", error.to_string()))?;
    let matrix = PerformanceMatrix::reviewed_v1()
        .map_err(|error| PerformanceCommandError::new("matrix", error.to_string()))?;
    let matrix_bytes = render_performance_matrix()
        .map_err(|error| PerformanceCommandError::new("matrix", error.to_string()))?;
    let matrix_sha256 = Sha256Hex::from_digest(Sha256::digest(matrix_bytes).into());
    let manifest_bytes = read_bounded_regular_file(&paths.manifest())
        .map_err(|message| PerformanceCommandError::new("manifest", message))?;
    let manifest: ReviewedManifest = toml::from_slice(&manifest_bytes)
        .map_err(|error| PerformanceCommandError::new("manifest", error.to_string()))?;
    if manifest.schema_version != 1
        || manifest.policy_sha256 != policy_sha256.as_str()
        || manifest.matrix_sha256 != matrix_sha256.as_str()
        || manifest.reviewed_reports.iter().any(|entry| {
            entry.is_empty()
                || entry.starts_with('/')
                || entry.contains("..")
                || !Path::new(entry)
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        })
    {
        return Err(PerformanceCommandError::new(
            "manifest",
            "reviewed manifest identity or report path is invalid",
        ));
    }
    Ok(PerformanceContract {
        policy: tracked.measurement_policy,
        policy_sha256,
        matrix,
        matrix_sha256,
    })
}

fn validate_tracked_policy(policy: &TrackedPolicy) -> Result<(), PerformanceCommandError> {
    let expected_bottlenecks = ["allocation", "cache", "scaling"];
    let expected_gates = ["differential", "determinism", "safety", "public_api"];
    if policy.schema_version != 1
        || policy.analysis_version != "phase12-performance-analysis-v1"
        || !matches!(policy.interval_method, IntervalMethod::StudentT95)
        || policy.measurement_policy != PerformancePolicy::reviewed_v1()
        || policy.minimum_profile_basis_points != 1_000
        || policy.bottleneck_kinds != expected_bottlenecks
        || policy.required_correctness_gates != expected_gates
        || policy.allowed_optimization_mode != "release_scalar"
        || policy.timing_authority != "unprofiled_wall_clock"
        || policy.claim_scope != "workload_only"
    {
        return Err(PerformanceCommandError::new(
            "policy",
            "tracked performance analysis policy drifted",
        ));
    }
    Ok(())
}

fn repository_root() -> Result<PathBuf, PerformanceCommandError> {
    let current = std::env::current_dir()
        .map_err(|error| PerformanceCommandError::new("workspace", error.to_string()))?;
    current
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").is_file()
                && candidate.join("tools/xtask/Cargo.toml").is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            PerformanceCommandError::new("workspace", "could not find Cargo workspace root")
        })
}
