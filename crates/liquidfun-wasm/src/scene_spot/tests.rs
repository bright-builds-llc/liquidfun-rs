use super::{
    RunSpread, SURVEY_SCENES, SceneSpotError, SceneSpotSample, SurveyCue, apply_cue,
    run_scene_spot, summarize,
};
use crate::scene::parse_scene_id;
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
    let result = run_scene_spot(0, 1, runs);

    // Assert
    assert_eq!(result, Err(SceneSpotError::ZeroRuns));
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
    assert!(!json.contains("wall_ms"));
    assert!(!json.contains("rust_over_cpp_ratio"));
}
