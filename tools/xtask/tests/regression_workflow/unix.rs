use std::{
    env,
    fmt::Write,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::{TestResult, workspace_root};

const CANDIDATE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const ORIGINAL_CANDIDATE: &str = "1111111111111111111111111111111111111111";
const FIX_COMMIT: &str = "2222222222222222222222222222222222222222";
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

struct Fixture {
    root: PathBuf,
    fake_bin: PathBuf,
    execution_list: PathBuf,
    command_log: PathBuf,
    event_log: PathBuf,
    candidate: String,
}

struct ConfinedResults {
    candidate: String,
    path: PathBuf,
}

impl ConfinedResults {
    fn new() -> TestResult<Self> {
        let candidate = format!("{:040x}", NEXT_ID.fetch_add(1, Ordering::Relaxed) + 0x9000);
        let path = workspace_root()
            .join("target/phase12-regressions")
            .join(&candidate);
        fs::create_dir_all(&path)?;
        let registry = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args([
                "safety-evidence",
                "validate-regressions",
                "--emit-execution-list",
            ])
            .current_dir(workspace_root())
            .output()?;
        assert!(registry.status.success());
        let entries: Vec<Value> = serde_json::from_slice(&registry.stdout)?;
        let records: Vec<_> = entries
            .iter()
            .map(|entry| {
                json!({
                    "regression_id": entry["regression_id"],
                    "candidate_sha": candidate,
                    "named_test_path": entry["named_test_path"],
                    "minimized_sha256": entry["minimized_sha256"],
                    "outcome": "passed",
                })
            })
            .collect();
        fs::write(
            path.join("completion.json"),
            serde_json::to_vec_pretty(&json!({
                "schema_version": 1,
                "candidate_sha": candidate,
                "complete": true,
                "results": records,
            }))?,
        )?;
        Ok(Self { candidate, path })
    }
}

impl Drop for ConfinedResults {
    fn drop(&mut self) {
        if self.path.exists() {
            fs::remove_dir_all(&self.path)
                .expect("test-owned confined results should be removable");
        }
    }
}

impl Fixture {
    fn new(entry: &Value) -> TestResult<Self> {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = env::temp_dir().join(format!(
            "liquidfun-regression-workflow-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("scripts"))?;
        fs::create_dir_all(root.join("scenarios/regressions"))?;
        let fake_bin = root.join("fake-bin");
        fs::create_dir_all(&fake_bin)?;
        fs::copy(
            workspace_root().join("scripts/phase12-regressions.sh"),
            root.join("scripts/phase12-regressions.sh"),
        )?;

        let minimized = b"reviewed-minimized-input\n";
        fs::write(root.join("scenarios/regressions/case.bin"), minimized)?;
        let execution_list = root.join("execution-list.json");
        fs::write(&execution_list, serde_json::to_vec_pretty(&json!([entry]))?)?;
        let command_log = root.join("commands.log");
        let event_log = root.join("events.log");
        fs::write(&command_log, [])?;
        fs::write(&event_log, [])?;

        write_executable(&fake_bin.join("cargo"), fake_cargo())?;
        write_executable(&fake_bin.join("git"), fake_git())?;

        Ok(Self {
            root,
            fake_bin,
            execution_list,
            command_log,
            event_log,
            candidate: CANDIDATE.to_owned(),
        })
    }

    fn with_entries(entries: &Value) -> TestResult<Self> {
        let fixture = Self::new(&valid_entry())?;
        fs::write(
            &fixture.execution_list,
            serde_json::to_vec_pretty(&entries)?,
        )?;
        Ok(fixture)
    }

    fn run(&self, mode: &str) -> TestResult<Output> {
        let inherited_path = env::var_os("PATH").ok_or("PATH must be available to tests")?;
        let joined_path = env::join_paths(
            std::iter::once(self.fake_bin.as_os_str()).chain(
                env::split_paths(&inherited_path)
                    .map(std::path::PathBuf::into_os_string)
                    .collect::<Vec<_>>()
                    .iter()
                    .map(std::ffi::OsString::as_os_str),
            ),
        )?;
        Ok(Command::new("timeout")
            .arg("30s")
            .arg("bash")
            .arg("scripts/phase12-regressions.sh")
            .arg("run")
            .arg(&self.candidate)
            .current_dir(&self.root)
            .env("PATH", joined_path)
            .env("FAKE_CANDIDATE", &self.candidate)
            .env("FAKE_COMMAND_LOG", &self.command_log)
            .env("FAKE_EVENT_LOG", &self.event_log)
            .env("FAKE_EXECUTION_LIST", &self.execution_list)
            .env("FAKE_REPOSITORY", &self.root)
            .env("FAKE_RESULT_MODE", mode)
            .env("GITHUB_WORKFLOW", "fixture-workflow")
            .env("GITHUB_JOB", "fixture-job")
            .env("GITHUB_RUN_ID", "42")
            .output()?)
    }

    fn output_directory(&self) -> PathBuf {
        self.root
            .join("target/phase12-regressions")
            .join(&self.candidate)
    }

    fn diagnostics(&self) -> PathBuf {
        self.root
            .join("target/phase12-regression-diagnostics")
            .join(&self.candidate)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if self.root.exists() {
            fs::remove_dir_all(&self.root)
                .expect("test-owned regression fixture should be removable");
        }
    }
}

fn write_executable(path: &Path, source: &str) -> TestResult {
    fs::write(path, source)?;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

fn minimized_sha256() -> String {
    format!("{:x}", Sha256::digest(b"reviewed-minimized-input\n"))
}

fn valid_entry() -> Value {
    json!({
        "regression_id": "case-v1",
        "named_test_path": "regressions::case_v1",
        "minimized_input": "scenarios/regressions/case.bin",
        "minimized_sha256": minimized_sha256(),
        "provenance": {
            "target": "world_mutation",
            "generator": "cargo-fuzz-0.13.2",
            "toolchain": "nightly-2026-07-15",
            "candidate_commit": ORIGINAL_CANDIDATE,
            "fix_commit": FIX_COMMIT,
            "oracle_identity": "oracle-debug@7f204021",
            "tolerance_identity": "phase12-v1",
            "first_divergence_signature": "checkpoint-1/world.bodies/exact",
            "failure_class": "PhysicsMismatch"
        }
    })
}

fn fake_git() -> &'static str {
    r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "$*" != "rev-parse HEAD" ]]; then
  exit 97
fi
printf '%s\n' "$FAKE_CANDIDATE"
"#
}

fn fake_cargo() -> &'static str {
    include_str!("cargo.sh")
}

