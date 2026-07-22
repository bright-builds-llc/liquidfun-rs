use crate::{
    CatalogDefinition, CatalogError, FloatBits, RigidAabbBits, RigidBodyKind,
    RigidContactDirectiveTarget, RigidFilterBits, RigidFixtureChildSelector,
    RigidPreSolveDirective, RigidQueryDirective, RigidQueryDirectiveRule, RigidRayDirective,
    RigidRayDirectiveRule, RigidWakePolicy, RigidWorldAction, RigidWorldWitness,
    SemanticEntityKind,
};

use super::{bits, configured_steps, default_settings, definition, entity_id, vec2};

/// Returns the reviewed native rigid-body scenario definitions.
///
/// # Errors
///
/// Returns [`CatalogError`] if a stable ID, schedule, setting, or coverage declaration is invalid.
#[allow(
    clippy::too_many_lines,
    reason = "the reviewed rigid scenario inventory is clearest as one ordered definition list"
)]
pub fn definitions() -> Result<Vec<CatalogDefinition>, CatalogError> {
    let body_a = entity_id(SemanticEntityKind::Body, 0)?;
    let body_b = entity_id(SemanticEntityKind::Body, 1)?;
    let fixture_a = entity_id(SemanticEntityKind::Fixture, 2)?;
    let fixture_b = entity_id(SemanticEntityKind::Fixture, 3)?;
    let settings = default_settings(1)?;
    let base_entities = vec![
        SemanticEntityKind::Body,
        SemanticEntityKind::Body,
        SemanticEntityKind::Fixture,
        SemanticEntityKind::Fixture,
    ];
    let base_setup = vec![
        RigidWorldAction::CreateBody {
            body_id: body_a.clone(),
        },
        RigidWorldAction::CreateBody {
            body_id: body_b.clone(),
        },
        RigidWorldAction::CreateFixture {
            fixture_id: fixture_a.clone(),
        },
        RigidWorldAction::CreateFixture {
            fixture_id: fixture_b.clone(),
        },
    ];

    let mut definitions = Vec::with_capacity(10);
    definitions.push(definition(
        "rigid-non-colliding-lifecycle",
        "Rigid Non-Colliding Lifecycle",
        "native-rigid-v1",
        &["rigid", "lifecycle"],
        "rigid-world-lifecycle-test",
        &[
            RigidWorldWitness::StaticKinematicOverlapRejected,
            RigidWorldWitness::ZeroContactStep,
        ],
        None,
        base_entities.clone(),
        append(
            &base_setup,
            vec![
                RigidWorldAction::InspectBody {
                    body_id: body_a.clone(),
                },
                RigidWorldAction::InspectFixture {
                    fixture_id: fixture_a.clone(),
                },
            ],
        ),
        configured_steps(settings, 1),
        1,
    )?);
    definitions.push(definition(
        "rigid-contact-lifecycle",
        "Rigid Contact Lifecycle",
        "native-rigid-v1",
        &["rigid", "contacts"],
        "rigid-world-contact-test",
        &[
            RigidWorldWitness::ContactCreated,
            RigidWorldWitness::ContactPersisted,
            RigidWorldWitness::ContactSolved,
        ],
        None,
        base_entities.clone(),
        base_setup.clone(),
        configured_steps(settings, 3),
        1,
    )?);

    let mut stack_entities = Vec::with_capacity(8);
    let mut stack_setup = Vec::with_capacity(8);
    for ordinal in 0..4 {
        stack_entities.push(SemanticEntityKind::Body);
        stack_setup.push(RigidWorldAction::CreateBody {
            body_id: entity_id(SemanticEntityKind::Body, ordinal)?,
        });
    }
    for ordinal in 4..8 {
        stack_entities.push(SemanticEntityKind::Fixture);
        stack_setup.push(RigidWorldAction::CreateFixture {
            fixture_id: entity_id(SemanticEntityKind::Fixture, ordinal)?,
        });
    }
    definitions.push(definition(
        "rigid-stack-stability",
        "Rigid Stack Stability",
        "native-rigid-v1",
        &["rigid", "stacks"],
        "rigid-world-stack-test",
        &[
            RigidWorldWitness::MultiContactIslandSolved,
            RigidWorldWitness::IslandTraversalOrdered,
            RigidWorldWitness::WarmStartApplied,
        ],
        None,
        stack_entities,
        stack_setup,
        configured_steps(settings, 8),
        1,
    )?);
    definitions.push(definition(
        "rigid-sleep-and-wake",
        "Rigid Sleep and Wake",
        "native-rigid-v1",
        &["rigid", "sleep"],
        "rigid-world-sleep-test",
        &[
            RigidWorldWitness::WholeIslandSlept,
            RigidWorldWitness::MutationWokeBody,
        ],
        None,
        base_entities.clone(),
        append(
            &base_setup,
            vec![
                RigidWorldAction::SetSleepingAllowed {
                    body_id: body_a.clone(),
                    sleeping_allowed: true,
                },
                RigidWorldAction::SetAwake {
                    body_id: body_a.clone(),
                    awake: false,
                },
                RigidWorldAction::ApplyForce {
                    body_id: body_a.clone(),
                    force: vec2(1.0, 0.0),
                    point: vec2(0.0, 0.0),
                    wake_policy: RigidWakePolicy::Wake,
                },
            ],
        ),
        configured_steps(settings, 4),
        1,
    )?);
    definitions.push(definition(
        "rigid-continuous-collision",
        "Rigid Continuous Collision",
        "native-rigid-v1",
        &["rigid", "continuous-collision"],
        "rigid-world-continuous-test",
        &[
            RigidWorldWitness::ContinuousPhysicsPreventedTunneling,
            RigidWorldWitness::BulletStateSelectedContinuousContact,
        ],
        None,
        base_entities.clone(),
        append(
            &base_setup,
            vec![
                RigidWorldAction::SetBullet {
                    body_id: body_b.clone(),
                    bullet: true,
                },
                RigidWorldAction::SetContinuousPhysics { enabled: true },
                RigidWorldAction::SetLinearVelocity {
                    body_id: body_b.clone(),
                    velocity: vec2(-50.0, 0.0),
                },
            ],
        ),
        configured_steps(settings, 2),
        1,
    )?);
    definitions.push(definition(
        "rigid-collision-filtering",
        "Rigid Collision Filtering",
        "native-rigid-v1",
        &["rigid", "filtering"],
        "rigid-world-filter-test",
        &[
            RigidWorldWitness::FilterRemovedContact,
            RigidWorldWitness::FilterRecreatedContact,
        ],
        None,
        base_entities.clone(),
        append(
            &base_setup,
            vec![
                RigidWorldAction::SetFixtureFilter {
                    fixture_id: fixture_b.clone(),
                    filter: RigidFilterBits::new(0x0002, 0x0001, 0),
                },
                RigidWorldAction::SetFixtureFilter {
                    fixture_id: fixture_b.clone(),
                    filter: RigidFilterBits::new(0x0002, 0xffff, 0),
                },
            ],
        ),
        configured_steps(settings, 2),
        1,
    )?);
    definitions.push(definition(
        "rigid-world-queries",
        "Rigid World Queries",
        "native-rigid-v1",
        &["rigid", "queries"],
        "rigid-world-query-test",
        &[
            RigidWorldWitness::QueryPreservedDuplicateOccurrences,
            RigidWorldWitness::RayNearestHitSelected,
        ],
        None,
        base_entities.clone(),
        append(&base_setup, query_actions(&fixture_a)),
        configured_steps(settings, 1),
        1,
    )?);
    definitions.push(definition(
        "rigid-callback-timing",
        "Rigid Callback Timing",
        "native-rigid-v1",
        &["rigid", "callbacks"],
        "rigid-world-callback-test",
        &[
            RigidWorldWitness::CallbackLifecycleOrderObserved,
            RigidWorldWitness::PreSolveMaterialObserved,
        ],
        None,
        base_entities.clone(),
        append(&base_setup, callback_actions(&fixture_a, &fixture_b)),
        configured_steps(settings, 2),
        1,
    )?);
    definitions.push(definition(
        "rigid-runtime-mutation",
        "Rigid Runtime Mutation",
        "native-rigid-v1",
        &["rigid", "mutation"],
        "rigid-world-mutation-test",
        &[
            RigidWorldWitness::BodyTypeChanged,
            RigidWorldWitness::SensorEnabled,
            RigidWorldWitness::MaterialChanged,
        ],
        None,
        base_entities.clone(),
        append(
            &base_setup,
            vec![
                RigidWorldAction::SetBodyType {
                    body_id: body_a.clone(),
                    body_kind: RigidBodyKind::Dynamic,
                },
                RigidWorldAction::SetFixtureSensor {
                    fixture_id: fixture_a.clone(),
                    sensor: true,
                },
                RigidWorldAction::SetFixtureMaterial {
                    fixture_id: fixture_a.clone(),
                    friction_bits: bits(0.5),
                    restitution_bits: bits(0.25),
                },
            ],
        ),
        configured_steps(settings, 1),
        1,
    )?);
    definitions.push(definition(
        "rigid-destruction-order",
        "Rigid Destruction Order",
        "native-rigid-v1",
        &["rigid", "destruction"],
        "rigid-world-destruction-test",
        &[
            RigidWorldWitness::FixtureDestroyed,
            RigidWorldWitness::BodyDestroyed,
            RigidWorldWitness::DestructionLifecycleOrderObserved,
        ],
        None,
        base_entities,
        append(
            &base_setup,
            vec![
                RigidWorldAction::DestroyFixture {
                    fixture_id: fixture_b,
                },
                RigidWorldAction::DestroyBody { body_id: body_b },
            ],
        ),
        configured_steps(settings, 1),
        1,
    )?);
    Ok(definitions)
}

