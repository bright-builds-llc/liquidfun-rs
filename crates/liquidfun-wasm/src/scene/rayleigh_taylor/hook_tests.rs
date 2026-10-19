//! Debug-safe Rayleigh–Taylor build, hook and session tests: the settle and
//! ramp, the two layer groups, the fail-closed preset and control seams.

use liquidfun::ParticleGroupId;
use liquidfun::math::Vec2;
use liquidfun::particle::ParticleFlags;

use super::geometry::tank_segments;
use super::{
    HEAVY_COLOR, LIGHT_COLOR, RayleighTaylorParts, TUNING, WEIGHT_RATIO_CONTROL,
    WEIGHT_RATIO_TOKENS, WeightRatio, build, build_parts, ramp_factor, ratio_from_presets,
};
use crate::scene::{ControlEffect, PointerKind, SceneHooks, SceneId, build_scene};
use crate::session::{SessionCore, SessionError};

const MAX_PARTICLES: usize = 3_000;

fn parts() -> RayleighTaylorParts {
    build_parts(WeightRatio::Two, &TUNING).expect("rayleigh-taylor parts")
}

fn preset(name: &str, value: &str) -> (String, String) {
    (name.to_owned(), value.to_owned())
}

/// Distinct group ids in row order.
fn group_ids(parts: &RayleighTaylorParts) -> Vec<Option<ParticleGroupId>> {
    let view = parts
        .world
        .particle_system_view(parts.particle_system)
        .expect("liquid");
    let mut ids = view.group_ids().to_vec();
    ids.dedup();
    ids
}

fn velocity_bits(parts: &RayleighTaylorParts) -> Vec<(u32, u32)> {
    parts
        .world
        .particle_system_view(parts.particle_system)
        .expect("liquid")
        .velocities()
        .iter()
        .map(|velocity| (velocity.x.to_bits(), velocity.y.to_bits()))
        .collect()
}

/// Highest particle center in each group: `(heavy, light)`.
fn group_tops(parts: &RayleighTaylorParts) -> (f32, f32) {
    let view = parts
        .world
        .particle_system_view(parts.particle_system)
        .expect("liquid");
    let mut tops = (f32::NEG_INFINITY, f32::NEG_INFINITY);
    for (position, group) in view.positions().iter().zip(view.group_ids()) {
        if *group == Some(parts.hooks.heavy_group) {
            tops.0 = tops.0.max(position.y);
        } else {
            tops.1 = tops.1.max(position.y);
        }
    }
    tops
}

#[test]
fn ramp_is_zero_through_the_settle_window() {
    for step in 0..30 {
        // Act
        let ramp = ramp_factor(step, 30, 18);

        // Assert
        assert!(ramp <= 0.0, "step {step}: {ramp}");
    }
}

#[test]
fn ramp_starts_one_step_in_after_the_settle() {
    // Act
    let ramp = ramp_factor(30, 30, 18);

    // Assert
    assert_eq!(ramp.to_bits(), (1.0_f32 / 18.0).to_bits());
}

#[test]
fn ramp_reaches_one_and_stays_there() {
    for step in 47..=u16::MAX {
        // Act
        let ramp = ramp_factor(step, 30, 18);

        // Assert
        assert_eq!(ramp.to_bits(), 1.0_f32.to_bits(), "step {step}");
    }
}

#[test]
fn ramp_never_decreases() {
    // Act
    let ramps: Vec<f32> = (0..100).map(|step| ramp_factor(step, 30, 18)).collect();

    // Assert
    assert!(ramps.windows(2).all(|pair| pair[0] <= pair[1]), "{ramps:?}");
}

#[test]
fn build_creates_exactly_two_groups_in_one_system() {
    // Act
    let parts = parts();

    // Assert
    let ids = group_ids(&parts);
    assert_eq!(ids.len(), 2, "{ids:?}");
    assert!(ids.iter().all(Option::is_some));
    assert_eq!(ids[1], Some(parts.hooks.heavy_group));
}

