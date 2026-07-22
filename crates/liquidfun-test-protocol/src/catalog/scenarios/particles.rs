use crate::{
    CatalogDefinition, CatalogError, CatalogEvidence, CatalogEvidenceId, Phase9ParticleAction,
    Phase10GroupDefinition, Phase10GroupDestination, Phase10GroupSource, Phase10Operation,
    Phase10Provenance, RigidWorldAction, SemanticEntityKind, TransformBits,
};

use super::{bits, definition_with_evidence, entity_id, vec2};

const ZOMBIE: u32 = 1 << 1;
const WALL: u32 = 1 << 2;
const SPRING: u32 = 1 << 3;
const ELASTIC: u32 = 1 << 4;
const VISCOUS: u32 = 1 << 5;
const POWDER: u32 = 1 << 6;
const TENSILE: u32 = 1 << 7;
const COLOR_MIXING: u32 = 1 << 8;
const BARRIER: u32 = 1 << 10;
const STATIC_PRESSURE: u32 = 1 << 11;
const REACTIVE: u32 = 1 << 12;
const REPULSIVE: u32 = 1 << 13;
const FIXTURE_CONTACT_LISTENER: u32 = 1 << 14;
const PARTICLE_CONTACT_LISTENER: u32 = 1 << 15;
const FIXTURE_CONTACT_FILTER: u32 = 1 << 16;
const PARTICLE_CONTACT_FILTER: u32 = 1 << 17;

/// Returns reviewed particle storage, lifecycle, force, pause, and solver-flag definitions.
///
/// # Errors
///
/// Returns [`CatalogError`] if a stable ID, exact action, evidence mapping, or bound is invalid.
pub fn definitions() -> Result<Vec<CatalogDefinition>, CatalogError> {
    let mut definitions = lifecycle_definitions()?;
    definitions.extend(flag_definitions()?);
    Ok(definitions)
}

fn lifecycle_definitions() -> Result<Vec<CatalogDefinition>, CatalogError> {
    Ok(vec![
        phase9_definition(
            "particle-storage-lifecycle",
            "Particle Storage and Lifecycle",
            "particle-lifecycle",
            &[
                "stable_ids_compact",
                "optional_lanes",
                "capacity_eviction",
                "teardown",
            ],
            3,
            vec![
                Phase9ParticleAction::InspectSystem {
                    system_id: entity_id(SemanticEntityKind::ParticleSystem, 0)?,
                },
                Phase9ParticleAction::InspectParticle {
                    particle_id: entity_id(SemanticEntityKind::Particle, 2)?,
                },
                Phase9ParticleAction::MarkForDestruction {
                    particle_id: entity_id(SemanticEntityKind::Particle, 2)?,
                },
                Phase9ParticleAction::Compact {
                    system_id: entity_id(SemanticEntityKind::ParticleSystem, 0)?,
                },
                Phase9ParticleAction::RequestStatistics {
                    system_id: entity_id(SemanticEntityKind::ParticleSystem, 0)?,
                },
            ],
        )?,
        phase9_definition(
            "particle-contacts-and-coupling",
            "Particle Contacts and Body Coupling",
            "particle-body-contacts",
            &[
                "particle_contact",
                "body_contact",
                "contact_order",
                "coupling_fields",
                "dynamic_body_reaction",
                "static_body_no_reaction",
            ],
            2,
            vec![
                Phase9ParticleAction::InspectParticleContact {
                    system_id: entity_id(SemanticEntityKind::ParticleSystem, 0)?,
                    contact_index: 0,
                },
                Phase9ParticleAction::InspectBodyContact {
                    system_id: entity_id(SemanticEntityKind::ParticleSystem, 0)?,
                    contact_index: 0,
                },
                Phase9ParticleAction::RequestStatistics {
                    system_id: entity_id(SemanticEntityKind::ParticleSystem, 0)?,
                },
            ],
        )?,
        phase9_definition(
            "particle-forces-and-statistics",
            "Particle Forces, Impulses, and Statistics",
            "particle-forces-statistics",
            &[
                "force_range",
                "impulse_range",
                "statistics_counts",
                "collision_energy",
                "stuck_candidates",
            ],
            3,
            vec![
                Phase9ParticleAction::ApplyForce {
                    particle_ids: particle_ids(3)?,
                    force: vec2(12.0, -6.0),
                },
                Phase9ParticleAction::ApplyImpulse {
                    particle_ids: particle_ids(3)?,
                    impulse: vec2(3.0, 1.5),
                },
                Phase9ParticleAction::RequestStatistics {
                    system_id: entity_id(SemanticEntityKind::ParticleSystem, 0)?,
                },
            ],
        )?,
        phase9_definition(
            "particle-system-pause-action",
            "Particle System Pause Action",
            "particle-lifecycle",
            &["paused_system"],
            1,
            vec![
                Phase9ParticleAction::SetPaused {
                    system_id: entity_id(SemanticEntityKind::ParticleSystem, 0)?,
                    paused: true,
                },
                Phase9ParticleAction::RequestStatistics {
                    system_id: entity_id(SemanticEntityKind::ParticleSystem, 0)?,
                },
                Phase9ParticleAction::SetPaused {
                    system_id: entity_id(SemanticEntityKind::ParticleSystem, 0)?,
                    paused: false,
                },
            ],
        )?,
    ])
}

