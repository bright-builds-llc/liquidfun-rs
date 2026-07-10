//! Thin allowlisted command dispatch for private differential workflows.

use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use liquidfun_differential::{
    DifferentialRunOutcome, MatchRun, MismatchReport, OraclePreset, SessionProfile, replay_exact,
    run_named,
};
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
    let command = CommandConfig::parse(env::args().skip(1))?;
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
    let stdout = io::stdout();
    let mut locked = stdout.lock();
    serde_json::to_writer(&mut locked, report)?;
    locked.write_all(b"\n")?;
    Ok(())
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
}
