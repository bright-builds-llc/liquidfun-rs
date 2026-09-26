use std::f32::consts::PI;

use liquidfun::{BodyType, JointDef, World, WorldObservationLimits};

use super::super::SceneId;
use super::build;
use crate::session::{SessionCore, SessionError};

#[test]
fn create_builds_wave_machine_with_particles() {
    // Arrange / Act
    let session = SessionCore::create(SceneId::WaveMachine)
        .expect("Wave Machine should construct the pinned motorized tank");

    // Assert
    assert!(
        session.particle_count() > 0,
        "fill box must create at least one particle"
    );
}

#[test]
fn default_wave_speed_matches_the_pinned_motor() {
    // Arrange
    let super::super::BuiltScene { world, .. } = build(&[]).expect("Wave Machine should construct");

    // Act
    let initial = revolute_motor_speed(&world);

    // Assert
    assert!(
        (initial - 0.05 * PI).abs() < 1.0e-5,
        "the tank must start at the pinned 1× speed (got {initial})"
    );
}

#[test]
fn zero_wave_speed_stays_stopped() {
    // Arrange
    let super::super::BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = build(&[]).expect("Wave Machine should construct");
    hooks
        .apply_control(&mut world, particle_system, "wave-speed", "0.0")
        .expect("0.0 stops the motor");

    // Act
    let initial = revolute_motor_speed(&world);
    for _ in 0..30 {
        hooks
            .on_advance(&mut world, particle_system)
            .expect("on_advance must keep a zero multiplier stopped");
    }
    let after = revolute_motor_speed(&world);

    // Assert
    assert!(
        initial.abs() < 1.0e-6,
        "zero wave speed must stop the tank (got {initial})"
    );
    assert!(
        after.abs() < 1.0e-6,
        "a zero wave speed must stay stopped after advances (got {after})"
    );
}

#[test]
fn original_speed_follows_sim_time_after_advances() {
    // Arrange — JS Step order: t += 1/60 then set_revolute_motor_speed(1 * 0.05 * cos(t) * π)
    let super::super::BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = build(&[]).expect("Wave Machine should construct the pinned motorized tank");
    let advances = 7_u32;
    hooks
        .apply_control(&mut world, particle_system, "wave-speed", "1.0")
        .expect("1.0 restores the pinned amplitude");

    // Act
    for _ in 0..advances {
        hooks
            .on_advance(&mut world, particle_system)
            .expect("on_advance must update motor from sim time");
    }
    let speed = revolute_motor_speed(&world);
    let expected = 0.05 * (advances as f32 / 60.0).cos() * PI;

    // Assert
    assert!(
        (speed - expected).abs() < 1.0e-5,
        "motor speed must track sim time at 1× (got {speed}, expected {expected})"
    );
}

#[test]
fn max_wave_speed_raises_frequency_without_raising_tilt() {
    // Arrange
    let super::super::BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = build(&[]).expect("Wave Machine should construct");
    let advances = 7_u32;
    hooks
        .apply_control(&mut world, particle_system, "wave-speed", "10.0")
        .expect("10.0 is the slider maximum");

    // Act
    for _ in 0..advances {
        hooks
            .on_advance(&mut world, particle_system)
            .expect("on_advance must scale rocking frequency");
    }
    let speed = revolute_motor_speed(&world);
    let time = advances as f32 / 60.0;
    let expected = 10.0 * 0.05 * (10.0 * time).cos() * PI;

    // Assert
    assert!(
        (speed - expected).abs() < 1.0e-4,
        "10× must speed the same tilt (got {speed}, expected {expected})"
    );
}

#[test]
fn commanded_tilt_stays_near_the_pinned_angle() {
    // Arrange — integrate the speed command the way a tracking motor would.
    let original_peak = 0.05 * PI;
    let steps = 400_u32;

    // Act
    let at_one = peak_commanded_angle(1.0, steps);
    let at_ten = peak_commanded_angle(10.0, steps);

    // Assert
    assert!(
        (at_one - original_peak).abs() < 0.01,
        "1× peak tilt must stay near 0.05π (got {at_one})"
    );
    assert!(
        (at_ten - original_peak).abs() < 0.02,
        "10× must keep that tilt instead of scaling it (got {at_ten})"
    );
}

#[test]
fn simulated_tank_keeps_the_pinned_tilt_when_sped_up() {
    // Arrange — 1× needs about a quarter period to reach the crest. 10× crests sooner.
    let pinned = 0.05 * PI;
    let at_one = peak_simulated_tilt("1.0", 100);
    let at_ten = peak_simulated_tilt("10.0", 24);

    // Assert — scaling speed alone used to swing about ten times as far.
    assert!(
        (at_one - pinned).abs() < 0.05,
        "1× tank tilt must stay near 0.05π (peak {at_one}, pinned {pinned})"
    );
    assert!(
        (at_ten - pinned).abs() < 0.08,
        "10× must keep that tilt (peak {at_ten}, pinned {pinned})"
    );
}

