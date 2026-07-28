//! Native Phase 6 rigid-world adapter integration tests.

#[path = "support/phase7_comparator.rs"]
mod phase7_comparator;
mod support;

use std::fs;
use std::process::Command;
use std::time::Duration;

use liquidfun_differential::{
    FailureBundleRequest, MinimizationBudget, NativeRigidWorldExecutor, OracleExecutable,
    OraclePreset, RigidComparisonFailure, RigidComparisonOutcome, RigidEngineSide, RigidEvaluation,
    RigidMismatchKind, compare_rigid_world_results, execute_rigid_world_process,
    minimize_rigid_world_request, persist_failure_bundle, validate_native_rigid_world_result,
};
use liquidfun_test_protocol::{
    HarnessLimits, Phase6PolicyProfile, RecordLimit, RigidWorldErrorKind, RigidWorldObservation,
    RigidWorldRequestRecord, RigidWorldResultRecord, RigidWorldWitnessFamily,
    decode_rigid_world_request_jsonl, decode_rigid_world_result_jsonl, encode_jsonl,
    rigid_world_checkpoint_live_identities, validate_rigid_world_result_against_request,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
const POLICY: &str = include_str!("../../../protocol/tolerances/phase6-v1.toml");
const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
include!("rigid_world/setup.rs");
include!("rigid_world/native.rs");
include!("rigid_world/comparison_validation.rs");
include!("rigid_world/comparison_outcomes.rs");
