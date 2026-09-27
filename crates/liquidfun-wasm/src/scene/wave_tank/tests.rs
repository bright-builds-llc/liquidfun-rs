use liquidfun::math::Vec2;
use liquidfun::{BodyType, JointDef, JointKind, WorldObservationLimits};

use super::build;
use crate::scene::{SceneId, build_scene};
use crate::session::{SessionCore, SessionError};

const FAR_WINDOW_MIN_X: f32 = 1.20;
const PARTICLE_DIAMETER: f32 = 0.05;

#[test]
fn pool_starts_as_a_still_band() {
    // Arrange
    let session = SessionCore::create(SceneId::WaveTank).expect("wave tank should construct");

    // Act
    let (live_count, positions, velocities) = session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should be live");
        (
            view.particle_ids().len(),
            view.positions().to_vec(),
            view.velocities().to_vec(),
        )
    });
    let far_window: Vec<Vec2> = positions
        .iter()
        .copied()
        .filter(|position| position.x >= FAR_WINDOW_MIN_X)
        .collect();

    // Assert
    assert!(live_count > 0, "the pool should start with water");
    assert!(
        velocities.iter().all(velocity_is_zero),
        "a fresh pool has no particle velocity"
    );
    assert!(
        !far_window.is_empty(),
        "the far-wall sample should start filled"
    );
    let pool_top = maximum_y(&positions);
    let far_top = maximum_y(&far_window);
    assert!(
        (pool_top - far_top).abs() <= PARTICLE_DIAMETER,
        "the far surface starts within one diameter of the pool surface"
    );
}

#[test]
fn only_the_platform_is_dynamic() {
    // Arrange
    let session = SessionCore::create(SceneId::WaveTank).expect("wave tank should construct");

    // Act
    let (dynamic_count, axis) = session.read_particles(|world, _system| {
        let observation = world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the platform");
        let dynamic_count = observation
            .bodies()
            .iter()
            .filter(|body| body.snapshot().body_type() == BodyType::Dynamic)
            .count();
        let joint = observation
            .joints()
            .iter()
            .find(|joint| joint.snapshot().kind() == JointKind::Prismatic)
            .expect("the platform rides a prismatic joint");
        let JointDef::Prismatic(definition) = joint.snapshot().definition() else {
            panic!("the platform joint should be prismatic");
        };
        (dynamic_count, definition.local_axis_a())
    });

    // Assert
    assert_eq!(dynamic_count, 1, "the platform is the only dynamic body");
    assert_eq!(axis.x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(axis.y.to_bits(), 1.0_f32.to_bits());
}

#[test]
fn unknown_controls_and_pointer_leave_the_pool_alone() {
    // Arrange
    let mut session = SessionCore::create(SceneId::WaveTank).expect("wave tank should construct");
    let before = session
        .live_particle_count()
        .expect("the starting water should be countable");

    // Act
    let period = session.apply_control("period", "2");
    let amplitude = session.apply_control("amplitude", "0.16");
    let stroke = session.apply_action("stroke");
    let pointer = session.apply_pointer("up", 0.5, 0.2);
    let after = session
        .live_particle_count()
        .expect("pointer up should leave the water in place");

    // Assert
    assert_eq!(period, Err(SessionError::UnknownControl));
    assert_eq!(amplitude, Err(SessionError::UnknownControl));
    assert_eq!(stroke, Err(SessionError::UnknownControl));
    assert_eq!(pointer, Ok(()));
    assert_eq!(after, before);
}

#[test]
fn non_empty_presets_are_rejected_and_gravity_is_stripped() {
    // Arrange
    let period = [("period".to_owned(), "2".to_owned())];
    let gravity = [("gravity".to_owned(), "10".to_owned())];

    // Act
    let rejected = build(&period);
    let built = build_scene(SceneId::WaveTank, &gravity);

    // Assert
    assert!(matches!(rejected, Err(SessionError::UnknownControl)));
    let built = built.expect("gravity is stripped before the scene builder");
    assert_eq!(built.world.gravity().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(built.world.gravity().y.to_bits(), (-10.0_f32).to_bits());
}

#[test]
fn far_wall_particles_rise_after_a_half_cycle() {
    // Arrange
    let mut session = SessionCore::create(SceneId::WaveTank).expect("wave tank should construct");
    let started = far_window_particles(&session);
    let start_count = session
        .live_particle_count()
        .expect("the starting pool should be countable");

    // Act
    for _ in 0..(HALF_PERIOD_STEPS / 4) {
        session
            .advance(4)
            .expect("a half period is fifteen batches of four steps");
    }
    let end_count = session
        .live_particle_count()
        .expect("the same water should still be live");
    let translation = prismatic_translation(&session);
    let rose = session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should still be live");
        started.iter().any(|(id, start)| {
            view.particle_ids()
                .iter()
                .zip(view.positions())
                .any(|(live, position)| {
                    live == id && position.y > start.y + (2.0 * PARTICLE_RADIUS)
                })
        })
    });

    // Assert
    assert!(
        !started.is_empty(),
        "the far window starts over the fixed floor"
    );
    assert!(
        translation >= 0.5 * STROKE,
        "the platform should be at least halfway up the stroke, was {translation}"
    );
    assert_eq!(end_count, start_count);
    assert!(
        rose,
        "an original far-wall particle should rise by one diameter"
    );
}

