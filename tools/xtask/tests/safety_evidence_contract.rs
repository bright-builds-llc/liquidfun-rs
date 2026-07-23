//! Typed safety-evidence contract and closed CLI tests.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "../src/safety_evidence/contract.rs"]
mod contract;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
const CANDIDATE: &str = "1111111111111111111111111111111111111111";
const FIX: &str = "2222222222222222222222222222222222222222";

struct TestRoot {
    path: PathBuf,
}

impl TestRoot {
    fn new(label: &str) -> TestResult<Self> {
        let path = std::env::temp_dir().join(format!(
            "liquidfun-safety-contract-{label}-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("scenarios/regressions"))?;
        Ok(Self { path })
    }

    fn manifest(&self) -> TestResult<String> {
        let input = b"minimized-regression-v1\n";
        fs::write(self.path.join("scenarios/regressions/case.bin"), input)?;
        Ok(format!(
            r#"schema_version = 1
record_fields = [
  "id",
  "minimized_path",
  "minimized_sha256",
  "target",
  "generator",
  "toolchain",
  "candidate_commit",
  "fix_commit",
  "oracle_identity",
  "tolerance_identity",
  "first_divergence_signature",
  "failure_class",
  "review_status",
  "named_test_path",
]

[[regressions]]
id = "case-v1"
minimized_path = "scenarios/regressions/case.bin"
minimized_sha256 = "{}"
target = "world_mutation"
generator = "cargo-fuzz-0.13.2"
toolchain = "nightly-2026-07-15"
candidate_commit = "{CANDIDATE}"
fix_commit = "{FIX}"
oracle_identity = "oracle-debug@7f204021"
tolerance_identity = "phase12-v1"
first_divergence_signature = "checkpoint-1/world.bodies/exact"
failure_class = "PhysicsMismatch"
review_status = "reviewed"
named_test_path = "regressions::case_v1"
"#,
            contract::sha256(input)
        ))
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        if self.path.exists() {
            fs::remove_dir_all(&self.path).expect("test-owned temporary root should be removable");
        }
    }
}

struct ResultDirectory {
    candidate: String,
    path: PathBuf,
}

impl ResultDirectory {
    fn new() -> TestResult<Self> {
        let candidate = format!("{:040x}", NEXT_ID.fetch_add(1, Ordering::Relaxed) + 0x1000);
        let path = workspace_root()
            .join("target/phase12-regressions")
            .join(&candidate);
        fs::create_dir_all(&path)?;
        Ok(Self { candidate, path })
    }

    fn relative(&self) -> String {
        format!("target/phase12-regressions/{}", self.candidate)
    }

    fn write_completion(&self, value: &serde_json::Value) -> TestResult {
        fs::write(
            self.path.join("completion.json"),
            serde_json::to_vec_pretty(value)?,
        )?;
        Ok(())
    }
}

impl Drop for ResultDirectory {
    fn drop(&mut self) {
        if self.path.exists() {
            fs::remove_dir_all(&self.path)
                .expect("test-owned confined result directory should be removable");
        }
    }
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask remains two levels below the workspace")
}

fn run(args: &[&str]) -> TestResult<Output> {
    Ok(Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(workspace_root())
        .output()?)
}

fn run_results(candidate: &str, results: &str) -> TestResult<Output> {
    run(&[
        "safety-evidence",
        "validate-regression-results",
        "--candidate",
        candidate,
        "--results",
        results,
    ])
}

fn empty_completion(candidate: &str) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "candidate_sha": candidate,
        "complete": true,
        "results": [],
    })
}

fn tracked_coverage() -> TestResult<Vec<u8>> {
    Ok(fs::read(
        workspace_root().join("reference/coverage/contract.json"),
    )?)
}

