use std::collections::{HashMap, HashSet};

use super::super::model::{CatalogError, CatalogErrorKind, ResolvedEntity};
use super::validation::{require_entity, require_finite_vec2};
use crate::{
    PHASE9_MAXIMUM_IDENTITIES, Phase9ParticleAction, Phase10GroupDestination, Phase10Operation,
    ScenarioId, SemanticEntityKind, validate_phase10_operation,
};

#[derive(Debug, Default)]
pub(super) struct CatalogSemanticState {
    created_systems: HashSet<ScenarioId>,
    live_systems: HashSet<ScenarioId>,
    created_particles: HashSet<ScenarioId>,
    live_particle_owners: HashMap<ScenarioId, ScenarioId>,
    pending_particles: HashSet<ScenarioId>,
    created_groups: HashSet<ScenarioId>,
    live_group_owners: HashMap<ScenarioId, ScenarioId>,
    maybe_group_provenance: Option<crate::Phase10Provenance>,
}

#[allow(
    clippy::too_many_lines,
    reason = "the closed Phase 9 action vocabulary keeps lifecycle checks explicit"
)]
pub(super) fn validate_particle_action(
    action: &Phase9ParticleAction,
    entities: &[ResolvedEntity],
    state: &mut CatalogSemanticState,
) -> Result<(), CatalogError> {
    match action {
        Phase9ParticleAction::CreateSystem { system_id } => {
            require_entity(entities, system_id, SemanticEntityKind::ParticleSystem)?;
            if !state.created_systems.insert(system_id.clone())
                || !state.live_systems.insert(system_id.clone())
            {
                return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
            }
        }
        Phase9ParticleAction::DestroySystem { system_id } => {
            require_live_system(entities, system_id, state)?;
            if state
                .live_group_owners
                .values()
                .any(|owner| owner == system_id)
            {
                return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
            }
            state.live_systems.remove(system_id);
            state
                .live_particle_owners
                .retain(|_, owner| owner != system_id);
            state
                .pending_particles
                .retain(|particle_id| state.live_particle_owners.contains_key(particle_id));
        }
        Phase9ParticleAction::CreateParticle { particle_id } => {
            require_entity(entities, particle_id, SemanticEntityKind::Particle)?;
            let owner = sole_live_system(state)?;
            if !state.created_particles.insert(particle_id.clone())
                || state
                    .live_particle_owners
                    .insert(particle_id.clone(), owner)
                    .is_some()
            {
                return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
            }
        }
        Phase9ParticleAction::InspectSystem { system_id }
        | Phase9ParticleAction::InspectParticleContact { system_id, .. }
        | Phase9ParticleAction::InspectBodyContact { system_id, .. }
        | Phase9ParticleAction::SetPaused { system_id, .. }
        | Phase9ParticleAction::Compact { system_id }
        | Phase9ParticleAction::RequestStatistics { system_id } => {
            require_live_system(entities, system_id, state)?;
            if let Phase9ParticleAction::InspectParticleContact { contact_index, .. }
            | Phase9ParticleAction::InspectBodyContact { contact_index, .. } = action
                && *contact_index >= PHASE9_MAXIMUM_IDENTITIES
            {
                return Err(CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded));
            }
            if matches!(action, Phase9ParticleAction::Compact { .. }) {
                state
                    .pending_particles
                    .retain(|particle_id| state.live_particle_owners.contains_key(particle_id));
            }
        }
        Phase9ParticleAction::InspectParticle { particle_id }
        | Phase9ParticleAction::SetPosition { particle_id, .. }
        | Phase9ParticleAction::SetVelocity { particle_id, .. } => {
            require_live_particle(entities, particle_id, state)?;
            if let Phase9ParticleAction::SetPosition { position, .. } = action {
                require_finite_vec2(*position)?;
            }
            if let Phase9ParticleAction::SetVelocity { velocity, .. } = action {
                require_finite_vec2(*velocity)?;
            }
        }
        Phase9ParticleAction::MarkForDestruction { particle_id } => {
            require_live_particle(entities, particle_id, state)?;
            state.live_particle_owners.remove(particle_id);
            state.pending_particles.insert(particle_id.clone());
        }
        Phase9ParticleAction::ApplyForce {
            particle_ids,
            force,
        }
        | Phase9ParticleAction::ApplyImpulse {
            particle_ids,
            impulse: force,
        } => {
            validate_particle_range(particle_ids, entities, state)?;
            require_finite_vec2(*force)?;
        }
        Phase9ParticleAction::InspectOccurrence { occurrence_index } => {
            if *occurrence_index >= PHASE9_MAXIMUM_IDENTITIES {
                return Err(CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded));
            }
        }
        Phase9ParticleAction::QueryAabb {
            system_id,
            lower,
            upper,
            ..
        } => {
            if let Some(system_id) = system_id {
                require_live_system(entities, system_id, state)?;
            }
            require_finite_vec2(*lower)?;
            require_finite_vec2(*upper)?;
            if lower.x_bits.to_f32() > upper.x_bits.to_f32()
                || lower.y_bits.to_f32() > upper.y_bits.to_f32()
            {
                return Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings));
            }
        }
        Phase9ParticleAction::RayCast {
            system_id,
            start,
            end,
            ..
        } => {
            if let Some(system_id) = system_id {
                require_live_system(entities, system_id, state)?;
            }
            require_finite_vec2(*start)?;
            require_finite_vec2(*end)?;
            if start == end {
                return Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings));
            }
        }
    }
    Ok(())
}

