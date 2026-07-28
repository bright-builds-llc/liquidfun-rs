use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use liquidfun_differential::{
    CapturedRigidWorld, EmptyWorldAdapter, MinimizationBudget, MinimizationStatus,
    NativeCollisionProbeExecutor, NativeMathProbeExecutor, NativeRigidWorldExecutor,
    OracleExecutable, OraclePreset, Phase4ComparisonEvidence, Phase4DiscreteMismatchReport,
    Phase4HarnessFailureReason, Phase4HarnessFailureReport, Phase4MathMismatchReport,
    RigidComparisonOutcome, RigidEvaluation, RigidFailureSignature,
    RigidMinimizationArtifactRequest, RigidMinimizationResult, RigidScenarioTransform,
    compare_collision_probe_results, compare_phase8_rigid_world_results,
    execute_collision_probe_process, execute_math_probe_process, execute_rigid_world_process,
    float_values_match_with_policy, minimize_rigid_world_request,
    persist_rigid_minimization_artifact, validate_oracle_checkout_identity,
};
use liquidfun_test_protocol::{
    BuildEvidenceTier, BuildIdentity, CollisionProbeRequestRecord, CollisionProbeResult,
    DivergenceHorizon, EvidenceTier, HarnessLimits, MathProbeHorizon, MathProbeRequestRecord,
    MathProbeResult, Phase4PolicyProfile, Phase5PolicyProfile, Phase6PolicyProfile,
    Phase7PolicyProfile, Phase8PolicyProfile, RigidWorldRequestRecord,
    decode_collision_probe_request_jsonl, decode_math_probe_request_jsonl,
    decode_rigid_world_request_jsonl,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::upstream;

mod catalog;
mod command;
mod comparison;
mod execution;
mod minimization;
mod options;
mod oracle;
mod process;

#[allow(
    clippy::wildcard_imports,
    reason = "the facade deliberately re-exports its private split-module contract"
)]
use command::*;
#[allow(
    clippy::wildcard_imports,
    reason = "the facade deliberately re-exports its private split-module contract"
)]
use comparison::*;
#[allow(
    clippy::wildcard_imports,
    reason = "the facade deliberately re-exports its private split-module contract"
)]
use execution::*;
#[allow(
    clippy::wildcard_imports,
    reason = "the facade deliberately re-exports its private split-module contract"
)]
use minimization::*;
#[allow(
    clippy::wildcard_imports,
    reason = "the facade deliberately re-exports its private split-module contract"
)]
use options::*;
#[allow(
    clippy::wildcard_imports,
    reason = "the facade deliberately re-exports its private split-module contract"
)]
use oracle::*;
#[allow(
    clippy::wildcard_imports,
    reason = "the facade deliberately re-exports its private split-module contract"
)]
use process::*;

const USAGE: &str = r"Usage: cargo xtask differential <command> [arguments]

Commands:
  check-protocol
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
const PHASE7_POLICY: &str = "protocol/tolerances/phase7-v1.toml";
const PHASE8_POLICY: &str = "protocol/tolerances/phase8-v1.toml";
const NATIVE_SOURCE_MANIFEST: &str = "crates/liquidfun-differential/native-math-sources.txt";
const RIGID_MINIMIZATION_MAXIMUM_ATTEMPTS: usize = 128;
const RIGID_MINIMIZATION_DEADLINE: Duration = Duration::from_secs(30);
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
    maybe_exit_code: Option<u8>,
}

impl DifferentialError {
    fn new(category: &'static str, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
            maybe_phase4_evidence: None,
            maybe_exit_code: None,
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new("usage", format!("{}\n\n{USAGE}", message.into()))
    }

    fn process(message: impl Into<String>) -> Self {
        Self::new("process", message)
    }

    fn process_exit(message: impl Into<String>, exit_code: u8) -> Self {
        Self {
            category: "process",
            message: message.into(),
            maybe_phase4_evidence: None,
            maybe_exit_code: Some(exit_code),
        }
    }

    pub(crate) fn exit_code(&self) -> u8 {
        match self.category {
            "catalog-usage" => 64,
            "catalog-scenario" => 65,
            "catalog-settings" => 66,
            "catalog-script" => 67,
            _ => self.maybe_exit_code.unwrap_or(1),
        }
    }

    fn phase4_evidence(category: &'static str, evidence: Phase4ComparisonEvidence) -> Self {
        Self {
            category,
            message: evidence.render_human(),
            maybe_phase4_evidence: Some(Box::new(evidence)),
            maybe_exit_code: None,
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
    if args == ["check-protocol"] {
        return check_protocol(&repository_root()?);
    }
    if let Some(command_args) = args.strip_prefix(&["catalog".to_owned()]) {
        return catalog::run(command_args);
    }
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

pub(crate) fn run_catalog(args: &[String]) -> Result<(), DifferentialError> {
    catalog::run(args)
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
mod tests;