fn flag_definitions() -> Result<Vec<CatalogDefinition>, CatalogError> {
    let flag_families = [
        (
            "particle-flags-water-zombie",
            "Water and Zombie Particle Flags",
            ZOMBIE,
            &["phase10/water", "phase10/zombie"][..],
        ),
        (
            "particle-flags-wall-barrier",
            "Wall and Barrier Particle Flags",
            WALL | BARRIER,
            &["phase10/wall", "phase10/barrier"],
        ),
        (
            "particle-flags-spring-elastic-reactive",
            "Spring, Elastic, and Reactive Particle Flags",
            SPRING | ELASTIC | REACTIVE,
            &["phase10/spring", "phase10/elastic", "phase10/reactive"],
        ),
        (
            "particle-flags-viscous-powder",
            "Viscous and Powder Particle Flags",
            VISCOUS | POWDER,
            &["phase10/viscous", "phase10/powder"],
        ),
        (
            "particle-flags-tensile-color",
            "Tensile and Color Mixing Particle Flags",
            TENSILE | COLOR_MIXING,
            &["phase10/tensile", "phase10/color_mixing"],
        ),
        (
            "particle-flags-pressure-repulsive",
            "Static Pressure and Repulsive Particle Flags",
            STATIC_PRESSURE | REPULSIVE,
            &["phase10/static_pressure", "phase10/repulsive"],
        ),
        (
            "particle-flags-contact-listeners",
            "Particle Contact Listener Flags",
            FIXTURE_CONTACT_LISTENER | PARTICLE_CONTACT_LISTENER,
            &[
                "inherited/listener_flag_enabled",
                "inherited/listener_flag_disabled",
            ],
        ),
        (
            "particle-flags-contact-filters",
            "Particle Contact Filter Flags",
            FIXTURE_CONTACT_FILTER | PARTICLE_CONTACT_FILTER,
            &[
                "inherited/filter_flag_enabled",
                "inherited/filter_flag_disabled",
            ],
        ),
    ];
    flag_families
        .into_iter()
        .map(|(slug, title, flags, evidence)| flag_definition(slug, title, flags, evidence))
        .collect()
}

fn phase9_definition(
    slug: &str,
    title: &str,
    test_id: &str,
    evidence: &[&str],
    particle_count: u32,
    actions: Vec<Phase9ParticleAction>,
) -> Result<CatalogDefinition, CatalogError> {
    let system_id = entity_id(SemanticEntityKind::ParticleSystem, 0)?;
    let mut entity_kinds = vec![SemanticEntityKind::ParticleSystem];
    let mut setup_actions = vec![particle_action(Phase9ParticleAction::CreateSystem {
        system_id: system_id.clone(),
    })];
    for ordinal in 1..=particle_count {
        entity_kinds.push(SemanticEntityKind::Particle);
        setup_actions.push(particle_action(Phase9ParticleAction::CreateParticle {
            particle_id: entity_id(SemanticEntityKind::Particle, ordinal)?,
        }));
    }
    let mut logical_actions = actions.into_iter().map(particle_action).collect::<Vec<_>>();
    logical_actions.push(particle_action(Phase9ParticleAction::DestroySystem {
        system_id,
    }));
    definition_with_evidence(
        slug,
        title,
        "native-particle-v1",
        &["particle", "headless"],
        test_id,
        evidence
            .iter()
            .map(|id| CatalogEvidenceId::new(*id).map(CatalogEvidence::Phase9))
            .collect::<Result<Vec<_>, _>>()?,
        None,
        entity_kinds,
        setup_actions,
        logical_actions,
        2,
    )
}

fn flag_definition(
    slug: &str,
    title: &str,
    particle_flags_bits: u32,
    evidence: &[&str],
) -> Result<CatalogDefinition, CatalogError> {
    let system_id = entity_id(SemanticEntityKind::ParticleSystem, 0)?;
    let group_id = entity_id(SemanticEntityKind::ParticleGroup, 1)?;
    let member_ids = vec![
        entity_id(SemanticEntityKind::Particle, 2)?,
        entity_id(SemanticEntityKind::Particle, 3)?,
    ];
    let group = group_definition(
        system_id.clone(),
        group_id.clone(),
        member_ids,
        particle_flags_bits,
        0,
    )?;
    definition_with_evidence(
        slug,
        title,
        "native-particle-flags-v1",
        &["particle", "solver-flags"],
        "particle-solver-flags",
        evidence
            .iter()
            .map(|id| CatalogEvidenceId::new(*id).map(CatalogEvidence::Phase10))
            .collect::<Result<Vec<_>, _>>()?,
        None,
        vec![
            SemanticEntityKind::ParticleSystem,
            SemanticEntityKind::ParticleGroup,
            SemanticEntityKind::Particle,
            SemanticEntityKind::Particle,
        ],
        vec![
            particle_action(Phase9ParticleAction::CreateSystem {
                system_id: system_id.clone(),
            }),
            group_action(Phase10Operation::CreateGroup { definition: group }),
        ],
        vec![
            group_action(Phase10Operation::Step {
                timestep_bits: bits(1.0 / 60.0),
                velocity_iterations: 8,
                position_iterations: 3,
                particle_iterations: 2,
            }),
            group_action(Phase10Operation::InspectState),
            group_action(Phase10Operation::DestroyGroup { group_id }),
            particle_action(Phase9ParticleAction::DestroySystem { system_id }),
        ],
        2,
    )
}