fn peak_simulated_tilt(speed: &str, steps: u32) -> f32 {
    use liquidfun::{NoDecisionHook, StepConfiguration, StepLimits};

    let super::super::BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = build(&[]).expect("Wave Machine should construct");
    hooks
        .apply_control(&mut world, particle_system, "wave-speed", speed)
        .expect("wave speed token should be accepted");
    let step = StepConfiguration::new(1.0 / 60.0, 8, 3)
        .expect("valid step")
        .with_particle_iterations(2)
        .expect("two particle iterations");
    let limits = StepLimits::default();
    let mut hook = NoDecisionHook;
    let mut peak = 0.0_f32;
    for _ in 0..steps {
        hooks
            .on_advance(&mut world, particle_system)
            .expect("on_advance must update the motor");
        world
            .step(step, &mut hook, limits)
            .expect("Wave Machine step should succeed");
        peak = peak.max(tank_angle(&world).abs());
    }
    peak
}

#[test]
fn first_advance_changes_original_motor_speed() {
    // Arrange
    let super::super::BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = build(&[]).expect("Wave Machine should construct");
    hooks
        .apply_control(&mut world, particle_system, "wave-speed", "1.0")
        .expect("1.0 restores the pinned amplitude");
    let initial = revolute_motor_speed(&world);

    // Act
    hooks
        .on_advance(&mut world, particle_system)
        .expect("first advance must update motor from sim time");
    let after = revolute_motor_speed(&world);
    let expected = 0.05 * (1.0_f32 / 60.0).cos() * PI;

    // Assert — not stalled at the create-time speed; matches increment-then-set
    assert!(
        (initial - 0.05 * PI).abs() < 1.0e-5,
        "1× before any advance must be 0.05 * PI (got {initial})"
    );
    assert!(
        (after - expected).abs() < 1.0e-5,
        "after one advance speed must be 0.05 * cos(1/60) * PI (got {after})"
    );
    assert!(
        (after - initial).abs() > 1.0e-6,
        "motor speed must change from initial after the first advance"
    );
}

#[test]
fn wave_speed_control_is_live() {
    // Arrange
    let mut session = SessionCore::create(SceneId::WaveMachine)
        .expect("Wave Machine should construct the pinned motorized tank");
    let before = session.particle_count();

    // Act
    let applied = session.apply_control("wave-speed", "2.5");

    // Assert
    assert_eq!(applied, Ok(false));
    assert_eq!(session.particle_count(), before);
}

#[test]
fn wave_speed_rejects_off_scale_tokens() {
    // Arrange
    let mut session = SessionCore::create(SceneId::WaveMachine)
        .expect("Wave Machine should construct the pinned motorized tank");

    // Act
    let rejected = ["1", "10", "10.1", "-1.0", "01.0", "1.00", "fast"]
        .map(|value| session.apply_control("wave-speed", value));

    // Assert
    assert!(
        rejected
            .iter()
            .all(|result| *result == Err(SessionError::UnknownControl))
    );
}

#[test]
fn unknown_control_is_rejected_and_pointer_is_noop() {
    // Arrange
    let mut session = SessionCore::create(SceneId::WaveMachine)
        .expect("Wave Machine should construct the pinned motorized tank");
    let before = session.particle_count();

    // Act
    let control = session.apply_control("stiffness", "high");
    let action = session.apply_action("refill");
    let pointer = session.apply_pointer("up", 0.0, 1.0);

    // Assert
    assert_eq!(control, Err(SessionError::UnknownControl));
    assert_eq!(action, Err(SessionError::UnknownControl));
    assert_eq!(pointer, Ok(()));
    assert_eq!(session.particle_count(), before);
}

#[test]
fn motor_on_pattern_is_not_water_wheel_motor_off() {
    // Arrange
    let source = include_str!("../wave_machine.rs");
    let impl_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation precedes tests");

    // Assert — production path must use live motor writes (Task 2)
    assert!(
        impl_source.contains("set_revolute_motor_speed"),
        "on_advance must call set_revolute_motor_speed"
    );
    assert!(
        impl_source.contains("0.05"),
        "motor formula constant 0.05 must be present"
    );
    assert!(
        !impl_source.contains("enable_motor: false") && !impl_source.contains("with_motor(false"),
        "Wave Machine must not copy Water Wheel motor-off"
    );
}

fn peak_commanded_angle(multiplier: f32, steps: u32) -> f32 {
    let mut time = 0.0_f32;
    let mut angle = 0.0_f32;
    let mut peak = 0.0_f32;
    let dt = 1.0 / 60.0;
    for _ in 0..steps {
        time += dt;
        angle += super::scaled_motor_speed(time, multiplier) * dt;
        peak = peak.max(angle.abs());
    }
    peak
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
    assert!(
        revolute[0].is_motor_enabled(),
        "Wave Machine revolute must enable_motor (not Water Wheel motor-off)"
    );
    revolute[0].motor_speed()
}
