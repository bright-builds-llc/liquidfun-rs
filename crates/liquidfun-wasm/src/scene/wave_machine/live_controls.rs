//! Live speed and tilt edits keep the wave absolute about level.

use std::f32::consts::PI;

use liquidfun::{
    BodyType, JointDef, NoDecisionHook, ParticleSystemId, StepConfiguration, StepLimits, World,
    WorldObservationLimits,
};

use super::super::{BuiltScene, SceneHooks};
use super::build;

#[test]
fn wave_speed_edit_keeps_the_current_phase() {
    // Arrange
    let BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = build(&[]).expect("Wave Machine should construct");
    let advances = 40_u16;
    for _ in 0..advances {
        hooks
            .on_advance(&mut world, particle_system)
            .expect("on_advance must accumulate phase");
    }

    // Act
    hooks
        .apply_control(&mut world, particle_system, "wave-speed", "4.0")
        .expect("4.0 is on the speed slider");
    let speed = revolute_motor_speed(&world);
    let phase = f32::from(advances) / 60.0;
    let kept = 0.05 * PI * 4.0 * phase.cos();
    let jumped = 0.05 * PI * 4.0 * (4.0 * phase).cos();

    // Assert
    assert!(
        (speed - kept).abs() < 1.0e-4,
        "speed must stay on the current phase (got {speed}, expected {kept})"
    );
    assert!(
        (speed - jumped).abs() > 0.05,
        "speed must not jump to speed * time (got {speed}, jumped {jumped})"
    );
}

#[test]
fn simulated_tilt_increase_rocks_about_level() {
    // Arrange — reach the pinned crest, then ask for a wider absolute peak.
    let BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = build(&[]).expect("Wave Machine should construct");
    hooks
        .apply_control(&mut world, particle_system, "wave-speed", "10.0")
        .expect("10× reaches the crest quickly");
    step_wave(&mut world, hooks.as_mut(), particle_system, 10);
    let leaned = tank_angle(&world);
    assert!(
        leaned > 0.10,
        "setup must leave level before the tilt edit (got {leaned})"
    );

    // Act
    hooks
        .apply_control(&mut world, particle_system, "wave-tilt", "18")
        .expect("18 degrees still contains the crest");
    let (min_angle, max_angle) = step_angle_range(&mut world, hooks.as_mut(), particle_system, 48);

    // Assert
    let center = 0.5 * (min_angle + max_angle);
    assert!(
        center.abs() < 0.08,
        "18° must rock about level (center {center}, range {min_angle}..{max_angle})"
    );
    assert!(
        max_angle > 0.22 && min_angle < -0.22,
        "18° must reach both sides of level (range {min_angle}..{max_angle})"
    );
}

#[test]
fn simulated_tilt_cut_eases_inside_the_new_peak() {
    // Arrange — reach the pinned crest, then cut below that angle.
    let BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = build(&[]).expect("Wave Machine should construct");
    hooks
        .apply_control(&mut world, particle_system, "wave-speed", "10.0")
        .expect("10× reaches the crest quickly");
    step_wave(&mut world, hooks.as_mut(), particle_system, 10);
    let leaned = tank_angle(&world);
    assert!(
        leaned > 0.10,
        "setup must reach a lean a 3° peak cannot contain (got {leaned})"
    );

    // Act
    hooks
        .apply_control(&mut world, particle_system, "wave-tilt", "3")
        .expect("3 degrees is inside the slider and below the current lean");
    step_wave(&mut world, hooks.as_mut(), particle_system, 70);
    let (min_angle, max_angle) = step_angle_range(&mut world, hooks.as_mut(), particle_system, 40);

    // Assert
    let center = 0.5 * (min_angle + max_angle);
    assert!(
        center.abs() < 0.06,
        "after a tilt cut the rock must center on level (center {center})"
    );
    assert!(
        max_angle < 0.12 && min_angle > -0.12,
        "3° peaks must stay inside the old crest (range {min_angle}..{max_angle})"
    );
    assert!(
        max_angle > 0.02 && min_angle < -0.02,
        "3° must still rock (range {min_angle}..{max_angle})"
    );
}

#[test]
fn simulated_zero_tilt_returns_to_level() {
    // Arrange — reach the pinned crest, then ask for level.
    let BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = build(&[]).expect("Wave Machine should construct");
    hooks
        .apply_control(&mut world, particle_system, "wave-speed", "10.0")
        .expect("10× reaches the crest quickly");
    step_wave(&mut world, hooks.as_mut(), particle_system, 10);
    let leaned = tank_angle(&world);
    assert!(
        leaned.abs() > 0.10,
        "setup must leave level before the tilt edit (got {leaned})"
    );

    // Act
    hooks
        .apply_control(&mut world, particle_system, "wave-tilt", "0")
        .expect("0 degrees asks for level");
    step_wave(&mut world, hooks.as_mut(), particle_system, 80);
    let settled = tank_angle(&world);
    step_wave(&mut world, hooks.as_mut(), particle_system, 30);
    let held = tank_angle(&world);

    // Assert
    assert!(
        settled.abs() < 0.05,
        "0° must ease the tank back to level (got {settled})"
    );
    assert!(
        held.abs() < 0.05,
        "0° must hold the tank level (got {held})"
    );
}

fn step_angle_range(
    world: &mut World,
    hooks: &mut dyn SceneHooks,
    particle_system: ParticleSystemId,
    steps: u32,
) -> (f32, f32) {
    let step = StepConfiguration::new(1.0 / 60.0, 8, 3)
        .expect("valid step")
        .with_particle_iterations(2)
        .expect("two particle iterations");
    let limits = StepLimits::default();
    let mut decision = NoDecisionHook;
    let mut min_angle = f32::MAX;
    let mut max_angle = f32::MIN;
    for _ in 0..steps {
        hooks
            .on_advance(world, particle_system)
            .expect("on_advance must update the motor");
        world
            .step(step, &mut decision, limits)
            .expect("Wave Machine step should succeed");
        let angle = tank_angle(world);
        min_angle = min_angle.min(angle);
        max_angle = max_angle.max(angle);
    }
    (min_angle, max_angle)
}

fn step_wave(
    world: &mut World,
    hooks: &mut dyn SceneHooks,
    particle_system: ParticleSystemId,
    steps: u32,
) {
    step_angle_range(world, hooks, particle_system, steps);
}

fn tank_angle(world: &World) -> f32 {
    let observation = world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("reviewed observation should include the tank");
    let dynamic: Vec<_> = observation
        .bodies()
        .iter()
        .map(|body| body.snapshot())
        .filter(|snapshot| snapshot.body_type() == BodyType::Dynamic)
        .collect();
    assert_eq!(dynamic.len(), 1, "exactly one dynamic tank");
    dynamic[0].angle()
}

fn revolute_motor_speed(world: &World) -> f32 {
    let observation = world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("reviewed observation should include the motorized revolute");
    let revolute: Vec<_> = observation
        .joints()
        .iter()
        .filter_map(|joint| match joint.snapshot().definition() {
            JointDef::Revolute(definition) => Some(definition),
            _ => None,
        })
        .collect();
    assert_eq!(revolute.len(), 1, "exactly one revolute joint");
    revolute[0].motor_speed()
}
