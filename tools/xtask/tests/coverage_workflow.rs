//! Clean-lane contracts for Phase 12 differential semantic coverage.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

static TEST_ORDINAL: AtomicU64 = AtomicU64::new(1);

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask remains two levels below the workspace")
}

fn differential_job(source: &str) -> Option<&str> {
    source
        .split_once("  differential-coverage:\n")
        .map(|(_, job)| job)
}

fn mutate_differential_job(source: &str, before: &str, after: &str) -> String {
    let (prefix, job) = source
        .split_once("  differential-coverage:\n")
        .expect("coverage workflow should contain the differential job");
    format!(
        "{prefix}  differential-coverage:\n{}",
        job.replacen(before, after, 1)
    )
}

fn workflow_contract_is_valid(source: &str) -> bool {
    let Some(job) = differential_job(source) else {
        return false;
    };
    let required_order = [
        "submodules: recursive",
        "clang++-22 --version",
        "cmake --version",
        "ninja --version",
        "cargo xtask upstream configure --preset oracle-debug",
        "cargo xtask upstream build --preset oracle-debug",
        "cargo xtask upstream configure --preset oracle-release",
        "cargo xtask upstream build --preset oracle-release",
        "scripts/phase12-coverage.sh differential",
    ];
    let mut previous = 0;
    for required in required_order {
        let Some(offset) = job.find(required) else {
            return false;
        };
        if offset < previous {
            return false;
        }
        previous = offset;
    }
    job.contains("timeout-minutes: 120")
        && job.contains("submodules: recursive")
        && !job.contains("submodules: false")
        && job.contains("9474ecd78b52aba6e923976b1e9773f5613027cc7e237b9956986cb536e02a36")
        && job.contains("927b2368a946c37269c3a66225ab00544e756459cdd0b5d0da438694fb9ff802")
        && job.contains("5749cbc4e668273514150a80e387a957f933c6ed3f5f11e03fb30955e2bbead6")
        && job.contains("cmake version 4.3.3")
        && job.contains("ninja --version | grep --fixed-strings \"1.13.2\"")
        && job.contains("clang version 22\\.1\\.8")
        && job.contains("LIQUIDFUN_XTASK_CXX: clang++-22")
}

fn producer_contract_is_valid(script: &str, round_trip: &str) -> bool {
    let Some((_, differential_run)) = script.split_once("run_differential_coverage() {") else {
        return false;
    };
    let Some(prerequisite) = differential_run.find("\trequire_differential_oracles") else {
        return false;
    };
    let Some(target_loop) = differential_run.find("\tfor target in") else {
        return false;
    };
    let Some(marker_guard) = round_trip.find("LIQUIDFUN_DIFFERENTIAL_LEAF_DIRECTORY\").is_none()")
    else {
        return false;
    };
    let Some(marker_emission) = round_trip.find("coverage_observation::observe") else {
        return false;
    };
    prerequisite < target_loop
        && marker_guard < marker_emission
        && script.contains("oracle-debug oracle-release")
        && script.contains("differential coverage requires the exact $preset oracle")
}

#[test]
fn clean_differential_job_builds_every_math_oracle_before_the_producer() -> TestResult {
    // Arrange
    let source = fs::read_to_string(workspace_root().join(".github/workflows/coverage.yml"))?;

    // Act
    let valid = workflow_contract_is_valid(&source);

    // Assert
    assert!(valid);
    Ok(())
}

#[test]
fn clean_differential_job_rejects_missing_upstream_toolchain_or_release_build() -> TestResult {
    // Arrange
    let source = fs::read_to_string(workspace_root().join(".github/workflows/coverage.yml"))?;
    let no_upstream =
        mutate_differential_job(&source, "submodules: recursive", "submodules: false");
    let no_toolchain =
        mutate_differential_job(&source, "LIQUIDFUN_XTASK_CXX: clang++-22", "CXX: c++");
    let no_release = mutate_differential_job(
        &source,
        "cargo xtask upstream build --preset oracle-release",
        "cargo xtask upstream verify",
    );

    // Act / Assert
    assert!(!workflow_contract_is_valid(&no_upstream));
    assert!(!workflow_contract_is_valid(&no_toolchain));
    assert!(!workflow_contract_is_valid(&no_release));
    Ok(())
}

#[test]
fn marker_producer_and_math_test_fail_closed_without_oracles() -> TestResult {
    // Arrange
    let script = fs::read_to_string(workspace_root().join("scripts/phase12-coverage.sh"))?;
    let round_trip = fs::read_to_string(
        workspace_root().join("crates/liquidfun-differential/tests/round_trip.rs"),
    )?;

    // Act
    let valid = producer_contract_is_valid(&script, &round_trip);

    // Assert
    assert!(valid);
    Ok(())
}

#[test]
#[cfg(unix)]
fn clean_producer_cannot_skip_a_missing_math_oracle() -> TestResult {
    // Arrange
    let ordinal = TEST_ORDINAL.fetch_add(1, Ordering::Relaxed);
    let reference_root = workspace_root()
        .join("target/xtask-coverage-workflow")
        .join(format!("{}-{ordinal}", std::process::id()));
    fs::create_dir_all(&reference_root)?;

    // Act
    let output = std::process::Command::new("bash")
        .current_dir(workspace_root())
        .env("PHASE12_COVERAGE_LIBRARY_ONLY", "1")
        .args([
            "-c",
            "source scripts/phase12-coverage.sh; require_differential_oracles \"$1\"",
            "coverage-prerequisite-test",
        ])
        .arg(&reference_root)
        .output()?;
    fs::remove_dir_all(&reference_root)?;

    // Assert
    assert_eq!(output.status.code(), Some(64));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("differential coverage requires the exact oracle-debug oracle")
    );
    Ok(())
}
