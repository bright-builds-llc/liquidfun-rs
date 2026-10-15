//! Debug-safe Archimedes Screw control tests: the `screw-speed` and
//! `screw-direction` allowlists, intent-only live effects and the reverse
//! motion they drive (D-09, D-10, D-11).

use liquidfun::math::Vec2;
use liquidfun::{BodySnapshot, NoDecisionHook, StepLimits};

use super::geometry::{
    AXIS, FRAME_MAX_X, FRAME_MAX_Y, FRAME_MIN_X, FRAME_MIN_Y, NORMAL, PITCH, TUBE_ORIGIN,
};
use super::test_support::step_configuration;
use super::{
    DEFAULT_SCREW_RPM, MAX_SCREW_RPM, SCREW_DIRECTION_CONTROL, SCREW_SPEED_CONTROL, SIM_DT,
    ScrewParts, axial_speed, build_parts, parse_screw_rpm,
};
use crate::scene::{ControlEffect, SceneHooks, SceneId};
use crate::session::{SessionCore, SessionError};

const POSITION_TOLERANCE: f32 = 1e-4;
const SPEED_TOLERANCE: f32 = 1e-5;

/// The web speed control, which must mirror `DEFAULT_SCREW_RPM` and `MAX_SCREW_RPM`.
const SPEED_TS: &str = include_str!("../../../../../web/src/catalog/archimedes-screw-speed.ts");
/// The web view bounds, which must mirror the landscape `FRAME_*` rectangle.
const VIEW_BOUNDS_TS: &str = include_str!("../../../../../web/src/catalog/scene-view-bounds.ts");
/// The web scene records, whose `screw-direction` options must parse in Rust.
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
        .unwrap_or_else(|| panic!("`{marker}` in the web speed control"));
    let (value, _) = rest
        .split_once(';')
        .unwrap_or_else(|| panic!("`;` after `{marker}`"));
    value
        .trim()
        .parse()
        .unwrap_or_else(|error| panic!("`{marker}` value {value:?}: {error}"))
}

/// Every `option("<token>"` token of the web `screw-direction` preset, in order.
fn web_direction_tokens() -> Vec<&'static str> {
    let marker = format!("runtimePreset(\"{SCREW_DIRECTION_CONTROL}\"");
    let (_, after) = CURIOSITY_TS
        .split_once(&marker)
        .unwrap_or_else(|| panic!("`{marker}` in the web scene records"));
    let (options, _) = after
        .split_once("])")
        .expect("`])` closing the direction options");
    options
        .split("option(\"")
        .skip(1)
        .map(|rest| {
            rest.split_once('"')
                .expect("closing quote of a direction token")
                .0
        })
        .collect()
}

fn train_snapshot(parts: &ScrewParts) -> BodySnapshot {
    parts
        .world
        .body_snapshot(parts.hooks.train)
        .expect("live train")
}

fn same_bits(left: Vec2, right: Vec2) -> bool {
    left.x.to_bits() == right.x.to_bits() && left.y.to_bits() == right.y.to_bits()
}

fn control(parts: &mut ScrewParts, name: &str, value: &str) -> Result<ControlEffect, SessionError> {
    let system = parts.particle_system;
    parts
        .hooks
        .apply_control(&mut parts.world, system, name, value)
}

#[test]
fn parse_screw_rpm_accepts_canonical_whole_numbers_up_to_the_maximum() {
    // Arrange
    let default_token = DEFAULT_SCREW_RPM.to_string();
    let max_token = MAX_SCREW_RPM.to_string();

    // Act
    let parsed = [
        parse_screw_rpm("0"),
        parse_screw_rpm("1"),
        parse_screw_rpm(&default_token),
        parse_screw_rpm(&max_token),
    ];

    // Assert
    assert_eq!(
        parsed,
        [
            Some(0),
            Some(1),
            Some(DEFAULT_SCREW_RPM),
            Some(MAX_SCREW_RPM)
        ]
    );
}

