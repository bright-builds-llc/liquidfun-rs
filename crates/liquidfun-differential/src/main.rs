//! Thin allowlisted command dispatch for private differential workflows.

use std::{
    collections::BTreeMap,
    env, fs,
    io::{self, Write},
    path::PathBuf,
    process::{Command, ExitCode},
};

use liquidfun_differential::{
    ArtifactKind, DifferentialRunOutcome, FailureBundleRequest, HarnessFailureRun, MatchRun,
    MismatchReport, NativeRigidWorldExecutor, OracleExecutable, OraclePreset, OracleSupervisor,
    PhysicsMismatchRun, ReviewMetadata, SessionProfile, StageRequest, persist_failure_bundle,
    promote_candidate, replay_exact, review_candidate, run_named, stage_candidate,
};
use liquidfun_test_protocol::{
    HarnessLimits, decode_rigid_world_request_jsonl, decode_scenario_request_jsonl,
};
use serde::Serialize;

mod minimize_command;

const ORACLE_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const EXIT_PHYSICS_MISMATCH: u8 = 2;
const EXIT_HARNESS_FAILURE: u8 = 3;
const EXIT_USAGE: u8 = 64;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("differential command failed: {error}");
            ExitCode::from(EXIT_USAGE)
        }
    }
}

fn run() -> Result<ExitCode, CliError> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments
        .first()
        .is_some_and(|argument| argument == "fixture")
    {
        return run_fixture(arguments.into_iter().skip(1));
    }
    if arguments
        .first()
        .is_some_and(|argument| argument == "native-rigid-world")
    {
        return run_native_rigid_world(arguments.into_iter().skip(1));
    }
    let command = CommandConfig::parse(arguments.into_iter())?;
    let repository_root = env::current_dir()?;
    let outcome = match &command.input {
        Input::Named(name) => run_named(
            &repository_root,
            name,
            command.preset,
            command.profile,
            ORACLE_REVISION,
        )?,
        Input::ExactRequest(path) => {
            let bytes = fs::read(path)?;
            replay_exact(
                &repository_root,
                &bytes,
                command.preset,
                command.profile,
                ORACLE_REVISION,
            )?
        }
    };
    if command.action == Action::Minimize {
        return minimize_command::run(&repository_root, command.preset, command.profile, outcome);
    }
    render_outcome(&repository_root, command.preset, command.profile, outcome)
}

fn run_native_rigid_world(
    mut arguments: impl Iterator<Item = String>,
) -> Result<ExitCode, CliError> {
    let request_path = match (
        arguments.next().as_deref(),
        arguments.next(),
        arguments.next(),
    ) {
        (Some("--request"), Some(path), None) => PathBuf::from(path),
        _ => return Err(CliError::Usage(usage())),
    };
    let bytes = fs::read(request_path)?;
    let request = decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())?;
    let result = NativeRigidWorldExecutor::execute(&request)?;
    write_json(&result)?;
    Ok(ExitCode::SUCCESS)
}

fn render_outcome(
    repository_root: &std::path::Path,
    preset: OraclePreset,
    profile: SessionProfile,
    outcome: DifferentialRunOutcome,
) -> Result<ExitCode, CliError> {
    match outcome {
        DifferentialRunOutcome::Match(run) => {
            write_machine(&MachineReport::matched(&run))?;
            eprintln!("match: {} validated request(s)", run.requests().len());
            Ok(ExitCode::SUCCESS)
        }
        DifferentialRunOutcome::PhysicsMismatch(run) => {
            let machine = MachineReport::mismatch(&run);
            let report_bytes = json_line(&machine)?;
            persist_outcome_bundle(
                repository_root,
                run.request(),
                run.request_jsonl(),
                &report_bytes,
                preset,
                profile,
                "physics_mismatch",
                Some(run.session_identity_sha256().as_str()),
                b"",
            )?;
            write_bytes(&report_bytes)?;
            eprintln!("{}", run.report().render_human());
            Ok(ExitCode::from(EXIT_PHYSICS_MISMATCH))
        }
        DifferentialRunOutcome::HarnessFailure(run) => {
            let machine = MachineReport::harness(&run);
            let report_bytes = json_line(&machine)?;
            persist_outcome_bundle(
                repository_root,
                run.request(),
                run.request_jsonl(),
                &report_bytes,
                preset,
                profile,
                "harness_failure",
                run.maybe_session_identity_sha256()
                    .map(liquidfun_test_protocol::Sha256Hex::as_str),
                run.failure().evidence().stderr().retained(),
            )?;
            write_bytes(&report_bytes)?;
            eprintln!("harness failure: {}", run.failure().kind().as_str());
            Ok(ExitCode::from(EXIT_HARNESS_FAILURE))
        }
    }
}

