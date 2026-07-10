//! End-to-end native/C++ comparison and CLI outcome tests.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

use liquidfun_differential::{
    DifferentialRunOutcome, OracleExecutable, OraclePreset, SessionProfile, replay_exact, run_named,
};

const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
static TEST_DIRECTORY_ID: AtomicU64 = AtomicU64::new(1);

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fake_repository(behavior: &str) -> PathBuf {
    let id = TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
    let root = repository_root()
        .join("target/round-trip-tests")
        .join(format!("{}-{id}", std::process::id()));
    let oracle_output = root.join("target/reference/oracle-debug");
    fs::create_dir_all(&oracle_output).expect("fake output should be creatable");
    let executable = oracle_output.join(if cfg!(windows) {
        "liquidfun-reference.exe"
    } else {
        "liquidfun-reference"
    });
    fs::copy(env!("CARGO_BIN_EXE_liquidfun-fake-oracle"), executable)
        .expect("fake oracle should copy");
    fs::write(oracle_output.join("behavior.txt"), behavior).expect("behavior should write");

    let fixture_output = root.join("protocol/fixtures/accepted");
    fs::create_dir_all(&fixture_output).expect("fixture output should be creatable");
    fs::copy(
        repository_root().join("protocol/fixtures/accepted/empty-world-request.jsonl"),
        fixture_output.join("empty-world-request.jsonl"),
    )
    .expect("request fixture should copy");
    root
}

fn run_cli(root: &Path, behavior: &str, arguments: &[&str]) -> std::process::Output {
    let fake_root = fake_repository(behavior);
    assert!(root == repository_root() || root == fake_root);
    Command::new(env!("CARGO_BIN_EXE_liquidfun-differential"))
        .current_dir(&fake_root)
        .args(arguments)
        .output()
        .expect("differential CLI should run")
}

#[test]
fn real_oracle_one_shot_and_two_request_reuse_match_or_skip_explicitly() {
    // Arrange
    let root = repository_root();
    if OracleExecutable::resolve(&root, OraclePreset::Debug).is_err() {
        eprintln!(
            "SKIP real oracle integration prerequisite: run cargo xtask upstream configure/build --preset oracle-debug"
        );
        return;
    }

    // Act
    let one_shot = run_named(
        &root,
        "empty-world",
        OraclePreset::Debug,
        SessionProfile::OneShot,
        REVISION,
    )
    .expect("real one-shot orchestration should run");
    let reuse = run_named(
        &root,
        "empty-world",
        OraclePreset::Debug,
        SessionProfile::Reuse,
        REVISION,
    )
    .expect("real reuse orchestration should run");

    // Assert
    assert!(matches!(one_shot, DifferentialRunOutcome::Match(_)));
    let DifferentialRunOutcome::Match(reused) = reuse else {
        panic!("real reuse should match");
    };
    assert_eq!(reused.requests().len(), 2);
    assert_eq!(reused.requests()[0].cpp_reset_epoch(), 1);
    assert_eq!(reused.requests()[1].cpp_reset_epoch(), 2);
}

#[test]
fn cli_compare_and_replay_emit_deterministic_match_reports() {
    // Arrange
    let root = repository_root();
    let compare_arguments = [
        "compare",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "reuse",
    ];
    let replay_arguments = [
        "replay",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
    ];

    // Act
    let compare = run_cli(&root, "valid", &compare_arguments);
    let replay = run_cli(&root, "valid", &replay_arguments);

    // Assert
    assert!(compare.status.success());
    assert!(replay.status.success());
    let compare_json: serde_json::Value =
        serde_json::from_slice(&compare.stdout).expect("compare report should be JSON");
    let replay_json: serde_json::Value =
        serde_json::from_slice(&replay.stdout).expect("replay report should be JSON");
    assert_eq!(compare_json["result_kind"], "match");
    assert_eq!(compare_json["requests"].as_array().map(Vec::len), Some(2));
    assert_eq!(replay_json["result_kind"], "match");
}

#[test]
fn exact_request_replay_preserves_serialized_source_metadata() {
    // Arrange
    let root = fake_repository("valid");
    let bytes = fs::read(root.join("protocol/fixtures/accepted/empty-world-request.jsonl"))
        .expect("exact request should be readable");

    // Act
    let outcome = replay_exact(
        &root,
        &bytes,
        OraclePreset::Debug,
        SessionProfile::OneShot,
        REVISION,
    )
    .expect("exact validated request should replay");

    // Assert
    let DifferentialRunOutcome::Match(run) = outcome else {
        panic!("exact replay should match");
    };
    assert_eq!(run.requests()[0].request_id(), "empty-world-request");
}

#[test]
fn cli_distinguishes_harness_failure_from_physics_mismatch_exit_codes() {
    // Arrange
    let root = repository_root();
    let arguments = [
        "compare",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
    ];

    // Act
    let harness = run_cli(&root, "malformed", &arguments);
    let mismatch = run_cli(&root, "value_mismatch", &arguments);

    // Assert
    assert_eq!(harness.status.code(), Some(3));
    assert_eq!(mismatch.status.code(), Some(2));
    let harness_json: serde_json::Value =
        serde_json::from_slice(&harness.stdout).expect("harness report should be JSON");
    let mismatch_json: serde_json::Value =
        serde_json::from_slice(&mismatch.stdout).expect("mismatch report should be JSON");
    assert_eq!(harness_json["result_kind"], "harness_failure");
    assert_eq!(harness_json["failure_kind"], "malformed_record");
    assert_eq!(mismatch_json["result_kind"], "physics_mismatch");
}