#[test]
fn invariant_class_flows_from_real_typed_projection_to_exact_replay() -> TestResult {
    // Arrange
    let mut entry = valid_entry();
    entry["provenance"]["failure_class"] = json!("InvariantViolation");
    entry["provenance"]
        .as_object_mut()
        .ok_or("provenance object")?
        .remove("oracle_identity");
    entry["provenance"]
        .as_object_mut()
        .ok_or("provenance object")?
        .remove("tolerance_identity");
    let fixture = Fixture::new(&entry)?;
    fs::create_dir_all(fixture.root.join("crates/liquidfun"))?;
    fs::create_dir_all(fixture.root.join("reference/regressions"))?;
    fs::write(fixture.root.join("Cargo.toml"), "[workspace]\n")?;
    fs::write(
        fixture.root.join("crates/liquidfun/Cargo.toml"),
        "[package]\nname=\"liquidfun\"\n",
    )?;
    let tracked = fs::read_to_string(workspace_root().join("reference/regressions/manifest.toml"))?;
    let header = tracked
        .split("[[regressions]]")
        .next()
        .ok_or("registry header")?;
    let mut manifest = format!("{header}[[regressions]]\nreview_status = \"reviewed\"\n");
    for (key, value) in entry["provenance"].as_object().ok_or("provenance object")? {
        writeln!(manifest, "{key} = {value}")?;
    }
    for (key, field) in [
        ("id", "regression_id"),
        ("minimized_path", "minimized_input"),
        ("minimized_sha256", "minimized_sha256"),
        ("named_test_path", "named_test_path"),
    ] {
        writeln!(manifest, "{key} = {}", entry[field])?;
    }
    fs::write(
        fixture.root.join("reference/regressions/manifest.toml"),
        manifest,
    )?;
    // Act
    let typed = Command::new("timeout")
        .arg("30s")
        .arg(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "safety-evidence",
            "validate-regressions",
            "--emit-execution-list",
        ])
        .current_dir(&fixture.root)
        .output()?;
    assert!(
        typed.status.success(),
        "{}",
        String::from_utf8_lossy(&typed.stderr)
    );
    fs::write(&fixture.execution_list, typed.stdout)?;
    let output = fixture.run("valid")?;
    // Assert
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        fs::read_to_string(&fixture.command_log)?
            .contains("test -p liquidfun --test regressions --all-features -- case_v1 --exact")
    );
    assert!(fixture.diagnostics().join("logs/case-v1.log").is_file());
    Ok(())
}