fn write_machine(report: &MachineReport<'_>) -> Result<(), CliError> {
    write_json(report)
}

fn write_json(report: &impl Serialize) -> Result<(), CliError> {
    write_bytes(&json_line(report)?)
}

fn json_line(report: &impl Serialize) -> Result<Vec<u8>, CliError> {
    let mut bytes = serde_json::to_vec(report)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn write_bytes(bytes: &[u8]) -> Result<(), CliError> {
    let stdout = io::stdout();
    let mut locked = stdout.lock();
    locked.write_all(bytes)?;
    Ok(())
}

#[allow(
    clippy::too_many_arguments,
    reason = "failure evidence has fixed request, command, identity, report, and stderr inputs"
)]
fn persist_outcome_bundle(
    repository_root: &std::path::Path,
    request: &liquidfun_test_protocol::ScenarioRequestRecord,
    request_bytes: &[u8],
    report_bytes: &[u8],
    preset: OraclePreset,
    profile: SessionProfile,
    result_kind: &'static str,
    maybe_session_identity_sha256: Option<&str>,
    stderr: &[u8],
) -> Result<(), CliError> {
    let identity = FailureIdentityReport {
        oracle_revision: ORACLE_REVISION,
        preset: preset_name(preset),
        session_profile: profile_name(profile),
        maybe_session_identity_sha256,
    };
    let identity_bytes = json_line(&identity)?;
    persist_failure_bundle(
        repository_root,
        &FailureBundleRequest {
            result_kind,
            request_id: request.request_id(),
            request_jsonl: request_bytes,
            report_json: report_bytes,
            identity_json: &identity_bytes,
            stderr,
        },
    )?;
    Ok(())
}

const fn preset_name(preset: OraclePreset) -> &'static str {
    match preset {
        OraclePreset::Debug => "oracle-debug",
        OraclePreset::Release => "oracle-release",
        OraclePreset::AsanUbsan => "oracle-asan-ubsan",
    }
}

const fn profile_name(profile: SessionProfile) -> &'static str {
    match profile {
        SessionProfile::OneShot => "one-shot",
        SessionProfile::Reuse => "reuse",
        SessionProfile::Sanitizer => "sanitizer",
    }
}

