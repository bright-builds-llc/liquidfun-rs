use super::config::StepConfiguration;
use super::contact_manager::ContactManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // The public step lifecycle consumes this key in Plan 07-09.
pub(super) struct ContinuousStepKey {
    time_step_bits: u32,
    velocity_iterations: u32,
    position_iterations: u32,
}

#[allow(dead_code)] // The public step lifecycle consumes this key in Plan 07-09.
impl ContinuousStepKey {
    pub(super) fn from_configuration(configuration: StepConfiguration) -> Self {
        Self {
            time_step_bits: configuration.time_step().to_bits(),
            velocity_iterations: configuration.velocity_iterations(),
            position_iterations: configuration.position_iterations(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // The public step lifecycle consumes this result in Plan 07-09.
pub(super) enum ContinuousStepKind {
    Fresh,
    Resumed,
}

#[derive(Debug, Default)]
pub(super) struct ContinuousStepState {
    maybe_pending: Option<ContinuousStepKey>,
}

impl ContinuousStepState {
    pub(super) const fn new() -> Self {
        Self {
            maybe_pending: None,
        }
    }

    #[allow(dead_code)] // The public step lifecycle calls this in Plan 07-09.
    pub(super) fn begin_step(
        &mut self,
        key: ContinuousStepKey,
        contact_manager: &mut ContactManager,
    ) -> ContinuousStepKind {
        if self.maybe_pending.take() == Some(key) {
            return ContinuousStepKind::Resumed;
        }
        contact_manager.reset_toi_state();
        ContinuousStepKind::Fresh
    }

    #[allow(dead_code)] // The public step lifecycle calls this in Plan 07-09.
    pub(super) fn mark_pending(&mut self, key: ContinuousStepKey) {
        self.maybe_pending = Some(key);
    }

    pub(super) fn invalidate(&mut self) {
        self.maybe_pending = None;
    }
}

#[cfg(test)]
mod tests {
    use super::{ContinuousStepKey, ContinuousStepKind, ContinuousStepState};
    use crate::collision::{CircleShape, FilterData, Shape};
    use crate::math::Vec2;
    use crate::math::settings::MAX_SUB_STEPS;
    use crate::{BodyDef, BodyType, FixtureDef, StepConfiguration, World};

    fn world_with_contact() -> (World, crate::BodyId) {
        let mut world = World::new().expect("test world key should remain available");
        let static_body = world
            .create_body(
                &BodyDef::new(BodyType::Static, Vec2::ZERO, 0.0, true)
                    .expect("test static body definition should be valid"),
            )
            .expect("test static body should fit");
        let dynamic_body = world
            .create_body(
                &BodyDef::new(BodyType::Dynamic, Vec2::new(0.5, 0.0), 0.0, true)
                    .expect("test dynamic body definition should be valid"),
            )
            .expect("test dynamic body should fit");
        let fixture = FixtureDef::new(
            Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test circle should be valid")),
            1.0,
            0.2,
            0.0,
            false,
            FilterData::default(),
        )
        .expect("test fixture definition should be valid");
        world
            .create_fixture(static_body, &fixture)
            .expect("test static fixture should fit");
        world
            .create_fixture(dynamic_body, &fixture)
            .expect("test dynamic fixture should fit");
        world.find_new_contacts();
        world.update_contacts();
        (world, dynamic_body)
    }

    fn step_key(time_step: f32) -> ContinuousStepKey {
        let configuration = StepConfiguration::new(time_step, 8, 3)
            .expect("test step configuration should be valid");
        ContinuousStepKey::from_configuration(configuration)
    }

    #[test]
    fn ccd_cache_is_invalidated_by_contact_and_sweep_changes() {
        // Arrange
        let (mut world, dynamic_body) = world_with_contact();
        let ordinal = world.contact_manager.contacts()[0].ordinal;
        world
            .contact_manager
            .seed_toi_state_for_test(ordinal, 0.25, MAX_SUB_STEPS + 1)
            .expect("bounded test TOI state should be accepted");

        // Act
        world.contact_manager.set_hook_enabled(ordinal, false);

        // Assert
        assert_eq!(
            world.contact_manager.toi_state_for_test(ordinal),
            Some((None, MAX_SUB_STEPS + 1))
        );
        assert!(
            world
                .contact_manager
                .increment_toi_count_for_test(ordinal)
                .is_err(),
            "the checked count must reject values above the strict upstream guard"
        );

        // Arrange
        world
            .contact_manager
            .seed_toi_state_for_test(ordinal, 0.5, 1)
            .expect("bounded test TOI state should be accepted");

        // Act
        world.contact_manager.invalidate_toi_for_body(dynamic_body);

        // Assert
        assert_eq!(
            world.contact_manager.toi_state_for_test(ordinal),
            Some((None, 1))
        );
    }

    #[test]
    fn pending_ccd_state_survives_only_the_matching_step() {
        // Arrange
        let (mut world, _dynamic_body) = world_with_contact();
        let ordinal = world.contact_manager.contacts()[0].ordinal;
        let mut state = ContinuousStepState::new();
        let matching = step_key(1.0 / 60.0);
        let different = step_key(1.0 / 30.0);
        world
            .contact_manager
            .seed_toi_state_for_test(ordinal, 0.25, 1)
            .expect("bounded test TOI state should be accepted");
        state.mark_pending(matching);

        // Act
        let matching_kind = state.begin_step(matching, &mut world.contact_manager);
        let retained_state = world.contact_manager.toi_state_for_test(ordinal);
        state.mark_pending(matching);
        let different_kind = state.begin_step(different, &mut world.contact_manager);
        let reset_state = world.contact_manager.toi_state_for_test(ordinal);
        let stale_kind = state.begin_step(matching, &mut world.contact_manager);

        // Assert
        assert_eq!(matching_kind, ContinuousStepKind::Resumed);
        assert_eq!(retained_state, Some((Some(0.25), 1)));
        assert_eq!(different_kind, ContinuousStepKind::Fresh);
        assert_eq!(reset_state, Some((None, 0)));
        assert_eq!(stale_kind, ContinuousStepKind::Fresh);
    }
}
