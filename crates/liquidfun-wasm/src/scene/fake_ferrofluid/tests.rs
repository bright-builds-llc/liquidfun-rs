//! Debug-safe Fake Ferrofluid tests: field math, geometry, build, hooks and
//! the surface-peak metric.

use liquidfun::collision::PolygonShape;
use liquidfun::math::Vec2;
use liquidfun::particle::ParticleFlags;
use liquidfun::{NoDecisionHook, StepLimits};

use super::field::{FIELD, envelope, magnet_acceleration};
use super::geometry::{
    FLOOR_BOTTOM_Y, FRAME_MAX_X, FRAME_MAX_Y, FRAME_MIN_X, FRAME_MIN_Y, MAGNET_HALF_HEIGHT,
    MAGNET_HALF_LENGTH, MAGNET_REST, PARTICLE_DIAMETER, dish_quads,
};
use super::test_support::{step_configuration, surface_profile};
use super::{DEFAULT_STRENGTH_PERCENT, LIQUID_COLOR, MAX_STRENGTH_PERCENT, build, build_parts};
use crate::scene::particle_force::GravityFrame;
use crate::scene::{PointerKind, SceneHooks, SceneId, build_scene};
use crate::session::SessionError;

const FRAME_MARGIN: f32 = 0.1;
const MIN_PARTICLES: usize = 800;
const MAX_PARTICLES: usize = 3_000;
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const TILTED_GRAVITY: Vec2 = Vec2::new(7.071_068, -7.071_068);

fn frame(gravity: Vec2) -> GravityFrame {
    GravityFrame::from_gravity(gravity).expect("gravity defines a frame")
}

fn full_strength() -> f32 {
    f32::from(MAX_STRENGTH_PERCENT) / 100.0
}

/// World points every 0.1 m over the dish and the space above it.
fn dish_grid() -> Vec<Vec2> {
    let mut points = Vec::new();
    for column in -33_i16..=33 {
        for row in -3_i16..=24 {
            points.push(Vec2::new(f32::from(column) * 0.1, f32::from(row) * 0.1));
        }
    }
    points
}

/// Field at an offset `(s, h)` from the resting magnet, gravity straight down.
fn field_at(strength: f32, s: f32, h: f32) -> Vec2 {
    magnet_acceleration(
        &FIELD,
        frame(GRAVITY),
        strength,
        MAGNET_REST,
        MAGNET_REST + Vec2::new(s, h),
    )
}

fn rotate_quarter_turn(vector: Vec2) -> Vec2 {
    Vec2::new(-vector.y, vector.x)
}

fn velocity_bits(velocities: &[Vec2]) -> Vec<(u32, u32)> {
    velocities
        .iter()
        .map(|velocity| (velocity.x.to_bits(), velocity.y.to_bits()))
        .collect()
}

/// A flat 0.5 m deep layer of particle centers across the dish.
fn flat_layer() -> Vec<Vec2> {
    let mut points = Vec::new();
    for column in 0_u16..74 {
        for row in 0_u16..7 {
            points.push(Vec2::new(
                -2.96 + f32::from(column) * PARTICLE_DIAMETER,
                0.04 + f32::from(row) * PARTICLE_DIAMETER,
            ));
        }
    }
    points
}

#[test]
fn zero_strength_gives_no_pull_anywhere() {
    for point in dish_grid() {
        // Act
        let acceleration = magnet_acceleration(&FIELD, frame(GRAVITY), 0.0, MAGNET_REST, point);

        // Assert
        assert_eq!(acceleration, Vec2::ZERO, "{point:?}");
    }
}

#[test]
fn pull_stays_under_the_cap_at_full_strength() {
    // Arrange
    let limit = FIELD.cap * 10.0 * full_strength() + 1e-4;

    for point in dish_grid() {
        // Act
        let acceleration =
            magnet_acceleration(&FIELD, frame(GRAVITY), full_strength(), MAGNET_REST, point);

        // Assert
        assert!(
            acceleration.length() <= limit,
            "{point:?}: {acceleration:?}"
        );
    }
}

#[test]
fn lift_points_up_directly_above_the_magnet() {
    // Arrange
    let heights = [0.1, 0.5, 1.0, 1.5];

    for h in heights {
        // Act
        let acceleration = field_at(1.0, 0.0, h);

        // Assert
        assert!(acceleration.y > 0.0, "h {h}: {acceleration:?}");
    }
}

