use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use liquidfun_differential::{
    CapturedRigidWorld, EmptyWorldAdapter, NativeCollisionProbeExecutor, NativeMathProbeExecutor,
    NativeRigidWorldExecutor, OracleExecutable, OraclePreset, Phase4ComparisonEvidence,
    Phase4DiscreteMismatchReport, Phase4HarnessFailureReason, Phase4HarnessFailureReport,
    Phase4MathMismatchReport, RigidComparisonOutcome, compare_collision_probe_results,
    compare_rigid_world_results, execute_collision_probe_process, execute_math_probe_process,
    execute_rigid_world_process, float_values_match_with_policy, validate_oracle_checkout_identity,
};
use liquidfun_test_protocol::{
    BuildEvidenceTier, BuildIdentity, CollisionProbeRequestRecord, CollisionProbeResult,
    DivergenceHorizon, EvidenceTier, HarnessLimits, MathProbeHorizon, MathProbeRequestRecord,
    MathProbeResult, Phase4PolicyProfile, Phase5PolicyProfile, Phase6PolicyProfile,
    RigidWorldRequestRecord, decode_collision_probe_request_jsonl, decode_math_probe_request_jsonl,
    decode_rigid_world_request_jsonl,
};
use sha2::{Digest, Sha256};

use crate::upstream;

const USAGE: &str = r"Usage: cargo xtask differential <command> [arguments]

Commands:
  compare  --scenario <empty-world|math-probes|collision-probes|rigid-world> --preset <oracle-debug|oracle-release|oracle-asan-ubsan> --session-profile <one-shot|reuse|sanitizer>
  replay   --scenario <empty-world|math-probes|collision-probes|rigid-world> --preset <oracle-debug|oracle-release|oracle-asan-ubsan> --session-profile <one-shot|reuse|sanitizer>
  minimize --scenario <empty-world|rigid-world> --preset <oracle-debug|oracle-release|oracle-asan-ubsan> --session-profile <one-shot|reuse|sanitizer>
  verify-determinism --scenario <math-probes|collision-probes|rigid-world> --preset <oracle-debug|oracle-release> --runs 2
  fixture stage   --scenario <empty-world|rigid-world> --preset <preset> --session-profile <profile> --artifact-kind <reviewed-trace|minimized-regression> --artifact-id <id>
  fixture review  --artifact-id <id> --reviewer <identity> --reviewed-at <UTC timestamp> --review-status <approved|rejected>
  fixture promote --artifact-id <id>";

const ORACLE_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const MATH_PROBE_REQUEST: &str = "protocol/fixtures/accepted/math-probe-request.jsonl";
const PHASE4_POLICY: &str = "protocol/tolerances/phase4-v1.toml";
const COLLISION_PROBE_REQUEST: &str = "protocol/fixtures/accepted/collision-probe-request.jsonl";
const PHASE5_POLICY: &str = "protocol/tolerances/phase5-v1.toml";
const RIGID_WORLD_REQUEST: &str = "protocol/fixtures/accepted/rigid-world-request.jsonl";
const PHASE6_POLICY: &str = "protocol/tolerances/phase6-v1.toml";
const NATIVE_SOURCE_MANIFEST: &str = "crates/liquidfun-differential/native-math-sources.txt";
const ALLOWED_SCENARIOS: [&str; 4] = [
    "empty-world",
    "math-probes",
    "collision-probes",
    "rigid-world",
];
const ALLOWED_PRESETS: [&str; 3] = ["oracle-debug", "oracle-release", "oracle-asan-ubsan"];
const MATH_PROBE_PRESETS: [&str; 2] = ["oracle-debug", "oracle-release"];
const ALLOWED_PROFILES: [&str; 3] = ["one-shot", "reuse", "sanitizer"];
const ALLOWED_ARTIFACT_KINDS: [&str; 2] = ["reviewed-trace", "minimized-regression"];
const ALLOWED_REVIEW_STATUSES: [&str; 2] = ["approved", "rejected"];

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DifferentialError {
    category: &'static str,
    message: String,
    maybe_phase4_evidence: Option<Box<Phase4ComparisonEvidence>>,
}

impl DifferentialError {
    fn new(category: &'static str, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
            maybe_phase4_evidence: None,
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new("usage", format!("{}\n\n{USAGE}", message.into()))
    }

    fn process(message: impl Into<String>) -> Self {
        Self::new("process", message)
    }

    fn phase4_evidence(category: &'static str, evidence: Phase4ComparisonEvidence) -> Self {
        Self {
            category,
            message: evidence.render_human(),
            maybe_phase4_evidence: Some(Box::new(evidence)),
        }
    }
}

