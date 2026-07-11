use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use liquidfun_differential::{
    EmptyWorldAdapter, NativeMathProbeExecutor, OracleExecutable, OraclePreset,
    execute_math_probe_process, float_values_match_with_policy,
};
use liquidfun_test_protocol::{
    BuildEvidenceTier, BuildIdentity, DivergenceHorizon, EvidenceTier, HarnessLimits,
    MathProbeHorizon, MathProbeRequestRecord, MathProbeResult, Phase4PolicyProfile,
    decode_math_probe_request_jsonl,
};
use sha2::{Digest, Sha256};

use crate::upstream;

const USAGE: &str = r"Usage: cargo xtask differential <command> [arguments]

Commands:
  compare  --scenario <empty-world|math-probes> --preset <oracle-debug|oracle-release|oracle-asan-ubsan> --session-profile <one-shot|reuse|sanitizer>
  replay   --scenario <empty-world|math-probes> --preset <oracle-debug|oracle-release|oracle-asan-ubsan> --session-profile <one-shot|reuse|sanitizer>
  minimize --scenario empty-world --preset <oracle-debug|oracle-release|oracle-asan-ubsan> --session-profile <one-shot|reuse|sanitizer>
  verify-determinism --scenario math-probes --preset <oracle-debug|oracle-release> --runs 2
  fixture stage   --scenario empty-world --preset <preset> --session-profile <profile> --artifact-kind <reviewed-trace|minimized-regression> --artifact-id <id>
  fixture review  --artifact-id <id> --reviewer <identity> --reviewed-at <UTC timestamp> --review-status <approved|rejected>
  fixture promote --artifact-id <id>";

const ORACLE_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const MATH_PROBE_REQUEST: &str = "protocol/fixtures/accepted/math-probe-request.jsonl";
const PHASE4_POLICY: &str = "protocol/tolerances/phase4-v1.toml";
const ALLOWED_SCENARIOS: [&str; 2] = ["empty-world", "math-probes"];
const ALLOWED_PRESETS: [&str; 3] = ["oracle-debug", "oracle-release", "oracle-asan-ubsan"];
const MATH_PROBE_PRESETS: [&str; 2] = ["oracle-debug", "oracle-release"];
const ALLOWED_PROFILES: [&str; 3] = ["one-shot", "reuse", "sanitizer"];
const ALLOWED_ARTIFACT_KINDS: [&str; 2] = ["reviewed-trace", "minimized-regression"];
const ALLOWED_REVIEW_STATUSES: [&str; 2] = ["approved", "rejected"];

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DifferentialError {
    category: &'static str,
    message: String,
}

impl DifferentialError {
    fn new(category: &'static str, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new("usage", format!("{}\n\n{USAGE}", message.into()))
    }

    fn process(message: impl Into<String>) -> Self {
        Self::new("process", message)
    }
}

impl Display for DifferentialError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "differential/{}: {}",
            self.category, self.message
        )
    }
}

impl Error for DifferentialError {}

