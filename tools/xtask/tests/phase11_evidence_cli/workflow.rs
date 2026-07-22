//! Phase 11 evidence shell, workflow, and documentation contracts.

use std::fs;

use super::{TestResult, workspace_root};

fn read(relative: &str) -> TestResult<String> {
    Ok(fs::read_to_string(workspace_root().join(relative))?)
}

fn job_section(workflow: &str, job: &str) -> TestResult<String> {
    let header = format!("  {job}:");
    let mut found = false;
    let mut lines = Vec::new();

    for line in workflow.lines() {
        if line == header {
            found = true;
        } else if found
            && line.starts_with("  ")
            && !line.starts_with("    ")
            && line.ends_with(':')
        {
            break;
        }

        if found {
            lines.push(line);
        }
    }

    if lines.is_empty() {
        return Err(std::io::Error::other(format!("workflow job `{job}` is missing")).into());
    }
    Ok(lines.join("\n"))
}

fn assert_exact_job_condition(section: &str, expected: &str) {
    let conditions = section
        .lines()
        .filter_map(|line| line.strip_prefix("    if: "))
        .collect::<Vec<_>>();
    assert_eq!(conditions, [expected]);
}

fn assert_actions_are_pinned(workflow: &str) {
    let mut actions = 0;
    for usage in workflow
        .lines()
        .filter_map(|line| line.trim().strip_prefix("uses: "))
    {
        let (action, revision) = usage
            .split_once('@')
            .expect("action use carries a revision");
        assert!(
            action.starts_with("actions/"),
            "unexpected action `{action}`"
        );
        assert_eq!(
            revision.len(),
            40,
            "action `{action}` is not pinned by full SHA"
        );
        assert!(
            revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "action `{action}` has a non-hex revision"
        );
        actions += 1;
    }
    assert!(actions > 0, "workflow contains no pinned actions");
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
    let canonical = job_section(&workflow, "canonical-linux")?;
    let sanitizer = job_section(&workflow, "sanitizer-linux")?;
    let phase11_canonical = job_section(&workflow, "phase11-canonical-linux")?;
    let phase11_sanitizer = job_section(&workflow, "phase11-sanitizer-linux")?;
    let macos = job_section(&workflow, "portability-macos")?;
    let windows = job_section(&workflow, "portability-windows")?;

    // Act
    let phase11_calls = workflow.matches("scripts/phase11-evidence.sh").count();

    // Assert
    assert!(workflow.contains("- phase11"));
    assert!(workflow.contains("Phase 11 canonical Linux oracle"));
    assert!(workflow.contains("Phase 11 fail-fast sanitizer"));
    assert_eq!(phase11_calls, 2);
    assert_eq!(
        phase11_canonical
            .matches("run: bash scripts/phase11-evidence.sh canonical target/oracle-evidence/phase11-canonical")
            .count(),
        1
    );
    assert_eq!(
        phase11_sanitizer
            .matches("run: bash scripts/phase11-evidence.sh sanitizer target/oracle-evidence/phase11-sanitizer")
            .count(),
        1
    );
    assert!(
        phase11_canonical
            .contains("name: phase11-canonical-${{ github.run_id }}-${{ github.sha }}")
    );
    assert!(
        phase11_sanitizer
            .contains("name: phase11-sanitizer-${{ github.run_id }}-${{ github.sha }}")
    );

    let legacy_phases = "(inputs.evidence_phase == 'phase8' || inputs.evidence_phase == 'phase9' || inputs.evidence_phase == 'phase10')";
    assert_exact_job_condition(
        &canonical,
        &format!(
            "github.event_name == 'pull_request' || github.event_name == 'push' || github.event_name == 'schedule' || (github.event_name == 'workflow_dispatch' && {legacy_phases})"
        ),
    );
    assert_exact_job_condition(
        &sanitizer,
        &format!(
            "github.event_name == 'schedule' || (github.event_name == 'workflow_dispatch' && {legacy_phases})"
        ),
    );
    let phase11_route = "github.event_name == 'schedule' || (github.event_name == 'workflow_dispatch' && inputs.evidence_phase == 'phase11')";
    assert_exact_job_condition(&phase11_canonical, phase11_route);
    assert_exact_job_condition(&phase11_sanitizer, phase11_route);
    let portability_route = format!("github.event_name == 'workflow_dispatch' && {legacy_phases}");
    assert_exact_job_condition(&macos, &portability_route);
    assert_exact_job_condition(&windows, &portability_route);
    assert!(!workflow.contains("inputs.evidence_phase != 'phase11'"));
    assert_actions_are_pinned(&workflow);
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
