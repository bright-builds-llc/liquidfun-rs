use crate::{
    CatalogDefinition, CatalogError, CatalogEvidence, CatalogEvidenceId, Phase10GroupDestination,
    Phase10Operation, SemanticEntityKind,
};

use super::{
    definition_with_evidence, entity_id,
    particles::{group_action, group_definition, particle_action},
};
use crate::Phase9ParticleAction;

const SOLID_GROUP: u32 = 1 << 0;
const RIGID_GROUP: u32 = 1 << 1;
const GROUP_CAN_BE_EMPTY: u32 = 1 << 2;
const REACTIVE: u32 = 1 << 12;

/// Returns reviewed particle-group construction, topology, solver, and destruction definitions.
///
/// # Errors
///
/// Returns [`CatalogError`] if an exact group definition or evidence mapping is invalid.
pub fn definitions() -> Result<Vec<CatalogDefinition>, CatalogError> {
    Ok(vec![
        construction_and_append()?,
        join_groups()?,
        split_reactive_group()?,
        solid_rigid_group()?,
        destruction_group()?,
    ])
}

fn construction_and_append() -> Result<CatalogDefinition, CatalogError> {
    let system_id = entity_id(SemanticEntityKind::ParticleSystem, 0)?;
    let group_id = entity_id(SemanticEntityKind::ParticleGroup, 1)?;
    let first_members = particle_range(2, 2)?;
    let appended_members = particle_range(4, 2)?;
    let first = group_definition(system_id.clone(), group_id.clone(), first_members, 0, 0)?;
    let mut append = group_definition(system_id.clone(), group_id.clone(), appended_members, 0, 0)?;
    append.destination = Phase10GroupDestination::AppendTo {
        target_group_id: group_id.clone(),
    };
    group_catalog_definition(
        "particle-group-construction-append",
        "Particle Group Construction and Append",
        &[
            "phase10/group_create",
            "phase10/group_append",
            "phase10/group_flags",
        ],
        vec![
            SemanticEntityKind::ParticleSystem,
            SemanticEntityKind::ParticleGroup,
            SemanticEntityKind::Particle,
            SemanticEntityKind::Particle,
            SemanticEntityKind::Particle,
            SemanticEntityKind::Particle,
        ],
        vec![
            create_system(system_id.clone()),
            group_action(Phase10Operation::CreateGroup { definition: first }),
        ],
        vec![
            group_action(Phase10Operation::CreateGroup { definition: append }),
            group_action(Phase10Operation::InspectState),
            group_action(Phase10Operation::DestroyGroup {
                group_id: group_id.clone(),
            }),
            destroy_system(system_id),
        ],
    )
}

fn join_groups() -> Result<CatalogDefinition, CatalogError> {
    let system_id = entity_id(SemanticEntityKind::ParticleSystem, 0)?;
    let target_group_id = entity_id(SemanticEntityKind::ParticleGroup, 1)?;
    let source_group_id = entity_id(SemanticEntityKind::ParticleGroup, 2)?;
    let target = group_definition(
        system_id.clone(),
        target_group_id.clone(),
        particle_range(3, 2)?,
        0,
        0,
    )?;
    let source = group_definition(
        system_id.clone(),
        source_group_id.clone(),
        particle_range(5, 2)?,
        0,
        0,
    )?;
    group_catalog_definition(
        "particle-group-join",
        "Particle Group Join Topology",
        &["phase10/group_join"],
        kinds_with_particles(2, 4),
        vec![
            create_system(system_id.clone()),
            group_action(Phase10Operation::CreateGroup { definition: target }),
            group_action(Phase10Operation::CreateGroup { definition: source }),
        ],
        vec![
            group_action(Phase10Operation::JoinGroups {
                target_group_id: target_group_id.clone(),
                source_group_id,
            }),
            group_action(Phase10Operation::InspectState),
            group_action(Phase10Operation::DestroyGroup {
                group_id: target_group_id,
            }),
            destroy_system(system_id),
        ],
    )
}

