//! Candidate-bound Phase 13.1 canonical native workflow contract.

use std::{fs, path::PathBuf};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const CHECKOUT: &str = "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0";
const UPLOAD: &str = "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a";
const WORKFLOW: &str = ".github/workflows/phase13-1-canonical-native.yml";

#[test]
fn workflow_binds_checkout_to_one_validated_candidate() -> TestResult {
    // Arrange
    let source = workflow_source()?;

    // Act
    let validation = source.find("^[0-9a-f]{40}$");
    let checkout = source.find(CHECKOUT);
    let head_assertion = source.find("git rev-parse HEAD");

    // Assert
    assert!(source.contains("name: Phase 13.1 Canonical Native Validation"));
    assert!(source.contains("candidate_sha:"));
    assert!(source.contains("required: true"));
    assert!(source.contains("type: string"));
    assert!(source.contains("permissions:\n  contents: read"));
    assert!(source.contains("cancel-in-progress: false"));
    assert!(source.contains("runs-on: ubuntu-24.04"));
    assert!(source.contains("ref: ${{ inputs.candidate_sha }}"));
    assert!(source.contains("fetch-depth: 0"));
    assert!(source.contains("persist-credentials: false"));
    assert!(source.contains("submodules: recursive"));
    assert_before(validation, checkout, "candidate validation before checkout")?;
    assert_before(checkout, head_assertion, "checkout before HEAD assertion")?;
    assert!(source.contains("git rev-parse 'HEAD^{tree}'"));
    Ok(())
}

#[test]
fn workflow_requires_the_exact_canonical_tool_identity() -> TestResult {
    // Arrange
    let source = workflow_source()?;

    // Act
    let upload_count = source.matches(UPLOAD).count();

    // Assert
    assert!(source.contains("CC: clang-22"));
    assert!(source.contains("CXX: clang++-22"));
    assert!(source.contains("LIQUIDFUN_XTASK_CMAKE: cmake"));
    assert!(source.contains("LIQUIDFUN_XTASK_NINJA: ninja"));
    assert!(source.contains("LIQUIDFUN_XTASK_CXX: clang++-22"));
    assert!(source.contains("rustup toolchain install 1.97.0 --profile minimal"));
    assert!(source.contains("rustc 1.97.0"));
    assert!(source.contains("clang version 22\\.1\\.8"));
    assert!(source.contains("cmake version 4.3.3"));
    assert!(source.contains("test \"$(ninja --version)\" = \"1.13.2\""));
    assert!(source.contains("927b2368a946c37269c3a66225ab00544e756459cdd0b5d0da438694fb9ff802"));
    assert!(source.contains("5749cbc4e668273514150a80e387a957f933c6ed3f5f11e03fb30955e2bbead6"));
    assert!(source.contains("9474ecd78b52aba6e923976b1e9773f5613027cc7e237b9956986cb536e02a36"));
    assert!(!source.contains("AppleClang"));
    assert_eq!(upload_count, 2);
    Ok(())
}

#[test]
fn workflow_runs_the_closed_native_matrix_in_order() -> TestResult {
    // Arrange
    let source = workflow_source()?;
    let commands = [
        "cargo test -p liquidfun-differential --test oracle_identity --all-features",
        "cargo test -p liquidfun-test-protocol --all-features canonical",
        "cargo xtask upstream configure --preset oracle-debug",
        "cargo xtask upstream build --preset oracle-debug",
        "cmake --build target/reference/oracle-debug --target liquidfun-reference-protocol-tests",
        "ctest --test-dir target/reference/oracle-debug --output-on-failure --no-tests=error -R '^liquidfun-reference-protocol$'",
        "cargo xtask upstream configure --preset oracle-release",
        "cargo xtask upstream build --preset oracle-release",
        "cmake --build target/reference/oracle-release --target liquidfun-reference-protocol-tests",
        "ctest --test-dir target/reference/oracle-release --output-on-failure --no-tests=error -R '^liquidfun-reference-protocol$'",
        "cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot",
        "cargo xtask differential compare --scenario rigid-world --preset oracle-release --session-profile one-shot",
        "cargo xtask differential replay --scenario rigid-world --preset oracle-debug --session-profile one-shot",
        "cargo xtask differential verify-determinism --scenario rigid-world --preset oracle-debug --runs 2",
    ];

    // Act
    let mut previous = None;
    for command in commands {
        let current = source
            .find(command)
            .ok_or_else(|| format!("workflow command is missing: {command}"))?;
        if let Some(previous) = previous {
            assert!(
                previous < current,
                "workflow command order changed at {command}"
            );
        }
        previous = Some(current);
    }

    // Assert
    let cmake = fs::read_to_string(workspace_root().join("tools/reference/CMakeLists.txt"))?;
    assert!(cmake.contains("-Werror"));
    assert!(cmake.contains("REFERENCE_FORBIDDEN_FP_PATTERN"));
    assert!(cmake.contains("REFERENCE_HAS_FORBIDDEN_FP_FLAGS"));
    assert!(!source.contains("CXXFLAGS:"));
    assert!(!source.contains("-Wno-error"));
    assert!(!source.contains("-ffast-math"));
    Ok(())
}

#[test]
fn workflow_publishes_identity_only_after_success() -> TestResult {
    // Arrange
    let source = workflow_source()?;

    // Act
    let matrix = source.find("name: Run the canonical native matrix");
    let compile_commands = source.find("name: Record normalized compile commands");
    let identity = source.find("name: Write terminal D1 identity");
    let success_upload = source.find("name: Upload successful canonical evidence");

    // Assert
    assert_before(
        matrix,
        compile_commands,
        "matrix before compile-command record",
    )?;
    assert_before(
        compile_commands,
        identity,
        "compile-command record before identity",
    )?;
    assert_before(identity, success_upload, "identity before success upload")?;
    assert!(source.contains("evidence_tier: \"D1\""));
    assert!(source.contains("candidate_sha: $candidate_sha"));
    assert!(source.contains("candidate_tree: $candidate_tree"));
    assert!(source.contains("command_order"));
    assert!(source.contains("command_exits"));
    assert!(source.contains("compile-commands.sha256"));
    assert!(source.contains("logs.sha256"));
    assert!(source.contains("if: failure()"));
    assert!(source.contains("if: success()"));
    assert!(source.contains("phase13-1-canonical-native-failure-"));
    assert!(source.contains("phase13-1-canonical-native-success-"));
    assert!(source.contains("FAILURE_DIRECTORY=$failure_directory"));
    assert!(source.contains("path: ${{ runner.temp }}/phase13-1-canonical-native-failure"));
    assert!(source.contains("path: target/phase13-1-canonical-native/${{ inputs.candidate_sha }}"));
    let failure_step = source
        .split("name: Upload bounded failure logs")
        .nth(1)
        .ok_or("failure upload step is missing")?;
    let failure_step = failure_step
        .split("- name:")
        .next()
        .ok_or("failure upload step is malformed")?;
    assert!(!failure_step.contains("identity.json"));
    Ok(())
}

fn workflow_source() -> Result<String, Box<dyn std::error::Error>> {
    Ok(fs::read_to_string(workspace_root().join(WORKFLOW))?)
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root must be present")
        .to_path_buf()
}

fn assert_before(
    maybe_first: Option<usize>,
    maybe_second: Option<usize>,
    label: &str,
) -> TestResult {
    let first = maybe_first.ok_or_else(|| format!("missing first marker for {label}"))?;
    let second = maybe_second.ok_or_else(|| format!("missing second marker for {label}"))?;
    assert!(first < second, "unexpected ordering: {label}");
    Ok(())
}