#[test]
fn regression_manifest_validates_exact_bytes_and_renders_stably() -> TestResult {
    // Arrange
    let root = TestRoot::new("manifest")?;
    let manifest_bytes = root.manifest()?;

    // Act
    let manifest =
        contract::validate_regression_manifest_bytes(&root.path, manifest_bytes.as_bytes())?;
    let first = contract::render_execution_list(&manifest)?;
    let second = contract::render_execution_list(&manifest)?;

    // Assert
    assert_eq!(manifest.regressions().len(), 1);
    assert_eq!(first, second);
    assert!(first.contains("\"regression_id\": \"case-v1\""));
    assert!(first.contains("\"minimized_input\": \"scenarios/regressions/case.bin\""));
    assert!(first.contains("\"failure_class\": \"PhysicsMismatch\""));
    Ok(())
}

#[test]
fn regression_manifest_rejects_unknown_duplicate_hash_and_class_changes() -> TestResult {
    // Arrange
    let root = TestRoot::new("manifest-negatives")?;
    let valid = root.manifest()?;
    let unknown = valid.replace("schema_version = 1", "schema_version = 1\nsurprise = true");
    let duplicate = format!(
        "{valid}\n{}",
        valid.split("[[regressions]]").nth(1).unwrap_or("")
    );
    let wrong_hash = valid.replace(
        &contract::sha256(b"minimized-regression-v1\n"),
        &"0".repeat(64),
    );
    let wrong_class = valid.replace("PhysicsMismatch", "MergedCoverage");

    // Act / Assert
    for malformed in [unknown, duplicate, wrong_hash, wrong_class] {
        assert!(
            contract::validate_regression_manifest_bytes(&root.path, malformed.as_bytes()).is_err()
        );
    }
    Ok(())
}

#[test]
fn coverage_contract_keeps_rust_cpp_and_differential_authority_distinct() -> TestResult {
    // Arrange
    let bytes = tracked_coverage()?;

    // Act
    let contract = contract::validate_coverage_contract_bytes(&bytes);

    // Assert
    assert!(contract.is_ok());
    Ok(())
}

#[test]
fn coverage_contract_rejects_unknown_merged_parity_and_incomplete_leaves() -> TestResult {
    // Arrange
    let valid: serde_json::Value = serde_json::from_slice(&tracked_coverage()?)?;
    let mut unknown = valid.clone();
    unknown["surprise"] = serde_json::json!(true);
    let mut merged = valid.clone();
    merged["cpp"]["evidence_kinds"] = serde_json::json!(["rust_coverage", "cpp_asan_ubsan"]);
    let mut parity = valid.clone();
    parity["parity_authority"] = serde_json::json!(true);
    let mut incomplete = valid;
    incomplete["subsystem_fields"] = serde_json::json!(["name", "exercised_files_or_leaves"]);

    // Act / Assert
    for malformed in [unknown, merged, parity, incomplete] {
        assert!(
            contract::validate_coverage_contract_bytes(&serde_json::to_vec(&malformed)?).is_err()
        );
    }
    Ok(())
}