fn split_reactive_group() -> Result<CatalogDefinition, CatalogError> {
    let system_id = entity_id(SemanticEntityKind::ParticleSystem, 0)?;
    let group_id = entity_id(SemanticEntityKind::ParticleGroup, 1)?;
    let created_group_ids = vec![
        entity_id(SemanticEntityKind::ParticleGroup, 2)?,
        entity_id(SemanticEntityKind::ParticleGroup, 3)?,
    ];
    let group = group_definition(
        system_id.clone(),
        group_id.clone(),
        particle_range(4, 4)?,
        REACTIVE,
        0,
    )?;
    group_catalog_definition(
        "particle-group-split-reactive",
        "Reactive Particle Group Split",
        &["phase10/group_split", "phase10/reactive"],
        kinds_with_particles(3, 4),
        vec![
            create_system(system_id.clone()),
            group_action(Phase10Operation::CreateGroup { definition: group }),
        ],
        vec![
            group_action(Phase10Operation::SplitGroup {
                group_id: group_id.clone(),
                created_group_ids: created_group_ids.clone().into_boxed_slice(),
            }),
            group_action(Phase10Operation::InspectState),
            group_action(Phase10Operation::DestroyGroup {
                group_id: group_id.clone(),
            }),
            group_action(Phase10Operation::DestroyGroup {
                group_id: created_group_ids[0].clone(),
            }),
            group_action(Phase10Operation::DestroyGroup {
                group_id: created_group_ids[1].clone(),
            }),
            destroy_system(system_id),
        ],
    )
}

fn solid_rigid_group() -> Result<CatalogDefinition, CatalogError> {
    let system_id = entity_id(SemanticEntityKind::ParticleSystem, 0)?;
    let group_id = entity_id(SemanticEntityKind::ParticleGroup, 1)?;
    let group = group_definition(
        system_id.clone(),
        group_id.clone(),
        particle_range(2, 2)?,
        0,
        SOLID_GROUP | RIGID_GROUP,
    )?;
    group_catalog_definition(
        "particle-group-solid-rigid",
        "Solid and Rigid Particle Group Flags",
        &["phase10/solid_group", "phase10/rigid_group"],
        kinds_with_particles(1, 2),
        vec![
            create_system(system_id.clone()),
            group_action(Phase10Operation::CreateGroup { definition: group }),
        ],
        vec![
            group_action(Phase10Operation::SetGroupFlags {
                group_id: group_id.clone(),
                group_flags_bits: SOLID_GROUP | RIGID_GROUP,
            }),
            group_action(Phase10Operation::Step {
                timestep_bits: crate::FloatBits::from_f32(1.0 / 60.0),
                velocity_iterations: 8,
                position_iterations: 3,
                particle_iterations: 2,
            }),
            group_action(Phase10Operation::InspectState),
            group_action(Phase10Operation::DestroyGroup { group_id }),
            destroy_system(system_id),
        ],
    )
}

fn destruction_group() -> Result<CatalogDefinition, CatalogError> {
    let system_id = entity_id(SemanticEntityKind::ParticleSystem, 0)?;
    let group_id = entity_id(SemanticEntityKind::ParticleGroup, 1)?;
    let group = group_definition(
        system_id.clone(),
        group_id.clone(),
        particle_range(2, 2)?,
        0,
        GROUP_CAN_BE_EMPTY,
    )?;
    group_catalog_definition(
        "particle-group-destruction",
        "Particle Group Destruction Lifecycle",
        &["phase10/group_destroy"],
        kinds_with_particles(1, 2),
        vec![
            create_system(system_id.clone()),
            group_action(Phase10Operation::CreateGroup { definition: group }),
        ],
        vec![
            group_action(Phase10Operation::InspectState),
            group_action(Phase10Operation::DestroyGroup { group_id }),
            destroy_system(system_id),
        ],
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "group definitions keep their exact schedule and coverage adjacent"
)]
fn group_catalog_definition(
    slug: &str,
    title: &str,
    evidence: &[&str],
    entity_kinds: Vec<SemanticEntityKind>,
    setup_actions: Vec<crate::RigidWorldAction>,
    logical_actions: Vec<crate::RigidWorldAction>,
) -> Result<CatalogDefinition, CatalogError> {
    definition_with_evidence(
        slug,
        title,
        "native-particle-group-v1",
        &["particle-group", "headless"],
        "particle-group-mutation",
        evidence
            .iter()
            .map(|id| CatalogEvidenceId::new(*id).map(CatalogEvidence::Phase10))
            .collect::<Result<Vec<_>, _>>()?,
        None,
        entity_kinds,
        setup_actions,
        logical_actions,
        2,
    )
}

