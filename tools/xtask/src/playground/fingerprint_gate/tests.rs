use super::{
    ChangedScene, GateReport, PlaygroundError, SceneFingerprint, compare_fingerprints,
    ensure_same_capture_parameters, gate_failure, parse_baseline, parse_observed,
    provenance_warnings, render_report,
};

const FP_A: &str = "0123456789abcdef";
const FP_B: &str = "fedcba9876543210";
const FP_C: &str = "00000000000000aa";
const FP_D: &str = "1111111111111111";

fn scene(id: &str, fingerprint: &str) -> SceneFingerprint {
    SceneFingerprint {
        id: id.to_owned(),
        fingerprint: fingerprint.to_owned(),
    }
}

fn baseline_json(scenes: &[(&str, &str)]) -> serde_json::Value {
    let records: Vec<serde_json::Value> = scenes
        .iter()
        .map(|(id, fingerprint)| serde_json::json!({ "id": id, "fingerprint": fingerprint }))
        .collect();
    serde_json::json!({
        "kind": "scene_fingerprint_baseline",
        "schema_version": 1,
        "provenance": {
            "git_commit": "abc",
            "captured_at": "2026-10-14T15-53-11Z",
            "warmup_steps": 60,
            "measured_steps": 120,
        },
        "scenes": records,
    })
}

fn observed_json(scenes: &[(&str, &str)]) -> serde_json::Value {
    let records: Vec<serde_json::Value> = scenes
        .iter()
        .map(|(id, fingerprint)| {
            serde_json::json!({ "scene": id, "fingerprint": fingerprint, "interaction": "default" })
        })
        .collect();
    serde_json::json!({
        "kind": "native_scene_spot",
        "git_head": "def",
        "full_catalog": true,
        "warmup_steps": 60,
        "measured_steps": 120,
        "scenes": records,
    })
}

#[test]
fn identical_protected_ids_pass_and_are_all_unchanged() {
    // Arrange
    let protected = [scene("a", FP_A), scene("b", FP_B)];
    let observed = [scene("b", FP_B), scene("a", FP_A)];

    // Act
    let report = compare_fingerprints(&protected, &observed).expect("valid inputs compare");

    // Assert
    assert!(report.passes());
    assert_eq!(report.unchanged, vec!["a".to_owned(), "b".to_owned()]);
}

#[test]
fn changed_protected_fingerprint_fails_with_both_values() {
    // Arrange
    let protected = [scene("a", FP_A), scene("b", FP_B)];
    let observed = [scene("a", FP_A), scene("b", FP_C)];

    // Act
    let report = compare_fingerprints(&protected, &observed).expect("valid inputs compare");

    // Assert
    assert!(!report.passes());
    assert_eq!(
        report.changed,
        vec![ChangedScene {
            id: "b".to_owned(),
            baseline: FP_B.to_owned(),
            observed: FP_C.to_owned(),
        }]
    );
}

#[test]
fn missing_protected_id_fails_and_is_named() {
    // Arrange
    let protected = [scene("a", FP_A), scene("b", FP_B)];
    let observed = [scene("a", FP_A)];

    // Act
    let report = compare_fingerprints(&protected, &observed).expect("valid inputs compare");

    // Assert
    assert!(!report.passes());
    assert_eq!(report.missing, vec!["b".to_owned()]);
}

#[test]
fn new_observed_id_passes_and_is_listed_as_added() {
    // Arrange
    let protected = [scene("a", FP_A)];
    let observed = [scene("a", FP_A), scene("new-scene", FP_D)];

    // Act
    let report = compare_fingerprints(&protected, &observed).expect("valid inputs compare");

    // Assert
    assert!(report.passes());
    assert_eq!(report.added, vec![scene("new-scene", FP_D)]);
}

#[test]
fn duplicate_baseline_id_is_rejected() {
    // Arrange
    let json = baseline_json(&[("a", FP_A), ("a", FP_B)]);

    // Act
    let result = parse_baseline(&json);

    // Assert
    assert!(result.is_err());
}