pub(super) fn group_definition(
    system_id: crate::ScenarioId,
    group_id: crate::ScenarioId,
    member_ids: Vec<crate::ScenarioId>,
    particle_flags_bits: u32,
    group_flags_bits: u32,
) -> Result<Phase10GroupDefinition, CatalogError> {
    let positions = std::iter::successors(Some(0.0_f32), |position| Some(*position + 0.5))
        .take(member_ids.len())
        .map(|position| vec2(position, 0.0))
        .collect::<Vec<_>>();
    Ok(Phase10GroupDefinition {
        provenance: Phase10Provenance {
            extension_version: 1,
            generator_id: crate::ScenarioId::new("phase11-catalog-generator")
                .map_err(|_| CatalogError::new(crate::CatalogErrorKind::InvalidIdentifier))?,
            generator_version: crate::ScenarioId::new("v1")
                .map_err(|_| CatalogError::new(crate::CatalogErrorKind::InvalidIdentifier))?,
            upstream_revision: crate::ScenarioId::new("7f20402173fd143a3988c921bc384459c6a858f2")
                .map_err(|_| {
                CatalogError::new(crate::CatalogErrorKind::InvalidIdentifier)
            })?,
            toolchain_id: crate::ScenarioId::new("rust-catalog-v1")
                .map_err(|_| CatalogError::new(crate::CatalogErrorKind::InvalidIdentifier))?,
            seed: 0,
        },
        system_id,
        group_id,
        member_ids: member_ids.into_boxed_slice(),
        source: Phase10GroupSource::Explicit {
            positions: positions.into_boxed_slice(),
        },
        destination: Phase10GroupDestination::New,
        particle_flags_bits,
        group_flags_bits,
        transform: TransformBits {
            position: vec2(0.0, 0.0),
            angle_bits: bits(0.0),
        },
        linear_velocity: vec2(0.0, 0.0),
        angular_velocity_bits: bits(0.0),
        color: [32, 128, 255, 255],
        strength_bits: bits(1.0),
        maybe_stride_bits: None,
        lifetime_bits: bits(0.0),
    })
}

fn particle_ids(count: u32) -> Result<Box<[crate::ScenarioId]>, CatalogError> {
    (1..=count)
        .map(|ordinal| entity_id(SemanticEntityKind::Particle, ordinal))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

pub(super) fn particle_action(action: Phase9ParticleAction) -> RigidWorldAction {
    RigidWorldAction::Particle { action }
}

pub(super) fn group_action(operation: Phase10Operation) -> RigidWorldAction {
    RigidWorldAction::ParticleGroup { operation }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{
        FloatBits, Phase9ParticleAction, ResolveRequest, RigidWorldAction, RunSettings,
        resolve_catalog,
    };

    use super::definitions;

    #[test]
    fn particle_definitions_cover_reviewed_phase9_families_deterministically() {
        // Arrange
        let definitions = definitions().expect("particle definitions should be valid");
        let settings = RunSettings::new(FloatBits::from_f32(1.0 / 60.0), 8, 3, 2)
            .expect("settings should be valid");
        let mut slugs = HashSet::new();

        // Act / Assert
        for definition in &definitions {
            assert!(slugs.insert(definition.slug().as_str()));
            let request = ResolveRequest::new(definition.slug().clone(), None, settings);
            let first = resolve_catalog(&definitions, &request).expect("definition should resolve");
            let second = resolve_catalog(&definitions, &request).expect("definition should repeat");
            assert_eq!(first.canonical_bytes(), second.canonical_bytes());
            assert!(definition.metadata().is_some());
        }
        assert_eq!(definitions.len(), 12);
    }

    #[test]
    fn particle_system_pause_is_a_typed_physics_action() {
        // Arrange
        let definitions = definitions().expect("particle definitions should be valid");
        let definition = definitions
            .iter()
            .find(|definition| definition.slug().as_str() == "particle-system-pause-action")
            .expect("pause definition exists");
        let settings = definition
            .metadata()
            .expect("metadata exists")
            .default_settings();

        // Act
        let resolved = resolve_catalog(
            &definitions,
            &ResolveRequest::new(definition.slug().clone(), None, settings),
        )
        .expect("pause definition resolves");

        // Assert
        assert!(resolved.actions().iter().any(|action| matches!(
            action.action(),
            RigidWorldAction::Particle {
                action: Phase9ParticleAction::SetPaused { paused: true, .. }
            }
        )));
    }
}