#[test]
fn coverage_records_require_five_distinct_complete_evidence_kinds() -> TestResult {
    // Arrange
    let root = TestRoot::new("coverage-records")?;
    fs::create_dir_all(root.path.join("target/coverage"))?;
    let coverage_contract = contract::validate_coverage_contract_bytes(&tracked_coverage()?)?;
    let subsystem = serde_json::json!({
        "name": "collision",
        "exercised_files_or_leaves": ["collision/tree.rs"],
        "missed_files_or_leaves": ["collision/toi.rs"],
    });
    let record = |kind: &str, toolchain: &str| -> TestResult<serde_json::Value> {
        let artifact_path = format!("target/coverage/{kind}.json");
        let artifact_bytes = format!("{kind}\n");
        fs::write(root.path.join(&artifact_path), artifact_bytes.as_bytes())?;
        Ok(serde_json::json!({
            "evidence_kind": kind,
            "candidate_commit": CANDIDATE,
            "toolchain_identity": toolchain,
            "artifact_path": artifact_path,
            "artifact_sha256": contract::sha256(artifact_bytes.as_bytes()),
            "subsystems": [subsystem.clone()],
        }))
    };
    let valid = serde_json::json!({
        "schema_version": 1,
        "candidate_commit": CANDIDATE,
        "parity_authority": false,
        "records": [
            record("rust_sanitizer", "nightly-2026-07-15")?,
            record("cpp_asan_ubsan", "clang-22.1.8")?,
            record("rust_coverage", "nightly-2026-07-15")?,
            record("cpp_coverage", "clang-22.1.8")?,
            record("differential_coverage", "semantic-leaf-v1")?,
        ],
    });
    let mut duplicate = valid.clone();
    duplicate["records"][4]["evidence_kind"] = serde_json::json!("rust_coverage");
    let mut incomplete = valid.clone();
    incomplete["records"][0]["subsystems"][0]["exercised_files_or_leaves"] = serde_json::json!([]);
    incomplete["records"][0]["subsystems"][0]["missed_files_or_leaves"] = serde_json::json!([]);
    let mut parity = valid.clone();
    parity["parity_authority"] = serde_json::json!(true);
    let mut wrong_hash = valid.clone();
    wrong_hash["records"][0]["artifact_sha256"] = serde_json::json!("0".repeat(64));
    let mut mixed_toolchain = valid.clone();
    mixed_toolchain["records"][0]["toolchain_identity"] = serde_json::json!("clang-22.1.8");

    // Act / Assert
    assert!(
        contract::validate_coverage_record_bytes(
            &root.path,
            &coverage_contract,
            &serde_json::to_vec(&valid)?
        )
        .is_ok()
    );
    for malformed in [duplicate, incomplete, parity, wrong_hash, mixed_toolchain] {
        assert!(
            contract::validate_coverage_record_bytes(
                &root.path,
                &coverage_contract,
                &serde_json::to_vec(&malformed)?
            )
            .is_err()
        );
    }
    Ok(())
}

#[test]
fn result_set_rejects_missing_duplicate_unregistered_and_mixed_identity() -> TestResult {
    // Arrange
    let root = TestRoot::new("results")?;
    let manifest =
        contract::validate_regression_manifest_bytes(&root.path, root.manifest()?.as_bytes())?;
    let registered = serde_json::json!({
        "regression_id": "case-v1",
        "candidate_sha": CANDIDATE,
        "named_test_path": "regressions::case_v1",
        "minimized_sha256": contract::sha256(b"minimized-regression-v1\n"),
        "outcome": "passed",
    });
    let valid = serde_json::json!({
        "schema_version": 1,
        "candidate_sha": CANDIDATE,
        "complete": true,
        "results": [registered.clone()],
    });
    let mut missing = valid.clone();
    missing["results"] = serde_json::json!([]);
    let mut duplicate = valid.clone();
    duplicate["results"] = serde_json::json!([registered.clone(), registered.clone()]);
    let mut unregistered = valid.clone();
    unregistered["results"][0]["regression_id"] = serde_json::json!("unknown");
    let mut mixed = valid.clone();
    mixed["results"][0]["candidate_sha"] = serde_json::json!(FIX);

    // Act / Assert
    assert!(
        contract::validate_regression_result_bytes(
            &manifest,
            CANDIDATE,
            &serde_json::to_vec(&valid)?
        )
        .is_ok()
    );
    for malformed in [missing, duplicate, unregistered, mixed] {
        assert!(
            contract::validate_regression_result_bytes(
                &manifest,
                CANDIDATE,
                &serde_json::to_vec(&malformed)?
            )
            .is_err()
        );
    }
    Ok(())
}

