//! Debug-safe Fake Ferrofluid seam tests: the pointer drag, the `magnet`
//! action and the `magnet-strength` allowlist store intent only, the magnet
//! stays clamped inside the frame, and the marker is drawn in the gravity
//! frame (D-06..D-11, D-17, UI-D74, UI-D81, UI-D82).

use liquidfun::math::Vec2;

use super::geometry::{
    FLOOR_BOTTOM_Y, FRAME_MAX_X, FRAME_MIN_Y, MAGNET_HALF_HEIGHT, MAGNET_HALF_LENGTH, MAGNET_REST,
    dish_segments, magnet_marker,
};
use super::{
    DEFAULT_STRENGTH_PERCENT, FerrofluidParts, MAGNET_ACTION, MAGNET_STRENGTH_CONTROL,
    MAX_STRENGTH_PERCENT, build_parts,
};
use crate::ProofFrame;
use crate::scene::particle_force::GravityFrame;
use crate::scene::{ControlEffect, PointerKind, RigidSegment, SceneHooks, SceneId};
use crate::session::{SessionCore, SessionError};

/// The frame's rigid segment cap.
const MAX_FRAME_SEGMENTS: usize = 512;
/// Geometry comparisons allow this much rounding, in meters.
const TOLERANCE: f32 = 1e-6;

fn pointer(parts: &mut FerrofluidParts, kind: PointerKind, x: f32, y: f32) {
    let system = parts.particle_system;
    parts
        .hooks
        .apply_pointer(&mut parts.world, system, kind, x, y)
        .expect("pointer events stay Ok");
}

fn action(parts: &mut FerrofluidParts, name: &str) -> Result<(), SessionError> {
    let system = parts.particle_system;
    parts.hooks.apply_action(&mut parts.world, system, name)
}

fn control(
    parts: &mut FerrofluidParts,
    name: &str,
    value: &str,
) -> Result<ControlEffect, SessionError> {
    let system = parts.particle_system;
    parts
        .hooks
        .apply_control(&mut parts.world, system, name, value)
}

fn particle_bits(parts: &FerrofluidParts) -> Vec<[u32; 4]> {
    let view = parts
        .world
        .particle_system_view(parts.particle_system)
        .expect("liquid");
    view.positions()
        .iter()
        .zip(view.velocities())
        .map(|(position, velocity)| {
            [
                position.x.to_bits(),
                position.y.to_bits(),
                velocity.x.to_bits(),
                velocity.y.to_bits(),
            ]
        })
        .collect()
}

fn frame_segment_bits(session: &SessionCore) -> Vec<u32> {
    let frame = ProofFrame::from(session.capture_frame().expect("frame capture"));
    frame
        .rigid_segments()
        .iter()
        .map(|value| value.to_bits())
        .collect()
}

fn gravity_frame(gravity: Vec2) -> Option<GravityFrame> {
    GravityFrame::from_gravity(gravity)
}

fn close(left: Vec2, right: Vec2) -> bool {
    (left - right).length() <= TOLERANCE
}

fn same_segments(left: &[RigidSegment], right: &[RigidSegment]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(a, b)| a.start == b.start && a.end == b.end)
}

#[test]
fn pointer_down_turns_the_magnet_on_at_the_pointer() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");
    parts.hooks.magnet_on = false;

    // Act
    pointer(&mut parts, PointerKind::Down, 1.0, 0.5);

    // Assert
    assert!(parts.hooks.pointer_held);
    assert!(parts.hooks.magnet_on);
    assert_eq!(parts.hooks.magnet_position, Vec2::new(1.0, 0.5));
}

#[test]
fn pointer_move_while_pressed_follows_the_pointer() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");
    pointer(&mut parts, PointerKind::Down, 1.0, 0.5);

    // Act
    pointer(&mut parts, PointerKind::Move, 1.2, 0.4);

    // Assert
    assert_eq!(parts.hooks.magnet_position, Vec2::new(1.2, 0.4));
}

#[test]
fn pointer_move_without_a_press_leaves_the_magnet() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");

    // Act
    pointer(&mut parts, PointerKind::Move, 1.2, 0.4);

    // Assert
    assert!(!parts.hooks.pointer_held);
    assert!(parts.hooks.magnet_on);
    assert_eq!(parts.hooks.magnet_position, MAGNET_REST);
}

#[test]
fn pointer_up_parks_the_magnet_off_at_rest() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");
    pointer(&mut parts, PointerKind::Down, 1.0, 0.5);

    // Act
    pointer(&mut parts, PointerKind::Up, 1.0, 0.5);

    // Assert
    assert!(!parts.hooks.pointer_held);
    assert!(!parts.hooks.magnet_on);
    assert_eq!(parts.hooks.magnet_position, MAGNET_REST);
}

