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
    rigid_commands::compare_passes_only_canonical_structured_arguments()
}

#[test]
fn replay_and_minimize_pass_only_named_scenario_arguments() -> TestResult {
    rigid_commands::replay_and_minimize_pass_only_named_scenario_arguments()
}

#[test]
fn math_probe_compare_and_replay_pass_only_reviewed_arguments() -> TestResult {
    rigid_commands::math_probe_compare_and_replay_pass_only_reviewed_arguments()
}

#[test]
fn collision_compare_replay_and_determinism_pass_only_reviewed_arguments() -> TestResult {
    rigid_commands::collision_compare_replay_and_determinism_pass_only_reviewed_arguments()
}

#[test]
fn rigid_compare_replay_and_minimize_pass_only_reviewed_arguments() -> TestResult {
    rigid_commands::rigid_compare_replay_and_minimize_pass_only_reviewed_arguments()
}

#[test]
fn rigid_determinism_accepts_exactly_two_debug_runs() -> TestResult {
    rigid_commands::rigid_determinism_accepts_exactly_two_debug_runs()
}

#[test]
fn rigid_fixture_stage_passes_only_fixed_lifecycle_metadata() -> TestResult {
    rigid_commands::rigid_fixture_stage_passes_only_fixed_lifecycle_metadata()
}

#[test]
fn rigid_fixture_real_binary_accepts_d1_and_rejects_d2_before_effects() -> TestResult {
    rigid_commands::rigid_fixture_real_binary_accepts_d1_and_rejects_d2_before_effects()
}

#[test]
fn rigid_fixture_stale_identity_real_binary_rejects_before_effects() -> TestResult {
    rigid_commands::rigid_fixture_stale_identity_real_binary_rejects_before_effects()
}

#[test]
fn rigid_commands_reject_unreviewed_shapes_before_effects() -> TestResult {
    rigid_commands::rigid_commands_reject_unreviewed_shapes_before_effects()
}

#[test]
fn rigid_child_failure_status_is_propagated() -> TestResult {
    validation_commands::rigid_child_failure_status_is_propagated()
}

#[test]
fn rigid_ci_commands_stay_in_the_native_reference_workflow() {
    validation_commands::rigid_ci_commands_stay_in_the_native_reference_workflow();
}

#[test]
fn sanitizer_rigid_protocol_and_compare_run_before_read_only_assertion() {
    validation_commands::sanitizer_rigid_protocol_and_compare_run_before_read_only_assertion();
}

#[test]
fn sanitizer_rigid_commands_use_fail_fast_environment_without_status_suppression() {
    validation_commands::sanitizer_rigid_commands_use_fail_fast_environment_without_status_suppression();
}

#[test]
fn sanitizer_rigid_compare_passes_only_the_reviewed_one_shot_shape() -> TestResult {
    validation_commands::sanitizer_rigid_compare_passes_only_the_reviewed_one_shot_shape()
}

#[test]
fn collision_compile_database_identity_is_covered_by_unit_digest_tests() {
    validation_commands::collision_compile_database_identity_is_covered_by_unit_digest_tests();
}

#[test]
fn collision_required_families_are_validated_before_oracle_execution() {
    validation_commands::collision_required_families_are_validated_before_oracle_execution();
}

#[test]
fn math_probe_determinism_accepts_only_two_reviewed_runs() -> TestResult {
    validation_commands::math_probe_determinism_accepts_only_two_reviewed_runs()
}

#[test]
fn math_probe_commands_reject_unreviewed_profiles_and_run_counts() -> TestResult {
    validation_commands::math_probe_commands_reject_unreviewed_profiles_and_run_counts()
}

#[test]
fn fixture_stage_passes_only_lifecycle_metadata() -> TestResult {
    validation_commands::fixture_stage_passes_only_lifecycle_metadata()
}

#[test]
fn fixture_review_and_promote_pass_only_candidate_metadata() -> TestResult {
    validation_commands::fixture_review_and_promote_pass_only_candidate_metadata()
}

#[test]
fn compare_rejects_unregistered_scenario_before_starting_runner() -> TestResult {
    validation_commands::compare_rejects_unregistered_scenario_before_starting_runner()
}

#[test]
fn compare_rejects_unregistered_preset_before_starting_runner() -> TestResult {
    validation_commands::compare_rejects_unregistered_preset_before_starting_runner()
}

#[test]
fn compare_rejects_unregistered_profile_before_starting_runner() -> TestResult {
    validation_commands::compare_rejects_unregistered_profile_before_starting_runner()
}

#[test]
fn replay_rejects_arbitrary_request_path_before_starting_runner() -> TestResult {
    validation_commands::replay_rejects_arbitrary_request_path_before_starting_runner()
}

#[test]
fn compare_rejects_extra_option_before_starting_runner() -> TestResult {
    validation_commands::compare_rejects_extra_option_before_starting_runner()
}

#[test]
fn compare_propagates_differential_child_failure() -> TestResult {
    validation_commands::compare_propagates_differential_child_failure()
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

fn debug_binary(name: &str) -> PathBuf {
    let target_directory = env::var_os("CARGO_TARGET_DIR")
        .map_or_else(|| workspace_root().join("target"), PathBuf::from);
    target_directory.join("debug").join(executable_name(name))
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
        workspace_root().join("protocol/tolerances/phase8-v1.toml"),
        root.join("protocol/tolerances/phase8-v1.toml"),
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
        debug_binary("liquidfun-fake-oracle"),
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

#[path = "differential_cli/rigid_commands.rs"]
mod rigid_commands;
#[path = "differential_cli/validation_commands.rs"]
mod validation_commands;
