use liquidfun::particle::ParticleFlags;

use crate::ProofFrame;
use crate::scene::SceneId;
use crate::session::SessionCore;

const TEAL: [u8; 4] = [57, 211, 199, 255];
const RED: [u8; 4] = [248, 113, 113, 255];

#[test]
fn create_succeeds_with_two_distinct_mixing_colors() {
    // Arrange / Act
    let session = SessionCore::create(SceneId::ColorMixer)
        .expect("Color Mixer should construct two mixing groups");
    let frame = capture(&session);
    let colors = frame.particle_colors();

    // Assert
    assert!((40..=220).contains(&session.particle_count()));
    assert!(
        colors_contain(colors.as_ref(), TEAL),
        "captured colors should include teal (57, 211, 199, 255)"
    );
    assert!(
        colors_contain(colors.as_ref(), RED),
        "captured colors should include destructive red (248, 113, 113, 255)"
    );
}

#[test]
fn every_constructed_particle_carries_color_mixing() {
    // Arrange / Act
    let super::BuiltScene {
        world,
        particle_system,
        ..
    } = super::build(&[]).expect("Color Mixer should construct");
    let view = world
        .particle_system_view(particle_system)
        .expect("constructed system should stay live");

    // Assert
    assert!(!view.flags().is_empty());
    assert!(
        view.flags()
            .iter()
            .all(|flags| flags.contains(ParticleFlags::COLOR_MIXING)),
        "every particle must carry COLOR_MIXING so contact mixing can run"
    );
}

#[test]
fn mix_strength_off_keeps_color_lanes_stable_after_stirred_steps() {
    // Arrange
    let mut session = SessionCore::create(SceneId::ColorMixer)
        .expect("Color Mixer should construct two mixing groups");
    let recreated = session
        .apply_control("mix-strength", "off")
        .expect("mix-strength=off should recreate");
    let before = capture(&session);
    let before_colors = before.particle_colors();
    let before_count = before.particle_count();

    // Act
    advance_steps(&mut session, 120);
    let after = capture(&session);

    // Assert
    assert!(recreated, "mix-strength must return Recreated");
    assert_eq!(after.particle_count(), before_count);
    assert_eq!(
        after.particle_colors().as_ref(),
        before_colors.as_ref(),
        "Off mix-strength must skip the engine color-mixing pass"
    );
}

#[test]
fn default_strong_changes_color_lanes_after_contact() {
    // Arrange
    let mut session = SessionCore::create(SceneId::ColorMixer)
        .expect("Color Mixer should construct two mixing groups");
    let before = capture(&session);
    let before_colors = before.particle_colors();
    let before_count = before.particle_count();

    // Act
    advance_steps(&mut session, 120);
    let after = capture(&session);

    // Assert
    assert_eq!(after.particle_count(), before_count);
    assert_ne!(
        after.particle_colors().as_ref(),
        before_colors.as_ref(),
        "default Strong mix-strength must change captured color-lane bytes"
    );
}

#[test]
fn stir_speed_presets_apply_live() {
    // Arrange
    let super::BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = super::build(&[]).expect("Color Mixer should construct");

    // Act
    let off = hooks
        .apply_control(&mut world, particle_system, "stir-speed", "off")
        .expect("stir-speed=off should apply");
    let slow = hooks
        .apply_control(&mut world, particle_system, "stir-speed", "slow")
        .expect("stir-speed=slow should apply");
    let fast = hooks
        .apply_control(&mut world, particle_system, "stir-speed", "fast")
        .expect("stir-speed=fast should apply");

    // Assert
    assert!(matches!(off, crate::scene::ControlEffect::Live));
    assert!(matches!(slow, crate::scene::ControlEffect::Live));
    assert!(matches!(fast, crate::scene::ControlEffect::Live));
}

#[test]
fn pointer_down_stirs_nearby_particles_without_rewriting_colors() {
    // Arrange
    let mut control = SessionCore::create(SceneId::ColorMixer)
        .expect("Color Mixer should construct two mixing groups");
    let mut stirred = SessionCore::create(SceneId::ColorMixer)
        .expect("Color Mixer should construct two mixing groups");

    // Act
    stirred
        .apply_pointer("down", 0.0, 2.2)
        .expect("down should store the stir origin");
    control.advance(1).expect("control step should succeed");
    stirred.advance(1).expect("stirred step should succeed");
    let control_frame = capture(&control);
    let stirred_frame = capture(&stirred);

    // Assert
    assert_ne!(
        stirred_frame.particle_positions().as_ref(),
        control_frame.particle_positions().as_ref(),
        "pointer stir should change at least one particle position versus a no-pointer world"
    );
    assert!(
        colors_contain(stirred_frame.particle_colors().as_ref(), TEAL),
        "colors must still come from engine lanes"
    );
    assert!(
        colors_contain(stirred_frame.particle_colors().as_ref(), RED),
        "colors must still come from engine lanes"
    );
}

