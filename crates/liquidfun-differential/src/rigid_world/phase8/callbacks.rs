//! Phase 8 collision callbacks and directive state.

use liquidfun::{
    CollisionDecisionHook, CollisionDirective, FixtureId, FixturePairView, FixtureParticleView,
    ParticlePairContactView, PreSolveDirective, PreSolveView, StepConfiguration, StepError,
    StepLimits, StepReport,
};
use liquidfun_test_protocol::{
    RigidContactDirectiveTarget, RigidPreSolveDirective, RigidWorldActionRecord,
    RigidWorldWitnessFamily,
};

use super::super::{NativeRigidWorldError, TimelineExecutor, action_error};

pub(super) fn step(
    executor: &mut TimelineExecutor,
    configuration: StepConfiguration,
    limits: StepLimits,
) -> Result<StepReport, StepError> {
    let mut hook = Phase8Hook {
        filter_directives: executor.filter_directives.clone(),
        pre_solve_directives: executor.pre_solve_directives.clone(),
        allow_unconfigured_contacts: captures_contact_behavior(executor.family),
    };
    executor.world.step(configuration, &mut hook, limits)
}

fn captures_contact_behavior(family: RigidWorldWitnessFamily) -> bool {
    // Solver-only timelines use fixtures solely to give moving bodies mass. Rejecting their
    // undeclared cross-family pairs keeps joint evidence independent of incidental contacts;
    // the C++ adapter must mirror this typed-family rule.
    matches!(
        family,
        RigidWorldWitnessFamily::MixedJointIslandOrderAndCollisionSuppression
            | RigidWorldWitnessFamily::ContactFilterListenerAndPreSolveTiming
            | RigidWorldWitnessFamily::DestructionListenerAndDependencyCascades
    ) || !RigidWorldWitnessFamily::PHASE8_REQUIRED.contains(&family)
}

pub(super) fn refresh_filter_pair(
    executor: &mut TimelineExecutor,
    fixtures: [FixtureId; 2],
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    for fixture in fixtures {
        let filter = executor
            .world
            .fixture_snapshot(fixture)
            .map_err(|error| action_error(action, error))?
            .filter_data();
        executor
            .world
            .set_fixture_filter(fixture, filter)
            .map_err(|error| action_error(action, error))?;
    }
    Ok(())
}

struct Phase8Hook {
    filter_directives: Vec<(FixtureId, FixtureId, bool)>,
    pre_solve_directives: Vec<(FixtureId, FixtureId, PreSolveDirective)>,
    allow_unconfigured_contacts: bool,
}

impl CollisionDecisionHook for Phase8Hook {
    fn should_collide(&mut self, pair: FixturePairView<'_>) -> CollisionDirective {
        pair_value(&self.filter_directives, pair.fixtures()).map_or_else(
            || {
                if self.allow_unconfigured_contacts {
                    CollisionDirective::Collide
                } else {
                    CollisionDirective::Ignore
                }
            },
            |should_collide| {
                if *should_collide {
                    CollisionDirective::Collide
                } else {
                    CollisionDirective::Ignore
                }
            },
        )
    }

    fn pre_solve(&mut self, contact: PreSolveView<'_>) -> PreSolveDirective {
        pair_value(&self.pre_solve_directives, contact.fixtures())
            .copied()
            .unwrap_or(PreSolveDirective::Enable)
    }

    fn should_collide_fixture_particle(
        &mut self,
        _contact: FixtureParticleView<'_>,
    ) -> CollisionDirective {
        CollisionDirective::Ignore
    }

    fn should_collide_particle_pair(
        &mut self,
        _contact: ParticlePairContactView<'_>,
    ) -> CollisionDirective {
        CollisionDirective::Ignore
    }
}

pub(super) fn directive_fixtures(
    executor: &TimelineExecutor,
    target: &RigidContactDirectiveTarget,
    action: &RigidWorldActionRecord,
) -> Result<[FixtureId; 2], NativeRigidWorldError> {
    Ok([
        executor.fixture(&target.fixture_a_id, action)?,
        executor.fixture(&target.fixture_b_id, action)?,
    ])
}

pub(super) fn pre_solve_directive(
    directive: RigidPreSolveDirective,
    action: &RigidWorldActionRecord,
) -> Result<PreSolveDirective, NativeRigidWorldError> {
    let mut value = if directive.enabled {
        PreSolveDirective::Enable
    } else {
        PreSolveDirective::Disable
    };
    if let Some(bits) = directive.maybe_friction_bits {
        value = value
            .with_friction(bits.to_f32())
            .map_err(|error| action_error(action, error))?;
    }
    if let Some(bits) = directive.maybe_restitution_bits {
        value = value
            .with_restitution(bits.to_f32())
            .map_err(|error| action_error(action, error))?;
    }
    if let Some(bits) = directive.maybe_tangent_speed_bits {
        value = value
            .with_tangent_speed(bits.to_f32())
            .map_err(|error| action_error(action, error))?;
    }
    Ok(value)
}

pub(super) fn upsert_pair<T: Copy>(
    entries: &mut Vec<(FixtureId, FixtureId, T)>,
    a: FixtureId,
    b: FixtureId,
    value: T,
) {
    if let Some(entry) = entries
        .iter_mut()
        .find(|(x, y, _)| same_pair([*x, *y], [a, b]))
    {
        entry.2 = value;
    } else {
        entries.push((a, b, value));
    }
}

fn pair_value<T>(entries: &[(FixtureId, FixtureId, T)], pair: [FixtureId; 2]) -> Option<&T> {
    entries
        .iter()
        .find_map(|(a, b, value)| same_pair([*a, *b], pair).then_some(value))
}

fn same_pair(first: [FixtureId; 2], second: [FixtureId; 2]) -> bool {
    first == second || first == [second[1], second[0]]
}