fn append(base: &[RigidWorldAction], extra: Vec<RigidWorldAction>) -> Vec<RigidWorldAction> {
    base.iter().cloned().chain(extra).collect()
}

fn query_actions(fixture_id: &crate::ScenarioId) -> Vec<RigidWorldAction> {
    let target = RigidFixtureChildSelector {
        fixture_id: fixture_id.clone(),
        child_index: 0,
    };
    vec![
        RigidWorldAction::QueryAabb {
            aabb: RigidAabbBits {
                lower: vec2(-2.0, -2.0),
                upper: vec2(2.0, 2.0),
            },
            directive_rules: vec![RigidQueryDirectiveRule {
                target: target.clone(),
                directive: RigidQueryDirective::Continue,
            }]
            .into_boxed_slice(),
        },
        RigidWorldAction::RayCast {
            start: vec2(-3.0, 0.0),
            end: vec2(3.0, 0.0),
            directive_rules: vec![RigidRayDirectiveRule {
                target,
                directive: RigidRayDirective::Clip {
                    fraction_bits: FloatBits::from_f32(0.5),
                },
            }]
            .into_boxed_slice(),
        },
    ]
}

fn callback_actions(
    fixture_a: &crate::ScenarioId,
    fixture_b: &crate::ScenarioId,
) -> Vec<RigidWorldAction> {
    let target = RigidContactDirectiveTarget {
        fixture_a_id: fixture_a.clone(),
        fixture_b_id: fixture_b.clone(),
    };
    vec![
        RigidWorldAction::SetContactFilterDirective {
            target: target.clone(),
            should_collide: true,
        },
        RigidWorldAction::SetPreSolveDirective {
            target,
            directive: RigidPreSolveDirective {
                enabled: true,
                maybe_friction_bits: Some(bits(0.25)),
                maybe_restitution_bits: Some(bits(0.1)),
                maybe_tangent_speed_bits: Some(bits(0.0)),
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{FloatBits, ResolveRequest, RunSettings, ScenarioConsumer, resolve_catalog};

    use super::definitions;

    #[test]
    fn rigid_definitions_are_unique_complete_and_deterministic() {
        // Arrange
        let definitions = definitions().expect("rigid definitions should be valid");
        let settings = RunSettings::new(FloatBits::from_f32(1.0 / 60.0), 8, 3, 1)
            .expect("default settings should be valid");
        let mut slugs = HashSet::new();

        // Act
        for definition in &definitions {
            assert!(slugs.insert(definition.slug().as_str()));
            let metadata = definition
                .metadata()
                .expect("native examples must declare metadata");
            assert_eq!(metadata.default_settings(), settings);
            assert!(!metadata.tags().is_empty());
            assert!(!metadata.coverage().test_ids().is_empty());
            assert!(!metadata.coverage().evidence_leaves().is_empty());
            for consumer in ScenarioConsumer::ALL {
                assert!(metadata.coverage().is_eligible(consumer));
            }
            let request = ResolveRequest::new(definition.slug().clone(), None, settings);
            let first =
                resolve_catalog(&definitions, &request).expect("rigid definition should resolve");
            let second = resolve_catalog(&definitions, &request)
                .expect("rigid definition should resolve identically");
            let entity_ids = first
                .entities()
                .iter()
                .map(|entity| entity.scenario_id().as_str())
                .collect::<HashSet<_>>();
            let action_ids = first
                .actions()
                .iter()
                .map(|action| action.action_id().as_str())
                .collect::<HashSet<_>>();

            // Assert
            assert_eq!(first.canonical_bytes(), second.canonical_bytes());
            assert_eq!(entity_ids.len(), first.entities().len());
            assert_eq!(action_ids.len(), first.actions().len());
        }
        assert_eq!(definitions.len(), 10);
    }
}
