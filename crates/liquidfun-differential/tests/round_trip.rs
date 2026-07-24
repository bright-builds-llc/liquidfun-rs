//! End-to-end native/C++ comparison and CLI outcome tests.

#[path = "support/coverage_observation.rs"]
mod coverage_observation;

use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use liquidfun_differential::{
    DifferentialRunOutcome, EmptyWorldAdapter, OracleExecutable, OraclePreset, SessionProfile,
    replay_exact, run_named,
};
use liquidfun_test_protocol::{
    HarnessLimits, MathProbeResult, RecordLimit, decode_math_probe_request_jsonl,
    decode_scenario_request_jsonl, encode_jsonl,
};
use sha2::{Digest, Sha256};

const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
static TEST_DIRECTORY_ID: AtomicU64 = AtomicU64::new(1);

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn real_oracle_path(preset: OraclePreset) -> Option<PathBuf> {
    let root = repository_root();
    let directory = match preset {
        OraclePreset::Debug => "oracle-debug",
        OraclePreset::Release => "oracle-release",
        OraclePreset::AsanUbsan => "oracle-asan-ubsan",
    };
    OracleExecutable::resolve(&root, preset).ok().map(|_| {
        root.join("target/reference")
            .join(directory)
            .join(if cfg!(windows) {
                "liquidfun-reference.exe"
            } else {
                "liquidfun-reference"
            })
    })
}

fn run_cpp_math_probe_twice(
    preset: OraclePreset,
) -> Option<(Vec<MathProbeResult>, Vec<serde_json::Value>)> {
    let maybe_executable = real_oracle_path(preset);
    let Some(executable) = maybe_executable else {
        eprintln!(
            "SKIP real oracle integration prerequisite: run cargo xtask upstream configure/build --preset oracle-debug"
        );
        return None;
    };
    let request_bytes =
        fs::read(repository_root().join("protocol/fixtures/accepted/math-probe-request.jsonl"))
            .expect("math probe request should be readable");
    let request =
        decode_math_probe_request_jsonl(&request_bytes, &HarnessLimits::phase2_default_v1())
            .expect("math probe request should decode");
    let expected = EmptyWorldAdapter::execute_math_probe(&request)
        .expect("native math probes should execute")
        .into_vec();
    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("real oracle should start");
    let mut stdin = child.stdin.take().expect("stdin should be piped");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout should be piped"));
    let mut handshake = String::new();
    stdout
        .read_line(&mut handshake)
        .expect("handshake should be readable");
    let mut ends = Vec::new();
    for _ in 0..2 {
        stdin
            .write_all(&request_bytes)
            .and_then(|()| stdin.flush())
            .expect("math request should write");
        let mut actual = Vec::with_capacity(expected.len());
        for _ in 0..expected.len() {
            let mut line = String::new();
            stdout
                .read_line(&mut line)
                .expect("result should be readable");
            actual.push(
                serde_json::from_str::<MathProbeResult>(&line)
                    .expect("C++ result should decode as the Rust contract"),
            );
        }
        for (actual_result, expected_result) in actual.iter().zip(&expected) {
            assert_eq!(actual_result.case_id(), expected_result.case_id());
            assert_eq!(actual_result.operation(), expected_result.operation());
            assert_eq!(actual_result.policy_path(), expected_result.policy_path());
            assert_eq!(actual_result.horizon(), expected_result.horizon());
            assert_eq!(
                actual_result
                    .values()
                    .iter()
                    .map(|value| value.field())
                    .collect::<Vec<_>>(),
                expected_result
                    .values()
                    .iter()
                    .map(|value| value.field())
                    .collect::<Vec<_>>()
            );
            assert_eq!(actual_result.discrete(), expected_result.discrete());
        }
        for witness in [
            "cancellation",
            "halfway-rounding",
            "overflow",
            "underflow",
            "fma-witness",
        ] {
            let actual_witness = actual
                .iter()
                .find(|result| result.case_id() == witness)
                .expect("C++ witness should exist");
            let expected_witness = expected
                .iter()
                .find(|result| result.case_id() == witness)
                .expect("Rust witness should exist");
            assert_eq!(actual_witness.values(), expected_witness.values());
        }
        let mut end = String::new();
        stdout
            .read_line(&mut end)
            .expect("end record should be readable");
        ends.push(serde_json::from_str(&end).expect("end record should be JSON"));
    }
    drop(stdin);
    let output = child.wait_with_output().expect("oracle should be reaped");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    Some((expected, ends))
}