fn validate_particle_range(
    particle_ids: &[ScenarioId],
    entities: &[ResolvedEntity],
    state: &CatalogSemanticState,
) -> Result<(), CatalogError> {
    if particle_ids.is_empty() || particle_ids.len() > PHASE9_MAXIMUM_IDENTITIES {
        return Err(CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded));
    }
    let mut unique = HashSet::with_capacity(particle_ids.len());
    let mut maybe_owner = None;
    for particle_id in particle_ids {
        if !unique.insert(particle_id) {
            return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
        }
        let owner = require_live_particle(entities, particle_id, state)?;
        if maybe_owner
            .as_ref()
            .is_some_and(|expected| *expected != owner)
        {
            return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
        }
        maybe_owner = Some(owner);
    }
    Ok(())
}

#[allow(
    clippy::too_many_lines,
    reason = "the closed Phase 10 operation vocabulary keeps ownership checks explicit"
)]
pub(super) fn validate_group_operation(
    operation: &Phase10Operation,
    settings: super::super::RunSettings,
    entities: &[ResolvedEntity],
    state: &mut CatalogSemanticState,
) -> Result<(), CatalogError> {
    validate_phase10_operation(operation)
        .map_err(|_| CatalogError::new(CatalogErrorKind::InvalidRunSettings))?;
    match operation {
        Phase10Operation::CreateGroup { definition } => {
            require_live_system(entities, &definition.system_id, state)?;
            if state
                .maybe_group_provenance
                .as_ref()
                .is_some_and(|provenance| provenance != &definition.provenance)
            {
                return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
            }
            state
                .maybe_group_provenance
                .get_or_insert_with(|| definition.provenance.clone());
            for particle_id in &definition.member_ids {
                require_entity(entities, particle_id, SemanticEntityKind::Particle)?;
                if state.created_particles.contains(particle_id)
                    || !state.created_particles.insert(particle_id.clone())
                {
                    return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
                }
            }
            match &definition.destination {
                Phase10GroupDestination::New => {
                    require_entity(
                        entities,
                        &definition.group_id,
                        SemanticEntityKind::ParticleGroup,
                    )?;
                    if !state.created_groups.insert(definition.group_id.clone())
                        || state
                            .live_group_owners
                            .insert(definition.group_id.clone(), definition.system_id.clone())
                            .is_some()
                    {
                        return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
                    }
                }
                Phase10GroupDestination::AppendTo { target_group_id } => {
                    require_live_group(entities, target_group_id, state)?;
                    if state.live_group_owners.get(target_group_id) != Some(&definition.system_id) {
                        return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
                    }
                }
            }
        }
        Phase10Operation::JoinGroups {
            target_group_id,
            source_group_id,
        } => {
            require_live_group(entities, target_group_id, state)?;
            require_live_group(entities, source_group_id, state)?;
            if target_group_id == source_group_id
                || state.live_group_owners.get(target_group_id)
                    != state.live_group_owners.get(source_group_id)
            {
                return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
            }
            state.live_group_owners.remove(source_group_id);
        }
        Phase10Operation::SplitGroup {
            group_id,
            created_group_ids,
        } => {
            require_live_group(entities, group_id, state)?;
            let owner = state
                .live_group_owners
                .get(group_id)
                .cloned()
                .ok_or_else(|| CatalogError::new(CatalogErrorKind::InvalidIdentifier))?;
            for created_group_id in created_group_ids {
                require_entity(
                    entities,
                    created_group_id,
                    SemanticEntityKind::ParticleGroup,
                )?;
                if !state.created_groups.insert(created_group_id.clone())
                    || state
                        .live_group_owners
                        .insert(created_group_id.clone(), owner.clone())
                        .is_some()
                {
                    return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
                }
            }
        }
        Phase10Operation::SetGroupFlags { group_id, .. } => {
            require_live_group(entities, group_id, state)?;
        }
        Phase10Operation::DestroyGroup { group_id } => {
            require_live_group(entities, group_id, state)?;
            state.live_group_owners.remove(group_id);
        }
        Phase10Operation::Step {
            timestep_bits,
            velocity_iterations,
            position_iterations,
            particle_iterations,
        } => {
            if *timestep_bits != settings.timestep_bits()
                || *velocity_iterations != settings.velocity_iterations()
                || *position_iterations != settings.position_iterations()
                || *particle_iterations != settings.particle_iterations()
            {
                return Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings));
            }
        }
        Phase10Operation::InspectState => {
            if state.maybe_group_provenance.is_none() {
                return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
            }
        }
    }
    Ok(())
}

