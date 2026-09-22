use liquidfun::particle::ParticleFlags;

use crate::ProofFrame;
use crate::scene::SceneId;
use crate::session::SessionCore;

#[test]
fn create_jelly_drop_builds_a_bounded_elastic_group_on_two_bars() {
    // Arrange / Act
    let session = SessionCore::create(SceneId::JellyDrop)
        .expect("Jelly Drop should construct a native elastic group");
    let frame = capture(&session);

    // Assert
    assert!((80..=2200).contains(&session.particle_count()));
    assert!(session.particle_count() <= 10240);
    assert!(
        frame.rigid_segments().len() >= 8,
        "two rigid bars report at least eight segment floats"
    );
}

#[test]
fn constructed_group_flags_include_elastic_and_spring() {
    // Arrange / Act
    let super::BuiltScene {
        world,
        particle_system,
        ..
    } = super::build(&[]).expect("Jelly Drop should construct");
    let view = world
        .particle_system_view(particle_system)
        .expect("constructed system should stay live");
    let expected = ParticleFlags::ELASTIC | ParticleFlags::SPRING;

    // Assert
    assert!(
        view.flags()
            .iter()
            .any(|flags| flags.contains(ParticleFlags::ELASTIC)
                && flags.contains(ParticleFlags::SPRING)),
        "group particles should carry {expected:?}"
    );
}

#[test]
fn thirty_steps_keep_the_particle_count_constant() {
    // Arrange
    let mut session = SessionCore::create(SceneId::JellyDrop)
        .expect("Jelly Drop should construct a native elastic group");
    let before = capture(&session).particle_count();

    // Act
    advance_steps(&mut session, 30);
    let after = capture(&session).particle_count();

    // Assert
    assert_eq!(after, before, "Jelly Drop must not emit extra particles");
    assert!((80..=2200).contains(&after));
}

#[test]
fn shape_and_softness_recreate_the_world_from_presets() {
    // Arrange
    let mut session = SessionCore::create(SceneId::JellyDrop)
        .expect("Jelly Drop should construct a native elastic group");
    let circle_count = session.particle_count();

    // Act
    let shape_recreated = session
        .apply_control("shape", "square")
        .expect("shape=square should recreate");
    let square_count = session.particle_count();
    let softness_recreated = session
        .apply_control("softness", "firm")
        .expect("softness=firm should recreate");

    let square_scene = super::build(&[
        ("shape".to_owned(), "square".to_owned()),
        ("softness".to_owned(), "firm".to_owned()),
    ])
    .expect("square firm presets should construct");
    let square_view = square_scene
        .world
        .particle_system_view(square_scene.particle_system)
        .expect("square system should stay live");

    // Assert
    assert!(shape_recreated);
    assert!(softness_recreated);
    assert!(
        square_count > circle_count,
        "filled square should sample more particles than the default circle"
    );
    assert_eq!(session.particle_count(), square_count);
    assert_eq!(square_view.positions().len(), square_count);
    assert!(square_view.flags().iter().any(|flags| {
        flags.contains(ParticleFlags::ELASTIC) && flags.contains(ParticleFlags::SPRING)
    }));
}

#[test]
fn poke_jelly_deforms_without_resetting_or_changing_count() {
    // Arrange
    let mut session = SessionCore::create(SceneId::JellyDrop)
        .expect("Jelly Drop should construct a native elastic group");
    let before = capture(&session);
    let before_count = before.particle_count();
    let before_positions = before.particle_positions();

    // Act
    session
        .apply_action("poke-jelly")
        .expect("poke-jelly should apply an in-engine impulse");
    advance_steps(&mut session, 4);
    let after = capture(&session);

    // Assert
    assert_eq!(after.particle_count(), before_count);
    assert_ne!(
        after.particle_positions().as_ref(),
        before_positions.as_ref(),
        "poke impulse should move particles without recreating the group"
    );
}

