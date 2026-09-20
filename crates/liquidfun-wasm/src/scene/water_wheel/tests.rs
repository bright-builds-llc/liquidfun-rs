use std::f32::consts::TAU;

use liquidfun::math::Vec2;
use liquidfun::{BodyType, JointDef, WorldObservationLimits};

use crate::ProofFrame;
use crate::scene::SceneId;
use crate::session::SessionCore;

#[test]
fn create_water_wheel_builds_hub_circle_and_paddle_segments() {
    // Arrange / Act
    let session = SessionCore::create(SceneId::WaterWheel)
        .expect("Water Wheel should construct a native pinned wheel");
    let frame = capture(&session);

    // Assert
    assert!(
        frame.rigid_circles().len() >= 3,
        "hub circle reports at least x, y, radius"
    );
    assert!(
        frame.rigid_segments().len() >= 16,
        "at least four paddle segments report start/end floats"
    );
    assert!(session.rigid_shape_count() >= 5);
}

#[test]
fn revolute_joint_is_created_without_a_motor() {
    // Arrange / Act
    let super::BuiltScene { world, .. } = super::build(&[]).expect("Water Wheel should construct");
    let observation = world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("reviewed observation should include the revolute hub");
    let revolute: Vec<_> = observation
        .joints()
        .iter()
        .filter_map(|joint| match joint.snapshot().definition() {
            JointDef::Revolute(definition) => Some(definition),
            _ => None,
        })
        .collect();

    // Assert
    assert_eq!(revolute.len(), 1, "exactly one revolute pin should exist");
    assert!(
        !revolute[0].is_motor_enabled(),
        "D-09 forbids a motor-driven water-wheel hub"
    );
    assert_eq!(revolute[0].motor_speed().to_bits(), 0.0_f32.to_bits());
    assert_eq!(revolute[0].max_motor_torque().to_bits(), 0.0_f32.to_bits());
}

#[test]
fn collect_segments_follow_a_forced_wheel_transform() {
    // Arrange
    let super::BuiltScene {
        mut world, hooks, ..
    } = super::build(&[]).expect("Water Wheel should construct");
    let before = flatten_segments(
        &hooks
            .collect_segments(&world)
            .expect("paddles should capture from the live pose"),
    );
    let wheel = world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("reviewed observation should include the wheel body")
        .bodies()
        .iter()
        .find(|body| body.snapshot().body_type() == BodyType::Dynamic)
        .expect("the paddle wheel is the dynamic body")
        .id();

    // Act
    world
        .set_body_transform(wheel, Vec2::new(0.0, 3.0), TAU / 8.0)
        .expect("test-only pose change proves capture uses the live transform");
    let after = flatten_segments(
        &hooks
            .collect_segments(&world)
            .expect("rotated paddles should still capture"),
    );

    // Assert
    assert!(after.len() >= 16);
    assert_ne!(
        before, after,
        "paddle segments must come from BodySnapshot::transform().apply, not constants"
    );
}

#[test]
fn medium_jet_with_emission_on_turns_the_wheel() {
    // Arrange
    let mut session = SessionCore::create(SceneId::WaterWheel)
        .expect("Water Wheel should construct a native pinned wheel");
    let recreated_strength = session
        .apply_control("jet-strength", "medium")
        .expect("medium jet should apply live");
    let recreated_emission = session
        .apply_control("emission", "on")
        .expect("emission on should apply live");
    let before = captured_paddle_angle(&session);

    // Act
    advance_steps(&mut session, 180);
    let after = captured_paddle_angle(&session);

    // Assert
    assert!(!recreated_strength);
    assert!(!recreated_emission);
    assert!(
        (after - before).abs() > 0.04,
        "native coupling should rotate the wheel; before={before} after={after}"
    );
}

#[test]
fn emission_off_leaves_the_wheel_angle_nearly_unchanged() {
    // Arrange
    let mut session = SessionCore::create(SceneId::WaterWheel)
        .expect("Water Wheel should construct a native pinned wheel");
    session
        .apply_control("emission", "off")
        .expect("emission off should apply live");
    let before = captured_paddle_angle(&session);

    // Act
    advance_steps(&mut session, 180);
    let after = captured_paddle_angle(&session);

    // Assert
    assert!(
        (after - before).abs() < 0.01,
        "motor-off wheel must not spin without the jet; before={before} after={after}"
    );
}

#[test]
fn two_hundred_forty_on_steps_plateau_at_or_below_the_particle_cap() {
    // Arrange
    let mut session = SessionCore::create(SceneId::WaterWheel)
        .expect("Water Wheel should construct a native pinned wheel");
    session
        .apply_control("jet-strength", "medium")
        .expect("medium jet should apply live");
    session
        .apply_control("emission", "on")
        .expect("emission on should apply live");

    // Act
    advance_steps(&mut session, 210);
    let mid_count = capture(&session).particle_count();
    advance_steps(&mut session, 30);
    let end_count = capture(&session).particle_count();

    // Assert
    assert!(end_count <= 3200);
    assert!(
        end_count <= mid_count,
        "count should not keep climbing over the last 30 steps: {mid_count} -> {end_count}"
    );
}