#[derive(Debug, PartialEq, Eq)]
struct RunnerInvocation {
    arguments: Vec<String>,
    oracle_dependent: bool,
    math_probe: Option<MathProbeInvocation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MathProbeAction {
    Compare,
    Replay,
    VerifyDeterminism,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MathProbeInvocation {
    action: MathProbeAction,
    preset: String,
    runs: usize,
}

pub(crate) fn run(args: &[String]) -> Result<(), DifferentialError> {
    let invocation = parse_invocation(args)?;
    let repository_root = repository_root()?;

    if invocation.oracle_dependent {
        upstream::run(&["verify".to_owned()]).map_err(|error| {
            DifferentialError::new("upstream", format!("oracle verification failed: {error}"))
        })?;
    }

    if env::var_os("LIQUIDFUN_XTASK_DIFFERENTIAL").is_some() {
        return run_differential(&repository_root, &invocation.arguments);
    }
    if let Some(math_probe) = invocation.math_probe {
        return run_math_probe_command(&repository_root, &math_probe);
    }
    run_differential(&repository_root, &invocation.arguments)
}

pub(crate) fn check_protocol(repository_root: &Path) -> Result<(), DifferentialError> {
    for (label, arguments) in [
        (
            "protocol schema presentations",
            [
                "test",
                "--package",
                "liquidfun-test-protocol",
                "--all-features",
                "--lib",
                "schema::tests",
            ]
            .as_slice(),
        ),
        (
            "protocol fixtures",
            [
                "test",
                "--package",
                "liquidfun-test-protocol",
                "--all-features",
                "--test",
                "fixtures",
            ]
            .as_slice(),
        ),
    ] {
        let cargo = env::var_os("LIQUIDFUN_XTASK_CARGO").unwrap_or_else(|| OsString::from("cargo"));
        run_process(&cargo, arguments, repository_root, label)?;
    }
    Ok(())
}

fn parse_invocation(args: &[String]) -> Result<RunnerInvocation, DifferentialError> {
    let Some((command, command_args)) = args.split_first() else {
        return Err(DifferentialError::usage("missing differential command"));
    };

    match command.as_str() {
        "compare" | "replay" | "minimize" => parse_scenario_command(command, command_args),
        "verify-determinism" => parse_determinism_command(command_args),
        "fixture" => parse_fixture_command(command_args),
        unknown => Err(DifferentialError::usage(format!(
            "unknown differential command `{unknown}`"
        ))),
    }
}

fn parse_scenario_command(
    command: &str,
    args: &[String],
) -> Result<RunnerInvocation, DifferentialError> {
    let options = parse_options(args)?;
    require_exact_options(&options, &["--scenario", "--preset", "--session-profile"])?;
    let scenario = require_allowed(&options, "--scenario", &ALLOWED_SCENARIOS)?;
    let preset = require_allowed(&options, "--preset", &ALLOWED_PRESETS)?;
    let profile = require_allowed(&options, "--session-profile", &ALLOWED_PROFILES)?;
    let math_probe = if scenario == "math-probes" {
        if command == "minimize" || profile != "one-shot" || !MATH_PROBE_PRESETS.contains(&preset) {
            return Err(DifferentialError::usage(
                "math-probes supports compare/replay with one-shot debug or release only",
            ));
        }
        Some(MathProbeInvocation {
            action: if command == "compare" {
                MathProbeAction::Compare
            } else {
                MathProbeAction::Replay
            },
            preset: preset.to_owned(),
            runs: 1,
        })
    } else {
        None
    };

    Ok(RunnerInvocation {
        arguments: option_arguments(
            &[command],
            &[
                ("--scenario", scenario),
                ("--preset", preset),
                ("--session-profile", profile),
            ],
        ),
        oracle_dependent: true,
        math_probe,
    })
}

fn parse_determinism_command(args: &[String]) -> Result<RunnerInvocation, DifferentialError> {
    let options = parse_options(args)?;
    require_exact_options(&options, &["--scenario", "--preset", "--runs"])?;
    let scenario = require_allowed(&options, "--scenario", &["math-probes"])?;
    let preset = require_allowed(&options, "--preset", &MATH_PROBE_PRESETS)?;
    let runs = require_allowed(&options, "--runs", &["2"])?;

    Ok(RunnerInvocation {
        arguments: option_arguments(
            &["verify-determinism"],
            &[
                ("--scenario", scenario),
                ("--preset", preset),
                ("--runs", runs),
            ],
        ),
        oracle_dependent: true,
        math_probe: Some(MathProbeInvocation {
            action: MathProbeAction::VerifyDeterminism,
            preset: preset.to_owned(),
            runs: 2,
        }),
    })
}

fn parse_fixture_command(args: &[String]) -> Result<RunnerInvocation, DifferentialError> {
    let Some((action, action_args)) = args.split_first() else {
        return Err(DifferentialError::usage("missing fixture action"));
    };
    let options = parse_options(action_args)?;

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
            let scenario = require_allowed(&options, "--scenario", &ALLOWED_SCENARIOS)?;
            let preset = require_allowed(&options, "--preset", &ALLOWED_PRESETS)?;
            let profile = require_allowed(&options, "--session-profile", &ALLOWED_PROFILES)?;
            let artifact_kind =
                require_allowed(&options, "--artifact-kind", &ALLOWED_ARTIFACT_KINDS)?;
            let artifact_id = required_option(&options, "--artifact-id")?;
            Ok(RunnerInvocation {
                arguments: option_arguments(
                    &["fixture", "stage"],
                    &[
                        ("--scenario", scenario),
                        ("--preset", preset),
                        ("--session-profile", profile),
                        ("--artifact-kind", artifact_kind),
                        ("--artifact-id", artifact_id),
                    ],
                ),
                oracle_dependent: true,
                math_probe: None,
            })
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
            let review_status =
                require_allowed(&options, "--review-status", &ALLOWED_REVIEW_STATUSES)?;
            Ok(RunnerInvocation {
                arguments: option_arguments(
                    &["fixture", "review"],
                    &[
                        ("--artifact-id", artifact_id),
                        ("--reviewer", reviewer),
                        ("--reviewed-at", reviewed_at),
                        ("--review-status", review_status),
                    ],
                ),
                oracle_dependent: false,
                math_probe: None,
            })
        }
        "promote" => {
            require_exact_options(&options, &["--artifact-id"])?;
            let artifact_id = required_option(&options, "--artifact-id")?;
            Ok(RunnerInvocation {
                arguments: option_arguments(
                    &["fixture", "promote"],
                    &[("--artifact-id", artifact_id)],
                ),
                oracle_dependent: false,
                math_probe: None,
            })
        }
        unknown => Err(DifferentialError::usage(format!(
            "unknown fixture action `{unknown}`"
        ))),
    }
}

