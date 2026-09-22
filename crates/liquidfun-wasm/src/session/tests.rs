use liquidfun::NoDecisionHook;
use liquidfun::math::Vec2;

use crate::ProofFrame;
use crate::scene::{SceneId, parse_scene_id};

use super::*;

fn new_session() -> SessionCore {
    SessionCore::create(SceneId::DamBreak).expect("fixed proof scene should construct")
}

#[test]
fn parse_scene_id_maps_allowlisted_tokens() {
    // Arrange
    let tokens = [
        ("dam-break", SceneId::DamBreak),
        ("fountain", SceneId::Fountain),
        ("float-or-sink", SceneId::FloatOrSink),
        ("color-mixer", SceneId::ColorMixer),
        ("jelly-drop", SceneId::JellyDrop),
        ("water-wheel", SceneId::WaterWheel),
        ("particles", SceneId::Particles),
        ("liquid-timer", SceneId::LiquidTimer),
        ("surface-tension", SceneId::SurfaceTension),
        ("elastic-particles", SceneId::ElasticParticles),
        ("rigid-particles", SceneId::RigidParticles),
        ("soup", SceneId::Soup),
        ("soup-stirrer", SceneId::SoupStirrer),
        ("impulse", SceneId::Impulse),
        ("wave-machine", SceneId::WaveMachine),
        ("theo-jansen", SceneId::TheoJansen),
    ];

    for (raw, expected) in tokens {
        // Act
        let parsed = parse_scene_id(raw);

        // Assert
        assert_eq!(parsed, Ok(expected));
    }
}

#[test]
fn parse_scene_id_rejects_unknown_tokens() {
    // Arrange
    let rejected = ["Dam-Break", "", "not-a-scene"];

    for raw in rejected {
        // Act
        let parsed = parse_scene_id(raw);

        // Assert
        assert_eq!(parsed, Err(SessionError::UnknownScene));
        assert_eq!(
            SessionError::UnknownScene.message(),
            "Rust/WASM scene id is not allowlisted"
        );
    }
}

#[test]
fn capture_uses_the_same_particle_cap_as_copied_frames() {
    // Arrange / Act / Assert
    assert_eq!(MAX_FRAME_PARTICLES, 10240);
}

#[test]
fn create_dam_break_matches_documented_medium_normal_world() {
    // Arrange
    let session =
        SessionCore::create(SceneId::DamBreak).expect("allowlisted Dam Break should construct");

    // Act
    let frame = capture(&session);

    // Assert
    let diagnostics = session.world.world_diagnostics();
    assert_eq!(session.world.gravity().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(session.world.gravity().y.to_bits(), (-10.0_f32).to_bits());
    assert_eq!(diagnostics.body_count(), 2);
    assert_eq!(diagnostics.fixture_count(), 4);
    assert_eq!(session.particle_count(), 1920);
    assert_eq!(session.rigid_shape_count(), 4);
    assert_eq!(frame.particle_count(), 1920);
    assert_eq!(
        frame.particle_colors(),
        [77, 163, 255, 255].repeat(1920).into_boxed_slice()
    );
    assert_eq!(frame.max_speed().to_bits(), 0.0_f32.to_bits());
    assert_eq!(frame.stuck_candidate_count(), 0);
}

#[test]
fn capture_reports_a_finite_speed_after_one_step() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::DamBreak).expect("allowlisted Dam Break should construct");

    // Act
    session.advance(1).expect("one step should succeed");
    let frame = capture(&session);

    // Assert
    assert!(frame.max_speed() > 0.0);
    assert!(frame.max_speed().is_finite());
    assert!(frame.stuck_candidate_count() <= frame.particle_count());
}

#[test]
fn fountain_live_particle_count_grows_after_advance() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::Fountain).expect("allowlisted Fountain should construct");
    let start = session
        .live_particle_count()
        .expect("Fountain snapshot should succeed");

    // Act
    session
        .advance(1)
        .expect("Fountain emission advance should succeed");
    let end = session
        .live_particle_count()
        .expect("Fountain snapshot after emit should succeed");

    // Assert
    assert!(end > start);
    assert_eq!(session.particle_count(), start);
}