#[test]
fn each_layer_carries_its_own_color() {
    // Arrange
    let parts = parts();
    let view = parts
        .world
        .particle_system_view(parts.particle_system)
        .expect("liquid");

    // Act
    let colors = view.maybe_colors().expect("layers are colored");

    // Assert
    for (color, group) in colors.iter().zip(view.group_ids()) {
        let expected = if *group == Some(parts.hooks.heavy_group) {
            HEAVY_COLOR
        } else {
            LIGHT_COLOR
        };
        assert_eq!(*color, expected);
    }
}

#[test]
fn every_particle_carries_the_tuned_flags_only() {
    // Arrange
    let parts = parts();
    let view = parts
        .world
        .particle_system_view(parts.particle_system)
        .expect("liquid");

    for flags in view.flags() {
        // Act
        let forbidden = flags.intersects(ParticleFlags::COLOR_MIXING | ParticleFlags::TENSILE);

        // Assert
        assert_eq!(*flags, TUNING.particle_flags);
        assert!(!forbidden, "{flags:?}");
    }
}

#[test]
fn build_stays_within_the_particle_budget() {
    // Act
    let parts = parts();

    // Assert
    let count = parts
        .world
        .particle_system_view(parts.particle_system)
        .expect("liquid")
        .positions()
        .len();
    assert!(count > 0 && count <= MAX_PARTICLES, "{count}");
}

#[test]
fn build_sets_gravity_ten_straight_down() {
    // Act
    let parts = parts();

    // Assert
    let gravity = parts.world.gravity();
    assert_eq!(
        (gravity.x.to_bits(), gravity.y.to_bits()),
        (0.0_f32.to_bits(), (-10.0_f32).to_bits())
    );
}

#[test]
fn heavy_group_starts_on_top() {
    // Act
    let (heavy_top, light_top) = group_tops(&parts());

    // Assert
    assert!(heavy_top > light_top, "{heavy_top} vs {light_top}");
}

#[test]
fn build_accepts_no_preset_and_a_listed_ratio() {
    // Act
    let defaulted = build(&[]);
    let three = build(&[preset(WEIGHT_RATIO_CONTROL, "3")]);

    // Assert
    assert!(defaulted.is_ok());
    assert!(three.is_ok());
}

#[test]
fn presets_pick_the_default_or_the_listed_ratio() {
    // Act
    let defaulted = ratio_from_presets(&[]);
    let three = ratio_from_presets(&[preset(WEIGHT_RATIO_CONTROL, "3")]);

    // Assert
    assert_eq!(defaulted, Ok(WeightRatio::Two));
    assert_eq!(three, Ok(WeightRatio::Three));
}

#[test]
fn build_rejects_unknown_presets_and_tokens() {
    for presets in [
        vec![preset(WEIGHT_RATIO_CONTROL, "2.0")],
        vec![preset("water-amount", "medium")],
        vec![preset("x", "1")],
    ] {
        // Act
        let result = build(&presets);

        // Assert
        assert!(
            matches!(result, Err(SessionError::UnknownControl)),
            "{presets:?}"
        );
    }
}

#[test]
fn last_weight_ratio_preset_wins() {
    // Arrange
    let presets = [
        preset(WEIGHT_RATIO_CONTROL, "1.2"),
        preset(WEIGHT_RATIO_CONTROL, "3"),
    ];

    // Act
    let ratio = ratio_from_presets(&presets);

    // Assert
    assert_eq!(ratio, Ok(WeightRatio::Three));
}

#[test]
fn build_scene_accepts_the_gravity_preset() {
    // Act
    let built = build_scene(SceneId::RayleighTaylor, &[preset("gravity", "10")]);

    // Assert
    assert!(built.is_ok());
}

#[test]
fn hooks_recreate_for_each_listed_ratio() {
    // Arrange
    let mut parts = parts();

    for token in WEIGHT_RATIO_TOKENS {
        // Act
        let effect = parts.hooks.apply_control(
            &mut parts.world,
            parts.particle_system,
            WEIGHT_RATIO_CONTROL,
            token,
        );

        // Assert
        assert!(matches!(effect, Ok(ControlEffect::Recreated)), "{token}");
    }
}