impl Display for DifferentialError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "differential/{}: {}",
            self.category, self.message
        )?;
        if let Some(evidence) = &self.maybe_phase4_evidence {
            let machine = evidence.render_machine().map_err(|_| fmt::Error)?;
            write!(formatter, "\n{}", String::from_utf8_lossy(&machine))?;
        }
        Ok(())
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
    Minimize,
    VerifyDeterminism,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeKind {
    Math,
    Collision,
    Rigid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MathProbeInvocation {
    kind: ProbeKind,
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
    if let Some(probe) = invocation.math_probe {
        return match probe.kind {
            ProbeKind::Math => run_math_probe_command(&repository_root, &probe),
            ProbeKind::Collision => run_collision_probe_command(&repository_root, &probe),
            ProbeKind::Rigid => run_rigid_world_command(&repository_root, &probe),
        };
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
    let math_probe = if matches!(scenario, "math-probes" | "collision-probes" | "rigid-world") {
        let sanitizer_rigid_compare =
            scenario == "rigid-world" && command == "compare" && preset == "oracle-asan-ubsan";
        let shape_is_reviewed = profile == "one-shot"
            && (MATH_PROBE_PRESETS.contains(&preset) || sanitizer_rigid_compare);
        let action_is_reviewed = scenario == "rigid-world" || command != "minimize";
        if !shape_is_reviewed || !action_is_reviewed {
            return Err(DifferentialError::usage(
                "fixed evidence scenarios support only their reviewed one-shot debug or release shape",
            ));
        }
        Some(MathProbeInvocation {
            kind: match scenario {
                "math-probes" => ProbeKind::Math,
                "collision-probes" => ProbeKind::Collision,
                "rigid-world" => ProbeKind::Rigid,
                _ => unreachable!("closed fixed evidence scenario"),
            },
            action: match command {
                "compare" => MathProbeAction::Compare,
                "replay" => MathProbeAction::Replay,
                "minimize" => MathProbeAction::Minimize,
                _ => unreachable!("closed scenario command"),
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
    let scenario = require_allowed(
        &options,
        "--scenario",
        &["math-probes", "collision-probes", "rigid-world"],
    )?;
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
            kind: match scenario {
                "math-probes" => ProbeKind::Math,
                "collision-probes" => ProbeKind::Collision,
                "rigid-world" => ProbeKind::Rigid,
                _ => unreachable!("closed determinism scenario"),
            },
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
    let expected_native_digest = native_source_manifest_sha256(repository_root)?;
    if native_adapter
        .build_identity()
        .adapter_content_sha256()
        .as_str()
        != expected_native_digest
    {
        return Err(DifferentialError::new(
            "identity",
            "native adapter digest differs from independently hashed reviewed math inputs",
        ));
    }
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
        MathProbeAction::Minimize => unreachable!("math probes do not support minimization"),
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

fn run_collision_probe_command(
    repository_root: &Path,
    invocation: &MathProbeInvocation,
) -> Result<(), DifferentialError> {
    let request_bytes = read_regular_file(repository_root, COLLISION_PROBE_REQUEST)?;
    let policy_bytes = read_regular_file(repository_root, PHASE5_POLICY)?;
    let request =
        decode_collision_probe_request_jsonl(&request_bytes, &HarnessLimits::phase2_default_v1())
            .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
    let policy_text = std::str::from_utf8(&policy_bytes)
        .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
    let policy = Phase5PolicyProfile::parse_toml(policy_text)
        .map_err(|error| DifferentialError::new("policy", error.to_string()))?;
    if request.tolerance_profile_sha256() != policy.profile_sha256() {
        return Err(DifferentialError::new(
            "policy",
            "collision request policy hash differs from the checked-in Phase 5 profile",
        ));
    }
    if invocation.action == MathProbeAction::VerifyDeterminism {
        return verify_collision_probe_determinism(
            repository_root,
            &request,
            &invocation.preset,
            invocation.runs,
        );
    }
    let capture = execute_collision_probe_once(repository_root, &request, &invocation.preset)?;
    let native = NativeCollisionProbeExecutor::execute(&request)
        .map_err(|error| DifferentialError::new("native", error.to_string()))?;
    compare_collision_probe_results(&request, &native, &capture.results, &policy).map_err(
        |divergence| {
            DifferentialError::new(
                "collision",
                format!(
                    "first divergence {}: {}",
                    divergence.signature_sha256().as_str(),
                    String::from_utf8_lossy(
                        &divergence
                            .render_machine()
                            .unwrap_or_else(|_| b"{}".to_vec())
                    )
                ),
            )
        },
    )?;
    let action = match invocation.action {
        MathProbeAction::Compare => "compare",
        MathProbeAction::Replay => "replay",
        MathProbeAction::Minimize => unreachable!("collision probes do not support minimization"),
        MathProbeAction::VerifyDeterminism => unreachable!("handled before execution"),
    };
    println!(
        "collision-probes {action}: {} ordered cases matched under {} ({})",
        capture.results.len(),
        policy.profile_id(),
        invocation.preset
    );
    Ok(())
}

fn run_rigid_world_command(
    repository_root: &Path,
    invocation: &MathProbeInvocation,
) -> Result<(), DifferentialError> {
    let policy_bytes = read_regular_file(repository_root, PHASE6_POLICY)?;
    let policy_text = std::str::from_utf8(&policy_bytes)
        .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
    let policy = Phase6PolicyProfile::parse_toml(policy_text)
        .map_err(|error| DifferentialError::new("policy", error.to_string()))?;
    let request = rigid_world_request(repository_root, &policy)?;

    if invocation.action == MathProbeAction::VerifyDeterminism {
        return verify_rigid_world_determinism(
            repository_root,
            &request,
            &invocation.preset,
            invocation.runs,
        );
    }

    let captured = execute_rigid_world_once(repository_root, &request, &invocation.preset)?;
    let native = NativeRigidWorldExecutor::execute(&request)
        .map_err(|error| DifferentialError::new("native", error.to_string()))?;
    let outcome = compare_rigid_world_results(&request, &native, captured.result(), &policy)
        .map_err(|error| {
            DifferentialError::new(
                "rigid-harness",
                serde_json::to_string(&error).unwrap_or_else(|_| format!("{error:?}")),
            )
        })?;
    let RigidComparisonOutcome::Match = outcome else {
        let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
            unreachable!("rigid comparison has exactly two outcomes")
        };
        return Err(DifferentialError::new(
            "physics-mismatch",
            String::from_utf8_lossy(
                &report
                    .render_machine()
                    .map_err(|error| DifferentialError::new("report", error.to_string()))?,
            )
            .into_owned(),
        ));
    };
    if invocation.action == MathProbeAction::Minimize {
        return Err(DifferentialError::new(
            "minimization",
            "rigid-world minimization requires a captured first-divergence signature",
        ));
    }

    let native_identity = EmptyWorldAdapter::new(ORACLE_REVISION)
        .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
    let action = match invocation.action {
        MathProbeAction::Compare => "compare",
        MathProbeAction::Replay => "replay",
        MathProbeAction::Minimize | MathProbeAction::VerifyDeterminism => {
            unreachable!("handled before matched output")
        }
    };
    println!(
        "rigid-world {action}: {} required families matched under {} ({}); oracle={}, native={}",
        request.scenario().timelines().len(),
        policy.profile_id(),
        invocation.preset,
        build_evidence_label(captured.identity().evidence_tier()),
        build_evidence_label(native_identity.build_identity().evidence_tier()),
    );
    Ok(())
}

fn rigid_world_request(
    repository_root: &Path,
    policy: &Phase6PolicyProfile,
) -> Result<RigidWorldRequestRecord, DifferentialError> {
    let request_bytes = read_regular_file(repository_root, RIGID_WORLD_REQUEST)?;
    let mut request_value: serde_json::Value = serde_json::from_slice(&request_bytes)
        .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
    request_value["tolerance_profile_sha256"] =
        serde_json::Value::String(policy.profile_sha256().as_str().to_owned());
    let mut bound_bytes = serde_json::to_vec(&request_value)
        .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
    bound_bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bound_bytes, &HarnessLimits::phase2_default_v1())
        .map_err(|error| DifferentialError::new("protocol", error.to_string()))
}

fn execute_rigid_world_once(
    repository_root: &Path,
    request: &RigidWorldRequestRecord,
    preset: &str,
) -> Result<CapturedRigidWorld, DifferentialError> {
    let oracle_preset = match preset {
        "oracle-debug" => OraclePreset::Debug,
        "oracle-release" => OraclePreset::Release,
        "oracle-asan-ubsan" => OraclePreset::AsanUbsan,
        _ => return Err(DifferentialError::usage("unregistered rigid-world preset")),
    };
    let oracle_program = OracleExecutable::resolve(repository_root, oracle_preset)
        .map_err(|error| DifferentialError::new("oracle", error.to_string()))?;
    let captured = execute_rigid_world_process(&oracle_program, request, ORACLE_REVISION).map_err(
        |error| {
            DifferentialError::process(format!(
                "{}; stderr bytes {}, killed {}, reaped {}: {}",
                error,
                error.stderr_bytes(),
                error.child_killed(),
                error.child_reaped(),
                String::from_utf8_lossy(error.retained_stderr()).trim_end()
            ))
        },
    )?;
    if captured.identity().cmake_preset() != preset || captured.identity().maybe_phase4().is_none()
    {
        return Err(DifferentialError::new(
            "identity",
            "oracle handshake lacks the requested rigid-world build identity",
        ));
    }
    validate_oracle_checkout_identity(repository_root, preset, captured.identity())
        .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
    Ok(captured)
}

fn verify_rigid_world_determinism(
    repository_root: &Path,
    request: &RigidWorldRequestRecord,
    preset: &str,
    runs: usize,
) -> Result<(), DifferentialError> {
    let mut maybe_oracle_baseline: Option<Vec<u8>> = None;
    let mut maybe_native_baseline: Option<Vec<u8>> = None;
    for run in 0..runs {
        let capture = execute_rigid_world_once(repository_root, request, preset)?;
        if let Some(expected) = &maybe_oracle_baseline
            && expected.as_slice() != capture.response_bytes()
        {
            return Err(DifferentialError::new(
                "determinism",
                format!("rigid oracle D0 response bytes changed on run {}", run + 1),
            ));
        }
        maybe_oracle_baseline = Some(capture.response_bytes().to_vec());

        let native = NativeRigidWorldExecutor::execute(request)
            .map_err(|error| DifferentialError::new("native", error.to_string()))?;
        let native_bytes = serde_json::to_vec(&native)
            .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
        if let Some(expected) = &maybe_native_baseline
            && expected != &native_bytes
        {
            return Err(DifferentialError::new(
                "determinism",
                format!("native rigid D0 bytes changed on run {}", run + 1),
            ));
        }
        maybe_native_baseline = Some(native_bytes);
    }
    println!("rigid-world D0: {runs} byte-identical native and {preset} runs");
    Ok(())
}

const fn build_evidence_label(tier: BuildEvidenceTier) -> &'static str {
    match tier {
        BuildEvidenceTier::D1Canonical => "d1_canonical",
        BuildEvidenceTier::D2Supported => "d2_supported",
        BuildEvidenceTier::D3Exploratory => "d3_exploratory",
    }
}

fn native_source_manifest_sha256(repository_root: &Path) -> Result<String, DifferentialError> {
    let manifest_path = repository_root.join(NATIVE_SOURCE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).map_err(|error| {
        DifferentialError::new(
            "identity",
            format!("failed to read {}: {error}", manifest_path.display()),
        )
    })?;
    native_source_digest_from_manifest(repository_root, &manifest)
}

fn native_source_digest_from_manifest(
    repository_root: &Path,
    manifest: &str,
) -> Result<String, DifferentialError> {
    let mut hasher = Sha256::new();
    for relative in manifest
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let path = Path::new(relative);
        if path.is_absolute()
            || path
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return Err(DifferentialError::new(
                "identity",
                format!("invalid native source manifest path `{relative}`"),
            ));
        }
        let bytes = fs::read(repository_root.join(path)).map_err(|error| {
            DifferentialError::new(
                "identity",
                format!("failed to hash native source `{relative}`: {error}"),
            )
        })?;
        let relative_len = u64::try_from(relative.len())
            .map_err(|_| DifferentialError::new("identity", "native source path is too long"))?;
        hasher.update(relative_len.to_be_bytes());
        hasher.update(relative.as_bytes());
        hasher.update(Sha256::digest(bytes));
    }
    Ok(format!("{:x}", hasher.finalize()))
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

struct CollisionProbeCapture {
    results: Vec<CollisionProbeResult>,
    response_bytes: Vec<u8>,
}

fn execute_collision_probe_once(
    repository_root: &Path,
    request: &CollisionProbeRequestRecord,
    preset: &str,
) -> Result<CollisionProbeCapture, DifferentialError> {
    let oracle_preset = match preset {
        "oracle-debug" => OraclePreset::Debug,
        "oracle-release" => OraclePreset::Release,
        _ => {
            return Err(DifferentialError::usage(
                "unregistered collision-probe preset",
            ));
        }
    };
    let executable = OracleExecutable::resolve(repository_root, oracle_preset)
        .map_err(|error| DifferentialError::new("oracle", error.to_string()))?;
    let capture = execute_collision_probe_process(&executable, request, ORACLE_REVISION).map_err(
        |error| {
            DifferentialError::process(format!(
                "{}; stderr bytes {}, killed {}, reaped {}: {}",
                error,
                error.stderr_bytes(),
                error.child_killed(),
                error.child_reaped(),
                String::from_utf8_lossy(error.retained_stderr()).trim_end()
            ))
        },
    )?;
    if capture.identity().cmake_preset() != preset || capture.identity().maybe_phase4().is_none() {
        return Err(DifferentialError::new(
            "identity",
            "oracle handshake lacks the requested collision build identity",
        ));
    }
    let expected_adapter_digest = liquidfun_differential::adapter_source_digest(repository_root)
        .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
    if capture.identity().adapter_content_sha256().as_str() != expected_adapter_digest {
        return Err(DifferentialError::new(
            "identity",
            "oracle adapter digest differs from checked-in inputs",
        ));
    }
    let expected_compile_digest =
        liquidfun_differential::effective_compile_command_sha256(repository_root, preset)
            .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
    if capture
        .identity()
        .maybe_phase4()
        .is_none_or(|identity| identity.compile_command_sha256() != expected_compile_digest)
    {
        return Err(DifferentialError::new(
            "identity",
            "oracle collision compile-command digest differs from the effective database",
        ));
    }
    Ok(CollisionProbeCapture {
        results: capture.results().to_vec(),
        response_bytes: capture.response_bytes().to_vec(),
    })
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
    let expected_adapter_digest = liquidfun_differential::adapter_source_digest(repository_root)
        .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
    if capture.identity().adapter_content_sha256().as_str() != expected_adapter_digest {
        return Err(DifferentialError::new(
            "identity",
            "oracle adapter digest differs from independently hashed checked-in inputs",
        ));
    }
    let expected_compile_digest =
        liquidfun_differential::effective_compile_command_sha256(repository_root, preset)
            .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
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

#[allow(
    clippy::too_many_lines,
    reason = "one ordered traversal keeps every typed Phase 4 failure path fail-closed"
)]
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
        return Err(phase4_harness_failure(
            request,
            policy,
            comparison_tier,
            oracle_identity,
            native_identity,
            Phase4HarnessFailureReason::ResultCount,
            None,
            expected.len().to_string(),
            actual.len().to_string(),
        )?);
    }
    for (case_index, (expected, actual)) in expected.iter().zip(actual).enumerate() {
        let structural_failure = if expected.case_id() != actual.case_id() {
            Some((
                Phase4HarnessFailureReason::CaseIdEcho,
                expected.case_id().to_owned(),
                actual.case_id().to_owned(),
            ))
        } else if expected.operation() != actual.operation() {
            Some((
                Phase4HarnessFailureReason::OperationEcho,
                format!("{:?}", expected.operation()),
                format!("{:?}", actual.operation()),
            ))
        } else if expected.policy_path() != actual.policy_path() {
            Some((
                Phase4HarnessFailureReason::PolicyPathEcho,
                expected.policy_path().as_str().to_owned(),
                actual.policy_path().as_str().to_owned(),
            ))
        } else if expected.horizon() != actual.horizon() {
            Some((
                Phase4HarnessFailureReason::HorizonEcho,
                format!("{:?}", expected.horizon()),
                format!("{:?}", actual.horizon()),
            ))
        } else if expected.values().len() != actual.values().len() {
            Some((
                Phase4HarnessFailureReason::ValueCount,
                expected.values().len().to_string(),
                actual.values().len().to_string(),
            ))
        } else if expected.discrete().len() != actual.discrete().len() {
            Some((
                Phase4HarnessFailureReason::DiscreteCount,
                expected.discrete().len().to_string(),
                actual.discrete().len().to_string(),
            ))
        } else {
            None
        };
        if let Some((reason, expected_context, actual_context)) = structural_failure {
            return Err(phase4_harness_failure(
                request,
                policy,
                comparison_tier,
                oracle_identity,
                native_identity,
                reason,
                Some(case_index),
                expected_context,
                actual_context,
            )?);
        }
        let Some(field_policy) = policy.field(expected.policy_path().as_str()) else {
            return Err(phase4_harness_failure(
                request,
                policy,
                comparison_tier,
                oracle_identity,
                native_identity,
                Phase4HarnessFailureReason::UnregisteredPolicy,
                Some(case_index),
                expected.policy_path().as_str(),
                "<missing>",
            )?);
        };
        if !horizons_match(expected.horizon(), field_policy.horizon()) {
            return Err(phase4_harness_failure(
                request,
                policy,
                comparison_tier,
                oracle_identity,
                native_identity,
                Phase4HarnessFailureReason::PolicyHorizon,
                Some(case_index),
                format!("{:?}", expected.horizon()),
                format!("{:?}", field_policy.horizon()),
            )?);
        }
        if !tier_authorizes(comparison_tier, field_policy.evidence_tier()) {
            return Err(phase4_harness_failure(
                request,
                policy,
                comparison_tier,
                oracle_identity,
                native_identity,
                Phase4HarnessFailureReason::PolicyTier,
                Some(case_index),
                format!("{:?}", field_policy.evidence_tier()),
                format!("{comparison_tier:?}"),
            )?);
        }
        for (expected_discrete, actual_discrete) in
            expected.discrete().iter().zip(actual.discrete())
        {
            if expected_discrete.field() != actual_discrete.field() {
                return Err(phase4_harness_failure(
                    request,
                    policy,
                    comparison_tier,
                    oracle_identity,
                    native_identity,
                    Phase4HarnessFailureReason::DiscreteFieldEcho,
                    Some(case_index),
                    format!("{:?}", expected_discrete.field()),
                    format!("{:?}", actual_discrete.field()),
                )?);
            }
            if expected_discrete.value() != actual_discrete.value() {
                let report = Phase4DiscreteMismatchReport::new(
                    request,
                    expected,
                    case_index,
                    *expected_discrete,
                    *actual_discrete,
                    policy.profile_id(),
                    policy.version(),
                    policy.profile_sha256(),
                    field_policy,
                    comparison_tier,
                    oracle_identity,
                    native_identity,
                )
                .map_err(|error| DifferentialError::new("report", error.to_string()))?;
                return Err(DifferentialError::phase4_evidence(
                    "physics-mismatch",
                    Phase4ComparisonEvidence::DiscreteMismatch(report),
                ));
            }
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
                let report = Phase4MathMismatchReport::new(
                    request,
                    expected,
                    case_index,
                    *expected_value,
                    *actual_value,
                    policy.profile_id(),
                    policy.version(),
                    policy.profile_sha256(),
                    field_policy,
                    comparison_tier,
                    oracle_identity.identity_sha256(),
                    native_identity.identity_sha256(),
                )
                .map_err(|error| DifferentialError::new("report", error.to_string()))?;
                return Err(DifferentialError::phase4_evidence(
                    "physics-mismatch",
                    Phase4ComparisonEvidence::NumericMismatch(report),
                ));
            }
        }
    }
    Ok(())
}

