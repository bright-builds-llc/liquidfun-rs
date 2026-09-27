use liquidfun::{BodyType, JointDef, JointKind, WorldObservationLimits};

use super::build;
use crate::ProofFrame;
use crate::scene::{SceneId, build_scene};
use crate::session::{SessionCore, SessionError};

const THROAT_X: f32 = 0.0;
const FOUNTAIN_SIDE_X: f32 = 0.04;
const ADVANCE_SPEED: f32 = 0.6;
const STEPS_PER_PERIOD: u32 = 120;

#[test]
fn fresh_build_places_water_only_in_the_piston_chamber() {
    // Arrange
    let session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");

    // Act
    let positions = session.read_particles(|world, system| {
        world
            .particle_system_view(system)
            .expect("the water system should be live")
            .positions()
            .to_vec()
    });

    // Assert
    assert!(
        !positions.is_empty(),
        "the piston chamber should start with water"
    );
    assert!(
        positions.iter().all(|position| position.x < THROAT_X),
        "every water particle starts on the piston side of the throat"
    );
    assert!(
        positions
            .iter()
            .all(|position| position.x <= FOUNTAIN_SIDE_X),
        "the fountain side starts empty"
    );
}

#[test]
fn piston_body_is_dynamic() {
    // Arrange
    let session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");

    // Act
    let dynamic_count = session.read_particles(|world, _system| {
        world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the piston")
            .bodies()
            .iter()
            .filter(|body| body.snapshot().body_type() == BodyType::Dynamic)
            .count()
    });

    // Assert
    assert_eq!(dynamic_count, 1, "the piston is the only dynamic body");
}

#[test]
fn motor_speed_follows_the_half_period() {
    // Arrange
    let mut session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");

    // Act
    session
        .advance(1)
        .expect("the first step should write the advance speed");
    let advance_speed = prismatic_motor_speed(&session);
    for _ in 0..16 {
        session
            .advance(4)
            .expect("the second half of the period should still step");
    }
    let retract_speed = prismatic_motor_speed(&session);

    // Assert
    assert_eq!(advance_speed.to_bits(), ADVANCE_SPEED.to_bits());
    assert_eq!(retract_speed.to_bits(), (-ADVANCE_SPEED).to_bits());
}

#[test]
fn controls_and_pointer_stay_watch_first() {
    // Arrange
    let mut session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");

    // Act
    let period = session.apply_control("period", "2");
    let aim = session.apply_action("aim");
    let pointer = session.apply_pointer("up", 0.0, 0.5);

    // Assert
    assert_eq!(period, Err(SessionError::UnknownControl));
    assert_eq!(aim, Err(SessionError::UnknownControl));
    assert_eq!(pointer, Ok(()));
}

#[test]
fn pointer_up_leaves_the_live_count_unchanged() {
    // Arrange
    let mut session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");
    let before = session
        .live_particle_count()
        .expect("the starting water should be countable");

    // Act
    session
        .apply_pointer("up", 0.0, 0.5)
        .expect("pointer up is a watch-first no-op");
    let after = session
        .live_particle_count()
        .expect("pointer up should leave the water in place");

    // Assert
    assert_eq!(after, before);
}