fn run_math_probe_command(
    repository_root: &Path,
    invocation: &MathProbeInvocation,
) -> Result<(), DifferentialError> {
    let request_bytes = read_regular_file(repository_root, MATH_PROBE_REQUEST)?;
    let policy_bytes = read_regular_file(repository_root, PHASE4_POLICY)?;
    let request =
        decode_math_probe_request_jsonl(&request_bytes, &HarnessLimits::phase2_default_v1())
            .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
    let policy_text = std::str::from_utf8(&policy_bytes)
        .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
    let policy = Phase4PolicyProfile::parse_toml(policy_text)
        .map_err(|error| DifferentialError::new("policy", error.to_string()))?;
    if request.tolerance_profile_sha256() != policy.profile_sha256() {
        return Err(DifferentialError::new(
            "policy",
            format!(
                "request policy hash {} does not match checked-in profile {}",
                request.tolerance_profile_sha256().as_str(),
                policy.profile_sha256().as_str()
            ),
        ));
    }

    if invocation.action == MathProbeAction::VerifyDeterminism {
        return verify_math_probe_determinism(
            repository_root,
            &request,
            &invocation.preset,
            invocation.runs,
        );
    }

    let capture = execute_math_probe_once(repository_root, &request, &invocation.preset)?;
    let native_adapter = EmptyWorldAdapter::new(ORACLE_REVISION)
        .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
    compare_math_probe_results(
        &request,
        &capture.results,
        &policy,
        &capture.oracle_identity,
        native_adapter.build_identity(),
    )?;
    let action = match invocation.action {
        MathProbeAction::Compare => "compare",
        MathProbeAction::Replay => "replay",
        MathProbeAction::VerifyDeterminism => unreachable!("handled before execution"),
    };
    println!(
        "math-probes {action}: {} ordered cases matched under {} ({})",
        capture.results.len(),
        policy.profile_id(),
        invocation.preset
    );
    Ok(())
}