#[test]
fn closed_commands_validate_tracked_authorities_and_execution_list_is_stable() -> TestResult {
    // Arrange / Act
    let regressions = run(&["safety-evidence", "validate-regressions"])?;
    let first = run(&[
        "safety-evidence",
        "validate-regressions",
        "--emit-execution-list",
    ])?;
    let second = run(&[
        "safety-evidence",
        "validate-regressions",
        "--emit-execution-list",
    ])?;
    let coverage = run(&["safety-evidence", "validate-coverage"])?;
    let help = run(&["safety-evidence", "validate-regression-results", "--help"])?;

    // Assert
    for output in [&regressions, &first, &second, &coverage, &help] {
        assert!(
            output.status.success(),
            "stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&first.stdout)?,
        serde_json::json!([])
    );
    assert!(String::from_utf8_lossy(&help.stdout).contains("--candidate FULL_SHA"));
    Ok(())
}

#[test]
fn confined_empty_result_set_validates_then_writes_identity_last() -> TestResult {
    // Arrange
    let directory = ResultDirectory::new()?;
    directory.write_completion(&empty_completion(&directory.candidate))?;

    // Act
    let output = run_results(&directory.candidate, &directory.relative())?;

    // Assert
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let identity: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.path.join("identity.json"))?)?;
    assert_eq!(identity["candidate_sha"], directory.candidate);
    Ok(())
}

#[test]
fn invalid_result_set_never_publishes_identity() -> TestResult {
    // Arrange
    let directory = ResultDirectory::new()?;
    let mut incomplete = empty_completion(&directory.candidate);
    incomplete["complete"] = serde_json::json!(false);
    directory.write_completion(&incomplete)?;

    // Act
    let output = run_results(&directory.candidate, &directory.relative())?;

    // Assert
    assert!(!output.status.success());
    assert!(!directory.path.join("identity.json").exists());
    Ok(())
}

#[test]
fn result_cli_rejects_absolute_parent_alternate_and_candidate_mismatch_paths() -> TestResult {
    // Arrange
    let candidate = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let paths = [
        "/tmp/phase12-results",
        "target/phase12-regressions/../escape",
        "alternate/phase12-regressions/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "target/phase12-regressions/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    ];

    // Act / Assert
    for results in paths {
        assert!(!run_results(candidate, results)?.status.success());
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn result_cli_rejects_symlinked_candidate_directory() -> TestResult {
    use std::os::unix::fs::symlink;

    // Arrange
    let directory = ResultDirectory::new()?;
    let outside = TestRoot::new("symlink-result")?;
    fs::remove_dir_all(&directory.path)?;
    symlink(&outside.path, &directory.path)?;

    // Act
    let output = run_results(&directory.candidate, &directory.relative())?;

    // Assert
    assert!(!output.status.success());
    fs::remove_file(&directory.path)?;
    Ok(())
}

#[test]
fn result_cli_rejects_missing_marker_extra_file_and_preexisting_identity() -> TestResult {
    // Arrange / Act / Assert: missing completion marker
    let missing = ResultDirectory::new()?;
    assert!(
        !run_results(&missing.candidate, &missing.relative())?
            .status
            .success()
    );

    // Arrange / Act / Assert: unexpected result file
    let extra = ResultDirectory::new()?;
    extra.write_completion(&empty_completion(&extra.candidate))?;
    fs::write(extra.path.join("unexpected.json"), b"{}\n")?;
    assert!(
        !run_results(&extra.candidate, &extra.relative())?
            .status
            .success()
    );

    // Arrange / Act / Assert: preexisting identity
    let identity = ResultDirectory::new()?;
    identity.write_completion(&empty_completion(&identity.candidate))?;
    fs::write(identity.path.join("identity.json"), b"{}\n")?;
    assert!(
        !run_results(&identity.candidate, &identity.relative())?
            .status
            .success()
    );
    Ok(())
}

#[test]
fn command_registration_uses_the_shared_contract_without_cli_owned_schema() -> TestResult {
    // Arrange
    let main_source = fs::read_to_string(workspace_root().join("tools/xtask/src/main.rs"))?;
    let cli_source =
        fs::read_to_string(workspace_root().join("tools/xtask/src/safety_evidence.rs"))?;

    // Act / Assert
    assert!(main_source.contains("\"safety-evidence\""));
    assert!(cli_source.contains("validate_regression_manifest_bytes"));
    assert!(cli_source.contains("validate_regression_result_bytes"));
    assert!(cli_source.contains("validate_coverage_contract_bytes"));
    assert!(!cli_source.contains("struct RegressionResult {"));
    assert!(!cli_source.contains("struct CoverageRecord {"));
    Ok(())
}
