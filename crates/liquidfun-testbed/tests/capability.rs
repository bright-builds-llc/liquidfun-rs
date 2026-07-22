//! Executable contract for the private Macroquad-first renderer gate.

use std::fs;
use std::path::PathBuf;

use liquidfun_testbed::{
    CapabilityArtifact, CapabilityOptions, REQUIRED_CAPABILITY_NAMES, run_capability_check,
};
use sha2::{Digest, Sha256};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("testbed package should be nested under the workspace root")
        .to_path_buf()
}

fn documented_report_identity(root: &std::path::Path) -> (usize, String) {
    let decision = fs::read_to_string(root.join("crates/liquidfun-testbed/CAPABILITY.md"))
        .expect("capability decision should be readable");
    let row = decision
        .lines()
        .find(|line| line.split('|').nth(1).map(str::trim) == Some("`capability-report.json`"))
        .expect("capability decision should record the report identity");
    let columns = row.split('|').map(str::trim).collect::<Vec<_>>();
    let bytes = columns[2]
        .replace(',', "")
        .parse()
        .expect("documented report bytes should be numeric");
    let sha256 = columns[3]
        .strip_prefix('`')
        .and_then(|value| value.strip_suffix('`'))
        .expect("documented report digest should be code-formatted")
        .to_owned();
    (bytes, sha256)
}

#[test]
fn macroquad_adapter_passes_every_required_capability_without_session_effects() {
    // Arrange
    let root = repository_root();
    let output = PathBuf::from("target/testbed-capability-tests/matrix");
    let options = CapabilityOptions::new(
        root.join("crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json"),
        output,
    );

    // Act
    let report = run_capability_check(&options).expect("capability matrix should pass");

    // Assert
    assert_eq!(report.adapter(), "macroquad-image-0.4.15");
    assert!(report.all_passed());
    assert_eq!(report.capability_names(), REQUIRED_CAPABILITY_NAMES);
    assert_eq!(report.session_logical_steps_before(), 0);
    assert_eq!(report.session_logical_steps_after(), 0);
    assert_eq!(report.session_captures_before(), 0);
    assert_eq!(report.session_captures_after(), 0);
    assert!(
        report
            .artifacts()
            .iter()
            .all(CapabilityArtifact::is_regular)
    );
    let report_bytes =
        fs::read(root.join("target/testbed-capability-tests/matrix/capability-report.json"))
            .expect("machine report should be readable");
    let (documented_bytes, documented_sha256) = documented_report_identity(&root);
    assert_eq!(report_bytes.len(), documented_bytes);
    assert_eq!(
        format!("{:x}", Sha256::digest(&report_bytes)),
        documented_sha256
    );
}

#[test]
fn capability_output_rejects_paths_outside_the_workspace_target() {
    // Arrange
    let root = repository_root();
    let options = CapabilityOptions::new(
        root.join("crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json"),
        PathBuf::from("../escaped-capability"),
    );

    // Act
    let error = run_capability_check(&options).expect_err("traversal should fail closed");

    // Assert
    assert_eq!(error.category(), "invalid_output_path");
}