fn read_regular_file(repository_root: &Path, relative: &str) -> Result<Vec<u8>, DifferentialError> {
    let path = repository_root.join(relative);
    let metadata = fs::symlink_metadata(&path).map_err(|error| {
        DifferentialError::new(
            "filesystem",
            format!("failed to inspect {}: {error}", path.display()),
        )
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(DifferentialError::new(
            "filesystem",
            format!("{} must be a regular checked-in file", path.display()),
        ));
    }
    fs::read(&path).map_err(|error| {
        DifferentialError::new(
            "filesystem",
            format!("failed to read {}: {error}", path.display()),
        )
    })
}

struct MathProbeCapture {
    results: Vec<MathProbeResult>,
    response_bytes: Vec<u8>,
    oracle_identity: BuildIdentity,
}

fn execute_math_probe_once(
    repository_root: &Path,
    request: &MathProbeRequestRecord,
    preset: &str,
) -> Result<MathProbeCapture, DifferentialError> {
    let oracle_preset = match preset {
        "oracle-debug" => OraclePreset::Debug,
        "oracle-release" => OraclePreset::Release,
        _ => return Err(DifferentialError::usage("unregistered math-probe preset")),
    };
    let executable = OracleExecutable::resolve(repository_root, oracle_preset)
        .map_err(|error| DifferentialError::new("oracle", error.to_string()))?;
    let capture =
        execute_math_probe_process(&executable, request, ORACLE_REVISION).map_err(|error| {
            let stderr = String::from_utf8_lossy(error.retained_stderr());
            DifferentialError::process(format!(
                "{}; stderr bytes {}, killed {}, reaped {}: {}",
                error,
                error.stderr_bytes(),
                error.child_killed(),
                error.child_reaped(),
                stderr.trim_end()
            ))
        })?;
    if capture.identity().cmake_preset() != preset || capture.identity().maybe_phase4().is_none() {
        return Err(DifferentialError::new(
            "identity",
            "oracle handshake lacks the requested Phase 4 build identity",
        ));
    }
    let expected_adapter_digest = upstream::adapter_source_digest(repository_root)
        .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
    if capture.identity().adapter_content_sha256().as_str() != expected_adapter_digest {
        return Err(DifferentialError::new(
            "identity",
            "oracle adapter digest differs from independently hashed checked-in inputs",
        ));
    }
    let expected_compile_digest = effective_compile_command_sha256(repository_root, preset)?;
    let phase4_identity = capture
        .identity()
        .maybe_phase4()
        .ok_or_else(|| DifferentialError::new("identity", "Phase 4 identity is missing"))?;
    if phase4_identity.compile_command_sha256() != expected_compile_digest {
        return Err(DifferentialError::new(
            "identity",
            "oracle compile-command digest differs from the effective compile database",
        ));
    }
    Ok(MathProbeCapture {
        results: capture.results().to_vec(),
        response_bytes: capture.response_bytes().to_vec(),
        oracle_identity: capture.identity().clone(),
    })
}

fn effective_compile_command_sha256(
    repository_root: &Path,
    preset: &str,
) -> Result<String, DifferentialError> {
    let path = repository_root
        .join("target/reference")
        .join(preset)
        .join("compile_commands.json");
    let bytes = fs::read(&path).map_err(|error| {
        DifferentialError::new(
            "identity",
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    compile_database_sha256(&bytes)
}

fn compile_database_sha256(bytes: &[u8]) -> Result<String, DifferentialError> {
    let entries: Vec<serde_json::Value> = serde_json::from_slice(bytes)
        .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
    let mut commands = entries
        .into_iter()
        .filter_map(|entry| {
            let source = entry.get("file")?.as_str()?;
            let filename = Path::new(source).file_name()?.to_str()?;
            if !matches!(filename, "math_probe.cpp" | "protocol_bits.cpp") {
                return None;
            }
            let command =
                if let Some(command) = entry.get("command").and_then(|value| value.as_str()) {
                    command.to_owned()
                } else {
                    let arguments = entry.get("arguments")?.as_array()?;
                    let mut command = String::new();
                    for argument in arguments {
                        command.push_str(argument.as_str()?);
                        command.push('\n');
                    }
                    command
                };
            Some(format!("{source}\n{command}"))
        })
        .collect::<Vec<_>>();
    if commands.len() != 2 {
        return Err(DifferentialError::new(
            "identity",
            format!(
                "expected two effective math/probe compile commands, found {}",
                commands.len()
            ),
        ));
    }
    commands.sort_unstable();
    Ok(format!("{:x}", Sha256::digest(commands.join("\n"))))
}

fn compare_math_probe_results(
    request: &MathProbeRequestRecord,
    actual: &[MathProbeResult],
    policy: &Phase4PolicyProfile,
    oracle_identity: &BuildIdentity,
    native_identity: &BuildIdentity,
) -> Result<(), DifferentialError> {
    for (engine, identity) in [
        ("C++ oracle", oracle_identity),
        ("native Rust", native_identity),
    ] {
        if identity.oracle_revision() != ORACLE_REVISION || identity.maybe_phase4().is_none() {
            return Err(DifferentialError::new(
                "identity",
                format!("{engine} identity is not bound to the Phase 4 oracle contract"),
            ));
        }
        if identity.evidence_tier() == BuildEvidenceTier::D3Exploratory {
            return Err(DifferentialError::new(
                "identity",
                format!("{engine} exploratory identity cannot authorize Phase 4 comparison"),
            ));
        }
    }
    let expected = NativeMathProbeExecutor::execute(request)
        .map_err(|error| DifferentialError::new("native", error.to_string()))?;
    let comparison_tier = comparison_evidence_tier(oracle_identity, native_identity);
    if expected.len() != actual.len() {
        return Err(math_probe_mismatch("result count differs"));
    }
    for (expected, actual) in expected.iter().zip(actual) {
        if expected.case_id() != actual.case_id()
            || expected.operation() != actual.operation()
            || expected.policy_path() != actual.policy_path()
            || expected.horizon() != actual.horizon()
            || expected.discrete() != actual.discrete()
            || expected.values().len() != actual.values().len()
        {
            return Err(math_probe_mismatch(format!(
                "structural mismatch at case {}",
                expected.case_id()
            )));
        }
        let field_policy = policy
            .field(expected.policy_path().as_str())
            .ok_or_else(|| math_probe_mismatch("policy path is not registered"))?;
        if !horizons_match(expected.horizon(), field_policy.horizon()) {
            return Err(math_probe_mismatch(format!(
                "request horizon does not match policy at case {}",
                expected.case_id()
            )));
        }
        if !tier_authorizes(comparison_tier, field_policy.evidence_tier()) {
            return Err(DifferentialError::new(
                "identity",
                format!(
                    "comparison tier {comparison_tier:?} cannot apply policy tier {:?} at case {}",
                    field_policy.evidence_tier(),
                    expected.case_id()
                ),
            ));
        }
        for (expected_value, actual_value) in expected.values().iter().zip(actual.values()) {
            if expected_value.field() != actual_value.field()
                || expected_value.class() != actual_value.class()
                || expected_value.is_negative() != actual_value.is_negative()
                || !float_values_match_with_policy(
                    expected_value.bits(),
                    actual_value.bits(),
                    field_policy,
                )
            {
                return Err(math_probe_mismatch(format!(
                    "numeric mismatch at case {} field {:?}: expected {:#010x}, actual {:#010x}",
                    expected.case_id(),
                    expected_value.field(),
                    expected_value.bits().bits(),
                    actual_value.bits().bits()
                )));
            }
        }
    }
    Ok(())
}

fn comparison_evidence_tier(
    oracle_identity: &BuildIdentity,
    native_identity: &BuildIdentity,
) -> EvidenceTier {
    match (
        oracle_identity.evidence_tier(),
        native_identity.evidence_tier(),
    ) {
        (BuildEvidenceTier::D1Canonical, BuildEvidenceTier::D1Canonical) => {
            EvidenceTier::D1Canonical
        }
        (BuildEvidenceTier::D3Exploratory, _) | (_, BuildEvidenceTier::D3Exploratory) => {
            EvidenceTier::D3Exploratory
        }
        _ => EvidenceTier::D2Supported,
    }
}

const fn horizons_match(request: MathProbeHorizon, policy: DivergenceHorizon) -> bool {
    match (request, policy) {
        (MathProbeHorizon::Operation, DivergenceHorizon::Operation) => true,
        (
            MathProbeHorizon::ScenarioSteps {
                steps: request_steps,
            },
            DivergenceHorizon::ScenarioSteps {
                steps: policy_steps,
            },
        ) => request_steps == policy_steps,
        _ => false,
    }
}

const fn tier_authorizes(actual: EvidenceTier, policy: EvidenceTier) -> bool {
    matches!(
        actual,
        EvidenceTier::D1Canonical | EvidenceTier::D2Supported
    ) && matches!(
        policy,
        EvidenceTier::D1Canonical | EvidenceTier::D2Supported
    )
}

fn math_probe_mismatch(message: impl Into<String>) -> DifferentialError {
    DifferentialError::new("physics-mismatch", message)
}

fn verify_math_probe_determinism(
    repository_root: &Path,
    request: &MathProbeRequestRecord,
    preset: &str,
    runs: usize,
) -> Result<(), DifferentialError> {
    let mut baseline = None;
    for run in 0..runs {
        let capture = execute_math_probe_once(repository_root, request, preset)?;
        if let Some(expected) = &baseline
            && expected != &capture.response_bytes
        {
            return Err(DifferentialError::new(
                "determinism",
                format!("D0 response bytes changed on run {}", run + 1),
            ));
        }
        baseline = Some(capture.response_bytes);
    }
    println!("math-probes D0: {runs} byte-identical {preset} runs");
    Ok(())
}

fn parse_options(args: &[String]) -> Result<BTreeMap<String, String>, DifferentialError> {
    let mut options = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        let option = &pair[0];
        if !option.starts_with("--") {
            return Err(DifferentialError::usage(format!(
                "unexpected positional argument `{option}`"
            )));
        }
        if options.insert(option.clone(), pair[1].clone()).is_some() {
            return Err(DifferentialError::usage(format!(
                "duplicate differential option `{option}`"
            )));
        }
    }
    if !args.chunks_exact(2).remainder().is_empty() {
        return Err(DifferentialError::usage(
            "every differential option requires one value",
        ));
    }
    Ok(options)
}

fn require_exact_options(
    options: &BTreeMap<String, String>,
    expected: &[&str],
) -> Result<(), DifferentialError> {
    if options.len() == expected.len()
        && options
            .keys()
            .all(|option| expected.contains(&option.as_str()))
    {
        return Ok(());
    }
    Err(DifferentialError::usage(
        "differential command options do not match the registered command shape",
    ))
}

fn required_option<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a str, DifferentialError> {
    options
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| DifferentialError::usage(format!("missing required option `{name}`")))
}

fn require_allowed<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
    allowed: &[&str],
) -> Result<&'a str, DifferentialError> {
    let value = required_option(options, name)?;
    if allowed.contains(&value) {
        return Ok(value);
    }
    Err(DifferentialError::usage(format!(
        "unregistered value `{value}` for `{name}`; allowed values: {}",
        allowed.join(", ")
    )))
}

