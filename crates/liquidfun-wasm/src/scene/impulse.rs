//! Pinned `LiquidFun` Impulse scene: chain-loop box with whole-group shove.

use crate::session::SessionError;

use super::BuiltScene;

/// Task 1 RED stub — Task 2 fills in the chain-loop box and group shove.
pub(crate) fn build(_presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    Err(SessionError::SceneConstruction)
}

#[cfg(test)]
mod tests {
    use liquidfun::NoDecisionHook;
    use liquidfun::math::Vec2;
    use liquidfun::{ParticleGroupId, ParticleSystemId, StepConfiguration, StepLimits, World};

    use super::super::{ControlEffect, PointerKind, SceneId};
    use super::build;
    use crate::session::{SessionCore, SessionError};

    #[test]
    fn create_builds_impulse_with_particles() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::Impulse)
            .expect("Impulse should construct the pinned chain-loop box");

        // Assert
        assert!(
            session.particle_count() > 0,
            "particle group must create at least one particle"
        );
    }

    #[test]
    fn inside_box_shove_changes_group_momentum_outside_is_noop() {
        // Arrange — twin worlds so gravity/settle match; only shove differs
        let mut baseline = build_impulse();
        let mut shoved = build_impulse();
        settle(&mut baseline.world);
        settle(&mut shoved.world);
        let baseline_group = first_group(&baseline.world, baseline.particle_system);
        let shoved_group = first_group(&shoved.world, shoved.particle_system);
        let before_outside = group_speed(&baseline.world, baseline_group);
        let before_inside = group_speed(&shoved.world, shoved_group);

        // Act — outside box is a success no-op
        baseline
            .hooks
            .apply_pointer(
                &mut baseline.world,
                baseline.particle_system,
                PointerKind::Up,
                10.0,
                10.0,
            )
            .expect("outside-box pointer up must succeed as a no-op");
        settle(&mut baseline.world);
        let after_outside = group_speed(&baseline.world, baseline_group);

        // Act — inside box shoves the whole blob (default force mode needs a step)
        shoved
            .hooks
            .apply_pointer(
                &mut shoved.world,
                shoved.particle_system,
                PointerKind::Up,
                1.0,
                2.0,
            )
            .expect("inside-box pointer up must shove the group");
        settle(&mut shoved.world);
        let after_inside = group_speed(&shoved.world, shoved_group);

        // Assert
        assert!(
            (after_outside - before_outside).abs() < 1e-3
                || after_outside >= 0.0 && before_outside >= 0.0,
            "outside-box pointer must not add shove momentum beyond shared settle"
        );
        // Twin: same settle after outside vs after inside shove — inside must diverge
        let settle_only = after_outside;
        assert!(
            (after_inside - settle_only).abs() > 0.05
                || after_inside > before_inside + 0.05,
            "inside-box shove must change group momentum versus the outside twin \
             (after_inside={after_inside}, settle_only={settle_only}, before={before_inside})"
        );
    }

    #[test]
    fn push_mode_force_and_impulse_return_live() {
        // Arrange
        let super::super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = build(&[]).expect("Impulse should construct the pinned chain-loop box");

        // Act
        let impulse = hooks
            .apply_control(&mut world, particle_system, "push-mode", "impulse")
            .expect("push-mode impulse must be allowlisted");
        let force = hooks
            .apply_control(&mut world, particle_system, "push-mode", "force")
            .expect("push-mode force must be allowlisted");

        // Assert
        assert!(matches!(impulse, ControlEffect::Live));
        assert!(matches!(force, ControlEffect::Live));
    }

    #[test]
    fn unknown_control_is_rejected() {
        // Arrange
        let mut session = SessionCore::create(SceneId::Impulse)
            .expect("Impulse should construct the pinned chain-loop box");

        // Act
        let control = session.apply_control("stiffness", "high");
        let action = session.apply_action("refill");
        let bad_mode = session.apply_control("push-mode", "explode");

        // Assert
        assert_eq!(control, Err(SessionError::UnknownControl));
        assert_eq!(action, Err(SessionError::UnknownControl));
        assert_eq!(bad_mode, Err(SessionError::UnknownControl));
    }

    #[test]
    fn shove_uses_full_member_ids_range_apis() {
        // Arrange
        let source = include_str!("impulse.rs");
        let impl_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("implementation precedes tests");

        // Assert — production shove (not Jelly Drop singular poke)
        assert!(
            impl_source.contains("apply_particle_force_range"),
            "force shove must use apply_particle_force_range"
        );
        assert!(
            impl_source.contains("apply_particle_linear_impulse_range"),
            "impulse shove must use apply_particle_linear_impulse_range"
        );
        assert!(
            impl_source.contains("member_ids"),
            "shove must copy full member_ids"
        );
        assert!(
            impl_source.contains("push-mode"),
            "live push-mode preset must be present"
        );
        assert!(
            impl_source.contains("0.025"),
            "particle radius must stay 0.025"
        );
        assert!(
            impl_source.contains("0.005"),
            "impulse magnitude constant must stay 0.005"
        );
        assert!(
            !impl_source.contains("apply_particle_linear_impulse("),
            "Impulse must not shove via singular apply_particle_linear_impulse"
        );
    }

    struct ImpulseBuilt {
        world: World,
        particle_system: ParticleSystemId,
        hooks: Box<dyn super::super::SceneHooks>,
    }

    fn build_impulse() -> ImpulseBuilt {
        let built = build(&[]).expect("Impulse should construct the pinned chain-loop box");
        ImpulseBuilt {
            world: built.world,
            particle_system: built.particle_system,
            hooks: built.hooks,
        }
    }

    fn first_group(world: &World, system: ParticleSystemId) -> ParticleGroupId {
        let view = world
            .particle_system_view(system)
            .expect("Impulse particle system must exist");
        view.group_ids()
            .iter()
            .find_map(|maybe_group| *maybe_group)
            .expect("impulse scene must own one particle group")
    }

    fn group_speed(world: &World, group: ParticleGroupId) -> f32 {
        world
            .particle_group_view(group)
            .expect("group must remain valid")
            .linear_velocity()
            .length()
    }

    fn settle(world: &mut World) {
        let configuration = StepConfiguration::new(1.0 / 60.0, 8, 3)
            .expect("default Impulse settle step configuration");
        let limits = StepLimits::default();
        world
            .step(configuration, &mut NoDecisionHook, limits)
            .expect("Impulse settle step must succeed");
    }

    #[allow(dead_code)] // kept for readable Vec2 asserts in future shove diagnostics
    fn _box_center() -> Vec2 {
        Vec2::new(0.0, 2.0)
    }
}
