use std::error::Error;
use std::fmt;

use crate::particle::ParticleGroupFlags;
use crate::particle::storage::{GroupPlanError, ParticleStorageError, SplitPlanError};
use crate::{
    ArenaInsertError, AssociationMap, DestroyedId, DestructionCause, DestructionRecord,
    HandleError, LifecycleEvent, MutationReport, ObjectSnapshot, ParticleGroupId,
};

use super::{ParticleGroup, World, group_topology_limits};

/// A failure while mutating particle-group membership or metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParticleGroupMutationError {
    /// The world is currently inside its locked step transaction.
    WorldLocked,
    /// One of the supplied stable identities is invalid for this operation.
    InvalidHandle(HandleError),
    /// A new split-group shell or diagnostic identity could not be allocated.
    Arena(ArenaInsertError),
    /// The application-owned association table could not reserve every split entry.
    AssociationCapacityExceeded,
    /// Connectivity or pair/triad topology could not produce a complete candidate.
    InvalidTopology,
    /// A join requires two distinct live group identities.
    SameGroup,
    /// Explicit empty-shell destruction was requested for a group with members.
    GroupNotEmpty,
}

impl fmt::Display for ParticleGroupMutationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorldLocked => formatter.write_str("world is locked by an active step"),
            Self::InvalidHandle(error) => {
                write!(formatter, "invalid particle-group handle: {error}")
            }
            Self::Arena(error) => write!(formatter, "could not store particle group: {error}"),
            Self::AssociationCapacityExceeded => {
                formatter.write_str("particle-group association capacity is exhausted")
            }
            Self::InvalidTopology => formatter.write_str("particle-group topology is invalid"),
            Self::SameGroup => formatter.write_str("cannot join a particle group to itself"),
            Self::GroupNotEmpty => formatter.write_str("particle-group shell still owns particles"),
        }
    }
}

impl Error for ParticleGroupMutationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidHandle(error) => Some(error),
            Self::Arena(error) => Some(error),
            Self::WorldLocked
            | Self::AssociationCapacityExceeded
            | Self::InvalidTopology
            | Self::SameGroup
            | Self::GroupNotEmpty => None,
        }
    }
}

impl From<HandleError> for ParticleGroupMutationError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

impl From<ArenaInsertError> for ParticleGroupMutationError {
    fn from(error: ArenaInsertError) -> Self {
        Self::Arena(error)
    }
}

pub(super) fn plan_join(
    world: &mut World,
    group_a: ParticleGroupId,
    group_b: ParticleGroupId,
) -> Result<MutationReport<ParticleGroupId>, ParticleGroupMutationError> {
    validate_mutation_state(world)?;
    if group_a == group_b {
        return Err(ParticleGroupMutationError::SameGroup);
    }
    let shell_a = world.particle_groups.get(group_a)?;
    let shell_b = world.particle_groups.get(group_b)?;
    if shell_a.system != shell_b.system {
        return Err(ParticleGroupMutationError::InvalidHandle(
            HandleError::WrongParticleSystem,
        ));
    }
    let system = shell_a.system;
    let source = world.particle_systems.get(system)?;
    let source_view = source
        .storage
        .group_view(group_b, 0.0)
        .map_err(storage_error)?;
    let mut particles = Vec::new();
    particles
        .try_reserve_exact(source_view.member_count())
        .map_err(|_error| allocation_error(source_view.member_count()))?;
    particles.extend_from_slice(source_view.member_ids());
    let invalidated = DestructionRecord {
        destroyed: DestroyedId::ParticleGroup(group_b),
        diagnostic_id: shell_b.diagnostic_id,
        cause: DestructionCause::Explicit,
        snapshot: ObjectSnapshot::ParticleGroup { system, particles },
    };
    let mut lifecycle = Vec::new();
    lifecycle
        .try_reserve_exact(1)
        .map_err(|_error| allocation_error(1))?;
    lifecycle.push(LifecycleEvent::Destruction(invalidated));

    let mut system_candidate = world.particle_systems.get(system)?.clone();
    let diameter = 2.0 * system_candidate.definition.radius();
    let join = system_candidate
        .storage
        .plan_join_groups(group_a, group_b, diameter, group_topology_limits())
        .map_err(group_plan_error)?;
    join.commit_group(&mut system_candidate.storage);
    system_candidate
        .groups
        .retain(|candidate| *candidate != group_b);

    let mut shell_candidate = world.particle_groups.clone();
    shell_candidate
        .remove(group_b)
        .expect("validated group shell remains live in the cloned arena");

    world.particle_groups = shell_candidate;
    *world
        .particle_systems
        .get_mut(system)
        .expect("validated system remains live until immediate commit") = system_candidate;
    Ok(MutationReport::new(group_a, lifecycle))
}