#[test]
fn create_color_mixer_constructs_a_live_world() {
    // Arrange / Act
    let session =
        SessionCore::create(SceneId::ColorMixer).expect("allowlisted Color Mixer should construct");

    // Assert
    assert!((400..=2200).contains(&session.particle_count()));
}

#[test]
fn dam_break_unknown_control_and_action_fail_closed() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::DamBreak).expect("allowlisted Dam Break should construct");

    // Act
    let control = session.apply_control("nope", "x");
    let action = session.apply_action("nope");

    // Assert
    assert_eq!(control, Err(SessionError::UnknownControl));
    assert_eq!(action, Err(SessionError::UnknownControl));
    assert_eq!(
        SessionError::UnknownControl.message(),
        "Rust/WASM control is not allowlisted"
    );
    assert_eq!(session.particle_count(), 1920);
    assert_eq!(session.scene_id, SceneId::DamBreak);
    assert!(session.presets.is_empty());
}

#[test]
fn store_preset_replaces_matching_name() {
    // Arrange
    let mut presets = vec![("water".to_owned(), "medium".to_owned())];

    // Act
    store_preset(&mut presets, "water", "large");
    store_preset(&mut presets, "gravity", "normal");

    // Assert
    assert_eq!(
        presets,
        vec![
            ("water".to_owned(), "large".to_owned()),
            ("gravity".to_owned(), "normal".to_owned()),
        ]
    );
}

fn capture(session: &SessionCore) -> ProofFrame {
    ProofFrame::from(
        session
            .capture_frame()
            .expect("fixed proof scene should capture"),
    )
}

