use crate::identity::{HandleIdentity, ParticleGroupId};
use crate::math::Transform;
use crate::particle::ParticleGroupFlags;
use crate::particle::storage::group::{GroupRecord, GroupStatisticsCache, InternalGroupFlags};
use crate::particle::topology::connectivity::{
    ConnectivityError, SplitConnectivityPlan, plan_split_connectivity,
};

use super::super::permutation::{TopologyRemapPolicy, prepare_group_reassignment_permutation};
use super::super::{ParticleStorage, ParticleStorageError};
use super::{
    MutationCandidate, MutationInvalidations, MutationPayload, SplitGroupCandidate,
    destruction_effects, require_no_removals,
};

type SplitPermutation = (Vec<Option<usize>>, Vec<Option<ParticleGroupId>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SplitPlanError {
    Storage(ParticleStorageError),
    Connectivity(ConnectivityError),
    GroupIdentityCount { required: usize, provided: usize },
}

impl From<ParticleStorageError> for SplitPlanError {
    fn from(error: ParticleStorageError) -> Self {
        Self::Storage(error)
    }
}

impl From<ConnectivityError> for SplitPlanError {
    fn from(error: ConnectivityError) -> Self {
        Self::Connectivity(error)
    }
}

pub(crate) struct SplitPlan {
    candidate: ParticleStorage,
    result_groups: Vec<ParticleGroupId>,
}

impl SplitPlan {
    pub(crate) fn result_groups(&self) -> &[ParticleGroupId] {
        &self.result_groups
    }

    pub(crate) fn commit(self, storage: &mut ParticleStorage) {
        *storage = self.candidate;
    }
}

impl ParticleStorage {
    pub(crate) fn split_group_count(
        &self,
        group: ParticleGroupId,
    ) -> Result<usize, SplitPlanError> {
        let record = live_group_record(self, group)?;
        self.check_invariants()?;
        let maybe_connectivity =
            plan_split_connectivity(record.range(), &self.particle_contacts, &self.flags)?;
        Ok(maybe_connectivity.map_or(1, |plan| plan.component_count()))
    }

    pub(crate) fn plan_split(
        &self,
        group: ParticleGroupId,
        new_groups: &[ParticleGroupId],
    ) -> Result<SplitPlan, SplitPlanError> {
        let source_record = live_group_record(self, group)?;
        self.check_invariants()?;
        let maybe_connectivity =
            plan_split_connectivity(source_record.range(), &self.particle_contacts, &self.flags)?;
        let required = maybe_connectivity
            .as_ref()
            .map_or(0, |plan| plan.later_components().len());
        if new_groups.len() != required {
            return Err(SplitPlanError::GroupIdentityCount {
                required,
                provided: new_groups.len(),
            });
        }
        validate_new_group_ids(self, group, new_groups)?;

        let mut candidate = self.clone();
        if let Some(connectivity) = maybe_connectivity {
            apply_split_candidate(&mut candidate, source_record, &connectivity, new_groups)?;
        }
        candidate.check_invariants()?;
        let mut result_groups = Vec::new();
        result_groups
            .try_reserve_exact(1 + new_groups.len())
            .map_err(|_error| ParticleStorageError::InvalidLaneBundle)?;
        result_groups.push(group);
        result_groups.extend_from_slice(new_groups);
        Ok(SplitPlan {
            candidate,
            result_groups,
        })
    }
}

impl MutationCandidate {
    fn prepare_split_group_reassignment(
        storage: &ParticleStorage,
        old_to_new: &[Option<usize>],
        groups_by_old_row: &[Option<ParticleGroupId>],
        group_records: &[GroupRecord],
    ) -> Result<Self, ParticleStorageError> {
        require_no_removals(old_to_new)?;
        let topology_policy = TopologyRemapPolicy::PreserveHistoricalOrder;
        let topology_mode = topology_policy.mode();
        let permutation = prepare_group_reassignment_permutation(
            storage,
            old_to_new,
            groups_by_old_row,
            group_records,
            topology_policy,
        )?;
        let lifecycle_effects = destruction_effects(&permutation)?;
        let payload = MutationPayload {
            permutation,
            topology_mode,
            invalidations: MutationInvalidations::AFFECTED_GROUPS,
            lifecycle_effects,
        };
        Ok(Self::SplitGroup(SplitGroupCandidate { payload }))
    }
}

fn live_group_record(
    storage: &ParticleStorage,
    group: ParticleGroupId,
) -> Result<GroupRecord, ParticleStorageError> {
    if group.identity().world() != storage.world {
        return Err(ParticleStorageError::WrongWorld);
    }
    let Some(record) = storage
        .group_records
        .iter()
        .copied()
        .find(|record| record.id == group)
    else {
        return Err(ParticleStorageError::StaleOrDestroyed);
    };
    if record.system != storage.system {
        return Err(ParticleStorageError::WrongParticleSystem);
    }
    if record
        .internal_flags
        .contains(InternalGroupFlags::WILL_BE_DESTROYED)
    {
        return Err(ParticleStorageError::StaleOrDestroyed);
    }
    Ok(record)
}

fn validate_new_group_ids(
    storage: &ParticleStorage,
    source: ParticleGroupId,
    new_groups: &[ParticleGroupId],
) -> Result<(), ParticleStorageError> {
    for (ordinal, group) in new_groups.iter().copied().enumerate() {
        if group == source
            || group.identity().world() != storage.world
            || storage
                .group_records
                .iter()
                .any(|record| record.id == group)
            || new_groups[..ordinal].contains(&group)
        {
            return Err(ParticleStorageError::InvalidGroupRange);
        }
    }
    Ok(())
}

fn apply_split_candidate(
    storage: &mut ParticleStorage,
    source_record: GroupRecord,
    connectivity: &SplitConnectivityPlan,
    new_groups: &[ParticleGroupId],
) -> Result<(), ParticleStorageError> {
    let (old_to_new, groups_by_old_row) = split_mapping(storage, connectivity, new_groups)?;
    let mut group_records = storage.group_records.clone();
    group_records
        .try_reserve_exact(new_groups.len())
        .map_err(|_error| ParticleStorageError::InvalidLaneBundle)?;
    group_records.extend(
        new_groups
            .iter()
            .copied()
            .map(|group| split_created_record(source_record, group)),
    );
    MutationCandidate::prepare_split_group_reassignment(
        storage,
        &old_to_new,
        &groups_by_old_row,
        &group_records,
    )?
    .commit(storage);
    Ok(())
}

fn split_mapping(
    storage: &ParticleStorage,
    connectivity: &SplitConnectivityPlan,
    new_groups: &[ParticleGroupId],
) -> Result<SplitPermutation, ParticleStorageError> {
    if connectivity.source_range().end > storage.len()
        || connectivity.later_components().len() != new_groups.len()
    {
        return Err(ParticleStorageError::InvalidPermutation);
    }
    let mut moved = vec![false; storage.len()];
    let mut groups_by_old_row = storage.groups.clone();
    let mut moved_members = Vec::new();
    moved_members
        .try_reserve_exact(connectivity.moved_members().count())
        .map_err(|_error| ParticleStorageError::InvalidLaneBundle)?;
    for (component, group) in connectivity
        .later_components()
        .iter()
        .zip(new_groups.iter().copied())
    {
        for old in component.iter().copied() {
            let Some(was_moved) = moved.get_mut(old) else {
                return Err(ParticleStorageError::InvalidPermutation);
            };
            if *was_moved || !connectivity.source_range().contains(&old) {
                return Err(ParticleStorageError::InvalidPermutation);
            }
            *was_moved = true;
            groups_by_old_row[old] = Some(group);
            moved_members.push(old);
        }
    }
    if connectivity
        .surviving_members()
        .iter()
        .any(|old| moved.get(*old).copied() != Some(false))
    {
        return Err(ParticleStorageError::InvalidPermutation);
    }
    let mut source_order = Vec::new();
    source_order
        .try_reserve_exact(storage.len())
        .map_err(|_error| ParticleStorageError::InvalidLaneBundle)?;
    source_order.extend((0..storage.len()).filter(|old| !moved[*old]));
    source_order.extend(moved_members);
    if source_order.len() != storage.len() {
        return Err(ParticleStorageError::InvalidPermutation);
    }
    let mut old_to_new = vec![None; storage.len()];
    for (new, old) in source_order.into_iter().enumerate() {
        old_to_new[old] = Some(new);
    }
    Ok((old_to_new, groups_by_old_row))
}

fn split_created_record(source: GroupRecord, id: ParticleGroupId) -> GroupRecord {
    let mut record = GroupRecord::new(id, source.system, 0..0);
    record.flags = source.flags;
    if record.flags.contains(ParticleGroupFlags::SOLID) {
        record
            .internal_flags
            .insert(InternalGroupFlags::NEEDS_UPDATE_DEPTH);
    }
    record.strength = 1.0;
    record.transform = Transform::IDENTITY;
    record.maybe_user_association = source.maybe_user_association;
    record.statistics = GroupStatisticsCache::INVALIDATED_ZERO;
    record
}

#[cfg(test)]
mod tests;