#[test]
fn non_empty_presets_are_rejected_and_gravity_is_stripped() {
    // Arrange
    let period = [("period".to_owned(), "2".to_owned())];
    let gravity = [("gravity".to_owned(), "10".to_owned())];

    // Act
    let rejected = build(&period);
    let built = build_scene(SceneId::HydraulicFountain, &gravity);

    // Assert
    assert!(matches!(rejected, Err(SessionError::UnknownControl)));
    let built = built.expect("gravity is stripped before the scene builder");
    assert_eq!(built.world.gravity().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(built.world.gravity().y.to_bits(), (-10.0_f32).to_bits());
}

#[test]
fn piston_chamber_particles_cross_the_throat_within_one_period() {
    // Arrange
    let mut session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");
    let started = session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should be live");
        view.particle_ids()
            .iter()
            .copied()
            .zip(view.positions().iter().copied())
            .collect::<Vec<_>>()
    });
    let start_count = session
        .live_particle_count()
        .expect("the piston chamber should start with water");

    // Act
    for _ in 0..(STEPS_PER_PERIOD / 4) {
        session
            .advance(4)
            .expect("one period is thirty batches of four steps");
    }
    let end_count = session
        .live_particle_count()
        .expect("the same water should still be live");
    let crossed = session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should still be live");
        started.iter().any(|(id, _start)| {
            view.particle_ids()
                .iter()
                .zip(view.positions())
                .any(|(live, position)| live == id && position.x > FOUNTAIN_SIDE_X)
        })
    });

    // Assert
    assert!(
        start_count > 0,
        "the piston chamber should start with water"
    );
    assert_eq!(
        started.len(),
        start_count,
        "the start snapshot includes every live id"
    );
    assert!(
        started.iter().all(|(_id, position)| position.x < THROAT_X),
        "every start position is on the piston side of the throat"
    );
    assert!(
        started
            .iter()
            .all(|(_id, position)| position.x <= FOUNTAIN_SIDE_X),
        "no start position is on the fountain side"
    );
    assert_eq!(end_count, start_count);
    assert!(
        crossed,
        "at least one original particle id is on the fountain side after one period"
    );
}

#[test]
fn rebuild_restores_the_initial_liquid_layout() {
    // Arrange
    let mut session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");
    for _ in 0..(STEPS_PER_PERIOD / 4) {
        session
            .advance(4)
            .expect("the first period should step before rebuild");
    }
    drop(session);

    // Act
    let rebuilt = SessionCore::create(SceneId::HydraulicFountain)
        .expect("a new session should restore the authored layout");
    let positions = rebuilt.read_particles(|world, system| {
        world
            .particle_system_view(system)
            .expect("the rebuilt water system should be live")
            .positions()
            .to_vec()
    });
    let translation = rebuilt.read_particles(|world, _system| {
        let observation = world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the piston joint");
        let mut prismatic = observation
            .joints()
            .iter()
            .filter(|joint| joint.snapshot().kind() == JointKind::Prismatic);
        let joint = prismatic
            .next()
            .expect("the rebuilt piston rides one prismatic joint");
        assert!(
            prismatic.next().is_none(),
            "rebuild has a single prismatic joint"
        );
        world
            .prismatic_joint_translation(joint.id())
            .expect("the retracted limit is readable")
    });

    // Assert
    assert!(
        !positions.is_empty(),
        "rebuild places the water group again"
    );
    assert!(
        positions.iter().all(|position| position.x < THROAT_X),
        "rebuild puts every particle back on the piston side"
    );
    assert_eq!(translation.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn captured_frame_draws_the_moving_piston() {
    // Arrange
    let mut session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");
    let initial_right_face = piston_right_face_x(&session);

    // Act
    for _ in 0..5 {
        session.advance(4).expect("the piston should keep stepping");
    }
    let later_right_face = piston_right_face_x(&session);

    // Assert
    assert!(
        later_right_face > initial_right_face,
        "the drawn piston face moves toward the throat"
    );
}

fn piston_right_face_x(session: &SessionCore) -> f32 {
    let segments = capture(session).rigid_segments();
    assert_eq!(
        segments.len(),
        32,
        "four wall segments and four piston edges"
    );
    segments
        .chunks(4)
        .filter_map(|segment| {
            let start_x = segment[0];
            let end_x = segment[2];
            if start_x < -0.2 && end_x < -0.2 {
                Some(start_x.max(end_x))
            } else {
                None
            }
        })
        .max_by(f32::total_cmp)
        .expect("the piston edges should be left of the throat")
}

fn capture(session: &SessionCore) -> ProofFrame {
    ProofFrame::from(
        session
            .capture_frame()
            .expect("hydraulic fountain should capture a frame"),
    )
}

fn prismatic_motor_speed(session: &SessionCore) -> f32 {
    session.read_particles(|world, _system| {
        let observation = world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the piston joint");
        let joint = observation
            .joints()
            .iter()
            .find(|joint| joint.snapshot().kind() == JointKind::Prismatic)
            .expect("the piston rides a prismatic joint");
        let JointDef::Prismatic(definition) = joint.snapshot().definition() else {
            panic!("the piston joint should be prismatic");
        };
        definition.motor_speed()
    })
}