#[test]
fn motor_speed_follows_the_sine() {
    // Arrange
    let mut first = SessionCore::create(SceneId::WaveTank).expect("wave tank should construct");
    let mut crest = SessionCore::create(SceneId::WaveTank).expect("wave tank should construct");
    let expected = PEAK_SPEED * (std::f32::consts::TAU * SIM_DT / PERIOD).sin();

    // Act
    first
        .advance(1)
        .expect("the first step should write the sine sample");
    let first_speed = prismatic_motor_speed(&first);
    for _ in 0..(HALF_PERIOD_STEPS / 4) {
        crest
            .advance(4)
            .expect("the half period should finish on a near-zero sine sample");
    }
    let crest_speed = prismatic_motor_speed(&crest);

    // Assert
    assert!(
        (first_speed - expected).abs() <= 1.0e-5,
        "first speed {first_speed} should match {expected}"
    );
    assert!(
        crest_speed.abs() <= 1.0e-3,
        "half-period speed should be near zero, was {crest_speed}"
    );
}

#[test]
fn rebuild_restores_the_still_pool() {
    // Arrange
    let mut session = SessionCore::create(SceneId::WaveTank).expect("wave tank should construct");
    for _ in 0..(HALF_PERIOD_STEPS / 4) {
        session
            .advance(4)
            .expect("the half cycle should step before rebuild");
    }
    drop(session);

    // Act
    let rebuilt = SessionCore::create(SceneId::WaveTank).expect("rebuild should construct");
    let translation = prismatic_translation(&rebuilt);

    // Assert
    assert_eq!(translation.to_bits(), 0.0_f32.to_bits());
    assert_still_band(&rebuilt);
}

#[test]
fn source_writes_a_prismatic_sine() {
    // Arrange
    let source = include_str!("../wave_tank.rs");

    // Act / Assert
    assert!(
        source.contains("set_prismatic_motor_speed"),
        "on_advance must call set_prismatic_motor_speed"
    );
    assert!(!source.contains("set_revolute_motor_speed"));
    assert!(!source.contains("create_particle_with_def"));
    assert!(!source.contains("with_destruction_by_age"));
}

const PARTICLE_RADIUS: f32 = 0.025;
const STROKE: f32 = 0.16;
const PERIOD: f32 = 2.0;
const SIM_DT: f32 = 1.0 / 60.0;
const PEAK_SPEED: f32 = STROKE * std::f32::consts::TAU / (2.0 * PERIOD);
const HALF_PERIOD_STEPS: u32 = (PERIOD * 30.0) as u32;

fn far_window_particles(session: &SessionCore) -> Vec<(liquidfun::ParticleId, Vec2)> {
    session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should be live");
        view.particle_ids()
            .iter()
            .copied()
            .zip(view.positions().iter().copied())
            .filter(|(_id, position)| position.x >= FAR_WINDOW_MIN_X)
            .collect()
    })
}

fn assert_still_band(session: &SessionCore) {
    let (live_count, positions, velocities) = session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should be live");
        (
            view.particle_ids().len(),
            view.positions().to_vec(),
            view.velocities().to_vec(),
        )
    });
    let far_window: Vec<Vec2> = positions
        .iter()
        .copied()
        .filter(|position| position.x >= FAR_WINDOW_MIN_X)
        .collect();
    assert!(live_count > 0, "the pool should start with water");
    assert!(
        velocities.iter().all(velocity_is_zero),
        "a fresh pool has no particle velocity"
    );
    assert!(
        !far_window.is_empty(),
        "the far-wall sample should start filled"
    );
    let pool_top = maximum_y(&positions);
    let far_top = maximum_y(&far_window);
    assert!(
        (pool_top - far_top).abs() <= PARTICLE_DIAMETER,
        "the far surface starts within one diameter of the pool surface"
    );
}

fn prismatic_translation(session: &SessionCore) -> f32 {
    session.read_particles(|world, _system| {
        let observation = world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the platform joint");
        let joint = observation
            .joints()
            .iter()
            .find(|joint| joint.snapshot().kind() == JointKind::Prismatic)
            .expect("the platform rides a prismatic joint");
        world
            .prismatic_joint_translation(joint.id())
            .expect("the platform translation should be readable")
    })
}

fn prismatic_motor_speed(session: &SessionCore) -> f32 {
    session.read_particles(|world, _system| {
        let observation = world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the platform joint");
        let joint = observation
            .joints()
            .iter()
            .find(|joint| joint.snapshot().kind() == JointKind::Prismatic)
            .expect("the platform rides a prismatic joint");
        let JointDef::Prismatic(definition) = joint.snapshot().definition() else {
            panic!("the platform joint should be prismatic");
        };
        definition.motor_speed()
    })
}

fn velocity_is_zero(velocity: &Vec2) -> bool {
    velocity.x.to_bits() == 0.0_f32.to_bits() && velocity.y.to_bits() == 0.0_f32.to_bits()
}

fn maximum_y(positions: &[Vec2]) -> f32 {
    positions
        .iter()
        .map(|position| position.y)
        .max_by(f32::total_cmp)
        .expect("the sample should contain a surface")
}
