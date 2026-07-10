//! Thin allowlisted command dispatch for private differential workflows.

use std::{
    collections::BTreeMap,
    env, fs,
    io::{self, Write},
    path::PathBuf,
    process::{Command, ExitCode},
};

use liquidfun_differential::{
    ArtifactKind, DifferentialRunOutcome, MatchRun, MismatchReport, OracleExecutable, OraclePreset,
    OracleSupervisor, ReviewMetadata, SessionProfile, StageRequest, promote_candidate,
    replay_exact, review_candidate, run_named, stage_candidate,
};
use liquidfun_test_protocol::{HarnessLimits, decode_scenario_request_jsonl};
use serde::Serialize;

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
    render_outcome(outcome)
}

fn render_outcome(outcome: DifferentialRunOutcome) -> Result<ExitCode, CliError> {
    match outcome {
        DifferentialRunOutcome::Match(run) => {
            write_machine(&MachineReport::matched(&run))?;
            eprintln!("match: {} validated request(s)", run.requests().len());
            Ok(ExitCode::SUCCESS)
        }
        DifferentialRunOutcome::PhysicsMismatch(report) => {
            write_machine(&MachineReport::mismatch(&report))?;
            eprintln!("{}", report.render_human());
            Ok(ExitCode::from(EXIT_PHYSICS_MISMATCH))
        }
        DifferentialRunOutcome::HarnessFailure(failure) => {
            write_machine(&MachineReport::harness(failure.kind().as_str()))?;
            eprintln!("harness failure: {}", failure.kind().as_str());
            Ok(ExitCode::from(EXIT_HARNESS_FAILURE))
        }
    }
}

fn write_machine(report: &MachineReport<'_>) -> Result<(), CliError> {
    write_json(report)
}

fn write_json(report: &impl Serialize) -> Result<(), CliError> {
    let stdout = io::stdout();
    let mut locked = stdout.lock();
    serde_json::to_writer(&mut locked, report)?;
    locked.write_all(b"\n")?;
    Ok(())
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
struct MachineReport<'a> {
    result_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    requests: Option<&'a [liquidfun_differential::MatchedRequest]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_kind: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mismatch: Option<&'a MismatchReport>,
}

impl<'a> MachineReport<'a> {
    fn matched(run: &'a MatchRun) -> Self {
        Self {
            result_kind: "match",
            requests: Some(run.requests()),
            failure_kind: None,
            mismatch: None,
        }
    }

    const fn mismatch(report: &'a MismatchReport) -> Self {
        Self {
            result_kind: "physics_mismatch",
            requests: None,
            failure_kind: None,
            mismatch: Some(report),
        }
    }

    const fn harness(failure_kind: &'a str) -> Self {
        Self {
            result_kind: "harness_failure",
            requests: None,
            failure_kind: Some(failure_kind),
            mismatch: None,
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
    _action: Action,
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
            _action: action,
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
    "usage: liquidfun-differential <compare|replay|minimize> --scenario empty-world \
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
    Executable(#[from] liquidfun_differential::OracleExecutableError),
    #[error(transparent)]
    Scenario(#[from] liquidfun_test_protocol::ScenarioDecodeError),
    #[error("oracle harness failure while staging: {0}")]
    Harness(String),
    #[error("could not determine generator revision: {0}")]
    GeneratorRevision(String),
}
