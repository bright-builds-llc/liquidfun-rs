use super::fingerprint::{Fnv1a, end_state_fingerprint, fingerprint_hex};
use super::{
    RunSpread, SURVEY_SCENES, SceneSpotError, SceneSpotSample, SurveyCue, apply_cue,
    resolve_scene_filter, run_scene_spot, summarize,
};
use crate::scene::{SceneId, parse_scene_id};
use crate::session::SessionCore;

const CATALOG_SCENES_TS: &str = include_str!("../../../../web/src/catalog/scenes.ts");

/// Test-only copy of the xtask `SCENE_IDS` bracket/comma split.
fn catalog_scene_ids(source: &str) -> Vec<&str> {
    let marker = "export const SCENE_IDS = [";
    let start = source
        .find(marker)
        .expect("scenes.ts should declare SCENE_IDS");
    let body = &source[start + marker.len()..];
    let close = body.find(']').expect("SCENE_IDS should close its array");
    body[..close]
        .split(',')
        .map(|item| item.trim().trim_matches('"'))
        .filter(|item| !item.is_empty())
        .collect()
}

#[test]
fn survey_scenes_match_catalog_scene_ids() {
    // Arrange
    let catalog = catalog_scene_ids(CATALOG_SCENES_TS);

    // Act
    let survey: Vec<&str> = SURVEY_SCENES.iter().map(|(id, _, _)| *id).collect();

    // Assert
    assert_eq!(survey, catalog);
}

#[test]
fn survey_scene_ids_resolve_through_parse_scene_id() {
    // Arrange
    let entries = SURVEY_SCENES;

    // Act
    let mismatches: Vec<&str> = entries
        .iter()
        .filter(|(id, scene, _)| parse_scene_id(id).ok() != Some(*scene))
        .map(|(id, _, _)| *id)
        .collect();

    // Assert
    assert!(mismatches.is_empty(), "unresolved ids: {mismatches:?}");
}

#[test]
fn only_idle_scenes_are_scripted() {
    // Arrange
    let entries = SURVEY_SCENES;

    // Act
    let scripted: Vec<&str> = entries
        .iter()
        .filter(|(_, _, cue)| cue.label() == "scripted")
        .map(|(id, _, _)| *id)
        .collect();
    let maybe_sparky_label = entries
        .iter()
        .find(|(id, _, _)| *id == "sparky")
        .map(|(_, _, cue)| cue.label());

    // Assert
    assert_eq!(scripted, ["float-or-sink", "impulse", "drawing-particles"]);
    assert_eq!(maybe_sparky_label, Some("default"));
}

#[test]
fn survey_cues_apply_to_fresh_sessions() {
    // Arrange
    let scripted = SURVEY_SCENES
        .iter()
        .filter(|(_, _, cue)| *cue != SurveyCue::Default);

    for (id, scene, cue) in scripted {
        let mut session = SessionCore::create(*scene).expect("scene should build");

        // Act
        let result = apply_cue(&mut session, id, *cue);

        // Assert
        assert_eq!(result, Ok(()), "{id} cue should apply");
    }
}

#[test]
fn summarize_odd_count_returns_middle_value() {
    // Arrange
    let mut values = [3.0, 1.0, 2.0];

    // Act
    let maybe_spread = summarize(&mut values);

    // Assert
    assert_eq!(
        maybe_spread,
        Some(RunSpread {
            median: 2.0,
            min: 1.0,
            max: 3.0,
        })
    );
}

#[test]
fn summarize_even_count_averages_middle_values() {
    // Arrange
    let mut values = [4.0, 1.0, 3.0, 2.0];

    // Act
    let maybe_spread = summarize(&mut values);

    // Assert
    assert_eq!(
        maybe_spread,
        Some(RunSpread {
            median: 2.5,
            min: 1.0,
            max: 4.0,
        })
    );
}

#[test]
fn summarize_empty_returns_none() {
    // Arrange
    let mut values: [f64; 0] = [];

    // Act
    let maybe_spread = summarize(&mut values);

    // Assert
    assert_eq!(maybe_spread, None);
}

#[test]
fn run_scene_spot_rejects_zero_runs() {
    // Arrange
    let runs = 0;

    // Act
    let result = run_scene_spot(0, 1, runs, &[]);

    // Assert
    assert_eq!(result, Err(SceneSpotError::ZeroRuns));
}

#[test]
fn run_scene_spot_rejects_zero_measured_steps() {
    // Arrange
    let measured_steps = 0;

    // Act
    let result = run_scene_spot(0, measured_steps, 1, &[]);

    // Assert
    assert_eq!(result, Err(SceneSpotError::ZeroMeasuredSteps));
}

