//! Pinned `LiquidFun` Wave Machine: motorized four-wall tank rocking from sim time.
//!
//! ANTI-PATTERN: Do not copy Water Wheel's motor-off revolute / empty motor writes.
//! Motor speed is written only from `on_advance` using `0.05 * cos(t) * π`.

use super::BuiltScene;
use crate::session::SessionError;

/// Stub for Sequential TDD RED — Task 2 implements the motorized tank.
pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let _ = presets;
    Err(SessionError::SceneConstruction)
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use liquidfun::{JointDef, World, WorldObservationLimits};

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
    fn motor_speed_follows_sim_time_after_advances() {
        // Arrange — JS Step order: t += 1/60 then set_revolute_motor_speed(0.05 * cos(t) * π)
        // Water Wheel motor-off is NOT the template.
        let super::super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = build(&[]).expect("Wave Machine should construct the pinned motorized tank");
        let advances = 7_u32;

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
            "motor speed must track sim time (got {speed}, expected {expected}); \
             set_revolute_motor_speed writes 0.05 * cos(t) * PI"
        );
    }

    #[test]
    fn first_advance_changes_motor_speed_from_initial() {
        // Arrange
        let super::super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = build(&[]).expect("Wave Machine should construct");
        let initial = revolute_motor_speed(&world);

        // Act
        hooks
            .on_advance(&mut world, particle_system)
            .expect("first advance must update motor from sim time");
        let after = revolute_motor_speed(&world);
        let expected = 0.05 * (1.0_f32 / 60.0).cos() * PI;

        // Assert — not stalled at create speed forever; matches increment-then-set
        assert!(
            (initial - 0.05 * PI).abs() < 1.0e-5,
            "initial motor speed must be 0.05 * PI before any advance (got {initial})"
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
        let source = include_str!("wave_machine.rs");
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
            !impl_source.contains("enable_motor: false")
                && !impl_source.contains("with_motor(false"),
            "Wave Machine must not copy Water Wheel motor-off"
        );
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
}