#[test]
fn duplicate_observed_id_is_rejected_by_compare() {
    // Arrange
    let protected = [scene("a", FP_A)];
    let observed = [scene("a", FP_A), scene("a", FP_A)];

    // Act
    let result = compare_fingerprints(&protected, &observed);

    // Assert
    assert!(result.is_err());
}

#[test]
fn duplicate_observed_id_is_rejected_by_parse() {
    // Arrange
    let json = observed_json(&[("a", FP_A), ("a", FP_A)]);

    // Act
    let result = parse_observed(&json);

    // Assert
    assert!(result.is_err());
}

#[test]
fn empty_baseline_scenes_are_rejected() {
    // Arrange
    let json = baseline_json(&[]);

    // Act
    let result = parse_baseline(&json);

    // Assert
    assert!(result.is_err());
}

#[test]
fn empty_protected_set_is_rejected_by_compare() {
    // Arrange
    let observed = [scene("a", FP_A)];

    // Act
    let result = compare_fingerprints(&[], &observed);

    // Assert
    assert!(result.is_err());
}

#[test]
fn malformed_baseline_fingerprint_is_rejected() {
    // Arrange
    let json = baseline_json(&[("a", "0123456789ABCDEF")]);

    // Act
    let result = parse_baseline(&json);

    // Assert
    assert!(result.is_err());
}

#[test]
fn malformed_observed_fingerprint_is_rejected() {
    // Arrange
    let json = observed_json(&[("a", "0123")]);

    // Act
    let result = parse_observed(&json);

    // Assert
    assert!(result.is_err());
}

#[test]
fn wrong_observed_kind_is_rejected() {
    // Arrange
    let mut json = observed_json(&[("a", FP_A)]);
    json["kind"] = serde_json::json!("native_scene_survey");

    // Act
    let result = parse_observed(&json);

    // Assert
    assert!(result.is_err());
}

#[test]
fn wrong_baseline_kind_is_rejected() {
    // Arrange
    let mut json = baseline_json(&[("a", FP_A)]);
    json["kind"] = serde_json::json!("native_scene_spot");

    // Act
    let result = parse_baseline(&json);

    // Assert
    assert!(result.is_err());
}

#[test]
fn unsupported_baseline_schema_version_is_rejected() {
    // Arrange
    let mut json = baseline_json(&[("a", FP_A)]);
    json["schema_version"] = serde_json::json!(2);

    // Act
    let result = parse_baseline(&json);

    // Assert
    assert!(result.is_err());
}

#[test]
fn valid_inputs_parse_in_file_order() {
    // Arrange
    let baseline = baseline_json(&[("b", FP_B), ("a", FP_A)]);
    let observed = observed_json(&[("a", FP_A), ("c", FP_C)]);

    // Act
    let parsed = (parse_baseline(&baseline), parse_observed(&observed));

    // Assert
    assert_eq!(
        parsed,
        (
            Ok(vec![scene("b", FP_B), scene("a", FP_A)]),
            Ok(vec![scene("a", FP_A), scene("c", FP_C)])
        )
    );
}

#[test]
fn passing_report_renders_the_exact_shape() {
    // Arrange
    let report = GateReport {
        unchanged: vec!["a".to_owned(), "b".to_owned()],
        changed: Vec::new(),
        missing: Vec::new(),
        added: Vec::new(),
    };

    // Act
    let rendered = render_report(&report, "base.json", "spot.json");

    // Assert
    assert_eq!(
        rendered,
        "Scene fingerprint gate: PASS\n\
         - protected: 2/2 unchanged\n\
         - changed: none\n\
         - missing: none\n\
         - added (accepted): none\n\
         - baseline: base.json\n\
         - observed: spot.json"
    );
}