fn run_fixture(arguments: impl Iterator<Item = String>) -> Result<ExitCode, CliError> {
    let repository_root = env::current_dir()?;
    let mut arguments = arguments;
    let action = arguments
        .next()
        .ok_or_else(|| CliError::Usage(fixture_usage()))?;
    let options = parse_fixture_options(arguments)?;
    match action.as_str() {
        "stage" => {
            require_exact_options(
                &options,
                &[
                    "--artifact-id",
                    "--artifact-kind",
                    "--preset",
                    "--scenario",
                    "--session-profile",
                ],
            )?;
            let scenario = required_option(&options, "--scenario")?;
            let artifact_id = required_option(&options, "--artifact-id")?;
            let preset = required_option(&options, "--preset")?;
            let session_profile = required_option(&options, "--session-profile")?;
            let artifact_kind = match required_option(&options, "--artifact-kind")?.as_str() {
                "reviewed-trace" => ArtifactKind::ReviewedTrace,
                "minimized-regression" => ArtifactKind::MinimizedRegression,
                _ => return Err(CliError::Usage(fixture_usage())),
            };
            if scenario != "empty-world" {
                return Err(CliError::Usage(fixture_usage()));
            }
            let parsed_preset = parse_preset(preset)?;
            let parsed_profile = parse_profile(session_profile)?;
            let generator_revision = generator_revision(&repository_root)?;
            let request_bytes = fs::read(
                repository_root.join("protocol/fixtures/accepted/empty-world-request.jsonl"),
            )?;
            let request =
                decode_scenario_request_jsonl(&request_bytes, &HarnessLimits::phase2_default_v1())?;
            let executable = OracleExecutable::resolve(&repository_root, parsed_preset)?;
            let mut supervisor = OracleSupervisor::new(executable, parsed_profile, ORACLE_REVISION);
            let captured = supervisor
                .execute_captured(&request)
                .map_err(|failure| CliError::Harness(failure.kind().as_str().to_owned()))?;
            let candidate = stage_candidate(
                &repository_root,
                StageRequest {
                    artifact_id,
                    artifact_kind,
                    scenario_id: scenario,
                    preset,
                    session_profile,
                    generator_revision: &generator_revision,
                    request_bytes: &request_bytes,
                    trace_bytes: captured.jsonl(),
                    stderr_bytes: b"",
                    maybe_failure_signature: None,
                },
            )?;
            write_json(&FixtureStageReport {
                result_kind: "fixture_staged",
                artifact_id: candidate.artifact_id(),
                candidate_directory: candidate.directory(),
            })?;
        }
        "review" => {
            require_exact_options(
                &options,
                &[
                    "--artifact-id",
                    "--review-status",
                    "--reviewed-at",
                    "--reviewer",
                ],
            )?;
            let artifact_id = required_option(&options, "--artifact-id")?;
            let reviewer = required_option(&options, "--reviewer")?;
            let reviewed_at = required_option(&options, "--reviewed-at")?;
            let metadata = match required_option(&options, "--review-status")?.as_str() {
                "approved" => ReviewMetadata::approved(reviewer, reviewed_at),
                "rejected" => ReviewMetadata::rejected(reviewer, reviewed_at),
                _ => return Err(CliError::Usage(fixture_usage())),
            };
            let receipt = review_candidate(&repository_root, artifact_id, metadata)?;
            write_json(&receipt)?;
        }
        "promote" => {
            require_exact_options(&options, &["--artifact-id"])?;
            let artifact_id = required_option(&options, "--artifact-id")?;
            let receipt = promote_candidate(&repository_root, artifact_id)?;
            write_json(&receipt)?;
        }
        _ => return Err(CliError::Usage(fixture_usage())),
    }
    Ok(ExitCode::SUCCESS)
}

fn require_exact_options(
    options: &BTreeMap<String, String>,
    expected: &[&str],
) -> Result<(), CliError> {
    if options.len() == expected.len()
        && options
            .keys()
            .all(|option| expected.contains(&option.as_str()))
    {
        return Ok(());
    }
    Err(CliError::Usage(fixture_usage()))
}

fn parse_fixture_options(
    mut arguments: impl Iterator<Item = String>,
) -> Result<BTreeMap<String, String>, CliError> {
    let mut options = BTreeMap::new();
    while let Some(option) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| CliError::Usage(fixture_usage()))?;
        if !option.starts_with("--") || options.insert(option, value).is_some() {
            return Err(CliError::Usage(fixture_usage()));
        }
    }
    Ok(options)
}

fn required_option<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a String, CliError> {
    options
        .get(name)
        .ok_or_else(|| CliError::Usage(fixture_usage()))
}