#[test]
fn construction_creates_exact_bounded_colored_scene() {
    // Arrange
    let session = new_session();

    // Act
    let frame = capture(&session);

    // Assert
    let diagnostics = session.world.world_diagnostics();
    assert_eq!(session.world.gravity().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(session.world.gravity().y.to_bits(), (-10.0_f32).to_bits());
    assert_eq!(diagnostics.body_count(), 2);
    assert_eq!(diagnostics.fixture_count(), 4);
    assert_eq!(session.world.particle_system_ids().len(), 1);
    let system = session
        .world
        .particle_system_snapshot(session.particle_system)
        .expect("proof particle system should remain live");
    assert_eq!(system.definition().maximum_count(), Some(10240));
    assert_eq!(system.particle_count(), 1920);
    assert_eq!(frame.step_index(), 0);
    assert_eq!(frame.particle_count(), 1920);
    assert_eq!(frame.rigid_shape_count(), 4);
    assert_eq!(frame.rigid_segments().len(), 12);
    assert_eq!(frame.rigid_circles().len(), 3);
    assert_eq!(
        frame.particle_radii(),
        vec![0.063_245_55; 1920].into_boxed_slice()
    );
    assert_eq!(
        frame.particle_colors(),
        [77, 163, 255, 255].repeat(1920).into_boxed_slice()
    );
}

#[test]
fn advance_rejects_out_of_range_counts_without_effect() {
    // Arrange
    let rejected_counts = [0, MAX_ADVANCE_STEPS + 1];

    for rejected_count in rejected_counts {
        let mut session = new_session();
        let before = capture(&session).particle_positions();

        // Act
        let result = session.advance(rejected_count);

        // Assert
        assert_eq!(result, Err(SessionError::StepCountOutOfRange));
        assert_eq!(session.step_index(), 0);
        assert_eq!(capture(&session).particle_positions(), before);
    }
}

#[test]
fn advance_increments_step_index_by_accepted_count() {
    // Arrange
    let mut session = new_session();

    // Act
    session
        .advance(MAX_ADVANCE_STEPS)
        .expect("bounded fixed steps should succeed");

    // Assert
    assert_eq!(session.step_index(), MAX_ADVANCE_STEPS);
}

#[test]
fn captured_frame_lanes_are_aligned_finite_and_positive() {
    // Arrange
    let session = new_session();

    // Act
    let frame = capture(&session);

    // Assert
    assert_eq!(frame.particle_positions().len(), 1920 * 2);
    assert_eq!(frame.particle_colors().len(), 1920 * 4);
    assert_eq!(frame.particle_radii().len(), 1920);
    assert!(
        frame
            .particle_positions()
            .iter()
            .all(|value| value.is_finite())
    );
    assert!(
        frame
            .particle_radii()
            .iter()
            .all(|radius| radius.is_finite() && *radius > 0.0)
    );
    assert!(frame.rigid_segments().iter().all(|value| value.is_finite()));
    assert!(frame.rigid_circles().iter().all(|value| value.is_finite()));
}

#[test]
fn four_steps_move_particle_or_dynamic_circle_state() {
    // Arrange
    let mut session = new_session();
    let before = capture(&session);

    // Act
    session
        .advance(MAX_ADVANCE_STEPS)
        .expect("bounded fixed steps should succeed");
    let after = capture(&session);

    // Assert
    assert!(
        before.particle_positions() != after.particle_positions()
            || before.rigid_circles() != after.rigid_circles()
    );
}

#[test]
fn step_index_overflow_is_rejected_before_world_effects() {
    // Arrange
    let mut session = new_session();
    session.step_index = u32::MAX;
    let before = capture(&session).particle_positions();

    // Act
    let result = session.advance(1);

    // Assert
    assert_eq!(result, Err(SessionError::StepIndexExhausted));
    assert_eq!(capture(&session).particle_positions(), before);
}

#[test]
fn set_gravity_overrides_until_authored_gravity_is_restored() {
    // Arrange
    let mut session = new_session();
    let built = session.world.gravity();

    // Act
    session
        .set_gravity(2.5, -1.5)
        .expect("finite gravity applies");
    let overridden = session.world.gravity();
    session
        .restore_authored_gravity()
        .expect("authored gravity restores");
    let restored = session.world.gravity();

    // Assert
    assert_eq!(overridden.x.to_bits(), 2.5_f32.to_bits());
    assert_eq!(overridden.y.to_bits(), (-1.5_f32).to_bits());
    assert_eq!(restored.x.to_bits(), built.x.to_bits());
    assert_eq!(restored.y.to_bits(), built.y.to_bits());
    assert!(session.set_gravity(f32::NAN, 0.0).is_err());
}

#[test]
fn a_particle_below_the_spatial_hash_makes_the_engine_step_fail() {
    // Arrange
    let mut session = new_session();
    let particle = session
        .world
        .particle_system_view(session.particle_system)
        .expect("dam break particles")
        .particle_ids()[0];
    session
        .world
        .set_particle_position(particle, Vec2::new(0.0, -400.0))
        .expect("a finite position is accepted");

    // Act
    let error = session
        .world
        .step(
            session.step_configuration,
            &mut NoDecisionHook,
            session.step_limits,
        )
        .expect_err("a particle hundreds of meters down leaves the spatial hash");

    // Assert
    let rendered = format!("{error:?}");
    assert!(rendered.contains("PositionOutOfTagRange"), "{rendered}");
}

#[test]
fn advance_removes_particles_that_fell_below_the_playfield() {
    // Arrange
    let mut session = new_session();
    let particle = session
        .world
        .particle_system_view(session.particle_system)
        .expect("dam break particles")
        .particle_ids()[0];
    session
        .world
        .set_particle_position(particle, Vec2::new(8.0, -20.0))
        .expect("a finite position is accepted");
    let before = session.live_particle_count().expect("live count");

    // Act
    session
        .advance(1)
        .expect("escaped particles are removed before the step");
    let after = session.live_particle_count().expect("live count");

    // Assert
    assert!(after < before);
    assert!(session.failure_detail().is_empty());
}

#[test]
fn escape_follows_gravity_and_falls_back_to_downward() {
    // Arrange
    let upright = Vec2::new(0.0, -10.0);
    let sideways = Vec2::new(10.0, 0.0);

    // Act / Assert
    assert!(escape::particle_has_escaped(Vec2::new(0.0, -12.1), upright));
    assert!(!escape::particle_has_escaped(
        Vec2::new(0.0, -11.0),
        upright
    ));
    assert!(escape::particle_has_escaped(Vec2::new(12.1, 0.0), sideways));
    assert!(!escape::particle_has_escaped(
        Vec2::new(0.0, -20.0),
        sideways
    ));
    assert!(escape::particle_has_escaped(
        Vec2::new(0.0, -12.1),
        Vec2::new(0.0, 0.0)
    ));
    assert!(escape::particle_has_escaped(
        Vec2::new(f32::NAN, 0.0),
        upright
    ));
}
