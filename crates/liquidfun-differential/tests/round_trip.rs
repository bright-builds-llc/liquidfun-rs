//! End-to-end native/C++ comparison and CLI outcome tests.

use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use liquidfun_differential::{
    DifferentialRunOutcome, OracleExecutable, OraclePreset, SessionProfile, replay_exact, run_named,
};
use liquidfun_test_protocol::HarnessLimits;
use sha2::{Digest, Sha256};

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
    for preset in ["oracle-debug", "oracle-asan-ubsan"] {
        let oracle_output = root.join("target/reference").join(preset);
        fs::create_dir_all(&oracle_output).expect("fake output should be creatable");
        let executable = oracle_output.join(if cfg!(windows) {
            "liquidfun-reference.exe"
        } else {
            "liquidfun-reference"
        });
        fs::copy(env!("CARGO_BIN_EXE_liquidfun-fake-oracle"), executable)
            .expect("fake oracle should copy");
        fs::write(oracle_output.join("behavior.txt"), behavior).expect("behavior should write");
    }

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
    run_cli_with_root(root, behavior, arguments).0
}

fn run_cli_with_root(
    root: &Path,
    behavior: &str,
    arguments: &[&str],
) -> (std::process::Output, PathBuf) {
    let fake_root = fake_repository(behavior);
    assert!(root == repository_root() || root == fake_root);
    let output = Command::new(env!("CARGO_BIN_EXE_liquidfun-differential"))
        .current_dir(&fake_root)
        .args(arguments)
        .output()
        .expect("differential CLI should run");
    (output, fake_root)
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
fn real_oracle_rejects_oversized_stdin_before_waiting_for_a_newline() {
    // Arrange
    let root = repository_root();
    if OracleExecutable::resolve(&root, OraclePreset::Debug).is_err() {
        eprintln!(
            "SKIP real oracle integration prerequisite: run cargo xtask upstream configure/build --preset oracle-debug"
        );
        return;
    }
    let executable = root
        .join("target/reference/oracle-debug")
        .join(if cfg!(windows) {
            "liquidfun-reference.exe"
        } else {
            "liquidfun-reference"
        });
    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("real oracle should start");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout should be piped"));
    let mut handshake = String::new();
    stdout
        .read_line(&mut handshake)
        .expect("oracle handshake should be readable");
    let mut stdin = child.stdin.take().expect("stdin should be piped");
    let oversized = vec![b' '; HarnessLimits::phase2_default_v1().input_record_bytes() + 1];

    // Act
    let write_result = stdin.write_all(&oversized).and_then(|()| stdin.flush());
    let deadline = Instant::now() + Duration::from_secs(2);
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .expect("oracle status should be observable")
        {
            break status;
        }
        if Instant::now() >= deadline {
            drop(stdin);
            child.kill().expect("stalled oracle should be killed");
            child.wait().expect("killed oracle should be reaped");
            panic!("oracle waited for the oversized record remainder");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    drop(stdin);
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr should be piped")
        .read_to_string(&mut stderr)
        .expect("oracle stderr should be readable");

    // Assert
    assert!(write_result.is_ok());
    assert!(!status.success());
    assert!(stderr.contains("input record exceeds reviewed byte limit"));
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
fn cli_sanitizer_profile_reuses_one_process_and_proves_reset() {
    // Arrange
    let root = repository_root();
    let arguments = [
        "compare",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-asan-ubsan",
        "--session-profile",
        "sanitizer",
    ];

    // Act
    let output = run_cli(&root, "valid", &arguments);

    // Assert
    assert!(output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("sanitizer report should be JSON");
    let requests = report["requests"]
        .as_array()
        .expect("sanitizer report should contain requests");
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0]["cpp_reset_epoch"], 1);
    assert_eq!(requests[1]["cpp_reset_epoch"], 2);
    assert_eq!(requests[0]["rust_reset_epoch"], 1);
    assert_eq!(requests[1]["rust_reset_epoch"], 2);
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
fn maximum_length_request_id_runs_in_reuse_and_sanitizer_profiles() {
    // Arrange
    let root = fake_repository("valid");
    let request_id = "r".repeat(128);
    let request =
        fs::read_to_string(root.join("protocol/fixtures/accepted/empty-world-request.jsonl"))
            .expect("exact request should be readable")
            .replace("empty-world-request", &request_id);
    let profiles = [
        (OraclePreset::Debug, SessionProfile::Reuse),
        (OraclePreset::AsanUbsan, SessionProfile::Sanitizer),
    ];

    // Act and Assert
    for (preset, profile) in profiles {
        let outcome = replay_exact(&root, request.as_bytes(), preset, profile, REVISION)
            .expect("maximum-length request identity should remain valid");
        let DifferentialRunOutcome::Match(run) = outcome else {
            panic!("bounded request identities should match");
        };
        assert_eq!(run.requests().len(), 2);
        assert_eq!(run.requests()[0].request_id(), request_id);
        assert!(run.requests()[1].request_id().len() <= 128);
    }
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

#[test]
fn cli_harness_failure_persists_bounded_hash_indexed_evidence() {
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
    let (output, fake_root) = run_cli_with_root(&root, "malformed", &arguments);
    let failure_root = fake_root.join("target/differential/failures");
    let directories = fs::read_dir(&failure_root)
        .expect("failure evidence root should exist")
        .collect::<Result<Vec<_>, _>>()
        .expect("failure evidence entries should be readable");

    // Assert
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(directories.len(), 1);
    let directory = directories[0].path();
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(directory.join("manifest.json")).expect("manifest should be readable"),
    )
    .expect("manifest should be valid JSON");
    for name in [
        "request.jsonl",
        "report.json",
        "identity.json",
        "stderr.txt",
    ] {
        let bytes = fs::read(directory.join(name)).expect("evidence file should be readable");
        assert!(bytes.len() <= HarnessLimits::phase2_default_v1().input_record_bytes());
        assert_eq!(
            manifest["files"][name]["sha256"],
            format!("{:x}", Sha256::digest(&bytes))
        );
        assert_eq!(manifest["files"][name]["bytes"], bytes.len());
    }
    assert_eq!(manifest["result_kind"], "harness_failure");
    let identity: serde_json::Value = serde_json::from_slice(
        &fs::read(directory.join("identity.json")).expect("identity should be readable"),
    )
    .expect("identity should be valid JSON");
    assert_eq!(identity["oracle_revision"], REVISION);
    assert_eq!(identity["preset"], "oracle-debug");
    assert_eq!(identity["session_profile"], "one-shot");
    assert_eq!(
        fs::read(directory.join("request.jsonl")).expect("request should be readable"),
        fs::read(fake_root.join("protocol/fixtures/accepted/empty-world-request.jsonl"))
            .expect("source request should be readable")
    );
}

#[test]
fn cli_minimize_persists_smaller_same_signature_scenario() {
    // Arrange
    let root = repository_root();
    let compare_arguments = [
        "compare",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
    ];
    let minimize_arguments = [
        "minimize",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
    ];

    // Act
    let comparison = run_cli(&root, "value_mismatch", &compare_arguments);
    let (minimized, fake_root) = run_cli_with_root(&root, "value_mismatch", &minimize_arguments);

    // Assert
    assert_eq!(comparison.status.code(), Some(2));
    assert!(minimized.status.success());
    let comparison_report: serde_json::Value =
        serde_json::from_slice(&comparison.stdout).expect("comparison report should be JSON");
    let minimization_report: serde_json::Value =
        serde_json::from_slice(&minimized.stdout).expect("minimization report should be JSON");
    assert_eq!(minimization_report["result_kind"], "minimization");
    assert_eq!(minimization_report["status"], "complete");
    assert_eq!(
        minimization_report["target_signature"],
        comparison_report["mismatch"]["signature"]
    );
    let minimized_commands = minimization_report["minimized_commands"]
        .as_u64()
        .expect("minimized command count should be an integer");
    let original_commands = minimization_report["original_commands"]
        .as_u64()
        .expect("original command count should be an integer");
    let minimized_checkpoints = minimization_report["minimized_checkpoints"]
        .as_u64()
        .expect("minimized checkpoint count should be an integer");
    let original_checkpoints = minimization_report["original_checkpoints"]
        .as_u64()
        .expect("original checkpoint count should be an integer");
    assert!(minimized_commands < original_commands || minimized_checkpoints < original_checkpoints);
    let artifact_root = fake_root.join("target/differential/minimized");
    let directories = fs::read_dir(artifact_root)
        .expect("minimized artifact root should exist")
        .collect::<Result<Vec<_>, _>>()
        .expect("minimized artifact entries should be readable");
    assert_eq!(directories.len(), 1);
    let minimized_scenario = fs::read(directories[0].path().join("scenario.json"))
        .expect("minimized scenario should be readable");
    let request: serde_json::Value = serde_json::from_slice(
        &fs::read(fake_root.join("protocol/fixtures/accepted/empty-world-request.jsonl"))
            .expect("source request should be readable"),
    )
    .expect("source request should be JSON");
    let original_scenario =
        serde_json::to_vec(&request["scenario"]).expect("source scenario should serialize");
    assert!(minimized_scenario.len() < original_scenario.len());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&minimized_scenario)
            .expect("minimized scenario should be JSON"),
        minimization_report["scenario"]
    );
}
