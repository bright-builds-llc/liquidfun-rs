//! Executable contract for the private replacement-renderer gate.

use std::fs;
use std::path::PathBuf;

use liquidfun_testbed::{
    CapabilityArtifact, CapabilityOptions, REQUIRED_CAPABILITY_NAMES, run_capability_check,
};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("testbed package should be nested under the workspace root")
        .to_path_buf()
}

#[test]
fn replacement_adapter_passes_every_required_capability_without_session_effects() {
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
    assert_eq!(report.adapter(), "eframe-egui-0.35.0+tiny-skia-0.12.0");
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
    assert_eq!(
        report
            .artifacts()
            .iter()
            .map(CapabilityArtifact::path)
            .collect::<Vec<_>>(),
        [
            "replacement-capability-640x480.png",
            "replacement-capability-800x600.png",
            "replacement-capability-1280x960.png",
        ]
    );
    let report_bytes =
        fs::read(root.join("target/testbed-capability-tests/matrix/capability-report.json"))
            .expect("machine report should be readable");
    let machine_report: serde_json::Value =
        serde_json::from_slice(&report_bytes).expect("machine report should be valid JSON");
    assert_eq!(machine_report["capability_profile"], "phase12-v1");
    assert_eq!(machine_report["fixture_profile"], "phase11-v1");
}

#[test]
fn replacement_capture_hashes_are_stable_across_independent_runs() {
    // Arrange
    let root = repository_root();
    let fixture = root.join("crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json");
    let first = CapabilityOptions::new(
        fixture.clone(),
        PathBuf::from("target/testbed-capability-tests/determinism-a"),
    );
    let second = CapabilityOptions::new(
        fixture,
        PathBuf::from("target/testbed-capability-tests/determinism-b"),
    );

    // Act
    let first_report = run_capability_check(&first).expect("first capability run should pass");
    let second_report = run_capability_check(&second).expect("second capability run should pass");

    // Assert
    let first_hashes = first_report
        .artifacts()
        .iter()
        .map(CapabilityArtifact::sha256)
        .collect::<Vec<_>>();
    let second_hashes = second_report
        .artifacts()
        .iter()
        .map(CapabilityArtifact::sha256)
        .collect::<Vec<_>>();
    assert_eq!(first_hashes, second_hashes);
}

#[test]
fn migrated_capability_and_viewport_sources_exclude_the_legacy_renderer() {
    // Arrange
    let root = repository_root();
    let sources = [
        "crates/liquidfun-testbed/src/capability.rs",
        "crates/liquidfun-testbed/src/capability/input.rs",
        "crates/liquidfun-testbed/src/capability/render.rs",
        "crates/liquidfun-testbed/src/capability/report.rs",
        "crates/liquidfun-testbed/src/ui/viewport/draw.rs",
        "crates/liquidfun-testbed/src/ui/protocol_viewport.rs",
    ];

    // Act
    let offenders = sources
        .into_iter()
        .filter(|relative| {
            fs::read_to_string(root.join(relative))
                .expect("migrated source should be readable")
                .contains("macroquad")
        })
        .collect::<Vec<_>>();

    // Assert
    assert!(
        offenders.is_empty(),
        "legacy renderer remains in migrated sources: {offenders:?}"
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