#[test]
fn to_json_reports_survey_fields() {
    // Arrange
    let sample = SceneSpotSample {
        scene: "impulse",
        interaction: "scripted",
        runs: 3,
        warmup_steps: 60,
        measured_steps: 120,
        start_particles: 10,
        end_particles: 40,
        median_ms_per_step: 0.9,
        min_ms_per_step: 0.8,
        max_ms_per_step: 1.0,
        timed_out: false,
        fingerprint: "00000000000000ab".to_owned(),
    };

    // Act
    let json = sample.to_json();

    // Assert
    for expected in [
        "\"interaction\":\"scripted\"",
        "\"runs\":3",
        "\"median_ms_per_step\":0.900000",
        "\"min_ms_per_step\":0.800000",
        "\"max_ms_per_step\":1.000000",
        "\"timed_out\":false",
    ] {
        assert!(json.contains(expected), "missing {expected} in {json}");
    }
    assert!(
        json.ends_with("\"timed_out\":false,\"fingerprint\":\"00000000000000ab\"}"),
        "fingerprint should follow timed_out last in {json}"
    );
    assert!(!json.contains("wall_ms"));
    assert!(!json.contains("rust_over_cpp_ratio"));
}

#[test]
fn fnv1a_of_empty_input_is_offset_basis() {
    // Arrange
    let hasher = Fnv1a::new();

    // Act
    let value = hasher.finish();

    // Assert
    assert_eq!(value, 0xcbf2_9ce4_8422_2325);
}

#[test]
fn fnv1a_mixes_u32_little_endian_bytes() {
    // Arrange
    let mut hasher = Fnv1a::new();
    let mut expected: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in [0x61_u8, 0x00, 0x00, 0x00] {
        expected ^= u64::from(byte);
        expected = expected.wrapping_mul(0x0100_0000_01b3);
    }

    // Act
    hasher.mix_u32(0x0000_0061);

    // Assert
    assert_eq!(hasher.finish(), expected);
}

#[test]
fn fingerprint_hex_is_sixteen_lowercase_digits() {
    // Arrange
    let value = 0xab;

    // Act
    let hex = fingerprint_hex(value);

    // Assert
    assert_eq!(hex, "00000000000000ab");
}

fn dam_break_fingerprint_after(steps: u32) -> Option<u64> {
    let mut session = SessionCore::create(SceneId::DamBreak).expect("dam break should build");
    session.advance(steps).expect("dam break should step");
    session.read_particles(end_state_fingerprint)
}

#[test]
fn end_state_fingerprint_is_deterministic_for_fresh_sessions() {
    // Arrange
    let steps = 3;

    // Act
    let first = dam_break_fingerprint_after(steps);
    let second = dam_break_fingerprint_after(steps);

    // Assert
    assert!(first.is_some(), "fingerprint should succeed");
    assert_eq!(first, second);
}

#[test]
fn end_state_fingerprint_changes_after_a_step() {
    // Arrange
    let mut session = SessionCore::create(SceneId::DamBreak).expect("dam break should build");
    let before = session.read_particles(end_state_fingerprint);

    // Act
    session.advance(1).expect("dam break should step");
    let after = session.read_particles(end_state_fingerprint);

    // Assert
    assert_ne!(before, after);
}

#[test]
fn scene_filter_keeps_catalog_order() {
    // Arrange
    let filter = ["tesla-valve", "liquid-tumbler"];

    // Act
    let resolved = resolve_scene_filter(&filter).expect("known ids should resolve");

    // Assert
    let ids: Vec<&str> = resolved.iter().map(|(id, _, _)| *id).collect();
    assert_eq!(ids, ["liquid-tumbler", "tesla-valve"]);
}

#[test]
fn scene_filter_rejects_unknown_id() {
    // Arrange
    let filter = ["not-a-scene"];

    // Act
    let result = resolve_scene_filter(&filter);

    // Assert
    assert_eq!(
        result,
        Err(SceneSpotError::UnknownScene {
            scene: "not-a-scene".to_owned()
        })
    );
}

#[test]
fn scene_filter_rejects_duplicate_id() {
    // Arrange
    let filter = ["soup", "soup"];

    // Act
    let result = resolve_scene_filter(&filter);

    // Assert
    assert_eq!(
        result,
        Err(SceneSpotError::DuplicateScene {
            scene: "soup".to_owned()
        })
    );
}

#[test]
fn empty_scene_filter_selects_full_catalog() {
    // Arrange
    let filter: [&str; 0] = [];

    // Act
    let resolved = resolve_scene_filter(&filter).expect("empty filter should resolve");

    // Assert
    assert_eq!(resolved, SURVEY_SCENES.to_vec());
}