fn generator_revision(repository_root: &std::path::Path) -> Result<String, CliError> {
    const GENERATOR_INPUTS: [&str; 12] = [
        ".gitmodules",
        "Cargo.lock",
        "Cargo.toml",
        "crates/liquidfun",
        "crates/liquidfun-differential",
        "crates/liquidfun-test-protocol",
        "protocol",
        "reference/artifacts/manifest.toml",
        "reference/upstream-lock.toml",
        "rust-toolchain.toml",
        "third_party/liquidfun",
        "tools/reference",
    ];
    let status = Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(["status", "--porcelain=v1", "--untracked-files=all", "--"])
        .args(GENERATOR_INPUTS)
        .output()?;
    if !status.status.success() {
        return Err(CliError::GeneratorRevision(
            String::from_utf8_lossy(&status.stderr).trim().to_owned(),
        ));
    }
    let dirty = String::from_utf8_lossy(&status.stdout);
    if !dirty.trim().is_empty() {
        return Err(CliError::GeneratorRevision(format!(
            "relevant generator inputs are dirty:\n{}",
            dirty.trim_end()
        )));
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(["rev-parse", "HEAD"])
        .output()?;
    if !output.status.success() {
        return Err(CliError::GeneratorRevision(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn fixture_usage() -> String {
    "usage: liquidfun-differential fixture stage --scenario empty-world --preset <oracle-debug|oracle-release|oracle-asan-ubsan> --session-profile <one-shot|reuse|sanitizer> --artifact-kind <reviewed-trace|minimized-regression> --artifact-id <id>; fixture review --artifact-id <id> --reviewer <identity> --reviewed-at <UTC timestamp> --review-status <approved|rejected>; fixture promote --artifact-id <id>".to_owned()
}

#[derive(Serialize)]
struct FixtureStageReport<'a> {
    result_kind: &'static str,
    artifact_id: &'a str,
    candidate_directory: &'a std::path::Path,
}

#[derive(Serialize)]
struct FailureIdentityReport<'a> {
    oracle_revision: &'static str,
    preset: &'static str,
    session_profile: &'static str,
    #[serde(
        rename = "session_identity_sha256",
        skip_serializing_if = "Option::is_none"
    )]
    maybe_session_identity_sha256: Option<&'a str>,
}

#[derive(Serialize)]
struct MachineReport<'a> {
    result_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    requests: Option<&'a [liquidfun_differential::MatchedRequest]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_kind: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mismatch: Option<&'a MismatchReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_identity_sha256: Option<&'a str>,
}

impl<'a> MachineReport<'a> {
    fn matched(run: &'a MatchRun) -> Self {
        Self {
            result_kind: "match",
            requests: Some(run.requests()),
            failure_kind: None,
            mismatch: None,
            request_id: None,
            session_identity_sha256: None,
        }
    }

    fn mismatch(run: &'a PhysicsMismatchRun) -> Self {
        Self {
            result_kind: "physics_mismatch",
            requests: None,
            failure_kind: None,
            mismatch: Some(run.report()),
            request_id: Some(run.request().request_id().as_str()),
            session_identity_sha256: Some(run.session_identity_sha256().as_str()),
        }
    }

