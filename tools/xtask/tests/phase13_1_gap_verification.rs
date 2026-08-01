//! Contracts for the Phase 13.1 gap-verification manifest and evidence tools.

use std::{collections::BTreeSet, fs, path::Path};

use serde_json::Value;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const DEFERRED_TARGET: &str = "phase13_acceptance_contract";
const SELECTED_TARGETS: [&str; 27] = [
    "catalog_cli",
    "corpus_closure",
    "corpus_discovery",
    "corpus_model",
    "coverage_workflow",
    "differential_cli",
    "docs_contract",
    "inventory_cli",
    "nightly_toolchain",
    "package_cli",
    "performance_cli",
    "performance_workflow",
    "phase10_evidence_cli",
    "phase11_evidence_cli",
    "phase13_1_canonical_native_workflow",
    "phase13_1_gap_verification",
    "phase13_evidence_contract",
    "phase13_promotion_contract",
    "phase9_evidence_cli",
    "phase9_witness_provenance",
    "platform_workflow",
    "provenance_cli",
    "regression_workflow",
    "release_attestation",
    "release_cli",
    "safety_evidence_contract",
    "upstream_cli",
];

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask remains two levels below the workspace")
}

fn integration_targets() -> TestResult<BTreeSet<String>> {
    let tests = workspace_root().join("tools/xtask/tests");
    let targets = fs::read_dir(tests)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .filter_map(|path| {
            path.file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
        })
        .collect();
    Ok(targets)
}

