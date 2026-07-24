//! Controlled-host Phase 12 performance workflow contract tests.

use std::{fs, path::Path};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CHECKOUT_ACTION: &str = "uses: actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0";
const UPLOAD_ACTION: &str =
    "uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a";
const PAIRED_COMMAND: &str = "run: scripts/phase12-performance.sh paired";
const CALIBRATE_COMMAND: &str = "run: scripts/phase12-performance.sh calibrate";
const VALIDATE_COMMAND: &str = "run: scripts/phase12-performance.sh validate";
const TYPED_VALIDATOR: &str = "cargo xtask performance validate";
const IDENTITY_PUBLICATION: &str =
    "mv -f -- \"$identity_tmp\" \"$evidence/producer-identity.json\"";

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask remains two levels below the workspace")
}

fn workflow_source() -> TestResult<String> {
    Ok(fs::read_to_string(
        workspace_root().join(".github/workflows/performance.yml"),
    )?)
}

fn actions_are_fully_pinned(source: &str) -> bool {
    let actions = source
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("uses: "))
        .collect::<Vec<_>>();
    actions.len() == 2
        && actions.iter().all(|reference| {
            reference
                .rsplit_once('@')
                .is_some_and(|(action, revision)| {
                    action.starts_with("actions/")
                        && revision.len() == 40
                        && revision.bytes().all(|byte| byte.is_ascii_hexdigit())
                })
        })
}

fn ordered_offsets(source: &str) -> Option<[usize; 7]> {
    Some([
        source.find(PAIRED_COMMAND)?,
        source.find(CALIBRATE_COMMAND)?,
        source.find(VALIDATE_COMMAND)?,
        source.rfind(TYPED_VALIDATOR)?,
        source.find("manifest-entry.json")?,
        source.find(IDENTITY_PUBLICATION)?,
        source.find(UPLOAD_ACTION)?,
    ])
}

fn workflow_contract_is_valid(source: &str) -> bool {
    let Some(offsets) = ordered_offsets(source) else {
        return false;
    };
    let ordered = offsets.windows(2).all(|pair| pair[0] < pair[1]);
    let trigger = source.split("permissions:").next().unwrap_or_default();

    ordered
        && trigger.contains("schedule:")
        && trigger.contains("workflow_dispatch:")
        && !trigger.contains("pull_request:")
        && !trigger.contains("push:")
        && trigger.contains("candidate_sha:")
        && trigger.contains("controlled_host_label:")
        && trigger.contains("controlled_host_identity:")
        && trigger.matches("required: true").count() == 3
        && source.contains("^[0-9a-f]{40}$")
        && source.contains("^[0-9a-f]{64}$")
        && source.contains("performance-controlled-linux-x64")
        && source.contains(
            "runs-on: ${{ github.event_name == 'workflow_dispatch' && inputs.controlled_host_label || 'ubuntu-24.04' }}",
        )
        && source.contains("submodules: recursive")
        && source.contains("persist-credentials: false")
        && source.contains("git symbolic-ref --quiet HEAD")
        && source.contains("test \"$(git rev-parse HEAD)\" = \"$CANDIDATE_SHA\"")
        && source.contains(
            "EVIDENCE_DISPOSITION: ${{ github.event_name == 'workflow_dispatch' && 'reviewed_controlled' || 'trend_diagnostic' }}",
        )
        && source.contains(
            "RELEASE_REVIEWED: ${{ github.event_name == 'workflow_dispatch' && 'true' || 'false' }}",
        )
        && source.contains("test \"$RELEASE_REVIEWED\" = \"false\"")
        && source.contains("test \"$EVIDENCE_DISPOSITION\" = \"trend_diagnostic\"")
        && source.contains("jq '[.cases[].workload] | unique | length'")
        && source.contains("test \"$workload_count\" = \"14\"")
        && source.contains("test \"$case_count\" = \"32\"")
        && source.contains("raw_samples")
        && source.contains("summary.json")
        && source.contains("manifest-entry.json")
        && source.contains("producer_workflow")
        && source.contains("producer_job")
        && source.contains("payload_sha256")
        && source.matches("${{ github.run_id }}").count() >= 2
        && source.contains(
            "name: phase12-performance-${{ github.run_id }}-${{ env.CANDIDATE_SHA }}",
        )
        && source.contains("path: target/phase12-performance")
        && source.contains("if-no-files-found: error")
        && source.matches(PAIRED_COMMAND).count() == 1
        && source.matches(CALIBRATE_COMMAND).count() == 1
        && source.matches(VALIDATE_COMMAND).count() == 1
        && source.matches(UPLOAD_ACTION).count() == 1
        && source.matches("uses: actions/upload-artifact@").count() == 1
        && actions_are_fully_pinned(source)
}