#[test]
fn poke_then_sixty_steps_keep_particles_inside_the_camera_box() {
    // Arrange
    let mut session = SessionCore::create(SceneId::JellyDrop)
        .expect("Jelly Drop should construct a native elastic group");

    // Act
    session
        .apply_action("poke-jelly")
        .expect("poke-jelly should apply an in-engine impulse");
    advance_steps(&mut session, 60);
    let frame = capture(&session);
    let positions = frame.particle_positions();

    // Assert
    assert_eq!(frame.particle_count(), session.particle_count());
    assert!(positions.iter().all(|value| value.is_finite()));
    for pair in positions.chunks_exact(2) {
        let x = pair[0];
        let y = pair[1];
        assert!(
            (-7.0..7.0).contains(&x) && (-2.0..9.0).contains(&y),
            "particle ({x}, {y}) escaped (-7,-2)..(7,9)"
        );
    }
}

#[test]
fn pointer_down_pokes_nearby_jelly_without_changing_count() {
    // Arrange
    let mut control = SessionCore::create(SceneId::JellyDrop)
        .expect("Jelly Drop should construct a native elastic group");
    let mut poked = SessionCore::create(SceneId::JellyDrop)
        .expect("Jelly Drop should construct a native elastic group");
    let before_count = poked.particle_count();

    // Act
    poked
        .apply_pointer("down", 0.0, 2.0)
        .expect("down should poke particles near the click");
    advance_steps(&mut control, 4);
    advance_steps(&mut poked, 4);
    let control_frame = capture(&control);
    let poked_frame = capture(&poked);

    // Assert
    assert_eq!(poked_frame.particle_count(), before_count);
    assert_eq!(control_frame.particle_count(), before_count);
    assert_ne!(
        poked_frame.particle_positions().as_ref(),
        control_frame.particle_positions().as_ref(),
        "pointer poke should move nearby jelly particles versus a no-pointer world"
    );
}

#[test]
fn pointer_cancel_after_down_does_not_apply_a_second_impulse() {
    // Arrange
    let mut down_only = SessionCore::create(SceneId::JellyDrop)
        .expect("Jelly Drop should construct a native elastic group");
    let mut canceled = SessionCore::create(SceneId::JellyDrop)
        .expect("Jelly Drop should construct a native elastic group");

    // Act
    down_only
        .apply_pointer("down", 0.0, 2.0)
        .expect("down should poke once");
    canceled
        .apply_pointer("down", 0.0, 2.0)
        .expect("down should poke once");
    canceled
        .apply_pointer("cancel", 0.0, 0.0)
        .expect("cancel after down must not poke again");
    advance_steps(&mut down_only, 4);
    advance_steps(&mut canceled, 4);

    // Assert
    assert_eq!(
        capture(&canceled).particle_positions().as_ref(),
        capture(&down_only).particle_positions().as_ref(),
        "cancel after down must not apply a second impulse"
    );
}

#[test]
fn labeled_poke_still_uses_contiguous_first_third_range() {
    // Arrange
    let source = include_str!("../jelly_drop.rs");
    let impl_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation precedes tests");

    // Assert
    assert!(
        impl_source.contains("apply_particle_linear_impulse("),
        "localized poke must call apply_particle_linear_impulse per nearby id"
    );
    assert!(
        impl_source.contains("apply_particle_linear_impulse_range"),
        "labeled poke-jelly must keep the contiguous range API"
    );
    assert!(
        impl_source.contains("members[..poke_len]"),
        "labeled poke must stay on the first-third contiguous slice"
    );
}

#[test]
fn unknown_shape_softness_and_actions_fail_closed() {
    // Arrange
    let mut session = SessionCore::create(SceneId::JellyDrop)
        .expect("Jelly Drop should construct a native elastic group");

    // Act
    let bad_shape = session.apply_control("shape", "triangle");
    let bad_softness = session.apply_control("softness", "jello");
    let unknown_name = session.apply_control("water-amount", "medium");
    let unknown_action = session.apply_action("drop-body");

    // Assert
    assert_eq!(bad_shape, Err(crate::session::SessionError::UnknownControl));
    assert_eq!(
        bad_softness,
        Err(crate::session::SessionError::UnknownControl)
    );
    assert_eq!(
        unknown_name,
        Err(crate::session::SessionError::UnknownControl)
    );
    assert_eq!(
        unknown_action,
        Err(crate::session::SessionError::UnknownControl)
    );
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
            .expect("Jelly Drop should capture a frame"),
    )
}
