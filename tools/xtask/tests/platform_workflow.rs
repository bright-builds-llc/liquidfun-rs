//! Artifact-first Phase 12 platform workflow contracts.

use std::fs;
use std::path::{Path, PathBuf};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask must live under tools/xtask")
        .to_path_buf()
}

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

fn assert_actions_are_pinned(workflow: &str) {
    let action_references = workflow
        .lines()
        .filter_map(|line| line.trim().strip_prefix("uses: "))
        .collect::<Vec<_>>();

    assert!(!action_references.is_empty());
    for reference in action_references {
        let Some((action, revision)) = reference.split_once('@') else {
            panic!("action reference `{reference}` is missing a revision");
        };
        assert!(action.starts_with("actions/"));
        assert_eq!(revision.len(), 40, "{action} must use a full commit SHA");
        assert!(revision.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
}

#[test]
fn platform_workflow_fans_out_one_exact_archive() -> TestResult {
    // Arrange
    let workflow = read(".github/workflows/platform.yml")?;
    let producer = job_section(&workflow, "package")?;
    let msrv = job_section(&workflow, "msrv")?;
    let native = job_section(&workflow, "native")?;
    let conditional = job_section(&workflow, "conditional-macos-intel")?;

    // Act
    let package_commands = workflow
        .matches("cargo xtask package create-artifact")
        .count();
    let consumer_commands = workflow.matches("scripts/phase12-platform.sh").count();

    // Assert
    assert_eq!(package_commands, 1);
    assert_eq!(consumer_commands, 3);
    assert!(producer.contains("runs-on: ubuntu-24.04"));
    assert!(producer.contains("candidate-commit \"$CANDIDATE_SHA\""));
    assert!(producer.contains("submodules: false"));
    assert!(msrv.contains("runs-on: ubuntu-24.04"));
    assert!(msrv.contains("RUSTUP_TOOLCHAIN: 1.92.0"));
    assert!(native.contains("RUSTUP_TOOLCHAIN: 1.97.0"));
    assert!(conditional.contains("runs-on: macos-15-intel"));
    assert!(conditional.contains("RUSTUP_TOOLCHAIN: 1.97.0"));
    assert!(!msrv.contains("cargo package"));
    assert!(!native.contains("cargo package"));
    assert!(!conditional.contains("cargo package"));
    assert_eq!(
        workflow
            .matches("actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a")
            .count(),
        5
    );
    assert_eq!(workflow.matches("actions/download-artifact@").count(), 3);
    assert_actions_are_pinned(&workflow);
    Ok(())
}

#[test]
fn native_matrix_names_exact_supported_runners_and_targets() -> TestResult {
    // Arrange
    let workflow = read(".github/workflows/platform.yml")?;
    let native = job_section(&workflow, "native")?;

    // Act
    let expected_pairs = [
        ("ubuntu-24.04", "x86_64-unknown-linux-gnu"),
        ("ubuntu-24.04-arm", "aarch64-unknown-linux-gnu"),
        ("macos-15", "aarch64-apple-darwin"),
        ("windows-2025", "x86_64-pc-windows-msvc"),
    ];

    // Assert
    for (runner, target) in expected_pairs {
        assert!(native.contains(&format!("runner: {runner}")));
        assert!(native.contains(&format!("target: {target}")));
    }
    assert!(workflow.contains("phase12-platform.sh x86_64-apple-darwin d2_supported"));
    assert!(!workflow.contains("macos-13"));
    assert!(!workflow.contains("macos-14-large"));
    Ok(())
}

#[test]
fn conditional_support_has_a_distinct_freshness_and_downgrade_path() -> TestResult {
    // Arrange
    let workflow = read(".github/workflows/platform.yml")?;
    let policy = job_section(&workflow, "conditional-policy")?;
    let conditional = job_section(&workflow, "conditional-macos-intel")?;
    let downgrade = job_section(&workflow, "conditional-downgrade")?;

    // Act
    let support_references = workflow.matches("reference/platform/support.json").count();

    // Assert
    assert!(support_references >= 3);
    assert!(policy.contains("max_age_days"));
    assert!(policy.contains("recorded_at_unix"));
    assert!(policy.contains("expires_at_unix"));
    assert!(policy.contains("macos-15-intel"));
    assert!(policy.contains("90"));
    assert!(conditional.contains("needs: [package, conditional-policy]"));
    assert!(conditional.contains("needs.conditional-policy.outputs.available == 'true'"));
    assert!(downgrade.contains("needs: conditional-policy"));
    assert!(downgrade.contains("needs.conditional-policy.outputs.available != 'true'"));
    assert!(downgrade.contains("tier: \"unsupported\""));
    assert!(downgrade.contains("reason: \"missing_or_expired_native_evidence\""));
    assert!(downgrade.contains("recorded_at_unix"));
    assert!(downgrade.contains("support_sha256"));
    Ok(())
}

#[test]
fn platform_script_is_closed_d2_only_and_identity_last() -> TestResult {
    // Arrange
    let script = read("scripts/phase12-platform.sh")?;

    // Act
    let verify = script.find("package verify-artifact");
    let documentation = script.find("cargo doc");
    let smoke = script.find("cargo test");
    let identity = script.rfind("identity.json");
    let lowered = script.to_ascii_lowercase();

    // Assert
    assert!(script.starts_with("#!/usr/bin/env bash\nset -euo pipefail\n"));
    assert!(script.contains("[[ $# -eq 4 ]]"));
    assert!(script.contains("<target> <tier> <archive> <identity>"));
    assert!(script.contains("d2_supported"));
    assert!(script.contains("archive_sha256"));
    assert!(script.contains("candidate_commit"));
    assert!(script.contains("strict_f32"));
    assert!(script.contains("rustc --version"));
    assert!(script.contains("reference/platform/support.json"));
    assert!(script.contains("GITHUB_WORKFLOW"));
    assert!(script.contains("GITHUB_RUN_ID"));
    assert!(script.contains("GITHUB_JOB"));
    assert!(script.contains("cd -- \"$unpacked_crate\""));
    assert!(verify.is_some_and(|offset| identity.is_some_and(|last| offset < last)));
    assert!(documentation.is_some_and(|offset| identity.is_some_and(|last| offset < last)));
    assert!(smoke.is_some_and(|offset| identity.is_some_and(|last| offset < last)));
    assert!(!lowered.contains("d1"));
    assert!(!lowered.contains("promotion"));
    assert!(!lowered.contains("fixture"));
    Ok(())
}

#[test]
fn platform_workflow_is_release_candidate_only_and_submodule_free() -> TestResult {
    // Arrange
    let workflow = read(".github/workflows/platform.yml")?;

    // Act
    let trigger = workflow
        .split("permissions:")
        .next()
        .ok_or("workflow permissions marker is missing")?;

    // Assert
    assert!(trigger.contains("schedule:"));
    assert!(trigger.contains("workflow_dispatch:"));
    assert!(!trigger.contains("pull_request:"));
    assert!(!trigger.contains("push:"));
    assert!(workflow.contains("candidate_sha:"));
    assert!(workflow.contains("^[0-9a-f]{40}$"));
    assert!(workflow.matches("submodules: false").count() >= 4);
    assert!(
        !workflow
            .to_ascii_lowercase()
            .contains("submodules: recursive")
    );
    Ok(())
}

#[test]
fn pull_request_ci_is_linux_quality_plus_mainstream_smokes() -> TestResult {
    // Arrange
    let workflow = read(".github/workflows/ci.yml")?;

    // Act
    let jobs_section = workflow
        .split_once("jobs:\n")
        .map(|(_, jobs)| jobs)
        .ok_or("CI jobs section is missing")?;
    let jobs = jobs_section
        .lines()
        .filter(|line| line.starts_with("  ") && !line.starts_with("    ") && line.ends_with(':'))
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(jobs, ["  quality:", "  default-features:"]);
    assert!(workflow.contains("runs-on: ubuntu-24.04"));
    assert!(workflow.contains("os: [ubuntu-24.04, macos-15, windows-2025]"));
    assert!(workflow.matches("submodules: false").count() >= 2);
    assert!(!workflow.contains("cargo package"));
    assert!(!workflow.contains("1.92.0"));
    assert_actions_are_pinned(&workflow);
    Ok(())
}
