//! Executable fail-closed contract for canonical compiler provisioning.

#[cfg(unix)]
#[path = "canonical_toolchain_workflow/execution.rs"]
mod execution;

use std::{fs, path::PathBuf};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const INSTALLER: &str = "scripts/install-canonical-clang.sh";
const UPSTREAM_COMMIT: &str = "eeed6742908255f0eeb12bb8e314366eff3c0a21";
const UPSTREAM_SHA256: &str = "9474ecd78b52aba6e923976b1e9773f5613027cc7e237b9956986cb536e02a36";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("xtask belongs to the workspace")
        .to_path_buf()
}

#[test]
fn acquisition_has_one_immutable_reviewed_source() -> TestResult {
    // Arrange
    let source = fs::read_to_string(root().join(INSTALLER))?;
    // Act
    let expected_url = format!(
        "https://raw.githubusercontent.com/opencollab/llvm-jenkins.debian.net/{UPSTREAM_COMMIT}/llvm.sh"
    );
    // Assert
    assert!(source.contains(&expected_url));
    assert!(source.contains(UPSTREAM_SHA256));
    assert!(source.contains("sha256sum --check --strict"));
    assert!(!source.contains("https://apt.llvm.org/llvm.sh"));
    Ok(())
}

#[test]
fn all_five_workflows_delegate_every_canonical_install() -> TestResult {
    for name in [
        "oracle",
        "coverage",
        "phase13-evidence-producer",
        "phase13-acceptance",
        "phase13-1-canonical-native",
    ] {
        // Arrange
        let source = fs::read_to_string(root().join(format!(".github/workflows/{name}.yml")))?;
        // Act
        let steps = source.matches("name: Install canonical LLVM 22").count();
        let invocations = source
            .matches("bash scripts/install-canonical-clang.sh")
            .count();
        // Assert
        assert!(steps > 0);
        assert_eq!(steps, invocations, "{name} must delegate every install");
        assert!(!source.contains("/llvm.sh"));
        assert!(source.contains("clang version 22\\.1\\.8"));
    }
    Ok(())
}

#[test]
fn canonical_native_builds_all_four_presets_without_modifying_upstream() -> TestResult {
    // Arrange
    let workflow =
        fs::read_to_string(root().join(".github/workflows/phase13-1-canonical-native.yml"))?;
    let presets: serde_json::Value =
        serde_json::from_slice(&fs::read(root().join("tools/reference/CMakePresets.json"))?)?;
    // Act and Assert
    for preset in [
        "oracle-debug",
        "oracle-release",
        "oracle-asan-ubsan",
        "upstream-tests",
    ] {
        assert!(workflow.contains(&format!("cargo xtask upstream configure --preset {preset}")));
        assert!(workflow.contains(&format!("cargo xtask upstream build --preset {preset}")));
        assert!(
            presets["configurePresets"]
                .as_array()
                .ok_or("configure presets required")?
                .iter()
                .any(|value| value["name"] == preset)
        );
    }
    assert!(workflow.contains("cargo xtask upstream verify"));
    assert!(workflow.contains(
        "ctest --test-dir target/reference/upstream-tests --output-on-failure --no-tests=error"
    ));
    Ok(())
}
