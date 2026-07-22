//! Phase 11 evidence shell, workflow, and documentation contracts.

use std::fs;

use super::{TestResult, workspace_root};

fn read(relative: &str) -> TestResult<String> {
    Ok(fs::read_to_string(workspace_root().join(relative))?)
}

#[test]
fn local_runner_is_fixed_fail_fast_and_identity_last() -> TestResult {
    // Arrange
    let script = read("scripts/phase11-evidence.sh")?;

    // Act
    let validation = script.find("phase11-evidence validate-content");
    let identity = script.rfind("identity.json\"");

    // Assert
    assert!(script.starts_with("#!/usr/bin/env bash\nset -euo pipefail\n"));
    assert!(script.contains("canonical | sanitizer"));
    assert!(script.contains("cargo xtask upstream verify"));
    assert!(script.contains("phase11-evidence render-records"));
    assert!(script.contains("oracle-debug"));
    assert!(script.contains("oracle-release"));
    assert!(script.contains("oracle-asan-ubsan"));
    assert!(script.contains("cargo_arguments+=(--release)"));
    assert!(validation.is_some_and(|offset| identity.is_some_and(|last| offset < last)));
    Ok(())
}

#[test]
fn oracle_workflow_produces_one_same_run_phase11_pair() -> TestResult {
    // Arrange
    let workflow = read(".github/workflows/oracle.yml")?;

    // Act
    let phase11_calls = workflow.matches("scripts/phase11-evidence.sh").count();

    // Assert
    assert!(workflow.contains("- phase11"));
    assert!(workflow.contains("Phase 11 canonical Linux oracle"));
    assert!(workflow.contains("Phase 11 fail-fast sanitizer"));
    assert_eq!(phase11_calls, 2);
    assert!(workflow.contains("phase11-canonical-${{ github.run_id }}-${{ github.sha }}"));
    assert!(workflow.contains("phase11-sanitizer-${{ github.run_id }}-${{ github.sha }}"));
    assert!(!workflow.contains("uses: actions/checkout@v"));
    assert!(!workflow.contains("uses: actions/upload-artifact@v"));
    Ok(())
}

#[test]
fn testing_guide_preserves_phase11_authority_boundaries() -> TestResult {
    // Arrange
    let guide = read("TESTING.md")?;

    // Act
    let maybe_phase11 = guide.split("## Phase 11 local evidence generation").nth(1);

    // Assert
    let phase11 = maybe_phase11
        .ok_or("Phase 11 evidence guidance is missing")?
        .to_ascii_lowercase();
    assert!(phase11.contains("d2"));
    assert!(phase11.contains("non-promotable"));
    assert!(phase11.contains("same-run d1"));
    assert!(phase11.contains("screenshots"));
    assert!(phase11.contains("wall-clock"));
    assert!(phase11.contains("acquisition"));
    assert!(phase11.contains("promotion"));
    Ok(())
}