pub(super) fn plan_split(
    world: &mut World,
    group: ParticleGroupId,
) -> Result<Vec<ParticleGroupId>, ParticleGroupMutationError> {
    validate_mutation_state(world)?;
    let shell = world.particle_groups.get(group)?;
    let system = shell.system;
    let source = world.particle_systems.get(system)?;
    let component_count = source
        .storage
        .split_group_count(group)
        .map_err(split_error)?;
    let new_group_count = component_count.saturating_sub(1);
    if new_group_count == 0 {
        let split = source.storage.plan_split(group, &[]).map_err(split_error)?;
        return Ok(split.result_groups().to_vec());
    }

    let (first_diagnostic_id, next_diagnostic_id) =
        world.preflight_diagnostic_ids(new_group_count)?;
    let mut shell_candidate = world.particle_groups.clone();
    let mut new_groups = Vec::new();
    new_groups
        .try_reserve_exact(new_group_count)
        .map_err(|_error| allocation_error(new_group_count))?;
    for ordinal in 0..new_group_count {
        let diagnostic_id = first_diagnostic_id
            .checked_add(
                u64::try_from(ordinal).map_err(|_error| ArenaInsertError::DiagnosticIdExhausted)?,
            )
            .ok_or(ArenaInsertError::DiagnosticIdExhausted)?;
        let new_group = shell_candidate.insert(ParticleGroup {
            diagnostic_id,
            system,
        })?;
        new_groups.push(new_group);
    }

    let mut system_candidate = source.clone();
    system_candidate
        .groups
        .try_reserve_exact(new_group_count)
        .map_err(|_error| allocation_error(new_group_count))?;
    let split = system_candidate
        .storage
        .plan_split(group, &new_groups)
        .map_err(split_error)?;
    let result_groups = split.result_groups().to_vec();
    split.commit(&mut system_candidate.storage);
    system_candidate.groups.extend_from_slice(&new_groups);

    world.particle_groups = shell_candidate;
    *world
        .particle_systems
        .get_mut(system)
        .expect("validated system remains live until immediate commit") = system_candidate;
    world.commit_next_diagnostic_id(next_diagnostic_id);
    Ok(result_groups)
}

pub(super) fn plan_split_with_association<UserAssociation: Clone>(
    world: &mut World,
    group: ParticleGroupId,
    associations: &mut AssociationMap<ParticleGroupId, UserAssociation>,
) -> Result<Vec<ParticleGroupId>, ParticleGroupMutationError> {
    validate_mutation_state(world)?;
    let shell = world.particle_groups.get(group)?;
    let source = world.particle_systems.get(shell.system)?;
    let component_count = source
        .storage
        .split_group_count(group)
        .map_err(split_error)?;
    let new_group_count = component_count.saturating_sub(1);
    let mut prepared_associations = Vec::new();
    if let Some(source_association) = associations.get(&group) {
        prepared_associations
            .try_reserve_exact(new_group_count)
            .map_err(|_error| allocation_error(new_group_count))?;
        for _ in 0..new_group_count {
            prepared_associations.push(source_association.clone());
        }
        associations
            .try_reserve(new_group_count)
            .map_err(|()| ParticleGroupMutationError::AssociationCapacityExceeded)?;
    }

    let groups = plan_split(world, group)?;
    for (new_group, association) in groups.iter().copied().skip(1).zip(prepared_associations) {
        let replaced = associations.insert(new_group, association);
        debug_assert!(replaced.is_none());
    }
    Ok(groups)
}