fn option_arguments(prefix: &[&str], options: &[(&str, &str)]) -> Vec<String> {
    prefix
        .iter()
        .copied()
        .chain(options.iter().flat_map(|(option, value)| [*option, *value]))
        .map(str::to_owned)
        .collect()
}

fn run_differential(repository_root: &Path, arguments: &[String]) -> Result<(), DifferentialError> {
    if let Some(program) = env::var_os("LIQUIDFUN_XTASK_DIFFERENTIAL") {
        return run_process(
            &program,
            arguments,
            repository_root,
            "run differential command",
        );
    }

    let cargo = env::var_os("LIQUIDFUN_XTASK_CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let mut cargo_arguments = [
        "run",
        "--quiet",
        "--package",
        "liquidfun-differential",
        "--bin",
        "liquidfun-differential",
        "--",
    ]
    .iter()
    .map(|argument| (*argument).to_owned())
    .collect::<Vec<_>>();
    cargo_arguments.extend_from_slice(arguments);
    run_process(
        &cargo,
        &cargo_arguments,
        repository_root,
        "run differential command",
    )
}

fn run_process<S: AsRef<std::ffi::OsStr>>(
    program: &std::ffi::OsStr,
    arguments: &[S],
    repository_root: &Path,
    operation: &str,
) -> Result<(), DifferentialError> {
    let status = Command::new(program)
        .args(arguments)
        .current_dir(repository_root)
        .status()
        .map_err(|error| {
            DifferentialError::process(format!(
                "failed to start `{}` while attempting to {operation}: {error}",
                program.to_string_lossy()
            ))
        })?;
    if status.success() {
        return Ok(());
    }

    let status = status.code().map_or_else(
        || "terminated by signal".to_owned(),
        |code| code.to_string(),
    );
    Err(DifferentialError::process(format!(
        "`{}` failed while attempting to {operation} (status {status})",
        program.to_string_lossy()
    )))
}