#[test]
fn oversized_test_log_is_bounded_and_cannot_publish_success() -> TestResult {
    // Arrange
    let fixture = Fixture::new(&valid_entry())?;
    // Act
    let output = fixture.run("oversized-log")?;
    // Assert
    assert!(!output.status.success());
    assert!(
        !fixture
            .output_directory()
            .join("producer-identity.json")
            .exists()
    );
    assert_eq!(
        fs::metadata(fixture.diagnostics().join("logs/case-v1.log"))?.len(),
        16 * 1024 * 1024
    );
    Ok(())
}

#[test]
fn failed_selected_test_preserves_log_without_success_identity() -> TestResult {
    // Arrange
    let fixture = Fixture::new(&valid_entry())?;
    // Act
    let output = fixture.run("failed-test")?;
    // Assert
    assert!(!output.status.success());
    assert!(
        !fixture
            .output_directory()
            .join("producer-identity.json")
            .exists()
    );
    assert!(
        fs::read_to_string(fixture.diagnostics().join("logs/case-v1.log"))?
            .contains("selected test failed")
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(fixture.diagnostics().join("terminal.json"))?)?["exit_code"],
        64
    );
    Ok(())
}

#[test]
fn zero_selected_tests_never_publish_success() -> TestResult {
    // Arrange
    let fixture = Fixture::new(&valid_entry())?;
    // Act
    let output = fixture.run("zero-tests")?;
    // Assert
    assert!(!output.status.success());
    assert!(
        !fixture
            .output_directory()
            .join("producer-identity.json")
            .exists()
    );
    Ok(())
}

#[test]
fn repeated_destination_preserves_original_attempt() -> TestResult {
    // Arrange
    let fixture = Fixture::new(&valid_entry())?;
    assert!(fixture.run("valid")?.status.success());
    let original = fs::read(fixture.output_directory().join("producer-identity.json"))?;
    // Act
    let output = fixture.run("valid")?;
    // Assert
    assert!(!output.status.success());
    assert_eq!(
        fs::read(fixture.output_directory().join("producer-identity.json"))?,
        original
    );
    Ok(())
}

