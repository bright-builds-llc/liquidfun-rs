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

struct RigidFixtureRepository {
    root: PathBuf,
    oracle_directory: PathBuf,
}

impl RigidFixtureRepository {
    fn new(behavior: &str) -> io::Result<Self> {
        let sequence = NEXT_REPOSITORY.fetch_add(1, Ordering::Relaxed);
        let root = workspace_root().join(format!(
            "target/rigid-fixture-workflow-{}-{sequence}",
            std::process::id()
        ));
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
        write_adapter_inputs(&root)?;
        run_git(&root, &["init", "--quiet"])?;
        run_git(&root, &["config", "user.name", "Fixture User"])?;
        run_git(&root, &["config", "user.email", "fixture@example.invalid"])?;
        run_git(&root, &["add", "."])?;
        run_git(&root, &["commit", "--quiet", "-m", "fixture"])?;

        let oracle_directory = root.join("target/reference/oracle-debug");
        fs::create_dir_all(&oracle_directory)?;
        fs::copy(
            env!("CARGO_BIN_EXE_liquidfun-fake-oracle"),
            oracle_directory.join(oracle_name()),
        )?;
        write_compile_database(&root, "-DREVIEWED=1")?;
        fs::write(oracle_directory.join("behavior.txt"), behavior)?;
        Ok(Self {
            root,
            oracle_directory,
        })
    }

    fn stage(&self, artifact_id: &str) -> io::Result<Output> {
        self.command(&[
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
            artifact_id,
        ])
    }

    fn review(&self, artifact_id: &str) -> io::Result<Output> {
        self.command(&[
            "fixture",
            "review",
            "--artifact-id",
            artifact_id,
            "--reviewer",
            "fixture-reviewer",
            "--reviewed-at",
            "2026-07-12T12:00:00Z",
            "--review-status",
            "approved",
        ])
    }

    fn promote(&self, artifact_id: &str) -> io::Result<Output> {
        self.command(&["fixture", "promote", "--artifact-id", artifact_id])
    }

    fn command(&self, arguments: &[&str]) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_liquidfun-differential"))
            .current_dir(&self.root)
            .args(arguments)
            .output()
    }

    fn set_behavior(&self, behavior: &str) -> io::Result<()> {
        fs::write(self.oracle_directory.join("behavior.txt"), behavior)
    }

    fn set_request_policy_hash(&self, hash: &str) -> io::Result<()> {
        let path = self
            .root
            .join("protocol/fixtures/accepted/rigid-world-request.jsonl");
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
        value["tolerance_profile_sha256"] = serde_json::Value::String(hash.to_owned());
        let mut bytes = serde_json::to_vec(&value)?;
        bytes.push(b'\n');
        fs::write(path, bytes)
    }

    fn candidate(&self, artifact_id: &str) -> PathBuf {
        self.root
            .join("target/differential/staging")
            .join(artifact_id)
    }

    fn adapter_input(&self) -> PathBuf {
        self.root.join("tools/reference/src/fixture_adapter.hpp")
    }

    fn compile_database(&self) -> PathBuf {
        self.oracle_directory.join("compile_commands.json")
    }
}

#[derive(Debug, PartialEq, Eq)]
struct FixtureMutationSnapshot {
    staging: Vec<(PathBuf, Vec<u8>)>,
    traces: Vec<(PathBuf, Vec<u8>)>,
    regressions: Vec<(PathBuf, Vec<u8>)>,
    manifest: Vec<u8>,
}

impl FixtureMutationSnapshot {
    fn capture(repository: &RigidFixtureRepository) -> io::Result<Self> {
        Ok(Self {
            staging: snapshot_tree(&repository.root.join("target/differential/staging"))?,
            traces: snapshot_tree(&repository.root.join("reference/artifacts/traces"))?,
            regressions: snapshot_tree(&repository.root.join("scenarios/regressions"))?,
            manifest: fs::read(repository.root.join("reference/artifacts/manifest.toml"))?,
        })
    }
}

impl Drop for RigidFixtureRepository {
    fn drop(&mut self) {
        let _ignored = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn transaction_real_binary_stages_replays_and_promotes_canonical_rigid_trace()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1")?;

    // Act
    let staged = repository.stage("canonical-rigid")?;
    let reviewed = repository.review("canonical-rigid")?;
    let promoted = repository.promote("canonical-rigid")?;

    // Assert
    assert!(staged.status.success(), "{}", stderr(&staged));
    assert!(reviewed.status.success(), "{}", stderr(&reviewed));
    assert!(promoted.status.success(), "{}", stderr(&promoted));
    assert!(
        repository
            .candidate("canonical-rigid")
            .join("review.toml")
            .is_file()
    );
    assert!(
        repository
            .root
            .join("reference/artifacts/traces/phase-08-rigid-world-v1.jsonl")
            .is_file()
    );
    let manifest = fs::read_to_string(repository.root.join("reference/artifacts/manifest.toml"))?;
    assert!(manifest.contains("phase-08-rigid-world-v1.jsonl"));
    Ok(())
}

#[test]
fn real_binary_rejects_d2_before_staging_or_accepted_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d2")?;
    let manifest_before = fs::read(repository.root.join("reference/artifacts/manifest.toml"))?;