fn repository_root() -> Result<PathBuf, DifferentialError> {
    let current_dir = env::current_dir().map_err(|error| {
        DifferentialError::new(
            "filesystem",
            format!("failed to read current directory: {error}"),
        )
    })?;
    let Some(root) = current_dir.ancestors().find(|candidate| {
        candidate.join("reference/upstream-lock.toml").is_file()
            && candidate.join(".gitmodules").is_file()
    }) else {
        return Err(DifferentialError::new(
            "repository",
            "could not find reference/upstream-lock.toml and .gitmodules",
        ));
    };
    Ok(root.to_path_buf())
}

#[cfg(test)]
mod tests {
    use liquidfun_test_protocol::{DivergenceHorizon, EvidenceTier, MathProbeHorizon};

    use super::{compile_database_sha256, horizons_match, tier_authorizes};

    #[test]
    fn effective_compile_digest_changes_with_material_command_inputs() {
        // Arrange
        let baseline = br#"[
          {"file":"/repo/math_probe.cpp","command":"clang++ -I/a -DFLAG=1 -march=x86-64 -c math_probe.cpp"},
          {"file":"/repo/protocol_bits.cpp","command":"clang++ -I/a -DFLAG=1 -c protocol_bits.cpp"}
        ]"#;
        let baseline_text = String::from_utf8(baseline.to_vec()).expect("fixture is UTF-8");
        let variants = [
            baseline_text.replace("-DFLAG=1", "-DFLAG=2"),
            baseline_text.replace("-I/a", "-I/b"),
            baseline_text.replace("-march=x86-64", "-march=x86-64-v3"),
        ];

