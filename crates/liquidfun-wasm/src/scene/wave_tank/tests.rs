use liquidfun::math::Vec2;
use liquidfun::{BodyType, JointDef, JointKind, WorldObservationLimits};

use super::build;
use crate::scene::{SceneId, build_scene};
use crate::session::{SessionCore, SessionError};

const FAR_WINDOW_MIN_X: f32 = 3.76;
/// Past the default 0.92 m paddle, on the flat floor.
const CHANNEL_MIN_X: f32 = 1.0;
const PARTICLE_DIAMETER: f32 = 0.11 * 1.5 * 4.0 / 50.0 * 2.0;

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
    assert_eq!(live_count, 2500, "the pool starts with 2500 particles");
    assert!(
        velocities.iter().copied().all(velocity_is_zero),
        "a fresh pool has no particle velocity"
    );
    assert!(
        !far_window.is_empty(),
        "the far-wall sample should start filled"
    );
    let channel_top = maximum_y(
        &positions
            .iter()
            .copied()
            .filter(|position| position.x >= CHANNEL_MIN_X)
            .collect::<Vec<_>>(),
    );
    let far_top = maximum_y(&far_window);
    assert!(
        (channel_top - far_top).abs() <= PARTICLE_DIAMETER,
        "the far surface starts within one diameter of the channel surface"
    );
}

#[test]
fn horizontal_device_tilt_preserves_the_pool_particles() {
    for gravity_x in [-10.0, 10.0] {
        // Arrange
        let mut session =
            SessionCore::create(SceneId::WaveTank).expect("wave tank should construct");
        let starting_count = session.live_particle_count().expect("water should be live");
        session
            .set_gravity(gravity_x, 0.0)
            .expect("horizontal device tilt should be supported");

        // Act
        session.advance(4).expect("the tilted pool should step");
        let final_count = session.live_particle_count().expect("water should remain");

        // Assert
        assert_eq!(final_count, starting_count);
    }
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
            .expect("the half period should finish on the crest");
    }
    let end_count = session
        .live_particle_count()
        .expect("the same water should still be live");
    let translation = prismatic_translation(&session);

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
        particles_stay_in_the_tank(&session),
        "particles should stay inside the tank"
    );

    let mut rose = false;
    for _ in 0..(FAR_WALL_WAIT_STEPS / 4) {
        session
            .advance(4)
            .expect("the wave should have time to reach the far wall");
        rose = session.read_particles(|world, system| {
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
        if rose {
            break;
        }
    }
    let (later_count, buried) = session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should still be live");
        let buried = view
            .positions()
            .iter()
            .copied()
            .find(|position| position.y < -0.024 || position.x < -0.012 || position.x > 4.012);
        (view.particle_ids().len(), buried)
    });
    assert!(
        rose,
        "an original far-wall particle should rise by one diameter"
    );
    assert_eq!(later_count, start_count);
    assert!(
        buried.is_none(),
        "water should stay above the floor, found {buried:?}"
    );
}

