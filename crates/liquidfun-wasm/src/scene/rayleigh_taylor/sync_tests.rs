//! Web/Rust sync tests: the web `Weight ratio` option ids and default, and
//! the `RAYLEIGH_TAYLOR_VIEW_BOUNDS` frame must mirror the Rust scene (R14).

use super::geometry::{FRAME_MAX_X, FRAME_MAX_Y, FRAME_MIN_X, FRAME_MIN_Y};
use super::{DEFAULT_WEIGHT_RATIO_TOKEN, WEIGHT_RATIO_CONTROL, WEIGHT_RATIO_TOKENS, WeightRatio};

/// The web ratio control, whose option ids must be the Rust tokens.
const RATIO_TS: &str = include_str!("../../../../../web/src/catalog/rayleigh-taylor-ratio.ts");
/// The web view bounds, which must mirror the `FRAME_*` rectangle.
const VIEW_BOUNDS_TS: &str = include_str!("../../../../../web/src/catalog/scene-view-bounds.ts");
/// The web preset defaults, whose `weight-ratio` entry must be the Rust default.
const SCENE_CONTROLS_TS: &str = include_str!("../../../../../web/src/components/scene-controls.ts");

/// Number literal after `key:` in a TypeScript object literal body.
fn ts_field(body: &str, key: &str) -> f32 {
    let marker = format!("{key}:");
    let (_, rest) = body
        .split_once(&marker)
        .unwrap_or_else(|| panic!("`{marker}` in the web view bounds"));
    let (value, _) = rest
        .split_once(',')
        .unwrap_or_else(|| panic!("`,` after `{marker}`"));
    value
        .trim()
        .parse()
        .unwrap_or_else(|error| panic!("`{marker}` value {value:?}: {error}"))
}

/// The ids of every `option("<id>", ...)` call in a TypeScript module, in order.
fn ts_option_ids(source: &str) -> Vec<&str> {
    source
        .split("option(\"")
        .skip(1)
        .map(|rest| {
            rest.split_once('"')
                .map(|(id, _)| id)
                .expect("closing quote after an option id")
        })
        .collect()
}

#[test]
fn web_view_bounds_match_the_frame() {
    // Arrange
    let (_, after) = VIEW_BOUNDS_TS
        .split_once("RAYLEIGH_TAYLOR_VIEW_BOUNDS = {")
        .expect("RAYLEIGH_TAYLOR_VIEW_BOUNDS in the web view bounds");
    let (body, _) = after.split_once('}').expect("closing brace");

    // Act
    let bounds = [
        ts_field(body, "minX"),
        ts_field(body, "minY"),
        ts_field(body, "maxX"),
        ts_field(body, "maxY"),
    ];

    // Assert
    let frame = [FRAME_MIN_X, FRAME_MIN_Y, FRAME_MAX_X, FRAME_MAX_Y];
    assert_eq!(
        bounds.map(f32::to_bits),
        frame.map(f32::to_bits),
        "{bounds:?} vs {frame:?}"
    );
}

#[test]
fn web_weight_ratio_tokens_match_rust() {
    // Arrange
    let control = format!("id: \"{WEIGHT_RATIO_CONTROL}\"");

    // Act
    let ids = ts_option_ids(RATIO_TS);

    // Assert
    assert!(
        RATIO_TS.contains(&control),
        "{control} in the web ratio control"
    );
    assert_eq!(ids, WEIGHT_RATIO_TOKENS, "web option ids vs Rust tokens");
    for id in ids {
        assert!(WeightRatio::parse(id).is_some(), "{id:?} parses in Rust");
    }
}

#[test]
fn web_default_weight_ratio_matches_rust() {
    // Arrange
    let entry = format!("\"{WEIGHT_RATIO_CONTROL}\": \"{DEFAULT_WEIGHT_RATIO_TOKEN}\"");

    // Act
    let has_entry = SCENE_CONTROLS_TS.contains(&entry);

    // Assert
    assert!(has_entry, "{entry} in the web preset defaults");
    assert_eq!(
        WeightRatio::parse(DEFAULT_WEIGHT_RATIO_TOKEN),
        Some(WeightRatio::DEFAULT)
    );
}
