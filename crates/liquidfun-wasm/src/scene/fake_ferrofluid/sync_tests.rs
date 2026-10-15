//! Web/Rust sync tests: the web `Magnet strength` constants, the
//! `FAKE_FERROFLUID_VIEW_BOUNDS` frame and the magnet ids must mirror the
//! Rust scene (R14).

use super::geometry::{FRAME_MAX_X, FRAME_MAX_Y, FRAME_MIN_X, FRAME_MIN_Y};
use super::{
    DEFAULT_STRENGTH_PERCENT, MAGNET_ACTION, MAGNET_STRENGTH_CONTROL, MAX_STRENGTH_PERCENT,
};

/// The web strength control, which must mirror the Rust strength range.
const STRENGTH_TS: &str =
    include_str!("../../../../../web/src/catalog/fake-ferrofluid-strength.ts");
/// The web view bounds, which must mirror the landscape `FRAME_*` rectangle.
const VIEW_BOUNDS_TS: &str = include_str!("../../../../../web/src/catalog/scene-view-bounds.ts");
/// The web scene records, whose `magnet` action must be the Rust action name.
const CURIOSITY_TS: &str =
    include_str!("../../../../../web/src/catalog/scene-records-curiosity.ts");

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

/// Number literal of `const NAME = <value>;` in a TypeScript module.
fn ts_const(source: &str, name: &str) -> f32 {
    let marker = format!("const {name} =");
    let (_, rest) = source
        .split_once(&marker)
        .unwrap_or_else(|| panic!("`{marker}` in the web strength control"));
    let (value, _) = rest
        .split_once(';')
        .unwrap_or_else(|| panic!("`;` after `{marker}`"));
    value
        .trim()
        .parse()
        .unwrap_or_else(|error| panic!("`{marker}` value {value:?}: {error}"))
}

#[test]
fn web_strength_constants_match_rust() {
    // Arrange
    let rust = [
        0.0_f32,
        f32::from(MAX_STRENGTH_PERCENT),
        f32::from(DEFAULT_STRENGTH_PERCENT),
    ];

    // Act
    let web = [
        ts_const(STRENGTH_TS, "MAGNET_STRENGTH_MIN"),
        ts_const(STRENGTH_TS, "MAGNET_STRENGTH_MAX"),
        ts_const(STRENGTH_TS, "MAGNET_STRENGTH_DEFAULT"),
    ];

    // Assert
    assert_eq!(
        web.map(f32::to_bits),
        rust.map(f32::to_bits),
        "{web:?} vs {rust:?}"
    );
}

#[test]
fn web_view_bounds_match_the_frame() {
    // Arrange
    let (_, after) = VIEW_BOUNDS_TS
        .split_once("FAKE_FERROFLUID_VIEW_BOUNDS = {")
        .expect("FAKE_FERROFLUID_VIEW_BOUNDS in the web view bounds");
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
fn web_magnet_ids_match_rust() {
    // Arrange
    let action = format!("action(\"{MAGNET_ACTION}\", \"Magnet\")");
    let control = format!("id: \"{MAGNET_STRENGTH_CONTROL}\"");

    // Act
    let has_action = CURIOSITY_TS.contains(&action);
    let has_control = STRENGTH_TS.contains(&control);

    // Assert
    assert!(has_action, "{action} in the web curiosity records");
    assert!(has_control, "{control} in the web strength control");
}