    // Act
    let output = repository.stage("noncanonical-rigid")?;

    // Assert
    assert!(!output.status.success());
    assert!(stderr(&output).contains("requires D1 canonical authority"));
    assert!(!repository.candidate("noncanonical-rigid").exists());
    assert!(!repository.root.join("target/differential/staging").exists());
    assert_eq!(
        fs::read(repository.root.join("reference/artifacts/manifest.toml"))?,
        manifest_before
    );
    assert!(
        !repository
            .root
            .join("reference/artifacts/traces/phase-08-rigid-world-v1.jsonl")
            .exists()
    );
    Ok(())
}

#[test]
fn stale_request_policy_rejects_before_staging_or_oracle_execution()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1")?;
    repository.set_request_policy_hash(&"0".repeat(64))?;
    repository.set_behavior("rigid_d1_nonzero")?;
    let before = FixtureMutationSnapshot::capture(&repository)?;

    // Act
    let error = stage_rigid_candidate(
        &repository.root,
        "stale-policy",
        ArtifactKind::ReviewedTrace,
        OraclePreset::Debug,
        "oracle-debug",
        "one-shot",
        "7f20402173fd143a3988c921bc384459c6a858f2",
        None,
    )
    .expect_err("stale request provenance must fail closed");

    // Assert
    assert!(error.to_string().contains("request policy hash"));
    assert_eq!(FixtureMutationSnapshot::capture(&repository)?, before);
    assert!(!repository.candidate("stale-policy").exists());
    Ok(())
}

#[test]
fn stale_adapter_real_binary_rejects_without_fixture_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1_stale_adapter")?;
    let before = FixtureMutationSnapshot::capture(&repository)?;

    // Act
    let output = repository.stage("stale-adapter")?;

    // Assert
    assert!(!output.status.success());
    let diagnostic = stderr(&output);
    assert!(diagnostic.contains("adapter digest differs from current checkout inputs"));
    assert!(diagnostic.len() < 1024);
    assert!(!diagnostic.contains(repository.root.to_string_lossy().as_ref()));
    assert_eq!(FixtureMutationSnapshot::capture(&repository)?, before);
    assert!(!repository.candidate("stale-adapter").exists());
    Ok(())
}

#[test]
fn stale_compile_real_binary_rejects_without_fixture_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1_stale_compile")?;
    let before = FixtureMutationSnapshot::capture(&repository)?;

    // Act
    let output = repository.stage("stale-compile")?;

    // Assert
    assert!(!output.status.success());
    let diagnostic = stderr(&output);
    assert!(diagnostic.contains("compile-command digest differs"));
    assert!(diagnostic.len() < 1024);
    assert!(!diagnostic.contains(repository.root.to_string_lossy().as_ref()));
    assert_eq!(FixtureMutationSnapshot::capture(&repository)?, before);
    assert!(!repository.candidate("stale-compile").exists());
    Ok(())
}

#[test]
fn review_and_promotion_recompute_checkout_identity_before_writes()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1")?;
    let staged = repository.stage("identity-drift")?;
    assert!(staged.status.success(), "{}", stderr(&staged));
    let adapter_path = repository.adapter_input();
    let adapter_before = fs::read(&adapter_path)?;
    fs::write(&adapter_path, b"fixture adapter interface changed\n")?;
    let before_review = FixtureMutationSnapshot::capture(&repository)?;

    // Act
    let stale_review = repository.review("identity-drift")?;

    // Assert
    assert!(!stale_review.status.success());
    assert!(stderr(&stale_review).contains("adapter digest differs"));
    assert_eq!(
        FixtureMutationSnapshot::capture(&repository)?,
        before_review
    );
    assert!(
        !repository
            .candidate("identity-drift")
            .join("review.toml")
            .exists()
    );

    // Arrange
    fs::write(&adapter_path, adapter_before)?;
    let reviewed = repository.review("identity-drift")?;
    assert!(reviewed.status.success(), "{}", stderr(&reviewed));
    let compile_path = repository.compile_database();
    let compile = fs::read_to_string(&compile_path)?;
    fs::write(
        &compile_path,
        compile.replace("-DREVIEWED=1", "-DREVIEWED=2"),
    )?;
    let before_promotion = FixtureMutationSnapshot::capture(&repository)?;

    // Act
    let stale_promotion = repository.promote("identity-drift")?;

    // Assert
    assert!(!stale_promotion.status.success());
    assert!(stderr(&stale_promotion).contains("compile-command digest differs"));
    assert_eq!(
        FixtureMutationSnapshot::capture(&repository)?,
        before_promotion
    );
    assert!(
        repository
            .candidate("identity-drift")
            .join("review.toml")
            .is_file()
    );
    assert!(
        !repository
            .root
            .join("reference/artifacts/traces/phase-08-rigid-world-v1.jsonl")
            .exists()
    );
    Ok(())
}