#[test]
fn parse_screw_rpm_rejects_malformed_and_out_of_range_tokens() {
    // Arrange
    let above_max = (MAX_SCREW_RPM + 1).to_string();
    let tokens = [
        "", "01", "-5", "+5", "1.0", " 30", "30 ", "abc", &above_max, "99999",
    ];

    // Act
    let accepted: Vec<&str> = tokens
        .iter()
        .copied()
        .filter(|token| parse_screw_rpm(token).is_some())
        .collect();

    // Assert
    assert!(accepted.is_empty(), "{accepted:?}");
}

#[test]
fn screw_speed_stores_intent_without_touching_the_world() {
    // Arrange
    let mut parts = build_parts().expect("screw parts");
    let before = train_snapshot(&parts);
    let token = (MAX_SCREW_RPM - 1).to_string();

    // Act
    let effect = control(&mut parts, SCREW_SPEED_CONTROL, &token);

    // Assert
    assert!(matches!(effect, Ok(ControlEffect::Live)));
    assert_eq!(parts.hooks.rpm, MAX_SCREW_RPM - 1);
    let after = train_snapshot(&parts);
    assert!(same_bits(after.position(), before.position()));
    assert!(same_bits(after.linear_velocity(), before.linear_velocity()));
}

#[test]
fn screw_speed_rejects_a_bad_value_and_keeps_the_speed() {
    // Arrange
    let mut parts = build_parts().expect("screw parts");
    let above_max = (MAX_SCREW_RPM + 1).to_string();

    // Act
    let results = [
        control(&mut parts, SCREW_SPEED_CONTROL, &above_max),
        control(&mut parts, SCREW_SPEED_CONTROL, "01"),
    ];

    // Assert
    assert!(
        results
            .iter()
            .all(|result| matches!(result, Err(SessionError::UnknownControl)))
    );
    assert_eq!(parts.hooks.rpm, DEFAULT_SCREW_RPM);
}

#[test]
fn screw_direction_reverse_sets_and_forward_clears_the_flag() {
    // Arrange
    let mut parts = build_parts().expect("screw parts");

    // Act
    let reverse = control(&mut parts, SCREW_DIRECTION_CONTROL, "reverse");
    let reversed = parts.hooks.reverse;
    let forward = control(&mut parts, SCREW_DIRECTION_CONTROL, "forward");

    // Assert
    assert!(matches!(reverse, Ok(ControlEffect::Live)));
    assert!(matches!(forward, Ok(ControlEffect::Live)));
    assert!(reversed);
    assert!(!parts.hooks.reverse);
}

#[test]
fn screw_direction_rejects_other_values_and_keeps_the_direction() {
    // Arrange
    let mut parts = build_parts().expect("screw parts");
    control(&mut parts, SCREW_DIRECTION_CONTROL, "reverse").expect("reverse is allowed");

    // Act
    let results = ["Reverse", "back", "", "forward "]
        .map(|value| control(&mut parts, SCREW_DIRECTION_CONTROL, value));

    // Assert
    assert!(
        results
            .iter()
            .all(|result| matches!(result, Err(SessionError::UnknownControl)))
    );
    assert!(parts.hooks.reverse);
}

#[test]
fn other_scene_controls_and_actions_fail_closed() {
    // Arrange
    let mut parts = build_parts().expect("screw parts");
    let system = parts.particle_system;

    // Act
    let drum = control(&mut parts, "drum-speed", "30");
    let motor = control(&mut parts, "motor-direction", "reverse");
    let action = parts
        .hooks
        .apply_action(&mut parts.world, system, "anything");

    // Assert
    assert!(matches!(drum, Err(SessionError::UnknownControl)));
    assert!(matches!(motor, Err(SessionError::UnknownControl)));
    assert!(matches!(action, Err(SessionError::UnknownControl)));
    assert_eq!(parts.hooks.rpm, DEFAULT_SCREW_RPM);
    assert!(!parts.hooks.reverse);
}

#[test]
fn the_session_applies_speed_and_direction_live_and_rebuilds_for_gravity() {
    // Arrange
    let mut session = SessionCore::create(SceneId::ArchimedesScrew).expect("screw scene");

    // Act
    let speed = session.apply_control(SCREW_SPEED_CONTROL, "0");
    let direction = session.apply_control(SCREW_DIRECTION_CONTROL, "reverse");
    let gravity = session.apply_control("gravity", "80");

    // Assert
    assert!(matches!(speed, Ok(false)), "{speed:?}");
    assert!(matches!(direction, Ok(false)), "{direction:?}");
    assert!(matches!(gravity, Ok(true)), "{gravity:?}");
}