        // Act
        let baseline_digest = compile_database_sha256(baseline).expect("baseline should hash");
        let changed_digests = variants
            .iter()
            .map(|changed| {
                compile_database_sha256(changed.as_bytes()).expect("changed command should hash")
            })
            .collect::<Vec<_>>();

        // Assert
        assert!(
            changed_digests
                .iter()
                .all(|changed| changed != &baseline_digest)
        );
    }

    #[test]
    fn request_horizon_must_exactly_match_field_policy() {
        // Arrange / Act / Assert
        assert!(horizons_match(
            MathProbeHorizon::ScenarioSteps { steps: 32 },
            DivergenceHorizon::ScenarioSteps { steps: 32 }
        ));
        assert!(!horizons_match(
            MathProbeHorizon::Operation,
            DivergenceHorizon::ScenarioSteps { steps: 32 }
        ));
        assert!(!horizons_match(
            MathProbeHorizon::ScenarioSteps { steps: 4 },
            DivergenceHorizon::ScenarioSteps { steps: 32 }
        ));
    }

    #[test]
    fn exploratory_or_replay_tier_cannot_apply_authoritative_policy() {
        // Arrange / Act / Assert
        assert!(tier_authorizes(
            EvidenceTier::D2Supported,
            EvidenceTier::D1Canonical
        ));
        assert!(!tier_authorizes(
            EvidenceTier::D3Exploratory,
            EvidenceTier::D2Supported
        ));
        assert!(!tier_authorizes(
            EvidenceTier::D1Canonical,
            EvidenceTier::D0Replay
        ));
    }
}