#[test]
fn valid_fixture_records_exact_execution_validation_and_identity_order() -> TestResult {
    // Arrange
    let fixture = Fixture::new(&valid_entry())?;

    // Act
    let output = fixture.run("valid")?;

    // Assert
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&fixture.event_log)?,
        "execution-list\ntest:case-v1\nvalidate-results\n"
    );
    let output_directory = fixture.output_directory();
    assert!(output_directory.join("completion.json").is_file());
    assert!(output_directory.join("identity.json").is_file());
    let producer: Value =
        serde_json::from_slice(&fs::read(output_directory.join("producer-identity.json"))?)?;
    assert_eq!(producer["candidate_sha"], CANDIDATE);
    assert_eq!(producer["producer_workflow"], "fixture-workflow");
    assert_eq!(producer["producer_job"], "fixture-job");
    assert_eq!(producer["run_id"], 42);
    assert_eq!(producer["run_attempt"], 1);
    assert_eq!(
        producer["diagnostics_sha256"],
        format!(
            "{:x}",
            Sha256::digest(fs::read(fixture.diagnostics().join("checksums.sha256"))?)
        )
    );
    assert!(
        fixture
            .diagnostics()
            .join("logs/case-v1.command.json")
            .is_file()
    );
    let checksums = Command::new("sha256sum")
        .arg("-c")
        .arg("checksums.sha256")
        .current_dir(fixture.diagnostics())
        .output()?;
    assert!(checksums.status.success());
    assert_eq!(producer["named_test_count"], 1);
    assert_eq!(producer["regression_manifest_sha256"], "a".repeat(64));
    assert_eq!(producer["payload_path"], "identity.json");
    assert_eq!(
        producer["payload_sha256"],
        format!(
            "{:x}",
            Sha256::digest(fs::read(output_directory.join("identity.json"))?)
        )
    );
    Ok(())
}

#[test]
fn real_typed_cli_validates_the_exact_confined_results_path() -> TestResult {
    // Arrange
    let results = ConfinedResults::new()?;
    let relative = format!("target/phase12-regressions/{}", results.candidate);

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "safety-evidence",
            "validate-regression-results",
            "--candidate",
            &results.candidate,
            "--results",
            &relative,
        ])
        .current_dir(workspace_root())
        .output()?;

    // Assert
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(results.path.join("identity.json").is_file());
    Ok(())
}

#[test]
fn empty_duplicate_and_invalid_provenance_lists_fail_before_identity() -> TestResult {
    // Arrange
    let duplicate = valid_entry();
    let mut wrong_hash = valid_entry();
    wrong_hash["minimized_sha256"] = json!("0".repeat(64));
    let mut missing_oracle = valid_entry();
    missing_oracle["provenance"]["oracle_identity"] = Value::Null;
    let mut missing_tolerance = valid_entry();
    missing_tolerance["provenance"]["tolerance_identity"] = Value::Null;
    let mut wrong_fix = valid_entry();
    wrong_fix["provenance"]["fix_commit"] = json!("short");
    let mut unknown_class = valid_entry();
    unknown_class["provenance"]["failure_class"] = json!("UnknownInvariant");
    let fixtures = [
        json!([]),
        json!([duplicate.clone(), duplicate]),
        json!([wrong_hash]),
        json!([missing_oracle]),
        json!([missing_tolerance]),
        json!([wrong_fix]),
        json!([unknown_class]),
    ];

    // Act / Assert
    for execution_list in fixtures {
        let fixture = Fixture::with_entries(&execution_list)?;
        let output = fixture.run("valid")?;
        assert!(!output.status.success());
        assert!(
            !fixture
                .output_directory()
                .join("producer-identity.json")
                .exists()
        );
    }
    Ok(())
}

#[test]
fn omitted_duplicated_and_unregistered_results_fail_typed_validation() -> TestResult {
    // Arrange / Act / Assert
    for mode in ["omitted", "duplicated", "unregistered"] {
        let fixture = Fixture::new(&valid_entry())?;
        let output = fixture.run(mode)?;
        assert!(!output.status.success(), "{mode}");
        assert!(!fixture.output_directory().join("identity.json").exists());
        assert!(
            !fixture
                .output_directory()
                .join("producer-identity.json")
                .exists()
        );
    }
    Ok(())
}
