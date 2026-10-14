use super::{LEVEL_IDS, parse_level_id};
use crate::scene::{ALL_SCENE_IDS, parse_scene_id};
use crate::session::SessionError;

const CATALOG_SCENES_TS: &str = include_str!("../../../../web/src/catalog/scenes.ts");
const CATALOG_PUZZLES_TS: &str = include_str!("../../../../web/src/catalog/puzzles.ts");
const SCENE_IDS_MARKER: &str = "export const SCENE_IDS = [";
const PUZZLE_IDS_MARKER: &str = "export const PUZZLE_IDS = [";

/// Quoted ids in the first array literal after `marker`.
fn quoted_ids_after<'a>(source: &'a str, marker: &str) -> Vec<&'a str> {
    let start = source
        .find(marker)
        .expect("catalog source should declare the id array marker");
    let body = &source[start + marker.len()..];
    let close = body.find(']').expect("catalog id array should close");
    body[..close]
        .split(',')
        .map(|item| item.trim().trim_matches('"'))
        .filter(|item| !item.is_empty())
        .collect()
}

#[test]
fn level_ids_round_trip_through_parse_level_id() {
    // Arrange
    let ids = LEVEL_IDS;

    // Act
    let parsed = ids.map(|id| parse_level_id(id.as_str()));

    // Assert
    assert_eq!(parsed, ids.map(Some));
}

#[test]
fn parse_level_id_rejects_unknown_and_case_variants() {
    // Arrange
    let raw = ["First-Pour", "first_pour", "", " first-pour", "dam-break"];

    // Act
    let parsed = raw.map(parse_level_id);

    // Assert
    assert_eq!(parsed, [None; 5]);
}

#[test]
fn no_level_id_parses_as_a_scene_id() {
    // Arrange
    let ids = LEVEL_IDS;

    // Act
    let parsed = ids.map(|id| parse_scene_id(id.as_str()));

    // Assert
    assert!(
        parsed
            .iter()
            .all(|result| *result == Err(SessionError::UnknownScene))
    );
}

#[test]
fn no_scene_id_parses_as_a_level_id() {
    // Arrange
    let scene_ids = quoted_ids_after(CATALOG_SCENES_TS, SCENE_IDS_MARKER);

    // Act
    let crossovers: Vec<&str> = scene_ids
        .iter()
        .copied()
        .filter(|id| parse_scene_id(id).is_err() || parse_level_id(id).is_some())
        .collect();

    // Assert
    assert_eq!(scene_ids.len(), ALL_SCENE_IDS.len());
    assert!(crossovers.is_empty(), "unexpected ids: {crossovers:?}");
}

#[test]
fn puzzle_ids_mirror_matches_the_rust_allowlist() {
    // Arrange
    let expected: Vec<&str> = LEVEL_IDS.iter().map(|id| id.as_str()).collect();

    // Act
    let mirrored = quoted_ids_after(CATALOG_PUZZLES_TS, PUZZLE_IDS_MARKER);

    // Assert
    assert_eq!(mirrored, expected);
}

#[test]
fn puzzle_ids_are_not_scene_ids() {
    // Arrange
    let scene_ids = quoted_ids_after(CATALOG_SCENES_TS, SCENE_IDS_MARKER);

    // Act
    let shared: Vec<&str> = quoted_ids_after(CATALOG_PUZZLES_TS, PUZZLE_IDS_MARKER)
        .into_iter()
        .filter(|id| scene_ids.contains(id))
        .collect();

    // Assert
    assert!(shared.is_empty(), "ids in both catalogs: {shared:?}");
}