#[test]
fn transaction_replay_rejects_dirty_rigid_candidate_before_review()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1")?;
    let staged = repository.stage("dirty-rigid")?;
    assert!(staged.status.success(), "{}", stderr(&staged));
    fs::write(
        repository.candidate("dirty-rigid").join("trace.jsonl"),
        b"tampered\n",
    )?;

    // Act
    let reviewed = repository.review("dirty-rigid")?;

    // Assert
    assert!(!reviewed.status.success());
    assert!(stderr(&reviewed).contains("SHA-256 mismatch"));
    assert!(
        !repository
            .candidate("dirty-rigid")
            .join("review.toml")
            .exists()
    );
    Ok(())
}

#[test]
fn real_binary_propagates_rigid_child_failure_without_candidate_creation()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1")?;
    repository.set_behavior("rigid_d1_nonzero")?;

    // Act
    let output = repository.stage("failed-child")?;

    // Assert
    assert!(!output.status.success());
    assert!(!repository.candidate("failed-child").exists());
    Ok(())
}

#[test]
fn minimized_regression_requires_reduced_complete_provenance()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1_mismatch")?;
    let original_bytes = fs::read(
        repository
            .root
            .join("protocol/fixtures/accepted/rigid-world-request.jsonl"),
    )?;
    let minimization = rigid_minimization(&repository.root, 4_096)?;
    assert_eq!(minimization.status(), MinimizationStatus::Complete);
    let minimized_native = NativeRigidWorldExecutor::execute(minimization.request())?;
    let minimized_value = serde_json::to_value(&minimized_native)?;
    assert!(
        minimized_value
            .pointer("/timelines/0/checkpoints/0/bodies/0/active")
            .is_some(),
        "the minimized request must retain the divergent body"
    );
    assert_fake_oracle_accepts(&repository, minimization.canonical_request_bytes())?;

    // Act
    let candidate = stage_rigid_candidate(
        &repository.root,
        "minimized-rigid",
        ArtifactKind::MinimizedRegression,
        OraclePreset::Debug,
        "oracle-debug",
        "one-shot",
        &git_head(&repository.root)?,
        Some(&minimization),
    )?;
    let reviewed = repository.review("minimized-rigid")?;

    // Assert
    let staged_request = fs::read(candidate.directory().join("request.jsonl"))?;
    assert_eq!(staged_request, minimization.canonical_request_bytes());
    assert_ne!(staged_request, original_bytes);
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(candidate.directory().join("report.json"))?)?;
    assert_eq!(report["status"], "complete");
    assert_eq!(
        report["target_signature_sha256"],
        minimization.target_signature().signature_sha256().as_str()
    );
    assert!(
        report["attempted_transforms"]
            .as_array()
            .is_some_and(|transforms| !transforms.is_empty())
    );
    assert!(
        report["accepted_transforms"]
            .as_array()
            .is_some_and(|transforms| !transforms.is_empty())
    );
    assert!(reviewed.status.success(), "{}", stderr(&reviewed));
    Ok(())
}

#[test]
fn minimized_regression_rejects_full_or_incomplete_requests_without_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1_mismatch")?;
    let incomplete = rigid_minimization(&repository.root, 0)?;
    assert!(matches!(
        incomplete.status(),
        MinimizationStatus::Incomplete(_)
    ));
    let before = FixtureMutationSnapshot::capture(&repository)?;

    // Act
    let missing = stage_rigid_candidate(
        &repository.root,
        "missing-minimization",
        ArtifactKind::MinimizedRegression,
        OraclePreset::Debug,
        "oracle-debug",
        "one-shot",
        &git_head(&repository.root)?,
        None,
    )
    .expect_err("a full request must not be labeled as minimized");
    let unfinished = stage_rigid_candidate(
        &repository.root,
        "incomplete-minimization",
        ArtifactKind::MinimizedRegression,
        OraclePreset::Debug,
        "oracle-debug",
        "one-shot",
        &git_head(&repository.root)?,
        Some(&incomplete),
    )
    .expect_err("an incomplete reduction must not be staged");

    // Assert
    assert!(
        missing
            .to_string()
            .contains("completed minimization result")
    );
    assert!(unfinished.to_string().contains("complete reduction"));
    assert_eq!(FixtureMutationSnapshot::capture(&repository)?, before);
    Ok(())
}