fn manifest() -> TestResult<Value> {
    let bytes =
        fs::read(workspace_root().join("scripts/phase13-1-gap-verification-manifest.json"))?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn validator_source() -> TestResult<String> {
    Ok(fs::read_to_string(
        workspace_root().join("scripts/phase13-1-validate-gap-evidence.sh"),
    )?)
}

fn command_argv(command: &Value) -> TestResult<Vec<&str>> {
    command["argv"]
        .as_array()
        .ok_or("argv must be an array")?
        .iter()
        .map(|argument| argument.as_str().ok_or("argv must contain strings"))
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

#[test]
fn manifest_selects_every_non_deferred_xtask_target_exactly_once() -> TestResult {
    // Arrange
    let inventory = integration_targets()?;
    let manifest = manifest()?;
    let selected = manifest["commands"]
        .as_array()
        .ok_or("commands must be an array")?
        .iter()
        .filter(|command| command["evidence_class"] == "xtask-integration")
        .map(|command| {
            command["argv"][5]
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or("xtask target must be argv[4]")
        })
        .collect::<Result<BTreeSet<_>, _>>()?;

    // Act
    let expected = inventory
        .iter()
        .filter(|target| target.as_str() != DEFERRED_TARGET)
        .cloned()
        .collect::<BTreeSet<_>>();

    // Assert
    assert_eq!(selected, expected);
    assert_eq!(selected.len(), 27);
    assert_eq!(
        manifest["deferred_xtask_targets"],
        serde_json::json!([DEFERRED_TARGET])
    );
    Ok(())
}

#[test]
fn manifest_preserves_exact_xtask_argv_environment_and_lexical_order() -> TestResult {
    // Arrange
    let manifest = manifest()?;
    let commands = manifest["commands"]
        .as_array()
        .ok_or("commands must be an array")?;
    let selected = commands
        .iter()
        .filter(|command| command["evidence_class"] == "xtask-integration")
        .collect::<Vec<_>>();

    // Act / Assert
    assert_eq!(selected.len(), SELECTED_TARGETS.len());
    for (command, expected_target) in selected.iter().zip(SELECTED_TARGETS) {
        assert_eq!(
            command_argv(command)?,
            [
                "cargo",
                "test",
                "-p",
                "xtask",
                "--test",
                expected_target,
                "--all-features"
            ]
        );
        assert_eq!(
            command["environment"],
            serde_json::json!({
                "CARGO_BUILD_JOBS": "1",
                "CARGO_TARGET_DIR": "${OUTPUT_ROOT}/cargo-target"
            })
        );
    }
    Ok(())
}

#[test]
fn manifest_rejects_broad_xtask_and_deferred_phase15_work() -> TestResult {
    // Arrange
    let manifest = manifest()?;
    let commands = manifest["commands"]
        .as_array()
        .ok_or("commands must be an array")?;
    let argv = commands
        .iter()
        .map(command_argv)
        .collect::<Result<Vec<_>, _>>()?;

    // Act
    let has_broad_xtask = argv.iter().any(|arguments| {
        arguments.starts_with(&["cargo", "test", "-p", "xtask"]) && !arguments.contains(&"--test")
    });
    let has_deferred_target = argv
        .iter()
        .any(|arguments| arguments.contains(&DEFERRED_TARGET));
    let has_acceptance_cli = argv.iter().any(|arguments| {
        arguments
            .windows(3)
            .any(|window| window == ["xtask", "phase13", "acceptance"])
    });

    // Assert
    assert!(!has_broad_xtask);
    assert!(!has_deferred_target);
    assert!(!has_acceptance_cli);
    Ok(())
}

#[test]
fn verification_report_records_the_same_sole_deferral() -> TestResult {
    // Arrange
    let verification = fs::read_to_string(workspace_root().join(
        ".planning/phases/13.1-restore-bright-builds-structural-compliance/13.1-VERIFICATION.md",
    ))?;

    // Act / Assert
    assert_eq!(
        verification.matches("addressed_in: \"Phase 15\"").count(),
        1
    );
    assert!(verification.contains("Exact-head Phase 13 acceptance remains green"));
    assert!(
        verification
            .contains("Current exact-head Phase 13 acceptance drift is deferred to Phase 15")
    );
    Ok(())
}

#[test]
fn evidence_validator_is_a_separate_fail_closed_executable() -> TestResult {
    // Arrange
    let source = validator_source()?;

    // Act / Assert
    assert!(source.starts_with("#!/usr/bin/env bash\nset -euo pipefail\n"));
    assert!(!source.contains("source scripts/phase13-1-gap-verification.sh"));
    assert!(!source.contains("phase13-1-gap-verification.sh\""));
    assert!(source.contains("phase13-1-gap-verification-evidence-v1"));
    assert!(source.contains("evidence_tier"));
    assert!(source.contains("git merge-base --is-ancestor"));
    Ok(())
}

#[cfg(unix)]
mod evidence_validator {
    use std::{
        env, fs,
        os::unix::fs::symlink,
        path::{Path, PathBuf},
        process::{Command, Output},
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::{TestResult, workspace_root};
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

    struct TemporaryDirectory(PathBuf);

    impl TemporaryDirectory {
        fn new() -> TestResult<Self> {
            let path = env::temp_dir().join(format!(
                "liquidfun-phase13-1-gap-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
            ));
            if path.exists() {
                return Err("test fixture path already exists".into());
            }
            fs::create_dir(&path)?;
            Ok(Self(path))
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TemporaryDirectory {
        fn drop(&mut self) {
            if self.0.exists() {
                fs::remove_dir_all(&self.0).expect("owned test fixture should be removable");
            }
        }
    }

    struct Fixture {
        _temporary: TemporaryDirectory,
        repository: PathBuf,
        retained: PathBuf,
        manifest: PathBuf,
        evidence: PathBuf,
    }

    fn hash_file(path: &Path) -> TestResult<String> {
        Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
    }

    fn run_git(repository: &Path, arguments: &[&str]) -> TestResult<String> {
        let output = Command::new("git")
            .args(arguments)
            .current_dir(repository)
            .output()?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
        }
        Ok(String::from_utf8(output.stdout)?.trim().to_owned())
    }

    fn write_json(path: &Path, value: &Value) -> TestResult {
        fs::write(path, serde_json::to_vec_pretty(value)?)?;
        Ok(())
    }

    impl Fixture {
        fn new() -> TestResult<Self> {
            let temporary = TemporaryDirectory::new()?;
            let repository = temporary.path().join("repository");
            let retained = temporary.path().join("retained");
            fs::create_dir_all(&repository)?;
            fs::create_dir_all(retained.join("logs"))?;
            fs::create_dir_all(retained.join("canonical/logs"))?;
            let retained = fs::canonicalize(retained)?;
            run_git(&repository, &["init", "-q", "-b", "main"])?;
            run_git(
                &repository,
                &["config", "user.email", "fixture@example.invalid"],
            )?;
            run_git(&repository, &["config", "user.name", "Fixture"])?;
            fs::write(repository.join("source.txt"), "base\n")?;
            run_git(&repository, &["add", "source.txt"])?;
            run_git(&repository, &["commit", "-q", "-m", "base"])?;
            let structural_parent = run_git(&repository, &["rev-parse", "HEAD"])?;
            fs::write(repository.join("source.txt"), "candidate\n")?;
            run_git(&repository, &["add", "source.txt"])?;
            run_git(&repository, &["commit", "-q", "-m", "candidate"])?;
            let candidate = run_git(&repository, &["rev-parse", "HEAD"])?;
            let tree = run_git(&repository, &["rev-parse", "HEAD^{tree}"])?;

            for name in ["one.stdout", "one.stderr", "two.stdout", "two.stderr"] {
                fs::write(retained.join("logs").join(name), format!("{name}\n"))?;
            }
            fs::write(retained.join("canonical/logs/native.log"), "canonical\n")?;
            let native_digest = hash_file(&retained.join("canonical/logs/native.log"))?;
            fs::write(
                retained.join("canonical/logs.sha256"),
                format!("{native_digest}  logs/native.log\n"),
            )?;
            let canonical_identity = json!({
                "candidate_sha": candidate,
                "candidate_tree": tree,
                "workflow_run_id": "7",
                "runner": {"os": "ubuntu-24.04", "architecture": "x86_64"},
                "tools": {"rust": "1.97.0", "clang": "22.1.8", "cmake": "4.3.3", "ninja": "1.13.2"},
                "evidence_tier": "D1",
                "command_exits": [{"name": "native", "exit_code": 0}],
                "log_digests": "logs.sha256"
            });
            write_json(
                &retained.join("canonical/identity.json"),
                &canonical_identity,
            )?;

            let commands = json!([
                {"id":"one","argv":["true"],"environment":{},"stdout_log":"logs/one.stdout","stderr_log":"logs/one.stderr","evidence_class":"fixture"},
                {"id":"two","argv":["true","two"],"environment":{"FIXTURE":"yes"},"stdout_log":"logs/two.stdout","stderr_log":"logs/two.stderr","evidence_class":"fixture"}
            ]);
            let manifest_json = json!({
                "schema": "phase13-1-gap-verification-manifest-v1",
                "test_fixture": true,
                "allowed_placeholders": ["CANDIDATE", "CANDIDATE_TREE", "OUTPUT_ROOT", "REMOTE_REF", "CANONICAL_RUN_ID"],
                "structural_source": {"commit": candidate, "parent": structural_parent},
                "deferred_xtask_targets": ["phase13_acceptance_contract"],
                "artifacts": [{"id":"canonical-native","identity":"canonical/identity.json","logs":"canonical/logs","evidence_tier":"D1"}],
                "commands": commands
            });
            let manifest = temporary.path().join("manifest.json");
            write_json(&manifest, &manifest_json)?;
            let evidence_commands = manifest_json["commands"]
                .as_array()
                .ok_or("fixture commands must be an array")?
                .iter()
                .map(|command| {
                    let mut record = command.clone();
                    let object = record
                        .as_object_mut()
                        .ok_or("fixture command must be an object")?;
                    object.insert("exit_code".into(), json!(0));
                    for stream in ["stdout", "stderr"] {
                        let relative = command[format!("{stream}_log")]
                            .as_str()
                            .ok_or("fixture log path must be a string")?;
                        object.insert(
                            format!("{stream}_sha256"),
                            json!(hash_file(&retained.join(relative))?),
                        );
                    }
                    Ok(record)
                })
                .collect::<TestResult<Vec<_>>>()?;
            let evidence_json = json!({
                "schema": "phase13-1-gap-verification-evidence-v1",
                "candidate_sha": candidate,
                "candidate_tree": tree,
                "output_root": retained.to_string_lossy(),
                "remote_ref": "main",
                "canonical_run_id": "7",
                "manifest_sha256": hash_file(&manifest)?,
                "commands": evidence_commands,
                "checker": {"findings": 0, "exceptions": 0},
                "canonical_identity": {"path":"canonical/identity.json","sha256":hash_file(&retained.join("canonical/identity.json"))?},
                "complete": true
            });
            let evidence = retained.join("final-verification.json.pending");
            write_json(&evidence, &evidence_json)?;
            Ok(Self {
                _temporary: temporary,
                repository,
                retained,
                manifest,
                evidence,
            })
        }

        fn run(&self) -> TestResult<Output> {
            Ok(
                Command::new(workspace_root().join("scripts/phase13-1-validate-gap-evidence.sh"))
                    .arg(&self.manifest)
                    .arg(&self.evidence)
                    .arg(&self.repository)
                    .arg(&self.retained)
                    .output()?,
            )
        }

        fn mutate_evidence(&self, mutation: impl FnOnce(&mut Value)) -> TestResult {
            let mut evidence: Value = serde_json::from_slice(&fs::read(&self.evidence)?)?;
            mutation(&mut evidence);
            write_json(&self.evidence, &evidence)
        }

        fn assert_rejected(&self) -> TestResult {
            let output = self.run()?;
            assert!(
                !output.status.success(),
                "validator unexpectedly accepted mutated evidence"
            );
            Ok(())
        }
    }

    #[test]
    fn evidence_validator_accepts_complete_candidate_bound_evidence() -> TestResult {
        let fixture = Fixture::new()?;
        let output = fixture.run()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }

    #[test]
    fn evidence_validator_rejects_failed_command_publication() -> TestResult {
        let fixture = Fixture::new()?;
        fixture.mutate_evidence(|evidence| evidence["commands"][0]["exit_code"] = json!(1))?;
        fixture.assert_rejected()
    }

    #[test]
    fn evidence_validator_rejects_missing_log() -> TestResult {
        let fixture = Fixture::new()?;
        fs::remove_file(fixture.retained.join("logs/one.stdout"))?;
        fixture.assert_rejected()
    }

    #[test]
    fn evidence_validator_rejects_digest_mismatch() -> TestResult {
        let fixture = Fixture::new()?;
        fs::write(fixture.retained.join("logs/one.stdout"), "mutated\n")?;
        fixture.assert_rejected()
    }

    #[test]
    fn evidence_validator_rejects_command_order_drift() -> TestResult {
        let fixture = Fixture::new()?;
        fixture.mutate_evidence(|evidence| {
            if let Some(commands) = evidence["commands"].as_array_mut() {
                commands.swap(0, 1);
            }
        })?;
        fixture.assert_rejected()
    }

    #[test]
    fn evidence_validator_rejects_environment_drift() -> TestResult {
        let fixture = Fixture::new()?;
        fixture.mutate_evidence(|evidence| {
            evidence["commands"][0]["environment"] = json!({"DRIFT":"yes"})
        })?;
        fixture.assert_rejected()
    }

    #[test]
    fn evidence_validator_rejects_candidate_tree_drift() -> TestResult {
        let fixture = Fixture::new()?;
        fixture.mutate_evidence(|evidence| {
            evidence["candidate_tree"] = json!("0000000000000000000000000000000000000000")
        })?;
        fixture.assert_rejected()
    }

    #[test]
    fn evidence_validator_rejects_candidate_commit_drift() -> TestResult {
        let fixture = Fixture::new()?;
        let parent = run_git(&fixture.repository, &["rev-parse", "HEAD^"])?;
        fixture.mutate_evidence(|evidence| evidence["candidate_sha"] = json!(parent))?;
        fixture.assert_rejected()
    }

    #[test]
    fn evidence_validator_rejects_structural_parent_failure() -> TestResult {
        let fixture = Fixture::new()?;
        let mut manifest: Value = serde_json::from_slice(&fs::read(&fixture.manifest)?)?;
        manifest["structural_source"]["parent"] = manifest["structural_source"]["commit"].clone();
        write_json(&fixture.manifest, &manifest)?;
        let manifest_digest = hash_file(&fixture.manifest)?;
        fixture.mutate_evidence(|evidence| evidence["manifest_sha256"] = json!(manifest_digest))?;
        fixture.assert_rejected()
    }

    #[test]
    fn evidence_validator_rejects_symlink_substitution() -> TestResult {
        let fixture = Fixture::new()?;
        let log = fixture.retained.join("logs/one.stdout");
        fs::remove_file(&log)?;
        symlink("../canonical/identity.json", &log)?;
        fixture.assert_rejected()
    }

    #[test]
    fn evidence_validator_rejects_path_escape() -> TestResult {
        let fixture = Fixture::new()?;
        fixture.mutate_evidence(|evidence| {
            evidence["commands"][0]["stdout_log"] = json!("../escape.log")
        })?;
        fixture.assert_rejected()
    }

    #[test]
    fn evidence_validator_rejects_dirty_post_run_tree() -> TestResult {
        let fixture = Fixture::new()?;
        fs::write(fixture.repository.join("dirty.txt"), "dirty\n")?;
        fixture.assert_rejected()
    }

    #[test]
    fn evidence_validator_rejects_false_canonical_d1_identity() -> TestResult {
        let fixture = Fixture::new()?;
        let identity_path = fixture.retained.join("canonical/identity.json");
        let mut identity: Value = serde_json::from_slice(&fs::read(&identity_path)?)?;
        identity["evidence_tier"] = json!("D2");
        write_json(&identity_path, &identity)?;
        let identity_digest = hash_file(&identity_path)?;
        fixture.mutate_evidence(|evidence| {
            evidence["canonical_identity"]["sha256"] = json!(identity_digest)
        })?;
        fixture.assert_rejected()
    }

    #[test]
    fn evidence_validator_rejects_missing_canonical_identity() -> TestResult {
        let fixture = Fixture::new()?;
        fs::remove_file(fixture.retained.join("canonical/identity.json"))?;
        fixture.assert_rejected()
    }
}
