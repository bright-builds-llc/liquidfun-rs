//! Contracts for the Phase 13.1 gap-verification manifest and evidence tools.

use std::{collections::BTreeSet, fs, path::Path};

use serde_json::Value;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const DEFERRED_TARGET: &str = "phase13_acceptance_contract";
const SELECTED_TARGETS: [&str; 27] = [
    "catalog_cli",
    "corpus_closure",
    "corpus_discovery",
    "corpus_model",
    "coverage_workflow",
    "differential_cli",
    "docs_contract",
    "inventory_cli",
    "nightly_toolchain",
    "package_cli",
    "performance_cli",
    "performance_workflow",
    "phase10_evidence_cli",
    "phase11_evidence_cli",
    "phase13_1_canonical_native_workflow",
    "phase13_1_gap_verification",
    "phase13_evidence_contract",
    "phase13_promotion_contract",
    "phase9_evidence_cli",
    "phase9_witness_provenance",
    "platform_workflow",
    "provenance_cli",
    "regression_workflow",
    "release_attestation",
    "release_cli",
    "safety_evidence_contract",
    "upstream_cli",
];

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask remains two levels below the workspace")
}

fn integration_targets() -> TestResult<BTreeSet<String>> {
    let tests = workspace_root().join("tools/xtask/tests");
    let targets = fs::read_dir(tests)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .filter_map(|path| {
            path.file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
        })
        .collect();
    Ok(targets)
}

fn manifest() -> TestResult<Value> {
    let bytes =
        fs::read(workspace_root().join("scripts/phase13-1-gap-verification-manifest.json"))?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn command_argv(command: &Value) -> TestResult<Vec<&str>> {
    command["argv"]
        .as_array()
        .ok_or("argv must be an array")?
        .iter()
        .map(|argument| argument.as_str().ok_or("argv must contain strings"))
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

#[test]
fn manifest_selects_every_non_deferred_xtask_target_exactly_once() -> TestResult {
    // Arrange
    let inventory = integration_targets()?;
    let manifest = manifest()?;
    let selected = manifest["commands"]
        .as_array()
        .ok_or("commands must be an array")?
        .iter()
        .filter(|command| command["evidence_class"] == "xtask-integration")
        .map(|command| {
            command["argv"][5]
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or("xtask target must be argv[4]")
        })
        .collect::<Result<BTreeSet<_>, _>>()?;

    // Act
    let expected = inventory
        .iter()
        .filter(|target| target.as_str() != DEFERRED_TARGET)
        .cloned()
        .collect::<BTreeSet<_>>();

    // Assert
    assert_eq!(selected, expected);
    assert_eq!(selected.len(), 27);
    assert_eq!(
        manifest["deferred_xtask_targets"],
        serde_json::json!([DEFERRED_TARGET])
    );
    Ok(())
}

#[test]
fn manifest_preserves_exact_xtask_argv_environment_and_lexical_order() -> TestResult {
    // Arrange
    let manifest = manifest()?;
    let commands = manifest["commands"]
        .as_array()
        .ok_or("commands must be an array")?;
    let selected = commands
        .iter()
        .filter(|command| command["evidence_class"] == "xtask-integration")
        .collect::<Vec<_>>();

    // Act / Assert
    assert_eq!(selected.len(), SELECTED_TARGETS.len());
    for (command, expected_target) in selected.iter().zip(SELECTED_TARGETS) {
        assert_eq!(
            command_argv(command)?,
            [
                "cargo",
                "test",
                "-p",
                "xtask",
                "--test",
                expected_target,
                "--all-features"
            ]
        );
        assert_eq!(
            command["environment"],
            serde_json::json!({
                "CARGO_BUILD_JOBS": "1",
                "CARGO_TARGET_DIR": "${OUTPUT_ROOT}/cargo-target"
            })
        );
    }
    Ok(())
}

#[test]
fn manifest_rejects_broad_xtask_and_deferred_phase15_work() -> TestResult {
    // Arrange
    let manifest = manifest()?;
    let commands = manifest["commands"]
        .as_array()
        .ok_or("commands must be an array")?;
    let argv = commands
        .iter()
        .map(command_argv)
        .collect::<Result<Vec<_>, _>>()?;

    // Act
    let has_broad_xtask = argv.iter().any(|arguments| {
        arguments.starts_with(&["cargo", "test", "-p", "xtask"]) && !arguments.contains(&"--test")
    });
    let has_deferred_target = argv
        .iter()
        .any(|arguments| arguments.contains(&DEFERRED_TARGET));
    let has_acceptance_cli = argv.iter().any(|arguments| {
        arguments
            .windows(3)
            .any(|window| window == ["xtask", "phase13", "acceptance"])
    });

    // Assert
    assert!(!has_broad_xtask);
    assert!(!has_deferred_target);
    assert!(!has_acceptance_cli);
    Ok(())
}

#[test]
fn verification_report_records_the_same_sole_deferral() -> TestResult {
    // Arrange
    let verification = fs::read_to_string(workspace_root().join(
        ".planning/phases/13.1-restore-bright-builds-structural-compliance/13.1-VERIFICATION.md",
    ))?;

    // Act / Assert
    assert_eq!(
        verification.matches("addressed_in: \"Phase 15\"").count(),
        1
    );
    assert!(verification.contains("Exact-head Phase 13 acceptance remains green"));
    assert!(
        verification
            .contains("Current exact-head Phase 13 acceptance drift is deferred to Phase 15")
    );
    Ok(())
}
