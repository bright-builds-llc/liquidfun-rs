//! Phase 12 named-regression producer and workflow contract tests.

use std::{fs, path::Path};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const VALIDATOR_COMMAND: &str = "cargo xtask safety-evidence validate-regression-results --candidate \"$candidate_sha\" --results \"target/phase12-regressions/$candidate_sha\"";

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask remains two levels below the workspace")
}

fn producer_source() -> TestResult<String> {
    Ok(fs::read_to_string(
        workspace_root().join("scripts/phase12-regressions.sh"),
    )?)
}

fn script_contract_is_valid(source: &str) -> bool {
    let maybe_completion =
        source.find("mv -- \"$completion_staging\" \"$output_directory/completion.json\"");
    let maybe_validator = source.find(VALIDATOR_COMMAND);
    let maybe_identity =
        source.find("write_producer_identity_last \"$output_directory\" \"$candidate_sha\"");
    let Some((completion, validator, identity)) = maybe_completion
        .zip(maybe_validator)
        .zip(maybe_identity)
        .map(|((completion, validator), identity)| (completion, validator, identity))
    else {
        return false;
    };

    source.starts_with("#!/usr/bin/env bash\nset -euo pipefail\n")
        && source.matches(VALIDATOR_COMMAND).count() == 1
        && completion < validator
        && validator < identity
        && source.contains("readonly PER_TEST_TIMEOUT_SECONDS=")
        && source.contains("readonly TOTAL_TIMEOUT_SECONDS=")
        && source.contains("readonly MAXIMUM_LOG_BYTES=")
        && source.contains("timeout --signal=TERM")
        && source.contains("validate-regressions --emit-execution-list")
        && source.contains("target/phase12-regressions/$candidate_sha")
        && !source.contains("retry")
        && !source.contains("--results \"$")
}

#[test]
fn producer_source_has_closed_bounded_completion_validation_identity_order() -> TestResult {
    // Arrange
    let source = producer_source()?;

    // Act
    let valid = script_contract_is_valid(&source);

    // Assert
    assert!(valid);
    Ok(())
}

#[test]
fn producer_projection_rejects_order_path_retry_and_unbounded_mutations() -> TestResult {
    // Arrange
    let source = producer_source()?;
    let validator_before_completion = format!("{VALIDATOR_COMMAND}\n{source}");
    let identity_before_validator =
        format!("write_producer_identity_last \"$output_directory\" \"$candidate_sha\"\n{source}");
    let alternate_results = source.replace(
        "--results \"target/phase12-regressions/$candidate_sha\"",
        "--results \"$caller_selected_results\"",
    );
    let retry = format!("{source}\nretry_failed_test\n");
    let unbounded = source.replace("timeout --signal=TERM", "command");

    // Act / Assert
    for invalid in [
        validator_before_completion,
        identity_before_validator,
        alternate_results,
        retry,
        unbounded,
    ] {
        assert!(!script_contract_is_valid(&invalid));
    }
    Ok(())
}