#[test]
fn pointer_aim_emits_positive_x_and_cancel_restores_default() {
    // Arrange
    let super::BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = super::build(&[]).expect("Water Wheel should construct");

    // Act
    hooks
        .apply_pointer(
            &mut world,
            particle_system,
            crate::scene::PointerKind::Down,
            0.0,
            3.15,
        )
        .expect("down should aim the jet toward the pointer");
    hooks
        .on_advance(&mut world, particle_system)
        .expect("aimed emit should succeed");
    let aimed_horizontal = {
        let view = world
            .particle_system_view(particle_system)
            .expect("aimed system should stay live");
        view.velocities()
            .iter()
            .any(|velocity| velocity.x > 1.0 && velocity.y.abs() < 0.15)
    };

    hooks
        .apply_pointer(
            &mut world,
            particle_system,
            crate::scene::PointerKind::Cancel,
            0.0,
            0.0,
        )
        .expect("cancel should clear the aim override");
    hooks
        .on_advance(&mut world, particle_system)
        .expect("default emit should succeed");
    let restored_default = {
        let view = world
            .particle_system_view(particle_system)
            .expect("restored system should stay live");
        view.velocities()
            .iter()
            .any(|velocity| velocity.x > 1.0 && (velocity.y + 0.4).abs() < 0.05)
    };

    // Assert
    assert!(
        aimed_horizontal,
        "pointer toward x=0 at nozzle height should emit a positive x velocity"
    );
    assert!(
        restored_default,
        "cancel must restore Vec2::new(speed, -0.4)"
    );
}

#[test]
fn jet_strength_strong_sets_speed_without_a_motor_write() {
    // Arrange
    let super::BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = super::build(&[]).expect("Water Wheel should construct");
    let source = include_str!("../water_wheel.rs");
    let impl_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation precedes tests");

    // Act
    let strong = hooks
        .apply_control(&mut world, particle_system, "jet-strength", "strong")
        .expect("jet-strength=strong should apply live");
    hooks
        .on_advance(&mut world, particle_system)
        .expect("strong emit should succeed");
    let strong_speed = {
        let view = world
            .particle_system_view(particle_system)
            .expect("strong system should stay live");
        view.velocities()
            .iter()
            .any(|velocity| (velocity.x - 12.0).abs() < 0.05 && (velocity.y + 0.4).abs() < 0.05)
    };

    // Assert
    assert!(matches!(strong, super::ControlEffect::Live));
    assert!(
        strong_speed,
        "jet-strength=strong must still emit at speed 12.0"
    );
    assert!(
        impl_source.contains("maybe_aim_velocity"),
        "Water Wheel must store maybe_aim_velocity for pointer aim"
    );
    assert!(
        impl_source.contains("Vec2::new(speed, -0.4)")
            || impl_source.contains("Vec2::new(self.jet_speed, -0.4)"),
        "cancel default must remain Vec2::new(speed, -0.4)"
    );
    assert!(
        !impl_source.contains("enable_motor") && !impl_source.contains(".motor_speed"),
        "pointer aim must not write a revolute motor"
    );
}

#[test]
fn jet_strength_and_emission_apply_live() {
    // Arrange
    let super::BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = super::build(&[]).expect("Water Wheel should construct");

    // Act
    let weak = hooks
        .apply_control(&mut world, particle_system, "jet-strength", "weak")
        .expect("weak jet should apply");
    let medium = hooks
        .apply_control(&mut world, particle_system, "jet-strength", "medium")
        .expect("medium jet should apply");
    let strong = hooks
        .apply_control(&mut world, particle_system, "jet-strength", "strong")
        .expect("strong jet should apply");
    let off = hooks
        .apply_control(&mut world, particle_system, "emission", "off")
        .expect("emission off should apply");
    let on = hooks
        .apply_control(&mut world, particle_system, "emission", "on")
        .expect("emission on should apply");

    // Assert
    assert!(matches!(weak, super::ControlEffect::Live));
    assert!(matches!(medium, super::ControlEffect::Live));
    assert!(matches!(strong, super::ControlEffect::Live));
    assert!(matches!(off, super::ControlEffect::Live));
    assert!(matches!(on, super::ControlEffect::Live));
}

#[test]
fn unknown_jet_and_emission_tokens_fail_closed() {
    // Arrange
    let mut session = SessionCore::create(SceneId::WaterWheel)
        .expect("Water Wheel should construct a native pinned wheel");

    // Act
    let bad_strength = session.apply_control("jet-strength", "max");
    let bad_emission = session.apply_control("emission", "maybe");
    let unknown_name = session.apply_control("aim-angle", "0");

    // Assert
    assert_eq!(
        bad_strength,
        Err(crate::session::SessionError::UnknownControl)
    );
    assert_eq!(
        bad_emission,
        Err(crate::session::SessionError::UnknownControl)
    );
    assert_eq!(
        unknown_name,
        Err(crate::session::SessionError::UnknownControl)
    );
}

fn captured_paddle_angle(session: &SessionCore) -> f32 {
    let segments = capture(session).rigid_segments();
    assert!(
        segments.len() >= 16,
        "captured frame must include at least four paddle segments"
    );
    let first_paddle = segments.len().saturating_sub(16);
    let start_x = segments[first_paddle];
    let start_y = segments[first_paddle + 1];
    let end_x = segments[first_paddle + 2];
    let end_y = segments[first_paddle + 3];
    (end_y - start_y).atan2(end_x - start_x)
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

fn flatten_segments(segments: &[crate::scene::RigidSegment]) -> Vec<f32> {
    let mut values = Vec::with_capacity(segments.len().saturating_mul(4));
    for segment in segments {
        values.extend_from_slice(&[
            segment.start.x,
            segment.start.y,
            segment.end.x,
            segment.end.y,
        ]);
    }
    values
}

fn capture(session: &SessionCore) -> ProofFrame {
    ProofFrame::from(
        session
            .capture_frame()
            .expect("Water Wheel should capture a frame"),
    )
}
