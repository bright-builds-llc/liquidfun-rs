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

fn workflow_source() -> TestResult<String> {
    Ok(fs::read_to_string(
        workspace_root().join(".github/workflows/regressions.yml"),
    )?)
}

fn actions_are_fully_pinned(source: &str) -> bool {
    let action_references = source
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("uses: "))
        .collect::<Vec<_>>();
    action_references.len() == 3
        && action_references.iter().all(|reference| {
            reference
                .rsplit_once('@')
                .is_some_and(|(_action, revision)| {
                    revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit())
                })
        })
}

fn workflow_contract_is_valid(source: &str) -> bool {
    const PRODUCER: &str = "run: scripts/phase12-regressions.sh run \"$CANDIDATE_SHA\"";
    const VERIFY: &str = "- name: Verify typed identity-last regression evidence";
    const UPLOAD: &str = "uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a";
    let Some((producer, verify, upload)) = source
        .find(PRODUCER)
        .zip(source.find(VERIFY))
        .zip(source.find(UPLOAD))
        .map(|((producer, verify), upload)| (producer, verify, upload))
    else {
        return false;
    };

    producer < verify
        && verify < upload
        && source.matches(PRODUCER).count() == 1
        && source.matches("uses: actions/upload-artifact@").count() == 2
        && source.matches("timeout-minutes:").count() == 1
        && source.contains("candidate_sha:")
        && source.contains("required: true")
        && source.contains("^[0-9a-f]{40}$")
        && source.contains("${{ github.run_id }}")
        && source.contains("persist-credentials: false")
        && source.contains("submodules: false")
        && source.contains(VALIDATOR_COMMAND)
        && source.contains("name: phase12-regressions-${{ env.CANDIDATE_SHA }}")
        && source.contains("path: target/phase12-regressions/${{ env.CANDIDATE_SHA }}")
        && source.contains("if: always()")
        && source.contains("name: phase12-regression-diagnostics-${{ github.run_id }}-${{ github.run_attempt }}-${{ env.CANDIDATE_SHA }}")
        && source.contains("path: target/phase12-regression-diagnostics/${{ env.CANDIDATE_SHA }}")
        && !source.contains("target/fuzz")
        && !source.contains("phase12-sanitizer")
        && !source.contains("phase12-coverage")
        && !source.contains("retry")
        && actions_are_fully_pinned(source)
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
fn workflow_projection_accepts_one_candidate_scoped_upload_after_identity() -> TestResult {
    // Arrange
    let source = workflow_source()?;

    // Act
    let valid = workflow_contract_is_valid(&source);

    // Assert
    assert!(valid);
    Ok(())
}

#[test]
fn workflow_projection_rejects_candidate_pin_order_and_cardinality_mutations() -> TestResult {
    // Arrange
    let source = workflow_source()?;
    let short_candidate = source.replace("^[0-9a-f]{40}$", "^[0-9a-f]{7,40}$");
    let floating_action = source.replace(
        "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0",
        "actions/checkout@v7",
    );
    let missing_timeout = source.replace("timeout-minutes:", "removed-timeout:");
    let missing_run_id = source.replace("${{ github.run_id }}", "missing-run-id");
    let duplicate_producer =
        format!("{source}\nrun: scripts/phase12-regressions.sh run \"$CANDIDATE_SHA\"\n");
    let upload_before_identity =
        format!("uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a\n{source}");
    let merged_upload = source.replace(
        "path: target/phase12-regressions/${{ env.CANDIDATE_SHA }}",
        "path: |\n            target/phase12-regressions/${{ env.CANDIDATE_SHA }}\n            target/fuzz-evidence",
    );
    let unscoped_name = source.replace(
        "name: phase12-regressions-${{ env.CANDIDATE_SHA }}",
        "name: phase12-regressions",
    );

    // Act / Assert
    for invalid in [
        short_candidate,
        floating_action,
        missing_timeout,
        missing_run_id,
        duplicate_producer,
        upload_before_identity,
        merged_upload,
        unscoped_name,
    ] {
        assert!(!workflow_contract_is_valid(&invalid));
    }
    Ok(())
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
#[path = "regression_workflow/unix.rs"]
mod unix;