#[test]
fn mirrored_offsets_mirror_the_pull() {
    // Arrange
    let offsets = [(0.2, 0.4), (0.9, 0.7), (1.7, 1.1), (2.3, -0.2)];

    for (s, h) in offsets {
        // Act
        let right = field_at(1.0, s, h);
        let left = field_at(1.0, -s, h);

        // Assert
        assert!(
            (right.y - left.y).abs() < 1e-5,
            "{s},{h}: {right:?} {left:?}"
        );
        assert!(
            (right.x + left.x).abs() < 1e-5,
            "{s},{h}: {right:?} {left:?}"
        );
    }
}

#[test]
fn tilted_gravity_lifts_along_the_tilted_up() {
    // Arrange
    let tilted = frame(TILTED_GRAVITY);
    let position = MAGNET_REST + tilted.up * 0.5;

    // Act
    let acceleration = magnet_acceleration(&FIELD, tilted, 1.0, MAGNET_REST, position);

    // Assert
    assert!(
        acceleration.cross(tilted.up).abs() < 1e-4,
        "{acceleration:?}"
    );
    assert!(acceleration.dot(tilted.up) > 0.0, "{acceleration:?}");
}

#[test]
fn quarter_turn_of_gravity_turns_the_pull() {
    // Arrange
    let offsets = [
        Vec2::new(0.7, 0.9),
        Vec2::new(-1.2, 0.3),
        Vec2::new(0.1, 1.4),
    ];
    let turned = frame(rotate_quarter_turn(GRAVITY));

    for offset in offsets {
        // Act
        let upright = magnet_acceleration(
            &FIELD,
            frame(GRAVITY),
            1.0,
            MAGNET_REST,
            MAGNET_REST + offset,
        );
        let rotated = magnet_acceleration(
            &FIELD,
            turned,
            1.0,
            MAGNET_REST,
            MAGNET_REST + rotate_quarter_turn(offset),
        );

        // Assert
        let error = (rotate_quarter_turn(upright) - rotated).length();
        assert!(error < 1e-4, "{offset:?}: {upright:?} {rotated:?}");
    }
}

#[test]
fn pull_vanishes_outside_the_envelope() {
    // Arrange
    let half_width = FIELD.envelope_half_width;
    let offsets = [
        (half_width, 0.5),
        (-half_width, 0.5),
        (half_width + 0.3, 1.0),
    ];

    for (s, h) in offsets {
        // Act
        let acceleration = field_at(1.0, s, h);

        // Assert
        assert_eq!(acceleration, Vec2::ZERO, "{s},{h}");
    }
}

#[test]
fn lift_vanishes_beyond_the_reach() {
    // Arrange
    let reach = FIELD.reach;
    let offsets = [
        (0.0, reach + 0.01),
        (0.0, -reach - 0.01),
        (0.65, reach + 0.4),
        (1.3, -reach - 0.2),
    ];

    for (s, h) in offsets {
        // Act
        let acceleration = field_at(1.0, s, h);

        // Assert
        assert_eq!(
            acceleration.y.abs().to_bits(),
            0,
            "{s},{h}: {acceleration:?}"
        );
    }
}

#[test]
fn pull_is_curl_free_where_the_cap_is_inactive() {
    // Arrange: column centers, where the uncapped pull stays below the cap.
    let samples = [
        (0.0, 0.9),
        (0.65, 1.0),
        (-0.65, 1.2),
        (1.3, 0.8),
        (-1.95, 1.1),
    ];
    let eps = 0.005;
    let cap = FIELD.cap * 10.0;

    for (s, h) in samples {
        // Act
        let stencil = [
            field_at(1.0, s + eps, h),
            field_at(1.0, s - eps, h),
            field_at(1.0, s, h + eps),
            field_at(1.0, s, h - eps),
        ];
        let lift_slope = (stencil[0].y - stencil[1].y) / (2.0 * eps);
        let lateral_slope = (stencil[2].x - stencil[3].x) / (2.0 * eps);

        // Assert
        assert!(
            stencil.iter().all(|value| value.length() < 0.95 * cap),
            "cap active at {s},{h}: {stencil:?}"
        );
        assert!(
            (lift_slope - lateral_slope).abs() < 1e-2,
            "{s},{h}: {lift_slope} vs {lateral_slope}"
        );
    }
}