#[test]
fn hooks_reject_unknown_control_values_and_names() {
    // Arrange
    let mut parts = parts();

    for (name, value) in [
        (WEIGHT_RATIO_CONTROL, "2.0"),
        (WEIGHT_RATIO_CONTROL, ""),
        ("gravity-ratio", "2"),
        ("water-amount", "2"),
        ("magnet-strength", "2"),
    ] {
        // Act
        let effect =
            parts
                .hooks
                .apply_control(&mut parts.world, parts.particle_system, name, value);

        // Assert
        assert!(
            matches!(effect, Err(SessionError::UnknownControl)),
            "{name}={value}"
        );
    }
}

#[test]
fn hooks_reject_every_action() {
    // Arrange
    let mut parts = parts();

    for name in ["flip", ""] {
        // Act
        let result = parts
            .hooks
            .apply_action(&mut parts.world, parts.particle_system, name);

        // Assert
        assert_eq!(result, Err(SessionError::UnknownControl), "{name:?}");
    }
}

#[test]
fn hooks_ignore_every_pointer_kind() {
    // Arrange
    let mut parts = parts();

    for kind in [
        PointerKind::Down,
        PointerKind::Move,
        PointerKind::Up,
        PointerKind::Cancel,
    ] {
        // Act
        let result =
            parts
                .hooks
                .apply_pointer(&mut parts.world, parts.particle_system, kind, 0.0, 1.0);

        // Assert
        assert_eq!(result, Ok(()), "{kind:?}");
    }
}

#[test]
fn control_action_and_pointer_seams_leave_velocities_unchanged() {
    // Arrange
    let mut parts = parts();
    let before = velocity_bits(&parts);
    let system = parts.particle_system;

    // Act
    let _effect = parts
        .hooks
        .apply_control(&mut parts.world, system, WEIGHT_RATIO_CONTROL, "3");
    let _bad = parts
        .hooks
        .apply_control(&mut parts.world, system, "x", "1");
    let _action = parts.hooks.apply_action(&mut parts.world, system, "flip");
    for kind in [PointerKind::Down, PointerKind::Move, PointerKind::Up] {
        let _pointer = parts
            .hooks
            .apply_pointer(&mut parts.world, system, kind, 0.0, 1.0);
    }

    // Assert
    assert_eq!(velocity_bits(&parts), before);
}

#[test]
fn hooks_draw_the_tank_outlines_and_no_circles() {
    // Arrange
    let parts = parts();
    let bits = |start: Vec2, end: Vec2| {
        [
            start.x.to_bits(),
            start.y.to_bits(),
            end.x.to_bits(),
            end.y.to_bits(),
        ]
    };

    // Act
    let segments = parts.hooks.collect_segments(&parts.world).expect("tank");
    let circles = parts.hooks.collect_circles(&parts.world).expect("none");

    // Assert
    let expected: Vec<_> = tank_segments()
        .iter()
        .map(|segment| bits(segment.start, segment.end))
        .collect();
    let actual: Vec<_> = segments
        .iter()
        .map(|segment| bits(segment.start, segment.end))
        .collect();
    assert_eq!(actual.len(), 16);
    assert_eq!(actual, expected);
    assert!(circles.is_empty());
}

#[test]
fn session_recreates_for_a_listed_ratio() {
    // Arrange
    let mut session = SessionCore::create(SceneId::RayleighTaylor).expect("session");

    // Act
    let recreated = session.apply_control(WEIGHT_RATIO_CONTROL, "3");

    // Assert
    assert_eq!(recreated, Ok(true));
}

#[test]
fn session_rejects_an_unlisted_ratio() {
    // Arrange
    let mut session = SessionCore::create(SceneId::RayleighTaylor).expect("session");

    // Act
    let result = session.apply_control(WEIGHT_RATIO_CONTROL, "5");

    // Assert
    assert_eq!(result, Err(SessionError::UnknownControl));
}

#[test]
fn session_rejects_actions_and_ignores_pointers() {
    // Arrange
    let mut session = SessionCore::create(SceneId::RayleighTaylor).expect("session");

    // Act
    let action = session.apply_action("flip");
    let pointer = session.apply_pointer("down", 0.0, 1.0);

    // Assert
    assert_eq!(action, Err(SessionError::UnknownControl));
    assert_eq!(pointer, Ok(()));
}
