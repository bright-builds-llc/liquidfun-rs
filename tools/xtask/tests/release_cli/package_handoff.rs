use super::*;

#[test]
fn package_import_preserves_bytes_and_rejects_substitution() {
    // Arrange
    let root = producer_fixture_root("package-import");
    let script = repository_root().join("tools/xtask/tests/release_cli/package_handoff.sh");

    // Act
    let output = Command::new("timeout")
        .args(["60", "bash"])
        .arg(script)
        .arg(&root)
        .arg(repository_root())
        .output()
        .expect("bounded package import regression");

    // Assert
    assert_success(&output);
}

#[test]
fn real_cargo_package_reaches_release_without_repackaging() {
    // Arrange
    let root = producer_fixture_root("real-package-handoff");
    let script = repository_root().join("tools/xtask/tests/release_cli/real_package_handoff.sh");

    // Act
    let output = Command::new("timeout")
        .args(["300", "bash"])
        .arg(script)
        .arg(&root)
        .arg(repository_root())
        .arg(env!("CARGO_BIN_EXE_xtask"))
        .output()
        .expect("bounded real Cargo package handoff");
    fs::write(root.join("stdout.log"), &output.stdout).expect("retained real package stdout");
    fs::write(root.join("stderr.log"), &output.stderr).expect("retained real package stderr");

    // Assert
    assert_success(&output);
}