#[test]
fn pointer_cancel_clears_leftover_local_stir() {
    // Arrange
    let mut canceled = SessionCore::create(SceneId::ColorMixer)
        .expect("Color Mixer should construct two mixing groups");
    let mut fresh = SessionCore::create(SceneId::ColorMixer)
        .expect("Color Mixer should construct two mixing groups");
    canceled
        .apply_control("stir-speed", "off")
        .expect("stir-speed=off should apply live");
    fresh
        .apply_control("stir-speed", "off")
        .expect("stir-speed=off should apply live");
    canceled
        .apply_pointer("down", 0.0, 2.2)
        .expect("down should store the stir origin");
    canceled
        .apply_pointer("move", 0.2, 2.3)
        .expect("move should update the stored point");

    // Act
    canceled
        .apply_pointer("cancel", 0.0, 0.0)
        .expect("cancel should clear leftover stir");
    advance_steps(&mut canceled, 4);
    advance_steps(&mut fresh, 4);
    let canceled_positions = capture(&canceled).particle_positions();
    let fresh_positions = capture(&fresh).particle_positions();

    // Assert
    for (canceled_value, fresh_value) in canceled_positions.iter().zip(fresh_positions.iter()) {
        assert!(
            (canceled_value - fresh_value).abs() <= 0.05,
            "cancel must drop leftover local force; {canceled_value} vs {fresh_value}"
        );
    }
}

#[test]
fn labeled_slow_stir_still_applies_global_force() {
    // Arrange
    let mut off = SessionCore::create(SceneId::ColorMixer)
        .expect("Color Mixer should construct two mixing groups");
    let mut slow = SessionCore::create(SceneId::ColorMixer)
        .expect("Color Mixer should construct two mixing groups");
    off.apply_control("stir-speed", "off")
        .expect("stir-speed=off should apply live");
    slow.apply_control("stir-speed", "slow")
        .expect("stir-speed=slow should apply live");

    // Act
    off.advance(1).expect("off step should succeed");
    slow.advance(1).expect("slow step should succeed");

    // Assert
    assert_ne!(
        capture(&slow).particle_positions().as_ref(),
        capture(&off).particle_positions().as_ref(),
        "labeled slow stir must still apply the global (8, 0) force"
    );
}

#[test]
fn pointer_stir_uses_per_particle_force_not_scattered_range() {
    // Arrange
    let source = include_str!("../color_mixer.rs");
    let impl_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation precedes tests");

    // Assert
    assert!(
        impl_source.contains("apply_particle_force("),
        "localized stir must call apply_particle_force per nearby id"
    );
    assert!(
        impl_source.contains("maybe_pointer"),
        "Color Mixer must store maybe_pointer for the captured stir point"
    );
    assert!(
        impl_source.contains("POINTER_STIR_RADIUS"),
        "localized stir radius must be named"
    );
    assert!(
        impl_source.contains("1.25"),
        "POINTER_STIR_RADIUS must be 1.25"
    );
    let range_uses = impl_source
        .lines()
        .filter(|line| line.contains("apply_particle_force_range"))
        .collect::<Vec<_>>();
    assert_eq!(
        range_uses.len(),
        1,
        "range API must stay on the full labeled-stir id list only, got {range_uses:?}"
    );
    assert!(
        range_uses[0].contains("&particles"),
        "labeled stir may use range on the full id vec"
    );
}

#[test]
fn unknown_mix_and_stir_tokens_fail_closed() {
    // Arrange
    let mut session = SessionCore::create(SceneId::ColorMixer)
        .expect("Color Mixer should construct two mixing groups");

    // Act
    let bad_mix = session.apply_control("mix-strength", "paint");
    let bad_stir = session.apply_control("stir-speed", "spin");
    let unknown_name = session.apply_control("water-amount", "medium");
    let unknown_action = session.apply_action("poke-jelly");

    // Assert
    assert_eq!(bad_mix, Err(crate::session::SessionError::UnknownControl));
    assert_eq!(bad_stir, Err(crate::session::SessionError::UnknownControl));
    assert_eq!(
        unknown_name,
        Err(crate::session::SessionError::UnknownControl)
    );
    assert_eq!(
        unknown_action,
        Err(crate::session::SessionError::UnknownControl)
    );
}

fn colors_contain(colors: &[u8], expected: [u8; 4]) -> bool {
    colors.chunks_exact(4).any(|chunk| chunk == expected)
}

fn advance_steps(session: &mut SessionCore, steps: u32) {
    let mut remaining = steps;
    while remaining > 0 {
        let chunk = remaining.min(4);
        session
            .advance(chunk)
            .expect("bounded native steps should succeed");
        remaining -= chunk;
    }
}

fn capture(session: &SessionCore) -> ProofFrame {
    ProofFrame::from(
        session
            .capture_frame()
            .expect("Color Mixer should capture a frame"),
    )
}