#[test]
fn envelope_is_one_over_the_magnet_and_zero_past_its_half_width() {
    // Act
    let center = envelope(
        &FIELD,
        frame(GRAVITY),
        MAGNET_REST,
        MAGNET_REST + Vec2::new(0.0, 0.4),
    );
    let outside = envelope(
        &FIELD,
        frame(GRAVITY),
        MAGNET_REST,
        MAGNET_REST + Vec2::new(FIELD.envelope_half_width, 0.4),
    );

    // Assert
    assert_eq!(center.to_bits(), 1.0_f32.to_bits());
    assert_eq!(outside.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn dish_quads_are_convex_and_counter_clockwise() {
    for quad in dish_quads() {
        // Act
        let turns = (0..4)
            .map(|index| {
                let a = quad[index];
                let b = quad[(index + 1) % 4];
                let c = quad[(index + 2) % 4];
                (b - a).cross(c - b)
            })
            .collect::<Vec<_>>();

        // Assert
        assert!(PolygonShape::new(&quad).is_ok(), "{quad:?}");
        assert!(turns.iter().all(|turn| *turn > 0.0), "{quad:?}");
    }
}

#[test]
fn walls_are_thick_and_overlap_the_floor_slab() {
    // Arrange
    let [floor, left, right] = dish_quads();

    for wall in [left, right] {
        // Act
        let thickness = wall[1].x - wall[0].x;
        let overlaps_x = wall[0].x < floor[1].x && wall[1].x > floor[0].x;
        let overlaps_y = wall[0].y < floor[2].y && wall[2].y > floor[0].y;

        // Assert
        assert!(thickness >= 3.0 * PARTICLE_DIAMETER, "{wall:?}");
        assert!(overlaps_x && overlaps_y, "{wall:?}");
    }
}

#[test]
fn resting_magnet_is_flush_with_the_slab_bottom() {
    // Act
    let top_face = MAGNET_REST.y + MAGNET_HALF_HEIGHT;

    // Assert
    assert!((top_face - FLOOR_BOTTOM_Y).abs() < 1e-6, "{top_face}");
}

#[test]
fn dish_and_resting_marker_fit_inside_the_frame() {
    // Arrange
    let marker_bottom = MAGNET_REST.y - MAGNET_HALF_HEIGHT;
    let mut points = dish_quads().concat();
    points.push(Vec2::new(MAGNET_REST.x - MAGNET_HALF_LENGTH, marker_bottom));
    points.push(Vec2::new(MAGNET_REST.x + MAGNET_HALF_LENGTH, marker_bottom));

    for point in points {
        // Act
        let inside = point.x >= FRAME_MIN_X + FRAME_MARGIN
            && point.x <= FRAME_MAX_X - FRAME_MARGIN
            && point.y >= FRAME_MIN_Y + FRAME_MARGIN
            && point.y <= FRAME_MAX_Y - FRAME_MARGIN;

        // Assert
        assert!(inside, "{point:?}");
    }
}

#[test]
fn build_fills_one_group_within_the_budget() {
    // Arrange
    let parts = build_parts().expect("ferrofluid parts");
    let view = parts
        .world
        .particle_system_view(parts.particle_system)
        .expect("liquid");
    let count = view.particle_ids().len();
    let group = view.group_ids()[0].expect("grouped liquid");

    // Act
    let members = parts
        .world
        .particle_group_view(group)
        .expect("group")
        .member_ids()
        .len();

    // Assert
    assert!((MIN_PARTICLES..=MAX_PARTICLES).contains(&count), "{count}");
    assert!(view.group_ids().iter().all(|id| *id == Some(group)));
    assert_eq!(members, count);
}

#[test]
fn every_particle_is_tensile_viscous_graphite_violet() {
    // Arrange
    let parts = build_parts().expect("ferrofluid parts");
    let view = parts
        .world
        .particle_system_view(parts.particle_system)
        .expect("liquid");

    // Act
    let colors = view.maybe_colors().expect("colored liquid");

    // Assert
    let flags = ParticleFlags::TENSILE | ParticleFlags::VISCOUS;
    assert!(view.flags().iter().all(|flag| flag.contains(flags)));
    assert!(colors.iter().all(|color| *color == LIQUID_COLOR));
    assert_eq!(
        LIQUID_COLOR,
        liquidfun::particle::ParticleColor::new(138, 132, 173, 255)
    );
}

#[test]
fn build_uses_straight_down_gravity() {
    // Act
    let gravity = build_parts().expect("ferrofluid parts").world.gravity();

    // Assert
    assert_eq!(gravity.x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(gravity.y.to_bits(), (-10.0_f32).to_bits());
}

#[test]
fn build_rejects_scene_presets() {
    // Act
    let result = build(&[("x".into(), "1".into())]);

    // Assert
    assert_eq!(result.err(), Some(SessionError::UnknownControl));
}

#[test]
fn build_scene_accepts_the_gravity_preset() {
    // Act
    let result = build_scene(SceneId::FakeFerrofluid, &[("gravity".into(), "10".into())]);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn fresh_hooks_hold_the_magnet_on_at_rest() {
    // Act
    let hooks = build_parts().expect("ferrofluid parts").hooks;

    // Assert
    assert!(hooks.magnet_on);
    assert_eq!(hooks.magnet_position, MAGNET_REST);
    assert_eq!(hooks.strength_percent, DEFAULT_STRENGTH_PERCENT);
}

#[test]
fn magnet_off_applies_no_force() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");
    let mut twin = build_parts().expect("twin parts");
    parts.hooks.magnet_on = false;

    // Act
    parts
        .hooks
        .on_advance(&mut parts.world, parts.particle_system)
        .expect("advance");
    parts
        .world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("step");
    twin.world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("twin step");

    // Assert
    let forced = parts
        .world
        .particle_system_view(parts.particle_system)
        .expect("liquid")
        .velocities()
        .to_vec();
    let untouched = twin
        .world
        .particle_system_view(twin.particle_system)
        .expect("twin liquid")
        .velocities()
        .to_vec();
    assert_eq!(velocity_bits(&forced), velocity_bits(&untouched));
}

#[test]
fn controls_and_actions_are_rejected_until_the_magnet_seam_lands() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");
    let system = parts.particle_system;

    // Act
    let control = parts
        .hooks
        .apply_control(&mut parts.world, system, "magnet-strength", "65");
    let action = parts.hooks.apply_action(&mut parts.world, system, "magnet");

    // Assert
    assert!(matches!(control, Err(SessionError::UnknownControl)));
    assert_eq!(action, Err(SessionError::UnknownControl));
}

#[test]
fn pointer_events_change_nothing_yet() {
    // Arrange
    let mut parts = build_parts().expect("ferrofluid parts");
    let system = parts.particle_system;
    let kinds = [
        PointerKind::Down,
        PointerKind::Move,
        PointerKind::Up,
        PointerKind::Cancel,
    ];

    for kind in kinds {
        // Act
        let result = parts
            .hooks
            .apply_pointer(&mut parts.world, system, kind, 1.0, 0.5);

        // Assert
        assert_eq!(result, Ok(()));
        assert!(parts.hooks.magnet_on);
        assert_eq!(parts.hooks.magnet_position, MAGNET_REST);
        assert_eq!(parts.hooks.strength_percent, DEFAULT_STRENGTH_PERCENT);
    }
}

#[test]
fn collect_segments_returns_only_the_dish_loops() {
    // Arrange
    let parts = build_parts().expect("ferrofluid parts");
    let expected = dish_quads()
        .iter()
        .flat_map(|quad| (0..4).map(move |index| (quad[index], quad[(index + 1) % 4])))
        .collect::<Vec<_>>();

    // Act
    let segments = parts
        .hooks
        .collect_segments(&parts.world)
        .expect("segments");
    let circles = parts.hooks.collect_circles(&parts.world).expect("circles");

    // Assert
    let actual = segments
        .iter()
        .map(|segment| (segment.start, segment.end))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    assert!(circles.is_empty());
}

#[test]
fn flat_layer_reads_flat_with_no_peaks() {
    // Act
    let profile = surface_profile(&flat_layer());

    // Assert
    assert_eq!(profile.peak_count(), 0);
    assert!(profile.is_flat());
}

#[test]
fn five_bumps_count_five_peaks_centered_on_zero() {
    // Arrange
    let mut points = flat_layer();
    for center in [-1.3, -0.65, 0.0, 0.65, 1.3] {
        for offset in [-PARTICLE_DIAMETER, 0.0, PARTICLE_DIAMETER] {
            for row in 1_u16..=4 {
                points.push(Vec2::new(center + offset, 0.52 + f32::from(row) * 0.075));
            }
        }
    }

    // Act
    let profile = surface_profile(&points);

    // Assert
    assert_eq!(profile.peak_count(), 5);
    let centroid = profile.peak_centroid_x().expect("peaks");
    assert!(centroid.abs() < 2.0 * PARTICLE_DIAMETER, "{centroid}");
    assert!(!profile.is_flat());
}

#[test]
fn one_particle_spike_is_smoothed_away() {
    // Arrange
    let mut points = flat_layer();
    points.push(Vec2::new(0.02, 0.82));

    // Act
    let profile = surface_profile(&points);

    // Assert
    assert_eq!(profile.peak_count(), 0);
    assert!(profile.is_flat());
}
