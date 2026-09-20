use std::fs;

use super::{TestResult, workspace_root};

#[test]
fn repository_text_is_forced_to_lf_for_cross_platform_evidence() -> TestResult {
    // Arrange
    let attributes_path = workspace_root().join(".gitattributes");

    // Act
    let attributes = fs::read_to_string(attributes_path)?;

    // Assert
    assert!(
        attributes
            .lines()
            .any(|line| line.trim() == "* text=auto eol=lf"),
        "repository text must retain LF bytes on Windows"
    );
    Ok(())
}

#[test]
fn upstream_clang_compatibility_covers_deprecated_copy_warning() -> TestResult {
    // Arrange
    let cmake_path = workspace_root().join("tools/reference/CMakeLists.txt");

    // Act
    let cmake = fs::read_to_string(cmake_path)?;

    // Assert
    assert!(
        cmake.contains("-Wno-error=deprecated-copy-with-user-provided-copy"),
        "the read-only upstream target must tolerate modern Clang's deprecated-copy diagnostic"
    );
    Ok(())
}