#[allow(
    clippy::too_many_arguments,
    reason = "the helper binds the failure to request, policy, tier, and both builds"
)]
fn phase4_harness_failure(
    request: &MathProbeRequestRecord,
    policy: &Phase4PolicyProfile,
    evidence_tier: EvidenceTier,
    oracle_identity: &BuildIdentity,
    native_identity: &BuildIdentity,
    reason: Phase4HarnessFailureReason,
    maybe_case_index: Option<usize>,
    expected: impl Into<String>,
    actual: impl Into<String>,
) -> Result<DifferentialError, DifferentialError> {
    let report = Phase4HarnessFailureReport::new(
        request,
        reason,
        maybe_case_index,
        expected,
        actual,
        policy.profile_id(),
        policy.version(),
        policy.profile_sha256(),
        evidence_tier,
        oracle_identity,
        native_identity,
    )
    .map_err(|error| DifferentialError::new("report", error.to_string()))?;
    Ok(DifferentialError::phase4_evidence(
        "harness-failure",
        Phase4ComparisonEvidence::HarnessFailure(report),
    ))
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

fn verify_collision_probe_determinism(
    repository_root: &Path,
    request: &CollisionProbeRequestRecord,
    preset: &str,
    runs: usize,
) -> Result<(), DifferentialError> {
    let mut baseline = None;
    for run in 0..runs {
        let capture = execute_collision_probe_once(repository_root, request, preset)?;
        if let Some(expected) = &baseline
            && expected != &capture.response_bytes
        {
            return Err(DifferentialError::new(
                "determinism",
                format!("collision D0 response bytes changed on run {}", run + 1),
            ));
        }
        baseline = Some(capture.response_bytes);
    }
    println!("collision-probes D0: {runs} byte-identical {preset} runs");
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
    use liquidfun_differential::{
        NativeMathProbeExecutor, Phase4ComparisonEvidence, Phase4HarnessFailureReason,
    };
    use liquidfun_test_protocol::{
        BuildIdentity, BuildIdentityFields, DivergenceHorizon, EvidenceTier, FloatBits,
        HarnessLimits, MathProbeDiscrete, MathProbeDiscreteField, MathProbeHorizon,
        MathProbeOperation, MathProbePolicyPath, MathProbeRequestRecord, MathProbeResult,
        MathProbeValue, Phase4BuildIdentityFields, Phase4PolicyProfile,
        decode_math_probe_request_jsonl,
    };

    use std::fs;

    use super::{
        ORACLE_REVISION, compare_math_probe_results, horizons_match,
        native_source_digest_from_manifest, tier_authorizes,
    };

    #[test]
    fn native_source_digest_changes_when_an_executor_input_changes() {
        // Arrange
        let root =
            std::env::temp_dir().join(format!("liquidfun-native-manifest-{}", std::process::id()));
        let source = root.join("crates/liquidfun-differential/src/math_probe.rs");
        fs::create_dir_all(source.parent().expect("fixture source has a parent"))
            .expect("temporary fixture directory should be created");
        fs::write(&source, b"executor v1").expect("first fixture should be written");
        let manifest = "crates/liquidfun-differential/src/math_probe.rs\n";
        let original = native_source_digest_from_manifest(&root, manifest)
            .expect("original manifest should hash");

        // Act
        fs::write(&source, b"executor v2").expect("changed fixture should be written");
        let changed = native_source_digest_from_manifest(&root, manifest)
            .expect("changed manifest should hash");

        // Assert
        assert_ne!(original, changed);
        fs::remove_dir_all(root).expect("temporary fixture should be removed");
    }

    #[test]
    fn actual_xtask_math_mismatch_carries_typed_machine_evidence() {
        // Arrange
        let request = decode_math_probe_request_jsonl(
            include_bytes!("../../../protocol/fixtures/accepted/math-probe-request.jsonl"),
            &HarnessLimits::phase2_default_v1(),
        )
        .expect("checked-in request should decode");
        let policy = Phase4PolicyProfile::parse_toml(include_str!(
            "../../../protocol/tolerances/phase4-v1.toml"
        ))
        .expect("checked-in policy should parse");
        let mut actual = NativeMathProbeExecutor::execute(&request)
            .expect("checked-in request should execute")
            .into_vec();
        let case_index = actual
            .iter()
            .position(|result| {
                result
                    .values()
                    .first()
                    .is_some_and(|value| value.bits().to_f32().is_finite())
            })
            .expect("fixture should contain a finite scalar result");
        let result = &actual[case_index];
        let mut values = result.values().to_vec();
        values[0] = MathProbeValue::new(values[0].field(), FloatBits::new(0x7f80_0000));
        actual[case_index] = MathProbeResult::new(
            result.case_id(),
            result.operation(),
            result.policy_path(),
            result.horizon(),
            values,
            result.discrete().to_vec(),
        );
        let oracle_identity = supported_math_identity("11");
        let native_identity = supported_math_identity("22");

        // Act
        let error = compare_math_probe_results(
            &request,
            &actual,
            &policy,
            &oracle_identity,
            &native_identity,
        )
        .expect_err("deliberate divergence should fail");

        // Assert
        assert_eq!(error.category, "physics-mismatch");
        let evidence = error
            .maybe_phase4_evidence
            .expect("actual xtask comparison should retain typed mismatch evidence");
        assert!(matches!(
            evidence.as_ref(),
            Phase4ComparisonEvidence::NumericMismatch(_)
        ));
        let machine = evidence
            .render_machine()
            .expect("typed report should serialize");
        let machine = String::from_utf8(machine).expect("JSON report should be UTF-8");
        assert!(machine.contains("\"policy_id\":\"phase4-v1\""));
        assert!(machine.contains("\"evidence_tier\":\"d2_supported\""));
        assert!(machine.contains("\"oracle_build_sha256\""));
        assert!(machine.contains("\"native_build_sha256\""));
        assert!(machine.contains("\"collection_policy\":\"ordered\""));
    }

    #[test]
    fn actual_xtask_result_count_failure_is_typed_harness_evidence() {
        // Arrange
        let (request, policy, mut actual, oracle_identity, native_identity) = math_fixture();
        actual.pop().expect("fixture should contain results");

        // Act
        let error = compare_math_probe_results(
            &request,
            &actual,
            &policy,
            &oracle_identity,
            &native_identity,
        )
        .expect_err("result count violation should fail");

        // Assert
        assert_harness_reason(error, Phase4HarnessFailureReason::ResultCount);
    }

    #[test]
    fn actual_xtask_structural_echo_failure_is_typed_harness_evidence() {
        // Arrange
        let (request, policy, mut actual, oracle_identity, native_identity) = math_fixture();
        let result = &actual[0];
        actual[0] = MathProbeResult::new(
            result.case_id(),
            MathProbeOperation::Abs,
            result.policy_path(),
            result.horizon(),
            result.values().to_vec(),
            result.discrete().to_vec(),
        );

        // Act
        let error = compare_math_probe_results(
            &request,
            &actual,
            &policy,
            &oracle_identity,
            &native_identity,
        )
        .expect_err("operation echo violation should fail");

        // Assert
        assert_harness_reason(error, Phase4HarnessFailureReason::OperationEcho);
    }

    #[test]
    fn actual_xtask_every_structural_failure_reason_is_typed_harness_evidence() {
        // Arrange
        let (request, policy, baseline, oracle_identity, native_identity) = math_fixture();
        let first = &baseline[0];
        let mut variants = Vec::new();

        let mut case_id = baseline.clone();
        case_id[0] = MathProbeResult::new(
            "changed-case-id",
            first.operation(),
            first.policy_path(),
            first.horizon(),
            first.values().to_vec(),
            first.discrete().to_vec(),
        );
        variants.push((case_id, Phase4HarnessFailureReason::CaseIdEcho));

        let mut policy_path = baseline.clone();
        policy_path[0] = MathProbeResult::new(
            first.case_id(),
            first.operation(),
            MathProbePolicyPath::MathOperationAbs,
            first.horizon(),
            first.values().to_vec(),
            first.discrete().to_vec(),
        );
        variants.push((policy_path, Phase4HarnessFailureReason::PolicyPathEcho));

        let mut horizon = baseline.clone();
        horizon[0] = MathProbeResult::new(
            first.case_id(),
            first.operation(),
            first.policy_path(),
            MathProbeHorizon::ScenarioSteps { steps: 4 },
            first.values().to_vec(),
            first.discrete().to_vec(),
        );
        variants.push((horizon, Phase4HarnessFailureReason::HorizonEcho));

        let value_index = baseline
            .iter()
            .position(|result| !result.values().is_empty())
            .expect("fixture should contain float values");
        let value_result = &baseline[value_index];
        let mut value_count = baseline.clone();
        let mut shortened_values = value_result.values().to_vec();
        shortened_values.pop().expect("selected result has a value");
        value_count[value_index] = MathProbeResult::new(
            value_result.case_id(),
            value_result.operation(),
            value_result.policy_path(),
            value_result.horizon(),
            shortened_values,
            value_result.discrete().to_vec(),
        );
        variants.push((value_count, Phase4HarnessFailureReason::ValueCount));

        let discrete_index = baseline
            .iter()
            .position(|result| !result.discrete().is_empty())
            .expect("fixture should contain discrete values");
        let discrete_result = &baseline[discrete_index];
        let mut discrete_count = baseline.clone();
        discrete_count[discrete_index] = MathProbeResult::new(
            discrete_result.case_id(),
            discrete_result.operation(),
            discrete_result.policy_path(),
            discrete_result.horizon(),
            discrete_result.values().to_vec(),
            Vec::new(),
        );
        variants.push((discrete_count, Phase4HarnessFailureReason::DiscreteCount));

        let mut discrete_field = baseline.clone();
        let mut changed_discrete = discrete_result.discrete().to_vec();
        changed_discrete[0] = MathProbeDiscrete::new(
            MathProbeDiscreteField::NonZeroDeterminant,
            changed_discrete[0].value(),
        );
        discrete_field[discrete_index] = MathProbeResult::new(
            discrete_result.case_id(),
            discrete_result.operation(),
            discrete_result.policy_path(),
            discrete_result.horizon(),
            discrete_result.values().to_vec(),
            changed_discrete,
        );
        variants.push((
            discrete_field,
            Phase4HarnessFailureReason::DiscreteFieldEcho,
        ));

        for (actual, expected_reason) in variants {
            // Act
            let error = compare_math_probe_results(
                &request,
                &actual,
                &policy,
                &oracle_identity,
                &native_identity,
            )
            .expect_err("structural violation should fail");

            // Assert
            assert_harness_reason(error, expected_reason);
        }
    }

    #[test]
    fn actual_xtask_unregistered_policy_is_typed_harness_evidence() {
        // Arrange
        let (request, _policy, actual, oracle_identity, native_identity) = math_fixture();
        let path = actual[0].policy_path().as_str();
        let policy_text = policy_without_path(
            include_str!("../../../protocol/tolerances/phase4-v1.toml"),
            path,
        );
        let policy = Phase4PolicyProfile::parse_toml(&policy_text)
            .expect("profile without one path remains structurally valid");

        // Act
        let error = compare_math_probe_results(
            &request,
            &actual,
            &policy,
            &oracle_identity,
            &native_identity,
        )
        .expect_err("missing policy should fail");

        // Assert
        assert_harness_reason(error, Phase4HarnessFailureReason::UnregisteredPolicy);
    }

    #[test]
    fn actual_xtask_policy_horizon_violation_is_typed_harness_evidence() {
        // Arrange
        let (request, _policy, actual, oracle_identity, native_identity) = math_fixture();
        let path = actual[0].policy_path().as_str();
        let policy_text = replace_in_policy_block(
            include_str!("../../../protocol/tolerances/phase4-v1.toml"),
            path,
            "horizon = { kind = \"operation\" }",
            "horizon = { kind = \"scenario_steps\", steps = 4 }",
        );
        let policy = Phase4PolicyProfile::parse_toml(&policy_text)
            .expect("alternate nonzero horizon remains structurally valid");

        // Act
        let error = compare_math_probe_results(
            &request,
            &actual,
            &policy,
            &oracle_identity,
            &native_identity,
        )
        .expect_err("policy horizon mismatch should fail");

        // Assert
        assert_harness_reason(error, Phase4HarnessFailureReason::PolicyHorizon);
    }

    #[test]
    fn actual_xtask_policy_tier_violation_is_typed_harness_evidence() {
        // Arrange
        let (request, _policy, actual, oracle_identity, native_identity) = math_fixture();
        let path = actual[0].policy_path().as_str();
        let policy_text = replace_in_policy_block(
            include_str!("../../../protocol/tolerances/phase4-v1.toml"),
            path,
            "evidence_tier = \"d1_canonical\"",
            "evidence_tier = \"d3_exploratory\"",
        );
        let policy = Phase4PolicyProfile::parse_toml(&policy_text)
            .expect("exploratory policy tier remains structurally valid");

        // Act
        let error = compare_math_probe_results(
            &request,
            &actual,
            &policy,
            &oracle_identity,
            &native_identity,
        )
        .expect_err("unauthorized policy tier should fail");

        // Assert
        assert_harness_reason(error, Phase4HarnessFailureReason::PolicyTier);
    }

    #[test]
    fn actual_xtask_discrete_difference_is_typed_mismatch_evidence() {
        // Arrange
        let (request, policy, mut actual, oracle_identity, native_identity) = math_fixture();
        let case_index = actual
            .iter()
            .position(|result| !result.discrete().is_empty())
            .expect("fixture should contain a discrete result");
        let result = &actual[case_index];
        let mut discrete = result.discrete().to_vec();
        discrete[0] = MathProbeDiscrete::new(discrete[0].field(), !discrete[0].value());
        actual[case_index] = MathProbeResult::new(
            result.case_id(),
            result.operation(),
            result.policy_path(),
            result.horizon(),
            result.values().to_vec(),
            discrete,
        );

        // Act
        let error = compare_math_probe_results(
            &request,
            &actual,
            &policy,
            &oracle_identity,
            &native_identity,
        )
        .expect_err("discrete semantic difference should fail");

        // Assert
        assert_eq!(error.category, "physics-mismatch");
        let evidence = error
            .maybe_phase4_evidence
            .expect("discrete mismatch should carry typed evidence");
        assert!(matches!(
            evidence.as_ref(),
            Phase4ComparisonEvidence::DiscreteMismatch(_)
        ));
        let machine = String::from_utf8(
            evidence
                .render_machine()
                .expect("discrete evidence should serialize"),
        )
        .expect("JSON evidence should be UTF-8");
        assert!(machine.contains("\"expected_value\""));
        assert!(machine.contains("\"actual_value\""));
        assert!(machine.contains("\"policy_id\":\"phase4-v1\""));
    }

    fn math_fixture() -> (
        MathProbeRequestRecord,
        Phase4PolicyProfile,
        Vec<MathProbeResult>,
        BuildIdentity,
        BuildIdentity,
    ) {
        let request = decode_math_probe_request_jsonl(
            include_bytes!("../../../protocol/fixtures/accepted/math-probe-request.jsonl"),
            &HarnessLimits::phase2_default_v1(),
        )
        .expect("checked-in request should decode");
        let policy = Phase4PolicyProfile::parse_toml(include_str!(
            "../../../protocol/tolerances/phase4-v1.toml"
        ))
        .expect("checked-in policy should parse");
        let actual = NativeMathProbeExecutor::execute(&request)
            .expect("checked-in request should execute")
            .into_vec();
        (
            request,
            policy,
            actual,
            supported_math_identity("11"),
            supported_math_identity("22"),
        )
    }

    fn assert_harness_reason(
        error: super::DifferentialError,
        expected_reason: Phase4HarnessFailureReason,
    ) {
        assert_eq!(error.category, "harness-failure");
        let evidence = error
            .maybe_phase4_evidence
            .expect("harness failure should carry typed evidence");
        let Phase4ComparisonEvidence::HarnessFailure(report) = evidence.as_ref() else {
            panic!("expected typed harness evidence");
        };
        assert_eq!(report.reason(), expected_reason);
        assert!(report.render_human().len() < 1024);
        let machine = serde_json::to_vec(report).expect("harness evidence should serialize");
        assert!(machine.len() < 4096);
        assert_eq!(report.signature_sha256().as_str().len(), 64);
    }

    fn policy_without_path(input: &str, path: &str) -> String {
        let mut output = String::new();
        for (index, section) in input.split("[[fields]]").enumerate() {
            if index == 0 {
                output.push_str(section);
                continue;
            }
            if section.contains(&format!("semantic_path = \"{path}\"")) {
                continue;
            }
            output.push_str("[[fields]]");
            output.push_str(section);
        }
        output
    }

    fn replace_in_policy_block(
        input: &str,
        path: &str,
        original: &str,
        replacement: &str,
    ) -> String {
        let mut output = String::new();
        for (index, section) in input.split("[[fields]]").enumerate() {
            if index == 0 {
                output.push_str(section);
                continue;
            }
            output.push_str("[[fields]]");
            if section.contains(&format!("semantic_path = \"{path}\"")) {
                output.push_str(&section.replacen(original, replacement, 1));
            } else {
                output.push_str(section);
            }
        }
        output
    }

    fn supported_math_identity(adapter_digest_byte: &str) -> BuildIdentity {
        let phase4 = Phase4BuildIdentityFields::new(
            "33".repeat(32),
            "AppleClang",
            "21.0.0",
            "arm64-apple-darwin",
            "baseline",
            "<none>",
            "<none>",
            "O0",
            "precise",
            "off",
            "ieee",
            "scalar baseline",
            "macos",
            "libSystem",
            "libSystem",
            "nearest_ties_even",
            true,
        );
        BuildIdentity::new(
            BuildIdentityFields::new(
                ORACLE_REVISION,
                "adapter-v1",
                adapter_digest_byte.repeat(32),
                "oracle-debug",
                "AppleClang",
                "21.0.0",
                "arm64-apple-darwin",
                "Debug",
                "reviewed",
                "none",
                "none",
            )
            .with_phase4(phase4),
        )
        .expect("supported fixture identity should validate")
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