#[test]
fn failing_report_lists_items_in_baseline_then_observed_order() {
    // Arrange
    let protected = [
        scene("z", FP_A),
        scene("m", FP_B),
        scene("y", FP_C),
        scene("b", FP_D),
    ];
    let observed = [
        scene("q", FP_A),
        scene("b", FP_A),
        scene("z", FP_B),
        scene("c", FP_C),
    ];
    let report = compare_fingerprints(&protected, &observed).expect("valid inputs compare");

    // Act
    let rendered = render_report(&report, "base.json", "spot.json");

    // Assert
    assert_eq!(
        rendered,
        format!(
            "Scene fingerprint gate: FAIL\n\
             - protected: 0/4 unchanged\n\
             - changed: z {FP_A} -> {FP_B}, b {FP_D} -> {FP_A}\n\
             - missing: m, y\n\
             - added (accepted): q {FP_A}, c {FP_C}\n\
             - baseline: base.json\n\
             - observed: spot.json"
        )
    );
}

#[test]
fn provenance_mismatch_warns_with_both_values() {
    // Arrange
    let baseline = serde_json::json!({
        "provenance": { "os": "macos", "arch": "aarch64", "compiler": "rustc 1.97.0" },
    });
    let observed =
        serde_json::json!({ "os": "macos", "arch": "x86_64", "compiler": "rustc 1.97.0" });

    // Act
    let warnings = provenance_warnings(&baseline, &observed);

    // Assert
    assert_eq!(
        warnings,
        vec![String::from(
            "observed arch `x86_64` differs from baseline arch `aarch64`; fingerprints may differ across hosts or toolchains"
        )]
    );
}

#[test]
fn matching_step_counts_are_accepted() {
    // Arrange
    let baseline = baseline_json(&[("a", FP_A)]);
    let observed = observed_json(&[("a", FP_A)]);

    // Act
    let result = ensure_same_capture_parameters(&baseline, &observed);

    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn mismatched_measured_steps_is_a_hard_error_naming_both_values() {
    // Arrange
    let baseline = baseline_json(&[("a", FP_A)]);
    let mut observed = observed_json(&[("a", FP_A)]);
    observed["measured_steps"] = serde_json::json!(10);

    // Act
    let result = ensure_same_capture_parameters(&baseline, &observed);

    // Assert
    assert_eq!(
        result,
        Err(PlaygroundError::new(
            "gate",
            "observed measured_steps 10 differs from baseline measured_steps 120; recapture with the baseline step counts before comparing fingerprints"
        ))
    );
}

#[test]
fn missing_observed_warmup_steps_is_a_hard_error() {
    // Arrange
    let baseline = baseline_json(&[("a", FP_A)]);
    let mut observed = observed_json(&[("a", FP_A)]);
    observed["warmup_steps"] = serde_json::Value::Null;

    // Act
    let result = ensure_same_capture_parameters(&baseline, &observed);

    // Assert
    assert_eq!(
        result,
        Err(PlaygroundError::new(
            "gate",
            "observed warmup_steps missing differs from baseline warmup_steps 60; recapture with the baseline step counts before comparing fingerprints"
        ))
    );
}

#[test]
fn filtered_capture_with_missing_ids_names_the_filter() {
    // Arrange
    let report = compare_fingerprints(&[scene("a", FP_A), scene("b", FP_B)], &[scene("a", FP_A)])
        .expect("valid inputs compare");
    let mut observed = observed_json(&[("a", FP_A)]);
    observed["full_catalog"] = serde_json::json!(false);

    // Act
    let error = gate_failure(&report, &observed);

    // Assert
    assert_eq!(
        error,
        PlaygroundError::new(
            "gate",
            "scene fingerprint gate failed: 0 changed, 1 missing; the observed capture is filtered (full_catalog false), so rerun `cargo xtask playground scene-spot` without --scene"
        )
    );
}

#[test]
fn full_catalog_capture_with_missing_ids_has_no_filter_hint() {
    // Arrange
    let report = compare_fingerprints(&[scene("a", FP_A), scene("b", FP_B)], &[scene("a", FP_A)])
        .expect("valid inputs compare");
    let observed = observed_json(&[("a", FP_A)]);

    // Act
    let error = gate_failure(&report, &observed);

    // Assert
    assert_eq!(
        error,
        PlaygroundError::new(
            "gate",
            "scene fingerprint gate failed: 0 changed, 1 missing"
        )
    );
}