fn sole_live_system(state: &CatalogSemanticState) -> Result<ScenarioId, CatalogError> {
    let mut systems = state.live_systems.iter();
    let system_id = systems
        .next()
        .cloned()
        .ok_or_else(|| CatalogError::new(CatalogErrorKind::InvalidIdentifier))?;
    if systems.next().is_some() {
        return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
    }
    Ok(system_id)
}

fn require_live_system(
    entities: &[ResolvedEntity],
    system_id: &ScenarioId,
    state: &CatalogSemanticState,
) -> Result<(), CatalogError> {
    require_entity(entities, system_id, SemanticEntityKind::ParticleSystem)?;
    if state.live_systems.contains(system_id) {
        return Ok(());
    }
    Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier))
}

fn require_live_particle<'a>(
    entities: &[ResolvedEntity],
    particle_id: &ScenarioId,
    state: &'a CatalogSemanticState,
) -> Result<&'a ScenarioId, CatalogError> {
    require_entity(entities, particle_id, SemanticEntityKind::Particle)?;
    state
        .live_particle_owners
        .get(particle_id)
        .ok_or_else(|| CatalogError::new(CatalogErrorKind::InvalidIdentifier))
}

fn require_live_group(
    entities: &[ResolvedEntity],
    group_id: &ScenarioId,
    state: &CatalogSemanticState,
) -> Result<(), CatalogError> {
    require_entity(entities, group_id, SemanticEntityKind::ParticleGroup)?;
    if state.live_group_owners.contains_key(group_id) {
        return Ok(());
    }
    Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier))
}