#[test]
fn reverse_moves_the_train_down_the_axis_at_the_current_speed() {
    // Arrange
    let mut parts = build_parts().expect("screw parts");
    control(&mut parts, SCREW_DIRECTION_CONTROL, "reverse").expect("reverse is allowed");
    let expected_speed = axial_speed(DEFAULT_SCREW_RPM, true);
    // From phase 0 one reverse step wraps the phase to `PITCH - step`.
    let expected_phase = PITCH + expected_speed * SIM_DT;
    let system = parts.particle_system;

    // Act
    parts
        .hooks
        .on_advance(&mut parts.world, system)
        .expect("the vane train advances");
    parts
        .world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("the screw world steps");

    // Assert
    let snapshot = train_snapshot(&parts);
    let velocity = snapshot.linear_velocity();
    let along = velocity.dot(AXIS);
    assert!(
        (along - expected_speed).abs() < SPEED_TOLERANCE,
        "{along} vs {expected_speed}"
    );
    assert!(velocity.dot(NORMAL).abs() < SPEED_TOLERANCE, "{velocity:?}");
    assert!(
        (parts.hooks.phase - expected_phase).abs() < POSITION_TOLERANCE,
        "{} vs {expected_phase}",
        parts.hooks.phase
    );
    let travelled = (snapshot.position() - TUBE_ORIGIN).dot(AXIS);
    assert!(
        (travelled - expected_phase).abs() < POSITION_TOLERANCE,
        "{travelled} vs {expected_phase}"
    );
}

#[test]
fn web_speed_default_and_maximum_match_rust() {
    // Arrange
    let rust = [f32::from(DEFAULT_SCREW_RPM), f32::from(MAX_SCREW_RPM)];

    // Act
    let web = [
        ts_const(SPEED_TS, "SCREW_SPEED_DEFAULT"),
        ts_const(SPEED_TS, "SCREW_SPEED_MAX"),
    ];

    // Assert
    assert_eq!(
        web.map(f32::to_bits),
        rust.map(f32::to_bits),
        "{web:?} vs {rust:?}"
    );
}

#[test]
fn web_speed_minimum_and_step_match_whole_rpm_tokens() {
    // Arrange
    let whole_rpm = [0.0_f32, 1.0];

    // Act
    let web = [
        ts_const(SPEED_TS, "SCREW_SPEED_MIN"),
        ts_const(SPEED_TS, "SCREW_SPEED_STEP"),
    ];

    // Assert
    assert_eq!(
        web.map(f32::to_bits),
        whole_rpm.map(f32::to_bits),
        "{web:?} vs {whole_rpm:?}"
    );
}

#[test]
fn web_direction_options_are_forward_then_reverse() {
    // Act
    let tokens = web_direction_tokens();

    // Assert
    assert_eq!(tokens, ["forward", "reverse"]);
}

#[test]
fn every_web_direction_option_is_a_live_rust_control() {
    // Arrange
    let mut parts = build_parts().expect("screw parts");
    let tokens = web_direction_tokens();

    // Act
    let rejected: Vec<&str> = tokens
        .iter()
        .copied()
        .filter(|token| {
            !matches!(
                control(&mut parts, SCREW_DIRECTION_CONTROL, token),
                Ok(ControlEffect::Live)
            )
        })
        .collect();

    // Assert
    assert!(!tokens.is_empty(), "no web direction options");
    assert!(rejected.is_empty(), "rejected web tokens: {rejected:?}");
}

#[test]
fn web_view_bounds_match_the_frame() {
    // Arrange
    let (_, after) = VIEW_BOUNDS_TS
        .split_once("ARCHIMEDES_SCREW_VIEW_BOUNDS = {")
        .expect("ARCHIMEDES_SCREW_VIEW_BOUNDS in the web view bounds");
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