#[test]
fn motor_speed_follows_the_sine() {
    // Arrange
    let mut first = SessionCore::create(SceneId::WaveTank).expect("wave tank should construct");
    let mut crest = SessionCore::create(SceneId::WaveTank).expect("wave tank should construct");
    let expected = PEAK_SPEED * (SPEED * std::f32::consts::TAU * SIM_DT / PERIOD).sin();

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
fn platform_anchor_starts_at_the_default_width() {
    // Arrange
    let session = SessionCore::create(SceneId::WaveTank).expect("wave tank should construct");

    // Act
    let position = dynamic_body_position(&session);

    // Assert
    assert_eq!(
        position.x.to_bits(),
        super::DEFAULT_PLATFORM_WIDTH.to_bits()
    );
    assert_eq!(position.y.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn wider_platform_preset_moves_the_anchor() {
    // Arrange
    let presets = [("platform-width".to_owned(), "0.64".to_owned())];

    // Act
    let built = build(&presets).expect("a wider paddle should still construct");
    let position = platform_position(&built.world);

    // Assert
    assert_eq!(position.x.to_bits(), (f32::from(64_u16) / 100.0).to_bits());
}

#[test]
fn speed_and_amplitude_scale_the_sine_without_rebuilding() {
    // Arrange
    let mut session = SessionCore::create(SceneId::WaveTank).expect("wave tank should construct");
    let doubled_speed = session.apply_control("platform-speed", "2.0");
    let doubled_stroke = session.apply_control("platform-amplitude", "0.080");
    let applied_peak = 0.08 * 2.0 * std::f32::consts::TAU / (2.0 * PERIOD);
    let expected = applied_peak * (2.0 * std::f32::consts::TAU * SIM_DT / PERIOD).sin();

    // Act
    session
        .advance(1)
        .expect("the first step should write the scaled sine sample");
    let speed = prismatic_motor_speed(&session);
    let count = session.live_particle_count().expect("water should remain");

    // Assert
    assert_eq!(doubled_speed, Ok(false));
    assert_eq!(doubled_stroke, Ok(false));
    assert!(
        (speed - expected).abs() <= 1.0e-5,
        "scaled speed {speed} should match {expected}"
    );
    assert_eq!(count, 2500);
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

const PARTICLE_RADIUS: f32 = 0.11 * 1.5 * 4.0 / 50.0;
const SPEED: f32 = 0.4;
const STROKE: f32 = 0.168;
const PERIOD: f32 = 2.4;
const SIM_DT: f32 = 1.0 / 60.0;
const PEAK_SPEED: f32 = STROKE * SPEED * std::f32::consts::TAU / (2.0 * PERIOD);
/// Half a cycle at 0.4× is 3 seconds.
const HALF_PERIOD_STEPS: u32 = 180;
/// The wave still needs a few seconds after the crest to reach the far wall.
const FAR_WALL_WAIT_STEPS: u32 = 600;

fn particles_stay_in_the_tank(session: &SessionCore) -> bool {
    session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should still be live");
        view.positions().iter().all(|position| {
            position.x > -PARTICLE_DIAMETER
                && position.x < 4.0 + PARTICLE_DIAMETER
                && position.y > -0.024
        })
    })
}

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
    assert_eq!(live_count, 2500, "the pool starts with 2500 particles");
    assert!(
        velocities.iter().copied().all(velocity_is_zero),
        "a fresh pool has no particle velocity"
    );
    assert!(
        !far_window.is_empty(),
        "the far-wall sample should start filled"
    );
    let channel_top = maximum_y(
        &positions
            .iter()
            .copied()
            .filter(|position| position.x >= CHANNEL_MIN_X)
            .collect::<Vec<_>>(),
    );
    let far_top = maximum_y(&far_window);
    assert!(
        (channel_top - far_top).abs() <= PARTICLE_DIAMETER,
        "the far surface starts within one diameter of the channel surface"
    );
}

fn dynamic_body_position(session: &SessionCore) -> Vec2 {
    session.read_particles(|world, _system| platform_position(world))
}

fn platform_position(world: &liquidfun::World) -> Vec2 {
    let observation = world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("reviewed observation should include the platform");
    observation
        .bodies()
        .iter()
        .find(|body| body.snapshot().body_type() == BodyType::Dynamic)
        .expect("the platform is the dynamic body")
        .snapshot()
        .position()
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

#[test]
fn platform_right_face_is_as_thick_as_the_tank_wall() {
    // Arrange
    let layout = super::wave_layout(super::DEFAULT_PLATFORM_WIDTH, 0.0);
    let rise = 0.0064;

    // Act
    let face = super::platform_face(&layout, 0.0);
    let raised = super::platform_face(&layout, rise);

    // Assert
    let thickness = face.half_width * 2.0;
    assert_eq!(thickness.to_bits(), (super::WALL_HALF * 2.0).to_bits());
    assert_eq!(
        (face.center.y + face.half_height).to_bits(),
        0.0_f32.to_bits()
    );
    assert_eq!(
        (face.center.y - face.half_height).to_bits(),
        super::FLOOR_BOTTOM_Y.to_bits()
    );
    let slab_right = layout.pad_center.x + layout.half_width;
    assert_eq!(
        (face.center.x + face.half_width).to_bits(),
        slab_right.to_bits()
    );
    let raised_bottom = rise + raised.center.y - raised.half_height;
    assert_eq!(raised_bottom.to_bits(), super::FLOOR_BOTTOM_Y.to_bits());
}

#[test]
fn default_slant_lifts_the_back_above_the_spill_edge() {
    // Arrange
    let flat = super::wave_layout(super::DEFAULT_PLATFORM_WIDTH, 0.0);
    let sloped = super::wave_layout(super::DEFAULT_PLATFORM_WIDTH, super::default_slant());

    // Act
    let flat_back = highest_corner(&super::plate_corners(&flat));
    let sloped_corners = super::plate_corners(&sloped);
    let sloped_back = highest_corner(&sloped_corners);
    let spill = nearest_origin(&sloped_corners);

    // Assert
    assert_eq!(flat_back.y.to_bits(), 0.0_f32.to_bits());
    assert!(sloped_back.y > 0.07, "a 5 degree plate raises the back");
    assert!(
        sloped_back.x < 0.0,
        "the high end stays toward the back wall"
    );
    assert!(spill.x.abs() < 1.0e-4 && spill.y.abs() < 1.0e-4);
}

#[test]
fn slant_recreates_and_amplitude_reaches_its_maximum() {
    // Arrange
    let mut session = SessionCore::create(SceneId::WaveTank).expect("wave tank should construct");

    // Act
    let slant = session.apply_control("platform-slant", "30");
    let rejected = session.apply_control("platform-slant", "31");
    let amplitude = session.apply_control("platform-amplitude", "0.320");

    // Assert
    assert_eq!(slant, Ok(true));
    assert!(matches!(rejected, Err(SessionError::UnknownControl)));
    assert_eq!(amplitude, Ok(false));
}

fn highest_corner(corners: &[Vec2; 4]) -> Vec2 {
    let mut highest = corners[0];
    for corner in corners.iter().skip(1) {
        if corner.y > highest.y {
            highest = *corner;
        }
    }
    highest
}

fn nearest_origin(corners: &[Vec2; 4]) -> Vec2 {
    let mut nearest = corners[0];
    let mut nearest_distance = corner_distance_squared(nearest);
    for corner in corners.iter().skip(1) {
        let distance = corner_distance_squared(*corner);
        if distance < nearest_distance {
            nearest = *corner;
            nearest_distance = distance;
        }
    }
    nearest
}

fn corner_distance_squared(corner: Vec2) -> f32 {
    corner.x * corner.x + corner.y * corner.y
}

fn velocity_is_zero(velocity: Vec2) -> bool {
    velocity.x.to_bits() == 0.0_f32.to_bits() && velocity.y.to_bits() == 0.0_f32.to_bits()
}

fn maximum_y(positions: &[Vec2]) -> f32 {
    positions
        .iter()
        .map(|position| position.y)
        .max_by(f32::total_cmp)
        .expect("the sample should contain a surface")
}
