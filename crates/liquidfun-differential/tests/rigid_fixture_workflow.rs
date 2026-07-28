//! Real-binary lifecycle coverage for canonical rigid-world evidence.

#[path = "rigid_fixture_workflow/provenance.rs"]
mod provenance;

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use liquidfun_differential::{
    ArtifactKind, MinimizationBudget, MinimizationStatus, NativeRigidWorldExecutor, OraclePreset,
    RigidComparisonOutcome, RigidEvaluation, RigidFailureSignature, RigidMinimizationResult,
    compare_phase8_rigid_world_results, minimize_rigid_world_request, stage_rigid_candidate,
};
use liquidfun_test_protocol::{
    HarnessLimits, Phase6PolicyProfile, Phase7PolicyProfile, Phase8PolicyProfile,
    RigidWorldRequestRecord, RigidWorldWitnessFamily, decode_rigid_world_request_jsonl,
    decode_rigid_world_result_jsonl,
};

static NEXT_REPOSITORY: AtomicU64 = AtomicU64::new(0);

#[test]
fn checked_in_request_locks_every_phase8_family_and_policy() {
    // Arrange
    let request_bytes =
        include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
    let policy = Phase8PolicyProfile::parse_toml(include_str!(
        "../../../protocol/tolerances/phase8-v1.toml"
    ))
    .expect("checked-in Phase 8 policy should parse");

    // Act
    let request =
        decode_rigid_world_request_jsonl(request_bytes, &HarnessLimits::phase2_default_v1())
            .expect("checked-in rigid request should decode");
    let families = request
        .scenario()
        .timelines()
        .iter()
        .map(liquidfun_test_protocol::RigidWorldTimeline::witness_family)
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(request.tolerance_profile_sha256(), policy.profile_sha256());
    assert_eq!(families, RigidWorldWitnessFamily::ALL);
}

include!("rigid_fixture_workflow/repository.rs");
include!("rigid_fixture_workflow/transactions.rs");
include!("rigid_fixture_workflow/minimization.rs");
include!("rigid_fixture_workflow/git_support.rs");
