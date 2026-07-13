//! Command-level coverage for safe differential contributor entrypoints.

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

const REPOSITORY: &str = "https://github.com/google/liquidfun.git";
const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";

static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
static FAKE_TOOLS: OnceLock<Result<FakeTools, String>> = OnceLock::new();

type TestResult = Result<(), Box<dyn Error>>;

#[derive(Debug)]
struct FakeTools {
    git: PathBuf,
    ninja: PathBuf,
    cxx: PathBuf,
    differential: PathBuf,
}

#[derive(Debug)]
struct RepositoryFixture {
    root: PathBuf,
    differential_marker: PathBuf,
}

impl RepositoryFixture {
    fn new() -> io::Result<Self> {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = workspace_root().join(format!(
            "target/xtask-differential-fixtures/{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("reference"))?;
        fs::create_dir_all(root.join("third_party/liquidfun"))?;
        fs::write(
            root.join("reference/upstream-lock.toml"),
            format!(
                "schema_version = 1\nrepository = \"{REPOSITORY}\"\nrevision = \"{REVISION}\"\nsubmodule_path = \"third_party/liquidfun\"\n"
            ),
        )?;
        fs::write(
            root.join(".gitmodules"),
            format!(
                "[submodule \"third_party/liquidfun\"]\n\tpath = third_party/liquidfun\n\turl = {REPOSITORY}\n"
            ),
        )?;

        let differential_marker = root.join("differential-arguments.txt");
        Ok(Self {
            root,
            differential_marker,
        })
    }

    fn command(&self) -> io::Result<Command> {
        let tools = fake_tools()?;
        let mut command = Command::new(env!("CARGO_BIN_EXE_xtask"));
        command
            .current_dir(&self.root)
            .env("LIQUIDFUN_XTASK_GIT", &tools.git)
            .env("LIQUIDFUN_XTASK_NINJA", &tools.ninja)
            .env("LIQUIDFUN_XTASK_CXX", &tools.cxx)
            .env("LIQUIDFUN_XTASK_DIFFERENTIAL", &tools.differential)
            .env("LIQUIDFUN_TEST_REVISION", REVISION)
            .env("LIQUIDFUN_TEST_REMOTE_URL", REPOSITORY)
            .env(
                "LIQUIDFUN_TEST_DIFFERENTIAL_MARKER",
                &self.differential_marker,
            );
        Ok(command)
    }

    fn differential_arguments(&self) -> io::Result<Vec<String>> {
        Ok(fs::read_to_string(&self.differential_marker)?
            .lines()
            .map(str::to_owned)
            .collect())
    }

    fn cleanup(self) -> io::Result<()> {
        fs::remove_dir_all(self.root)
    }
}

#[test]
fn compare_passes_only_canonical_structured_arguments() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "differential",
        "compare",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fixture.differential_arguments()?,
        [
            "compare",
            "--scenario",
            "empty-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
        ]
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn replay_and_minimize_pass_only_named_scenario_arguments() -> TestResult {
    // Arrange
    for action in ["replay", "minimize"] {
        let fixture = RepositoryFixture::new()?;
        let mut command = fixture.command()?;
        command.args([
            "differential",
            action,
            "--scenario",
            "empty-world",
            "--preset",
            "oracle-release",
            "--session-profile",
            "reuse",
        ]);

        // Act
        let output = command.output()?;

        // Assert
        assert!(output.status.success(), "{}", stderr(&output));
        assert_eq!(
            fixture.differential_arguments()?,
            [
                action,
                "--scenario",
                "empty-world",
                "--preset",
                "oracle-release",
                "--session-profile",
                "reuse",
            ]
        );
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn math_probe_compare_and_replay_pass_only_reviewed_arguments() -> TestResult {
    // Arrange
    for action in ["compare", "replay"] {
        let fixture = RepositoryFixture::new()?;
        let mut command = fixture.command()?;
        let arguments = [
            "differential",
            action,
            "--scenario",
            "math-probes",
            "--preset",
            "oracle-release",
            "--session-profile",
            "one-shot",
        ];
        command.args(arguments);

        // Act
        let output = command.output()?;

        // Assert
        assert!(output.status.success(), "{}", stderr(&output));
        assert_eq!(fixture.differential_arguments()?, &arguments[1..]);
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn collision_compare_replay_and_determinism_pass_only_reviewed_arguments() -> TestResult {
    // Arrange
    for action in ["compare", "replay"] {
        let fixture = RepositoryFixture::new()?;
        let mut command = fixture.command()?;
        let arguments = [
            "differential",
            action,
            "--scenario",
            "collision-probes",
            "--preset",
            "oracle-release",
            "--session-profile",
            "one-shot",
        ];
        command.args(arguments);

        // Act
        let output = command.output()?;

        // Assert
        assert!(output.status.success(), "{}", stderr(&output));
        assert_eq!(fixture.differential_arguments()?, &arguments[1..]);
        fixture.cleanup()?;
    }

    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "differential",
        "verify-determinism",
        "--scenario",
        "collision-probes",
        "--preset",
        "oracle-debug",
        "--runs",
        "2",
    ]);
    let output = command.output()?;
    assert!(output.status.success(), "{}", stderr(&output));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn rigid_compare_replay_and_minimize_pass_only_reviewed_arguments() -> TestResult {
    // Arrange
    for action in ["compare", "replay", "minimize"] {
        let fixture = RepositoryFixture::new()?;
        let mut command = fixture.command()?;
        let arguments = [
            "differential",
            action,
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
        ];
        command.args(arguments);

        // Act
        let output = command.output()?;

        // Assert
        assert!(output.status.success(), "{}", stderr(&output));
        assert_eq!(fixture.differential_arguments()?, &arguments[1..]);
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn rigid_determinism_accepts_exactly_two_debug_runs() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    let arguments = [
        "differential",
        "verify-determinism",
        "--scenario",
        "rigid-world",
        "--preset",
        "oracle-debug",
        "--runs",
        "2",
    ];
    command.args(arguments);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(fixture.differential_arguments()?, &arguments[1..]);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn rigid_fixture_stage_passes_only_fixed_lifecycle_metadata() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    let arguments = [
        "differential",
        "fixture",
        "stage",
        "--scenario",
        "rigid-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
        "--artifact-kind",
        "reviewed-trace",
        "--artifact-id",
        "rigid-trace-1",
    ];
    command.args(arguments);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(fixture.differential_arguments()?, &arguments[1..]);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn rigid_fixture_real_binary_accepts_d1_and_rejects_d2_before_effects() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    prepare_real_rigid_repository(&fixture.root, "rigid_d1")?;
    let real_differential = workspace_root()
        .join("target/debug")
        .join(executable_name("liquidfun-differential"));
    let arguments = [
        "differential",
        "fixture",
        "stage",
        "--scenario",
        "rigid-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
        "--artifact-kind",
        "reviewed-trace",
        "--artifact-id",
        "xtask-rigid-d1",
    ];
    let mut accepted = fixture.command()?;
    accepted
        .env("LIQUIDFUN_XTASK_DIFFERENTIAL", &real_differential)
        .args(arguments);

    // Act
    let accepted_output = accepted.output()?;
    fs::write(
        fixture
            .root
            .join("target/reference/oracle-debug/behavior.txt"),
        "rigid_d2",
    )?;
    let mut rejected = fixture.command()?;
    rejected
        .env("LIQUIDFUN_XTASK_DIFFERENTIAL", &real_differential)
        .args([
            "differential",
            "fixture",
            "stage",
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
            "--artifact-kind",
            "reviewed-trace",
            "--artifact-id",
            "xtask-rigid-d2",
        ]);
    let rejected_output = rejected.output()?;

    // Assert
    assert!(
        accepted_output.status.success(),
        "{}",
        stderr(&accepted_output)
    );
    assert!(
        fixture
            .root
            .join("target/differential/staging/xtask-rigid-d1/candidate.toml")
            .is_file()
    );
    assert!(!rejected_output.status.success());
    assert!(stderr(&rejected_output).contains("requires D1 canonical authority"));
    assert!(
        !fixture
            .root
            .join("target/differential/staging/xtask-rigid-d2")
            .exists()
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn rigid_fixture_stale_identity_real_binary_rejects_before_effects() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    prepare_real_rigid_repository(&fixture.root, "rigid_d1_stale_adapter")?;
    let manifest_path = fixture.root.join("reference/artifacts/manifest.toml");
    let manifest_before = fs::read(&manifest_path)?;
    let real_differential = workspace_root()
        .join("target/debug")
        .join(executable_name("liquidfun-differential"));
    let mut command = fixture.command()?;
    command
        .env("LIQUIDFUN_XTASK_DIFFERENTIAL", &real_differential)
        .args([
            "differential",
            "fixture",
            "stage",
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
            "--artifact-kind",
            "reviewed-trace",
            "--artifact-id",
            "xtask-rigid-stale-adapter",
        ]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(!output.status.success());
    assert!(stderr(&output).contains("adapter digest differs from current checkout inputs"));
    assert!(!fixture.root.join("target/differential/staging").exists());
    assert_eq!(fs::read(manifest_path)?, manifest_before);
    assert!(
        !fixture
            .root
            .join("reference/artifacts/traces/phase-07-rigid-world-v1.jsonl")
            .exists()
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn rigid_commands_reject_unreviewed_shapes_before_effects() -> TestResult {
    // Arrange
    let scenario_file_option = ["--scenario", "-file"].concat();
    let cases = [
        vec![
            "differential",
            "compare",
            "--scenario",
            "../rigid-world.jsonl",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
        ],
        vec![
            "differential",
            "compare",
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "reuse",
        ],
        vec![
            "differential",
            "verify-determinism",
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--runs",
            "3",
        ],
        vec![
            "differential",
            "compare",
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
            scenario_file_option.as_str(),
            "../outside.jsonl",
        ],
    ];

    for arguments in cases {
        let fixture = RepositoryFixture::new()?;
        let mut command = fixture.command()?;
        command.args(arguments);

        // Act
        let output = command.output()?;

        // Assert
        assert_failure_category(&output, "differential/usage");
        assert!(!fixture.differential_marker.exists());
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn rigid_child_failure_status_is_propagated() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command
        .args([
            "differential",
            "compare",
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
        ])
        .env("LIQUIDFUN_TEST_DIFFERENTIAL_FAIL", "1");

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "differential/process");
    assert!(stderr(&output).contains("status 42"));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn local_rigid_identity_cannot_authorize_canonical_promotion() {
    // Arrange
    let identity = liquidfun_differential::EmptyWorldAdapter::new(REVISION)
        .expect("local native identity should validate");

    // Act
    let result = liquidfun_differential::validate_rigid_promotion_authority(
        identity.build_identity(),
        liquidfun_differential::ArtifactKind::ReviewedTrace,
    );

    // Assert
    assert!(result.is_err());
}

#[test]
fn rigid_ci_commands_stay_in_the_native_reference_workflow() {
    // Arrange
    let native_reference_workflow = include_str!("../../../.github/workflows/oracle.yml");
    let cargo_workflow = include_str!("../../../.github/workflows/ci.yml");
    let required = [
        "cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot",
        "cargo xtask differential compare --scenario rigid-world --preset oracle-release --session-profile one-shot",
        "cargo xtask differential replay --scenario rigid-world --preset oracle-debug --session-profile one-shot",
        "cargo xtask differential verify-determinism --scenario rigid-world --preset oracle-debug --runs 2",
    ];

    // Act
    let all_reference_commands_present = required
        .iter()
        .all(|command| native_reference_workflow.contains(command));
    let cargo_only_isolated = ["submodules: recursive", "cmake", "oracle", "rigid-world"]
        .iter()
        .all(|forbidden| !cargo_workflow.to_ascii_lowercase().contains(forbidden));

    // Assert
    assert!(all_reference_commands_present);
    assert!(cargo_only_isolated);
}

#[test]
fn sanitizer_rigid_protocol_and_compare_run_before_read_only_assertion() {
    // Arrange
    let workflow = include_str!("../../../.github/workflows/oracle.yml");
    let sanitizer_job = workflow
        .split("  sanitizer-linux:")
        .nth(1)
        .and_then(|suffix| suffix.split("  portability-macos:").next())
        .expect("sanitizer job must remain in the Oracle workflow");
    let build = "cargo xtask upstream build --preset oracle-asan-ubsan";
    let protocol_build = "cmake --build target/reference/oracle-asan-ubsan --target liquidfun-reference-protocol-tests";
    let protocol = "ctest --test-dir target/reference/oracle-asan-ubsan";
    let rigid = "cargo xtask differential compare --scenario rigid-world --preset oracle-asan-ubsan --session-profile one-shot";
    let read_only = "git diff --exit-code -- protocol scenarios reference COMPATIBILITY.md";

    // Act
    let positions = [build, protocol_build, protocol, rigid, read_only]
        .map(|marker| sanitizer_job.find(marker));

    // Assert
    assert!(positions.iter().all(Option::is_some));
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn sanitizer_rigid_commands_use_fail_fast_environment_without_status_suppression() {
    // Arrange
    let workflow = include_str!("../../../.github/workflows/oracle.yml");
    let sanitizer_job = workflow
        .split("  sanitizer-linux:")
        .nth(1)
        .and_then(|suffix| suffix.split("  portability-macos:").next())
        .expect("sanitizer job must remain in the Oracle workflow");
    let fail_fast = "UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1 ASAN_OPTIONS=abort_on_error=1:halt_on_error=1";
    let required_commands = [
        "ctest --test-dir target/reference/oracle-asan-ubsan",
        "cargo xtask differential compare --scenario rigid-world --preset oracle-asan-ubsan --session-profile one-shot",
    ];

    // Act / Assert
    for command in required_commands {
        let expected = format!("{fail_fast} {command}");
        assert!(sanitizer_job.contains(&expected), "missing `{expected}`");
    }
    assert!(!sanitizer_job.contains("continue-on-error:"));
    assert!(!sanitizer_job.contains("|| true"));
    assert!(!sanitizer_job.contains("|| echo"));
}

#[test]
fn sanitizer_rigid_compare_passes_only_the_reviewed_one_shot_shape() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    let arguments = [
        "differential",
        "compare",
        "--scenario",
        "rigid-world",
        "--preset",
        "oracle-asan-ubsan",
        "--session-profile",
        "one-shot",
    ];
    command.args(arguments);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(fixture.differential_arguments()?, &arguments[1..]);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn collision_compile_database_identity_is_covered_by_unit_digest_tests() {
    // The unit digest fixtures name collision_probe.cpp explicitly and are run by this filter.
    assert!(
        include_str!("../../../crates/liquidfun-differential/src/oracle_identity.rs")
            .contains("collision_probe.cpp")
    );
}

#[test]
fn collision_required_families_are_validated_before_oracle_execution() {
    let bytes = include_bytes!("../../../protocol/fixtures/accepted/collision-probe-request.jsonl");
    let request = liquidfun_test_protocol::decode_collision_probe_request_jsonl(
        bytes,
        &liquidfun_test_protocol::HarnessLimits::phase2_default_v1(),
    )
    .expect("checked-in collision corpus should be fail-closed and complete");
    for family in liquidfun_test_protocol::CollisionWitnessFamily::REQUIRED {
        assert!(
            request
                .scenario()
                .cases()
                .iter()
                .any(|case| case.witness_family() == family)
        );
    }
}

#[test]
fn math_probe_determinism_accepts_only_two_reviewed_runs() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "differential",
        "verify-determinism",
        "--scenario",
        "math-probes",
        "--preset",
        "oracle-debug",
        "--runs",
        "2",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fixture.differential_arguments()?,
        [
            "verify-determinism",
            "--scenario",
            "math-probes",
            "--preset",
            "oracle-debug",
            "--runs",
            "2",
        ]
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn math_probe_commands_reject_unreviewed_profiles_and_run_counts() -> TestResult {
    // Arrange
    let cases = [
        vec![
            "differential",
            "compare",
            "--scenario",
            "math-probes",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "reuse",
        ],
        vec![
            "differential",
            "verify-determinism",
            "--scenario",
            "math-probes",
            "--preset",
            "oracle-debug",
            "--runs",
            "3",
        ],
    ];

    for arguments in cases {
        let fixture = RepositoryFixture::new()?;
        let mut command = fixture.command()?;
        command.args(arguments);

        // Act
        let output = command.output()?;

        // Assert
        assert_failure_category(&output, "differential/usage");
        assert!(!fixture.differential_marker.exists());
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn fixture_stage_passes_only_lifecycle_metadata() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "differential",
        "fixture",
        "stage",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
        "--artifact-kind",
        "reviewed-trace",
        "--artifact-id",
        "trace-1",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fixture.differential_arguments()?,
        [
            "fixture",
            "stage",
            "--scenario",
            "empty-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
            "--artifact-kind",
            "reviewed-trace",
            "--artifact-id",
            "trace-1",
        ]
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn fixture_review_and_promote_pass_only_candidate_metadata() -> TestResult {
    // Arrange
    let cases = [
        vec![
            "fixture",
            "review",
            "--artifact-id",
            "trace-1",
            "--reviewer",
            "maintainer",
            "--reviewed-at",
            "2026-07-10T11:30:00Z",
            "--review-status",
            "approved",
        ],
        vec!["fixture", "promote", "--artifact-id", "trace-1"],
    ];

    for arguments in cases {
        let fixture = RepositoryFixture::new()?;
        let mut command = fixture.command()?;
        command.arg("differential").args(&arguments);

        // Act
        let output = command.output()?;

        // Assert
        assert!(output.status.success(), "{}", stderr(&output));
        assert_eq!(fixture.differential_arguments()?, arguments);
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn compare_rejects_unregistered_scenario_before_starting_runner() -> TestResult {
    assert_rejected_value("--scenario", "../outside")
}

#[test]
fn compare_rejects_unregistered_preset_before_starting_runner() -> TestResult {
    assert_rejected_value("--preset", "untrusted")
}

#[test]
fn compare_rejects_unregistered_profile_before_starting_runner() -> TestResult {
    assert_rejected_value("--session-profile", "unbounded")
}

#[test]
fn replay_rejects_arbitrary_request_path_before_starting_runner() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "differential",
        "replay",
        "--exact-request",
        "../outside.jsonl",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "differential/usage");
    assert!(!fixture.differential_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn compare_rejects_extra_option_before_starting_runner() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "differential",
        "compare",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
        "--output",
        "../outside.jsonl",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "differential/usage");
    assert!(!fixture.differential_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn compare_propagates_differential_child_failure() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command
        .args([
            "differential",
            "compare",
            "--scenario",
            "empty-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
        ])
        .env("LIQUIDFUN_TEST_DIFFERENTIAL_FAIL", "1");

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "differential/process");
    assert!(stderr(&output).contains("status 42"));
    assert!(stderr(&output).contains("simulated differential failure"));
    fixture.cleanup()?;
    Ok(())
}

fn assert_rejected_value(option: &str, value: &str) -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    let mut arguments = vec![
        "differential",
        "compare",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
    ];
    let index = arguments
        .iter()
        .position(|argument| *argument == option)
        .ok_or_else(|| io::Error::other(format!("missing test option {option}")))?;
    arguments[index + 1] = value;
    command.args(arguments);

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "differential/usage");
    assert!(!fixture.differential_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}

fn prepare_real_rigid_repository(root: &Path, behavior: &str) -> io::Result<()> {
    fs::create_dir_all(root.join("protocol/fixtures/accepted"))?;
    fs::create_dir_all(root.join("protocol/tolerances"))?;
    fs::create_dir_all(root.join("reference/artifacts"))?;
    fs::create_dir_all(root.join("scenarios/regressions"))?;
    fs::copy(
        workspace_root().join("protocol/fixtures/accepted/rigid-world-request.jsonl"),
        root.join("protocol/fixtures/accepted/rigid-world-request.jsonl"),
    )?;
    fs::copy(
        workspace_root().join("protocol/tolerances/phase6-v1.toml"),
        root.join("protocol/tolerances/phase6-v1.toml"),
    )?;
    fs::copy(
        workspace_root().join("protocol/tolerances/phase7-v1.toml"),
        root.join("protocol/tolerances/phase7-v1.toml"),
    )?;
    fs::copy(
        workspace_root().join("reference/artifacts/manifest.toml"),
        root.join("reference/artifacts/manifest.toml"),
    )?;
    fs::write(root.join("THIRD_PARTY_NOTICES.md"), "fixture notices\n")?;
    write_real_adapter_inputs(root)?;
    run_system_git(root, &["init", "--quiet"])?;
    run_system_git(root, &["config", "user.name", "Fixture User"])?;
    run_system_git(root, &["config", "user.email", "fixture@example.invalid"])?;
    run_system_git(root, &["add", "."])?;
    run_system_git(root, &["commit", "--quiet", "-m", "fixture"])?;
    let oracle_directory = root.join("target/reference/oracle-debug");
    fs::create_dir_all(&oracle_directory)?;
    fs::copy(
        workspace_root()
            .join("target/debug")
            .join(executable_name("liquidfun-fake-oracle")),
        oracle_directory.join(executable_name("liquidfun-reference")),
    )?;
    write_real_compile_database(root)?;
    fs::write(oracle_directory.join("behavior.txt"), behavior)
}

fn write_real_adapter_inputs(root: &Path) -> io::Result<()> {
    let source = root.join("tools/reference/src");
    fs::create_dir_all(&source)?;
    fs::write(
        root.join("tools/reference/adapter-inputs.txt"),
        "tools/reference/src/fixture_adapter.cpp\ntools/reference/src/fixture_adapter.hpp\n",
    )?;
    fs::write(
        source.join("fixture_adapter.cpp"),
        b"fixture adapter implementation\n",
    )?;
    fs::write(
        source.join("fixture_adapter.hpp"),
        b"fixture adapter interface\n",
    )
}

fn write_real_compile_database(root: &Path) -> io::Result<()> {
    let build = root.join("target/reference/oracle-debug");
    fs::create_dir_all(&build)?;
    let units = [
        "collision_probe.cpp",
        "math_probe.cpp",
        "protocol_bits.cpp",
        "rigid_world.cpp",
    ];
    let entries = units
        .map(|unit| {
            let source = root.join("tools/reference/src").join(unit);
            serde_json::json!({
                "directory": build,
                "file": source,
                "command": format!(
                    "clang++ -I{}/tools/reference/src -DREVIEWED=1 -o {}/{unit}.o -c {}",
                    root.display(),
                    build.display(),
                    source.display()
                ),
            })
        })
        .to_vec();
    fs::write(
        build.join("compile_commands.json"),
        serde_json::to_vec_pretty(&entries)?,
    )
}

fn run_system_git(root: &Path, arguments: &[&str]) -> io::Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "git {} failed: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr).trim()
    )))
}

fn fake_tools() -> io::Result<&'static FakeTools> {
    let result = FAKE_TOOLS.get_or_init(compile_fake_tools);
    match result {
        Ok(tools) => Ok(tools),
        Err(message) => Err(io::Error::other(message.clone())),
    }
}

fn compile_fake_tools() -> Result<FakeTools, String> {
    let output_dir = workspace_root().join(format!(
        "target/xtask-differential-tools/{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;
    let upstream_source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake_upstream_tool.rs");
    let upstream_base = compile_fake_tool(&upstream_source, &output_dir, "fake-upstream")?;
    let differential_source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake_differential_tool.rs");
    let differential = compile_fake_tool(&differential_source, &output_dir, "fake-differential")?;

    Ok(FakeTools {
        git: copy_fake_tool(&upstream_base, &output_dir, "fake-git")?,
        ninja: copy_fake_tool(&upstream_base, &output_dir, "fake-ninja")?,
        cxx: copy_fake_tool(&upstream_base, &output_dir, "fake-cxx")?,
        differential,
    })
}

fn compile_fake_tool(source: &Path, output_dir: &Path, name: &str) -> Result<PathBuf, String> {
    let destination = output_dir.join(executable_name(name));
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let output = Command::new(rustc)
        .arg(source)
        .arg("--edition=2024")
        .arg("-o")
        .arg(&destination)
        .output()
        .map_err(|error| format!("failed to compile {}: {error}", source.display()))?;
    if !output.status.success() {
        return Err(format!(
            "failed to compile {}: {}",
            source.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(destination)
}

fn copy_fake_tool(base: &Path, output_dir: &Path, name: &str) -> Result<PathBuf, String> {
    let destination = output_dir.join(executable_name(name));
    fs::copy(base, &destination).map_err(|error| error.to_string())?;
    Ok(destination)
}

fn executable_name(stem: &str) -> String {
    format!("{stem}{}", env::consts::EXE_SUFFIX)
}

fn assert_failure_category(output: &Output, category: &str) {
    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert!(
        stderr(output).contains(category),
        "expected category `{category}` in stderr: {}",
        stderr(output)
    );
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