fn particle_range(start: u32, count: u32) -> Result<Vec<crate::ScenarioId>, CatalogError> {
    (start..start + count)
        .map(|ordinal| entity_id(SemanticEntityKind::Particle, ordinal))
        .collect()
}

fn kinds_with_particles(group_count: usize, particle_count: usize) -> Vec<SemanticEntityKind> {
    let mut kinds = vec![SemanticEntityKind::ParticleSystem];
    kinds.extend(std::iter::repeat_n(
        SemanticEntityKind::ParticleGroup,
        group_count,
    ));
    kinds.extend(std::iter::repeat_n(
        SemanticEntityKind::Particle,
        particle_count,
    ));
    kinds
}

fn create_system(system_id: crate::ScenarioId) -> crate::RigidWorldAction {
    particle_action(Phase9ParticleAction::CreateSystem { system_id })
}

fn destroy_system(system_id: crate::ScenarioId) -> crate::RigidWorldAction {
    particle_action(Phase9ParticleAction::DestroySystem { system_id })
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{
        FloatBits, Phase10Operation, ResolveRequest, RigidWorldAction, RunSettings, resolve_catalog,
    };

    use super::definitions;

    #[test]
    fn group_definitions_cover_reviewed_phase10_families_deterministically() {
        // Arrange
        let definitions = definitions().expect("group definitions should be valid");
        let settings = RunSettings::new(FloatBits::from_f32(1.0 / 60.0), 8, 3, 2)
            .expect("settings should be valid");
        let mut slugs = HashSet::new();

        // Act / Assert
        for definition in &definitions {
            assert!(slugs.insert(definition.slug().as_str()));
            let request = ResolveRequest::new(definition.slug().clone(), None, settings);
            let first = resolve_catalog(&definitions, &request).expect("definition resolves");
            let second = resolve_catalog(&definitions, &request).expect("definition repeats");
            assert_eq!(first.canonical_bytes(), second.canonical_bytes());
        }
        assert_eq!(definitions.len(), 5);
    }

    #[test]
    fn group_catalog_uses_closed_phase10_operations() {
        // Arrange
        let definitions = definitions().expect("group definitions should be valid");
        let settings = RunSettings::new(FloatBits::from_f32(1.0 / 60.0), 8, 3, 2)
            .expect("settings should be valid");

        // Act
        let operations = definitions
            .iter()
            .map(|definition| {
                resolve_catalog(
                    &definitions,
                    &ResolveRequest::new(definition.slug().clone(), None, settings),
                )
                .expect("definition resolves")
            })
            .collect::<Vec<_>>();
        let operations = operations
            .iter()
            .flat_map(crate::ResolvedScenario::actions)
            .filter_map(|action| match action.action() {
                RigidWorldAction::ParticleGroup { operation } => Some(operation),
                _ => None,
            })
            .collect::<Vec<_>>();

        // Assert
        assert!(
            operations
                .iter()
                .any(|operation| matches!(operation, Phase10Operation::JoinGroups { .. }))
        );
        assert!(
            operations
                .iter()
                .any(|operation| matches!(operation, Phase10Operation::SplitGroup { .. }))
        );
        assert!(
            operations
                .iter()
                .any(|operation| matches!(operation, Phase10Operation::SetGroupFlags { .. }))
        );
    }
}
