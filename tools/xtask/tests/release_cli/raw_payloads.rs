use super::*;

#[test]
fn retained_payload_boundaries_reject_unsafe_and_tampered_bytes() {
    // Arrange
    let root = producer_fixture_root("raw-payload-boundaries");
    let script = repository_root().join("tools/xtask/tests/release_cli/raw_payloads.sh");

    // Act
    let output = Command::new("timeout")
        .args(["90", "bash"])
        .arg(script)
        .arg(&root)
        .arg(repository_root())
        .output()
        .expect("bounded raw payload controls");
    fs::write(root.join("stdout.log"), &output.stdout).expect("retain stdout");
    fs::write(root.join("stderr.log"), &output.stderr).expect("retain stderr");

    // Assert
    assert_success(&output);
}