#[test]
fn pointer_cancel_parks_the_magnet_like_up() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");
    pointer(&mut parts, PointerKind::Down, 1.0, 0.5);

    // Act
    pointer(&mut parts, PointerKind::Cancel, 1.0, 0.5);

    // Assert
    assert!(!parts.hooks.pointer_held);
    assert!(!parts.hooks.magnet_on);
    assert_eq!(parts.hooks.magnet_position, MAGNET_REST);
}

#[test]
fn pointer_up_without_a_press_parks_the_magnet_off() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");
    let system = parts.particle_system;

    // Act
    let result = parts
        .hooks
        .apply_pointer(&mut parts.world, system, PointerKind::Up, 2.0, 1.0);

    // Assert
    assert_eq!(result, Ok(()));
    assert!(!parts.hooks.magnet_on);
    assert_eq!(parts.hooks.magnet_position, MAGNET_REST);
}

#[test]
fn pointer_down_outside_the_frame_clamps_to_the_inset() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");
    let expected = Vec2::new(
        FRAME_MAX_X - MAGNET_HALF_LENGTH,
        FRAME_MIN_Y + MAGNET_HALF_HEIGHT,
    );

    // Act
    pointer(&mut parts, PointerKind::Down, 50.0, -50.0);

    // Assert
    assert_eq!(parts.hooks.magnet_position, expected);
}

#[test]
fn session_rejects_a_non_finite_pointer_and_keeps_the_frame() {
    // Arrange
    let mut session = SessionCore::create(SceneId::FakeFerrofluid).expect("ferrofluid scene");
    let before = frame_segment_bits(&session);

    // Act
    let rejected = session.apply_pointer("down", f32::NAN, 0.0);

    // Assert
    assert_eq!(rejected, Err(SessionError::InvalidPointer));
    assert_eq!(frame_segment_bits(&session), before);
}

#[test]
fn magnet_action_toggles_without_moving_the_magnet() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");
    pointer(&mut parts, PointerKind::Down, 1.0, 0.5);

    // Act
    let off = action(&mut parts, MAGNET_ACTION);
    let was_on = parts.hooks.magnet_on;
    let on = action(&mut parts, MAGNET_ACTION);

    // Assert
    assert_eq!((off, on), (Ok(()), Ok(())));
    assert!(!was_on);
    assert!(parts.hooks.magnet_on);
    assert_eq!(parts.hooks.magnet_position, Vec2::new(1.0, 0.5));
}

#[test]
fn magnet_action_after_a_release_energizes_at_rest() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");
    pointer(&mut parts, PointerKind::Down, 1.0, 0.5);
    pointer(&mut parts, PointerKind::Up, 1.0, 0.5);

    // Act
    let result = action(&mut parts, MAGNET_ACTION);

    // Assert
    assert_eq!(result, Ok(()));
    assert!(parts.hooks.magnet_on);
    assert_eq!(parts.hooks.magnet_position, MAGNET_REST);
}

#[test]
fn other_actions_fail_closed_and_change_nothing() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");

    // Act
    let results = ["squeeze", "", "Magnet", "magnet "].map(|name| action(&mut parts, name));

    // Assert
    assert!(
        results
            .iter()
            .all(|result| *result == Err(SessionError::UnknownControl)),
        "{results:?}"
    );
    assert!(parts.hooks.magnet_on);
    assert_eq!(parts.hooks.magnet_position, MAGNET_REST);
}

#[test]
fn magnet_strength_accepts_canonical_whole_percents() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");
    let default_token = DEFAULT_STRENGTH_PERCENT.to_string();
    let max_token = MAX_STRENGTH_PERCENT.to_string();
    let tokens = ["0", "1", default_token.as_str(), max_token.as_str()];

    for (token, expected) in
        tokens
            .into_iter()
            .zip([0, 1, DEFAULT_STRENGTH_PERCENT, MAX_STRENGTH_PERCENT])
    {
        // Act
        let effect = control(&mut parts, MAGNET_STRENGTH_CONTROL, token);

        // Assert
        assert!(matches!(effect, Ok(ControlEffect::Live)), "{token}");
        assert_eq!(parts.hooks.strength_percent, expected, "{token}");
    }
}

#[test]
fn magnet_strength_rejects_malformed_tokens_and_keeps_the_strength() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");
    let above_max = (MAX_STRENGTH_PERCENT + 1).to_string();
    let tokens = [
        "", "065", "-5", "+5", "6.5", " 65", "65 ", &above_max, "99999", "abc",
    ];

    // Act
    let accepted: Vec<&str> = tokens
        .iter()
        .copied()
        .filter(|token| control(&mut parts, MAGNET_STRENGTH_CONTROL, token).is_ok())
        .collect();

    // Assert
    assert!(accepted.is_empty(), "{accepted:?}");
    assert_eq!(parts.hooks.strength_percent, DEFAULT_STRENGTH_PERCENT);
}