fn rigid_minimization(
    root: &Path,
    max_attempts: usize,
) -> Result<RigidMinimizationResult, Box<dyn std::error::Error>> {
    let effective_max_attempts = max_attempts.saturating_mul(4);
    let limits = HarnessLimits::phase2_default_v1();
    let request_bytes =
        fs::read(root.join("protocol/fixtures/accepted/rigid-world-request.jsonl"))?;
    let request = decode_rigid_world_request_jsonl(&request_bytes, &limits)?;
    let phase6 = Phase6PolicyProfile::parse_toml(&fs::read_to_string(
        root.join("protocol/tolerances/phase6-v1.toml"),
    )?)?;
    let phase7 = Phase7PolicyProfile::parse_toml(&fs::read_to_string(
        root.join("protocol/tolerances/phase7-v1.toml"),
    )?)?;
    let phase8 = Phase8PolicyProfile::parse_toml(&fs::read_to_string(
        root.join("protocol/tolerances/phase8-v1.toml"),
    )?)?;
    let Some(target) = rigid_mismatch_signature(&request, &phase6, &phase7, &phase8) else {
        return Err(io::Error::other("injected rigid mismatch unexpectedly matched").into());
    };
    minimize_rigid_world_request(
        &request,
        &target,
        MinimizationBudget::new(effective_max_attempts, Duration::from_secs(1)),
        |candidate| {
            RigidEvaluation::new(
                rigid_mismatch_signature(candidate, &phase6, &phase7, &phase8),
                Duration::ZERO,
            )
        },
    )
    .map_err(Into::into)
}

fn rigid_mismatch_signature(
    request: &RigidWorldRequestRecord,
    phase6: &Phase6PolicyProfile,
    phase7: &Phase7PolicyProfile,
    phase8: &Phase8PolicyProfile,
) -> Option<RigidFailureSignature> {
    let limits = HarnessLimits::phase2_default_v1();
    let native = NativeRigidWorldExecutor::execute(request).ok()?;
    let mut oracle_value = serde_json::to_value(&native).ok()?;
    let active = oracle_value.pointer_mut("/timelines/0/checkpoints/0/bodies/0/active")?;
    *active = serde_json::Value::Bool(false);
    let mut oracle_bytes = serde_json::to_vec(&oracle_value).ok()?;
    oracle_bytes.push(b'\n');
    let oracle = decode_rigid_world_result_jsonl(&oracle_bytes, &limits).ok()?;
    let outcome =
        compare_phase8_rigid_world_results(request, &native, &oracle, phase6, phase7, phase8)
            .ok()?;
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        return None;
    };
    Some(report.signature().clone())
}

fn assert_fake_oracle_accepts(
    repository: &RigidFixtureRepository,
    request_bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new(repository.oracle_directory.join(oracle_name()))
        .current_dir(&repository.root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("fake oracle stdin is unavailable"))?
        .write_all(request_bytes)?;
    let output = child.wait_with_output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "fake oracle rejected minimized request: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    ))
    .into())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should be present")
        .to_path_buf()
}

fn oracle_name() -> &'static str {
    if cfg!(windows) {
        "liquidfun-reference.exe"
    } else {
        "liquidfun-reference"
    }
}

fn run_git(root: &Path, arguments: &[&str]) -> io::Result<()> {
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

fn git_head(root: &Path) -> io::Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned());
    }
    Err(io::Error::other(format!(
        "git rev-parse failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    )))
}

fn write_adapter_inputs(root: &Path) -> io::Result<()> {
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

fn write_compile_database(root: &Path, common_flag: &str) -> io::Result<()> {
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
                    "clang++ -I{}/tools/reference/src {common_flag} -o {}/{unit}.o -c {}",
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

fn snapshot_tree(root: &Path) -> io::Result<Vec<(PathBuf, Vec<u8>)>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect_snapshot(root, Path::new(""), &mut files)?;
    Ok(files)
}

fn collect_snapshot(
    root: &Path,
    relative: &Path,
    files: &mut Vec<(PathBuf, Vec<u8>)>,
) -> io::Result<()> {
    let mut entries = fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let child_relative = relative.join(entry.file_name());
        if path.is_dir() {
            collect_snapshot(&path, &child_relative, files)?;
        } else {
            files.push((child_relative, fs::read(path)?));
        }
    }
    Ok(())
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