pub(super) fn set_flags(
    world: &mut World,
    group: ParticleGroupId,
    flags: ParticleGroupFlags,
) -> Result<(), ParticleGroupMutationError> {
    validate_mutation_state(world)?;
    let shell = world.particle_groups.get(group)?;
    let system = shell.system;
    let mut candidate = world.particle_systems.get(system)?.clone();
    candidate
        .storage
        .set_group_flags_internal(group, flags)
        .map_err(storage_error)?;
    *world
        .particle_systems
        .get_mut(system)
        .expect("validated system remains live until immediate commit") = candidate;
    Ok(())
}

pub(super) fn destroy_empty(
    world: &mut World,
    group: ParticleGroupId,
) -> Result<DestructionRecord, ParticleGroupMutationError> {
    validate_mutation_state(world)?;
    let shell = world.particle_groups.get(group)?;
    let system = shell.system;
    let source = world.particle_systems.get(system)?;
    if source
        .storage
        .group_view(group, 0.0)
        .map_err(storage_error)?
        .member_count()
        != 0
    {
        return Err(ParticleGroupMutationError::GroupNotEmpty);
    }

    let mut system_candidate = source.clone();
    let particles = system_candidate
        .storage
        .clear_group(group)
        .map_err(storage_error)?;
    debug_assert!(particles.is_empty());
    system_candidate
        .groups
        .retain(|candidate| *candidate != group);
    let mut shell_candidate = world.particle_groups.clone();
    let removed = shell_candidate
        .remove(group)
        .expect("validated empty group remains live in the cloned arena");
    let record = DestructionRecord {
        destroyed: crate::DestroyedId::ParticleGroup(group),
        diagnostic_id: removed.diagnostic_id,
        cause: DestructionCause::Explicit,
        snapshot: crate::ObjectSnapshot::ParticleGroup { system, particles },
    };

    world.particle_groups = shell_candidate;
    *world
        .particle_systems
        .get_mut(system)
        .expect("validated system remains live until immediate commit") = system_candidate;
    Ok(record)
}

fn validate_mutation_state(world: &World) -> Result<(), ParticleGroupMutationError> {
    world.ensure_not_poisoned_for_handle()?;
    if world.step_state.is_locked() {
        return Err(ParticleGroupMutationError::WorldLocked);
    }
    Ok(())
}

fn group_plan_error(error: GroupPlanError) -> ParticleGroupMutationError {
    match error {
        GroupPlanError::Storage(error) => storage_error(error),
        GroupPlanError::Topology => ParticleGroupMutationError::InvalidTopology,
    }
}

fn split_error(error: SplitPlanError) -> ParticleGroupMutationError {
    match error {
        SplitPlanError::Storage(error) => storage_error(error),
        SplitPlanError::Connectivity(_) | SplitPlanError::GroupIdentityCount { .. } => {
            ParticleGroupMutationError::InvalidTopology
        }
    }
}

fn storage_error(error: ParticleStorageError) -> ParticleGroupMutationError {
    match error {
        ParticleStorageError::WrongWorld => {
            ParticleGroupMutationError::InvalidHandle(HandleError::WrongWorld)
        }
        ParticleStorageError::WrongParticleSystem => {
            ParticleGroupMutationError::InvalidHandle(HandleError::WrongParticleSystem)
        }
        ParticleStorageError::StaleOrDestroyed => {
            ParticleGroupMutationError::InvalidHandle(HandleError::StaleOrDestroyed)
        }
        ParticleStorageError::PendingDelete => {
            ParticleGroupMutationError::InvalidHandle(HandleError::PendingDelete)
        }
        ParticleStorageError::CapacityExceeded { limit } => {
            ParticleGroupMutationError::Arena(ArenaInsertError::CapacityExceeded { limit })
        }
        ParticleStorageError::IdentityExhausted => {
            ParticleGroupMutationError::Arena(ArenaInsertError::GenerationExhausted)
        }
        ParticleStorageError::InvalidPermutation
        | ParticleStorageError::LaneLengthMismatch
        | ParticleStorageError::InvalidDerivedReference
        | ParticleStorageError::InvalidGroupRange
        | ParticleStorageError::InvalidLaneBundle => ParticleGroupMutationError::InvalidTopology,
    }
}

fn allocation_error(limit: usize) -> ParticleGroupMutationError {
    ParticleGroupMutationError::Arena(ArenaInsertError::CapacityExceeded { limit })
}