#[test]
fn cpp_protocol_self_test_executes_phase8_after_plan_08_21() {
    // Arrange
    let root = repository_root();
    if real_oracle_path(OraclePreset::Debug).is_none() {
        eprintln!(
            "SKIP real oracle integration prerequisite: run cargo xtask upstream configure/build --preset oracle-debug"
        );
        return;
    }

    // Act
    let output = Command::new("ctest")
        .args([
            "--test-dir",
            root.join("target/reference/oracle-debug")
                .to_str()
                .expect("repository path should be UTF-8"),
            "--output-on-failure",
            "-R",
            "liquidfun-reference-protocol",
        ])
        .output()
        .expect("C++ protocol self-test should run");

    // Assert
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn cpp_math_probe_matches_operation_contract() {
    // Arrange / Act / Assert
    for preset in [OraclePreset::Debug, OraclePreset::Release] {
        let Some((results, ends)) = run_cpp_math_probe_twice(preset) else {
            return;
        };
        assert_eq!(results.len(), 39);
        assert_eq!(ends[0]["result_count"], 39);
        assert_eq!(ends[0]["reset_epoch"], 1);
        assert_eq!(ends[1]["reset_epoch"], 2);
        assert_eq!(ends[0]["reset_verified"], true);
        assert_eq!(ends[1]["reset_verified"], true);
    }
    coverage_observation::observe(&[
        "public-api.liquidfun-box2d-box2d-common-b2math-h",
        "subsystem.common-math-and-settings",
    ])
    .expect("successful math comparison should emit its covered leaves");
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
fn real_oracle_rejects_invalid_query_child_without_poisoning_process() {
    // Arrange
    let Some(executable) = real_oracle_path(OraclePreset::Debug) else {
        eprintln!(
            "SKIP real oracle integration prerequisite: run cargo xtask upstream configure/build --preset oracle-debug"
        );
        return;
    };
    let request_bytes =
        fs::read(repository_root().join("protocol/fixtures/accepted/rigid-world-request.jsonl"))
            .expect("rigid-world request should be readable");
    let mut request: serde_json::Value =
        serde_json::from_slice(&request_bytes).expect("rigid-world request should be JSON");
    let query_timeline = request["scenario"]["timelines"]
        .as_array_mut()
        .expect("timelines should be an array")
        .iter_mut()
        .find(|timeline| timeline["witness_family"] == "world_query_and_ray_cast")
        .expect("query timeline should exist");
    let terminate_query = query_timeline["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .iter_mut()
        .find(|action| action["action_id"] == "query-07")
        .expect("terminating query should exist");
    terminate_query["action"]["directive_rules"][0]["target"]["child_index"] = serde_json::json!(1);
    let request = encode_jsonl(
        &request,
        &HarnessLimits::phase2_default_v1(),
        RecordLimit::Input,
    )
    .expect("mutated request should encode");
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

    // Act
    stdin
        .write_all(&request)
        .and_then(|()| stdin.flush())
        .expect("invalid query request should write");
    drop(stdin);
    let mut result_records = String::new();
    stdout
        .read_to_string(&mut result_records)
        .expect("oracle result stream should be readable");
    let output = child.wait_with_output().expect("oracle should be reaped");

    // Assert
    assert!(
        output.status.success(),
        "a rejected request must not poison the reusable oracle process"
    );
    assert!(result_records.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("query directive references invalid fixture child")
    );
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
fn cli_reuse_and_sanitizer_bundles_bind_the_second_request_and_session_identity() {
    // Arrange
    let root = repository_root();
    let original_request_id = "empty-world-request";
    let expected_request_id = format!("reuse-{:x}", Sha256::digest(original_request_id.as_bytes()));
    let profiles = [
        ("oracle-debug", "reuse"),
        ("oracle-asan-ubsan", "sanitizer"),
    ];
    let cases = [
        ("second_malformed", 3, "harness_failure"),
        ("second_value_mismatch", 2, "physics_mismatch"),
    ];

    // Act and Assert
    for (preset, profile) in profiles {
        let arguments = [
            "compare",
            "--scenario",
            "empty-world",
            "--preset",
            preset,
            "--session-profile",
            profile,
        ];
        for (behavior, exit_code, result_kind) in cases {
            let (output, fake_root) = run_cli_with_root(&root, behavior, &arguments);
            assert_eq!(
                output.status.code(),
                Some(exit_code),
                "{profile}/{behavior}"
            );
            let directory = only_failure_directory(&fake_root);
            let manifest: serde_json::Value = serde_json::from_slice(
                &fs::read(directory.join("manifest.json")).expect("manifest should be readable"),
            )
            .expect("manifest should be JSON");
            let request_bytes =
                fs::read(directory.join("request.jsonl")).expect("request should be readable");
            let request =
                decode_scenario_request_jsonl(&request_bytes, &HarnessLimits::phase2_default_v1())
                    .expect("persisted request should validate");
            let canonical = encode_jsonl(
                &request,
                &HarnessLimits::phase2_default_v1(),
                RecordLimit::Input,
            )
            .expect("persisted request should re-encode");
            let report: serde_json::Value = serde_json::from_slice(
                &fs::read(directory.join("report.json")).expect("report should be readable"),
            )
            .expect("report should be JSON");
            let identity: serde_json::Value = serde_json::from_slice(
                &fs::read(directory.join("identity.json")).expect("identity should be readable"),
            )
            .expect("identity should be JSON");
            let session_identity = identity["session_identity_sha256"]
                .as_str()
                .expect("validated session identity should be present");

            assert_eq!(request_bytes, canonical, "{profile}/{behavior}");
            assert_eq!(
                request.request_id().as_str(),
                expected_request_id,
                "{profile}/{behavior}"
            );
            assert_eq!(
                manifest["request_id"], expected_request_id,
                "{profile}/{behavior}"
            );
            assert_eq!(manifest["result_kind"], result_kind, "{profile}/{behavior}");
            assert_eq!(
                report["request_id"], expected_request_id,
                "{profile}/{behavior}"
            );
            assert_eq!(report["result_kind"], result_kind, "{profile}/{behavior}");
            assert_eq!(
                report["session_identity_sha256"], session_identity,
                "{profile}/{behavior}"
            );
            assert_eq!(session_identity.len(), 64, "{profile}/{behavior}");
            if result_kind == "physics_mismatch" {
                assert_eq!(
                    report["mismatch"]["request_id"], expected_request_id,
                    "{profile}/{behavior}"
                );
            }
        }
    }
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

fn only_failure_directory(repository_root: &Path) -> PathBuf {
    let directories = fs::read_dir(repository_root.join("target/differential/failures"))
        .expect("failure evidence root should exist")
        .collect::<Result<Vec<_>, _>>()
        .expect("failure evidence entries should be readable");
    assert_eq!(directories.len(), 1);
    directories[0].path()
}