#[cfg(unix)]
mod unix {
    use std::{
        env, fs,
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

    impl Fixture {
        fn new(entry: Value) -> TestResult<Self> {
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

        fn with_entries(entries: Value) -> TestResult<Self> {
            let fixture = Self::new(valid_entry())?;
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
                        .map(|path| path.into_os_string())
                        .collect::<Vec<_>>()
                        .iter()
                        .map(std::ffi::OsString::as_os_str),
                ),
            )?;
            Ok(Command::new("bash")
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
        r#"#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$FAKE_COMMAND_LOG"
case "$*" in
  "xtask safety-evidence validate-regressions --emit-execution-list")
    printf 'execution-list\n' >> "$FAKE_EVENT_LOG"
    command cat "$FAKE_EXECUTION_LIST"
    ;;
  test\ --all-features\ --\ *)
    test "$LIQUIDFUN_REGRESSION_ID" = "case-v1"
    test "$LIQUIDFUN_REGRESSION_INPUT" = "scenarios/regressions/case.bin"
    test "$LIQUIDFUN_REGRESSION_INPUT_SHA256" != ""
    test "$LIQUIDFUN_REGRESSION_TARGET" = "world_mutation"
    test "$LIQUIDFUN_REGRESSION_GENERATOR" = "cargo-fuzz-0.13.2"
    test "$LIQUIDFUN_REGRESSION_TOOLCHAIN" = "nightly-2026-07-15"
    test "$LIQUIDFUN_REGRESSION_ORIGINAL_CANDIDATE" = "1111111111111111111111111111111111111111"
    test "$LIQUIDFUN_REGRESSION_FIX_COMMIT" = "2222222222222222222222222222222222222222"
    test "$LIQUIDFUN_REGRESSION_ORACLE_IDENTITY" = "oracle-debug@7f204021"
    test "$LIQUIDFUN_REGRESSION_TOLERANCE_IDENTITY" = "phase12-v1"
    test "$LIQUIDFUN_REGRESSION_FIRST_DIVERGENCE" = "checkpoint-1/world.bodies/exact"
    test "$LIQUIDFUN_REGRESSION_FAILURE_CLASS" = "PhysicsMismatch"
    printf 'test:%s\n' "$LIQUIDFUN_REGRESSION_ID" >> "$FAKE_EVENT_LOG"
    printf 'named regression passed\n'
    ;;
  "xtask safety-evidence validate-regression-results --candidate $FAKE_CANDIDATE --results target/phase12-regressions/$FAKE_CANDIDATE")
    evidence="$FAKE_REPOSITORY/target/phase12-regressions/$FAKE_CANDIDATE"
    test -s "$evidence/completion.json"
    test ! -e "$evidence/identity.json"
    test ! -e "$evidence/producer-identity.json"
    printf 'validate-results\n' >> "$FAKE_EVENT_LOG"
    case "$FAKE_RESULT_MODE" in
      valid) ;;
      omitted)
        jq '.results = []' "$evidence/completion.json" > "$evidence/mutated.json"
        mv "$evidence/mutated.json" "$evidence/completion.json"
        exit 9
        ;;
      duplicated)
        jq '.results += [.results[0]]' "$evidence/completion.json" > "$evidence/mutated.json"
        mv "$evidence/mutated.json" "$evidence/completion.json"
        exit 9
        ;;
      unregistered)
        jq '.results[0].regression_id = "unregistered"' "$evidence/completion.json" > "$evidence/mutated.json"
        mv "$evidence/mutated.json" "$evidence/completion.json"
        exit 9
        ;;
      *) exit 98 ;;
    esac
    completion_sha256=$(sha256sum "$evidence/completion.json" | awk '{print $1}')
    jq -n \
      --arg candidate_sha "$FAKE_CANDIDATE" \
      --arg regression_manifest_sha256 "$(printf 'a%.0s' {1..64})" \
      --arg completion_sha256 "$completion_sha256" \
      '{schema_version: 1, candidate_sha: $candidate_sha, regression_manifest_sha256: $regression_manifest_sha256, completion_sha256: $completion_sha256}' \
      > "$evidence/identity.json"
    ;;
  *) exit 99 ;;
esac
"#
    }

    #[test]
    fn valid_fixture_records_exact_execution_validation_and_identity_order() -> TestResult {
        // Arrange
        let fixture = Fixture::new(valid_entry())?;

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
        assert_eq!(producer["named_test_count"], 1);
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
        let fixtures = [
            json!([]),
            json!([duplicate.clone(), duplicate]),
            json!([wrong_hash]),
            json!([missing_oracle]),
            json!([missing_tolerance]),
            json!([wrong_fix]),
        ];

        // Act / Assert
        for execution_list in fixtures {
            let fixture = Fixture::with_entries(execution_list)?;
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
            let fixture = Fixture::new(valid_entry())?;
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
}
