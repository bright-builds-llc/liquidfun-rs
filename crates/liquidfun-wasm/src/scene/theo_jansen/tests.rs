use liquidfun::{JointDef, World, WorldObservationLimits};

use super::super::{ControlEffect, SceneId};
use super::build;
use crate::session::{SessionCore, SessionError};

#[test]
fn create_builds_theo_jansen_with_particles() {
    // Arrange / Act
    let session = SessionCore::create(SceneId::TheoJansen)
        .expect("Theo Jansen should construct the pinned walker under particle load");

    // Assert
    assert!(
        session.particle_count() > 0,
        "particle slab must create at least one particle"
    );
}

#[test]
fn soft_distance_joints_and_motorized_revolute_exist() {
    // Arrange / Act
    let BuiltScene { world, .. } =
        build(&[]).expect("Theo Jansen should construct with soft legs");
    let (distance_count, soft_count, motorized) = joint_inventory(&world);

    // Assert — CreateLeg soft suspension, not welded polygons
    assert!(
        distance_count >= 1,
        "walker must expose at least one distance joint (got {distance_count})"
    );
    assert!(
        soft_count >= 1,
        "at least one soft distance joint (freq 10, damp 0.5) must exist (got {soft_count})"
    );
    assert!(
        motorized,
        "chassis↔wheel revolute must enable_motor (not Water Wheel motor-off)"
    );
}

#[test]
fn motor_direction_reverse_and_forward_are_live() {
    // Arrange
    let BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = build(&[]).expect("Theo Jansen should construct the pinned walker");
    let initial = motorized_revolute_speed(&world);
    assert!(
        initial > 0.0,
        "fresh create defaults to forward (+2.0); got {initial}"
    );

    // Act
    let reverse = hooks
        .apply_control(&mut world, particle_system, "motor-direction", "reverse")
        .expect("motor-direction reverse must be allowlisted");
    let reverse_speed = motorized_revolute_speed(&world);
    let forward = hooks
        .apply_control(&mut world, particle_system, "motor-direction", "forward")
        .expect("motor-direction forward must be allowlisted");
    let forward_speed = motorized_revolute_speed(&world);

    // Assert — Live sign flip via set_revolute_motor_speed; world not recreated
    assert!(matches!(reverse, ControlEffect::Live));
    assert!(matches!(forward, ControlEffect::Live));
    assert!(
        reverse_speed < 0.0,
        "reverse must set motor speed negative (got {reverse_speed})"
    );
    assert!(
        forward_speed > 0.0,
        "forward must set motor speed positive (got {forward_speed})"
    );
    assert!(
        (reverse_speed + 2.0).abs() < 1.0e-5,
        "reverse magnitude must stay 2.0 (got {reverse_speed})"
    );
    assert!(
        (forward_speed - 2.0).abs() < 1.0e-5,
        "forward magnitude must stay 2.0 (got {forward_speed})"
    );
}

#[test]
fn unknown_control_is_rejected_and_pointer_is_noop() {
    // Arrange
    let mut session = SessionCore::create(SceneId::TheoJansen)
        .expect("Theo Jansen should construct the pinned walker");
    let before = session.particle_count();

    // Act
    let control = session.apply_control("stiffness", "high");
    let bad_direction = session.apply_control("motor-direction", "left");
    let action = session.apply_action("refill");
    let pointer = session.apply_pointer("up", 0.0, 8.0);

    // Assert
    assert_eq!(control, Err(SessionError::UnknownControl));
    assert_eq!(bad_direction, Err(SessionError::UnknownControl));
    assert_eq!(action, Err(SessionError::UnknownControl));
    assert_eq!(pointer, Ok(()));
    assert_eq!(session.particle_count(), before);
}

#[test]
fn soft_legs_and_live_motor_are_not_water_wheel_motor_off() {
    // Arrange
    let source = include_str!("../theo_jansen.rs");
    let impl_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation precedes tests");

    // Assert — production path must use soft distance + live motor reverse
    assert!(
        impl_source.contains("with_frequency(10") || impl_source.contains("with_frequency(10.0"),
        "CreateLeg soft distance must set frequency 10"
    );
    assert!(
        impl_source.contains("with_damping_ratio(0.5"),
        "CreateLeg soft distance must set damping ratio 0.5"
    );
    assert!(
        impl_source.contains("motor-direction"),
        "live motor-direction preset must be present"
    );
    assert!(
        impl_source.contains("set_revolute_motor_speed"),
        "motor-direction must call set_revolute_motor_speed"
    );
    assert!(
        !impl_source.contains("enable_motor: false")
            && !impl_source.contains("with_motor(false"),
        "Theo Jansen must not copy Water Wheel motor-off"
    );
}

use super::super::BuiltScene;

fn joint_inventory(world: &World) -> (usize, usize, bool) {
    let observation = world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("reviewed observation should include walker joints");
    let mut distance_count = 0_usize;
    let mut soft_count = 0_usize;
    let mut motorized = false;
    for joint in observation.joints() {
        match joint.snapshot().definition() {
            JointDef::Distance(definition) => {
                distance_count += 1;
                if (definition.frequency() - 10.0).abs() < 1.0e-5
                    && (definition.damping_ratio() - 0.5).abs() < 1.0e-5
                {
                    soft_count += 1;
                }
            }
            JointDef::Revolute(definition) if definition.is_motor_enabled() => {
                motorized = true;
            }
            _ => {}
        }
    }
    (distance_count, soft_count, motorized)
}

fn motorized_revolute_speed(world: &World) -> f32 {
    let observation = world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("reviewed observation should include the motorized revolute");
    let motorized: Vec<_> = observation
        .joints()
        .iter()
        .filter_map(|joint| match joint.snapshot().definition() {
            JointDef::Revolute(definition) if definition.is_motor_enabled() => Some(definition),
            _ => None,
        })
        .collect();
    assert_eq!(
        motorized.len(),
        1,
        "exactly one motorized revolute (chassis↔wheel)"
    );
    motorized[0].motor_speed()
}
