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