    fn harness(run: &'a HarnessFailureRun) -> Self {
        Self {
            result_kind: "harness_failure",
            requests: None,
            failure_kind: Some(run.failure().kind().as_str()),
            mismatch: None,
            request_id: Some(run.request().request_id().as_str()),
            session_identity_sha256: run
                .maybe_session_identity_sha256()
                .map(liquidfun_test_protocol::Sha256Hex::as_str),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Compare,
    Replay,
    Minimize,
}

enum Input {
    Named(String),
    ExactRequest(PathBuf),
}

struct CommandConfig {
    action: Action,
    input: Input,
    preset: OraclePreset,
    profile: SessionProfile,
}

impl CommandConfig {
    fn parse(arguments: impl Iterator<Item = String>) -> Result<Self, CliError> {
        let mut arguments = arguments;
        let action = match arguments.next().as_deref() {
            Some("compare") => Action::Compare,
            Some("replay") => Action::Replay,
            Some("minimize") => Action::Minimize,
            _ => return Err(CliError::Usage(usage())),
        };
        let mut maybe_scenario = None;
        let mut maybe_exact_request = None;
        let mut maybe_preset = None;
        let mut maybe_profile = None;
        while let Some(option) = arguments.next() {
            let value = arguments.next().ok_or_else(|| CliError::Usage(usage()))?;
            match option.as_str() {
                "--scenario" if maybe_scenario.is_none() => maybe_scenario = Some(value),
                "--exact-request" if maybe_exact_request.is_none() => {
                    maybe_exact_request = Some(PathBuf::from(value));
                }
                "--preset" if maybe_preset.is_none() => maybe_preset = Some(parse_preset(&value)?),
                "--session-profile" if maybe_profile.is_none() => {
                    maybe_profile = Some(parse_profile(&value)?);
                }
                _ => return Err(CliError::Usage(usage())),
            }
        }
        let preset = maybe_preset.ok_or_else(|| CliError::Usage(usage()))?;
        let profile = maybe_profile.ok_or_else(|| CliError::Usage(usage()))?;
        let input = match (maybe_scenario, maybe_exact_request) {
            (Some(name), None) => Input::Named(name),
            (None, Some(path)) if action == Action::Replay => Input::ExactRequest(path),
            _ => return Err(CliError::Usage(usage())),
        };
        Ok(Self {
            action,
            input,
            preset,
            profile,
        })
    }
}

fn parse_preset(value: &str) -> Result<OraclePreset, CliError> {
    match value {
        "oracle-debug" => Ok(OraclePreset::Debug),
        "oracle-release" => Ok(OraclePreset::Release),
        "oracle-asan-ubsan" => Ok(OraclePreset::AsanUbsan),
        _ => Err(CliError::Usage(usage())),
    }
}

fn parse_profile(value: &str) -> Result<SessionProfile, CliError> {
    match value {
        "one-shot" => Ok(SessionProfile::OneShot),
        "reuse" => Ok(SessionProfile::Reuse),
        "sanitizer" => Ok(SessionProfile::Sanitizer),
        _ => Err(CliError::Usage(usage())),
    }
}

fn usage() -> String {
    "usage: liquidfun-differential native-rigid-world --request <file>; or \
     <compare|replay|minimize> --scenario empty-world \
     --preset <oracle-debug|oracle-release|oracle-asan-ubsan> \
     --session-profile <one-shot|reuse|sanitizer>; replay also accepts --exact-request <file>"
        .to_owned()
}

#[derive(Debug, thiserror::Error)]
enum CliError {
    #[error("{0}")]
    Usage(String),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Runner(#[from] liquidfun_differential::DifferentialRunnerError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Fixture(#[from] liquidfun_differential::FixtureError),
    #[error(transparent)]
    FailureBundle(#[from] liquidfun_differential::FailureBundleError),
    #[error(transparent)]
    Executable(#[from] liquidfun_differential::OracleExecutableError),
    #[error(transparent)]
    Scenario(#[from] liquidfun_test_protocol::ScenarioDecodeError),
    #[error(transparent)]
    RigidWorld(#[from] liquidfun_test_protocol::RigidWorldDecodeError),
    #[error(transparent)]
    NativeRigidWorld(#[from] liquidfun_differential::NativeRigidWorldError),
    #[error("oracle harness failure while staging: {0}")]
    Harness(String),
    #[error("could not determine generator revision: {0}")]
    GeneratorRevision(String),
    #[error("minimize requires an initial physics mismatch")]
    MinimizeRequiresMismatch,
    #[error("minimization candidate encountered harness failure `{0}`")]
    MinimizationHarness(String),
    #[error(transparent)]
    Minimization(#[from] liquidfun_differential::MinimizationError),
}