#[test]
fn other_scene_controls_fail_closed() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");

    // Act
    let results = [
        control(&mut parts, "screw-speed", "30"),
        control(&mut parts, "drum-speed", "30"),
        control(&mut parts, "magnet", "1"),
    ];

    // Assert
    assert!(
        results
            .iter()
            .all(|result| matches!(result, Err(SessionError::UnknownControl)))
    );
    assert_eq!(parts.hooks.strength_percent, DEFAULT_STRENGTH_PERCENT);
}

#[test]
fn the_session_applies_strength_live_and_rebuilds_for_gravity() {
    // Arrange
    let mut session = SessionCore::create(SceneId::FakeFerrofluid).expect("ferrofluid scene");

    // Act
    let strength = session.apply_control(MAGNET_STRENGTH_CONTROL, "0");
    let gravity = session.apply_control("gravity", "80");

    // Assert
    assert!(matches!(strength, Ok(false)), "{strength:?}");
    assert!(matches!(gravity, Ok(true)), "{gravity:?}");
}

#[test]
fn seam_hooks_store_intent_without_touching_the_world() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");
    let before = particle_bits(&parts);

    // Act
    pointer(&mut parts, PointerKind::Down, 1.0, 0.5);
    pointer(&mut parts, PointerKind::Move, 1.5, 0.2);
    pointer(&mut parts, PointerKind::Up, 1.5, 0.2);
    action(&mut parts, MAGNET_ACTION).expect("magnet action");
    control(&mut parts, MAGNET_STRENGTH_CONTROL, "100").expect("full strength");

    // Assert
    assert_eq!(particle_bits(&parts), before);
}

#[test]
fn energized_marker_draws_outline_divider_and_three_ticks() {
    // Act
    let marker = magnet_marker(MAGNET_REST, gravity_frame(Vec2::new(0.0, -10.0)), true);

    // Assert
    assert_eq!(marker.len(), 8);
}

#[test]
fn parked_marker_draws_only_the_outline() {
    // Act
    let marker = magnet_marker(MAGNET_REST, gravity_frame(Vec2::new(0.0, -10.0)), false);

    // Assert
    assert_eq!(marker.len(), 4);
}

#[test]
fn resting_marker_top_face_is_flush_with_the_slab_bottom() {
    // Act
    let marker = magnet_marker(MAGNET_REST, gravity_frame(Vec2::new(0.0, -10.0)), true);

    // Assert
    let top = marker[..4]
        .iter()
        .flat_map(|segment| [segment.start.y, segment.end.y])
        .fold(f32::MIN, f32::max);
    assert!((top - FLOOR_BOTTOM_Y).abs() < TOLERANCE, "{top}");
}

#[test]
fn sideways_gravity_turns_the_bar_a_quarter_turn() {
    // Arrange
    let center = Vec2::new(0.5, 0.5);

    // Act
    let marker = magnet_marker(center, gravity_frame(Vec2::new(10.0, 0.0)), false);

    // Assert
    let long_side = marker[0].end - marker[0].start;
    assert!(
        close(long_side, Vec2::new(0.0, 2.0 * MAGNET_HALF_LENGTH)),
        "{long_side:?}"
    );
}

#[test]
fn marker_without_a_gravity_frame_is_an_axis_aligned_outline() {
    // Arrange
    let center = Vec2::new(0.5, 0.5);
    let (dx, dy) = (MAGNET_HALF_LENGTH, MAGNET_HALF_HEIGHT);
    let corners = [
        Vec2::new(center.x - dx, center.y - dy),
        Vec2::new(center.x + dx, center.y - dy),
        Vec2::new(center.x + dx, center.y + dy),
        Vec2::new(center.x - dx, center.y + dy),
    ];

    // Act
    let marker = magnet_marker(center, None, true);

    // Assert
    assert_eq!(marker.len(), 4);
    for (index, segment) in marker.iter().enumerate() {
        assert!(close(segment.start, corners[index]), "{segment:?}");
        assert!(close(segment.end, corners[(index + 1) % 4]), "{segment:?}");
    }
}

#[test]
fn collect_segments_appends_the_marker_after_the_dish() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");
    let dish = dish_segments();
    pointer(&mut parts, PointerKind::Down, 1.0, 0.5);
    let marker = magnet_marker(
        Vec2::new(1.0, 0.5),
        gravity_frame(parts.world.gravity()),
        true,
    );

    // Act
    let segments = parts
        .hooks
        .collect_segments(&parts.world)
        .expect("segments");
    let circles = parts.hooks.collect_circles(&parts.world).expect("circles");

    // Assert
    assert!(segments.len() <= MAX_FRAME_SEGMENTS);
    assert!(same_segments(&segments[..dish.len()], &dish));
    assert!(same_segments(&segments[dish.len()..], &marker));
    assert!(circles.is_empty());
}