#[test]
fn workflow_accepts_one_candidate_bound_identity_last_artifact() -> TestResult {
    // Arrange
    let source = workflow_source()?;

    // Act
    let valid = workflow_contract_is_valid(&source);

    // Assert
    assert!(valid);
    Ok(())
}

#[test]
fn workflow_rejects_missing_or_short_candidate_sha() -> TestResult {
    // Arrange
    let source = workflow_source()?;
    let missing = source.replacen("      candidate_sha:\n", "      removed_candidate:\n", 1);
    let short = source.replace("^[0-9a-f]{40}$", "^[0-9a-f]{7,40}$");

    // Act / Assert
    assert!(!workflow_contract_is_valid(&missing));
    assert!(!workflow_contract_is_valid(&short));
    Ok(())
}

#[test]
fn workflow_rejects_absent_controlled_host_identity() -> TestResult {
    // Arrange
    let source = workflow_source()?;
    let missing = source.replace("controlled_host_identity", "removed_host_identity");

    // Act / Assert
    assert!(!workflow_contract_is_valid(&missing));
    Ok(())
}

#[test]
fn workflow_rejects_shared_host_claim_promotion() -> TestResult {
    // Arrange
    let source = workflow_source()?;
    let promoted = source.replace(
        "github.event_name == 'workflow_dispatch' && 'true' || 'false'",
        "github.event_name == 'schedule' && 'true' || 'false'",
    );

    // Act / Assert
    assert!(!workflow_contract_is_valid(&promoted));
    Ok(())
}

#[test]
fn workflow_rejects_validation_after_upload() -> TestResult {
    // Arrange
    let source = workflow_source()?;
    let delayed = format!("{UPLOAD_ACTION}\n{source}");

    // Act / Assert
    assert!(!workflow_contract_is_valid(&delayed));
    Ok(())
}

#[test]
fn workflow_rejects_missing_run_id() -> TestResult {
    // Arrange
    let source = workflow_source()?;
    let missing = source.replace("${{ github.run_id }}", "missing-run-id");

    // Act / Assert
    assert!(!workflow_contract_is_valid(&missing));
    Ok(())
}

#[test]
fn workflow_rejects_floating_action_tags() -> TestResult {
    // Arrange
    let source = workflow_source()?;
    let floating = source.replace(CHECKOUT_ACTION, "uses: actions/checkout@v7");

    // Act / Assert
    assert!(!workflow_contract_is_valid(&floating));
    Ok(())
}

#[test]
fn workflow_rejects_wrong_runner_order() -> TestResult {
    // Arrange
    let source = workflow_source()?;
    let wrong = source
        .replace(PAIRED_COMMAND, "run: temporary-order-marker")
        .replace(CALIBRATE_COMMAND, PAIRED_COMMAND)
        .replace("run: temporary-order-marker", CALIBRATE_COMMAND);

    // Act / Assert
    assert!(!workflow_contract_is_valid(&wrong));
    Ok(())
}

#[test]
fn workflow_rejects_wrong_artifact_cardinality() -> TestResult {
    // Arrange
    let source = workflow_source()?;
    let duplicate = format!("{source}\n{UPLOAD_ACTION}\n");

    // Act / Assert
    assert!(!workflow_contract_is_valid(&duplicate));
    Ok(())
}
