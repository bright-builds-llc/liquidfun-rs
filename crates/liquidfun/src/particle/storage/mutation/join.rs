use crate::identity::{HandleIdentity, ParticleGroupId};
use crate::particle::storage::group::{GroupRecord, InternalGroupFlags};
use crate::particle::storage::validation::rebuild_group_records_for_system;
use crate::particle::topology::VoronoiLimits;
use crate::particle::topology::constraints::{
    ConnectionFilter, ConstraintError, TopologyGroup, TopologyInput, generate_pairs_and_triads,
};

use super::super::{ParticleIndex, ParticleStorage, ParticleStorageError};
use super::MutationCandidate;

#[derive(Debug, Clone, Copy)]
pub(super) struct JoinTopologyParameters {
    particle_diameter: f32,
    voronoi_limits: VoronoiLimits,
}

impl JoinTopologyParameters {
    pub(super) const fn new(particle_diameter: f32, voronoi_limits: VoronoiLimits) -> Self {
        Self {
            particle_diameter,
            voronoi_limits,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum JoinPlanError {
    Storage(ParticleStorageError),
    Constraints(ConstraintError),
}

impl From<ParticleStorageError> for JoinPlanError {
    fn from(error: ParticleStorageError) -> Self {
        Self::Storage(error)
    }
}

impl From<ConstraintError> for JoinPlanError {
    fn from(error: ConstraintError) -> Self {
        Self::Constraints(error)
    }
}

pub(super) struct JoinPlan {
    candidate: ParticleStorage,
}

impl JoinPlan {
    pub(super) fn commit(self, storage: &mut ParticleStorage) {
        *storage = self.candidate;
    }
}

impl ParticleStorage {
    pub(super) fn plan_join(
        &self,
        group_a: ParticleGroupId,
        group_b: ParticleGroupId,
        topology: JoinTopologyParameters,
    ) -> Result<JoinPlan, JoinPlanError> {
        let (record_a, record_b) = validate_join_handles(self, group_a, group_b)?;
        self.check_invariants()?;
        validate_connection_bounds(self)?;

        let old_to_new = join_mapping(self, record_a, record_b)?;
        let mut candidate = self.clone();
        MutationCandidate::prepare_exact_join_groups(
            &candidate,
            &old_to_new,
            Vec::new(),
            Vec::new(),
        )?
        .commit(&mut candidate);

        let (range, threshold) = joined_range_and_threshold(&candidate, group_a, group_b)?;
        let generated = generate_cross_constraints(&candidate, range, threshold, topology)?;
        validate_generated_bounds(&candidate, &generated.pairs, &generated.triads)?;
        let identity = (0..candidate.len()).map(Some).collect::<Vec<_>>();
        MutationCandidate::prepare_exact_join_groups(
            &candidate,
            &identity,
            generated.pairs,
            generated.triads,
        )?
        .commit(&mut candidate);

        finish_join(&mut candidate, group_a, group_b)?;
        candidate.check_invariants()?;
        Ok(JoinPlan { candidate })
    }
}

fn validate_connection_bounds(storage: &ParticleStorage) -> Result<(), ParticleStorageError> {
    let pair_limit = combination_limit(storage.len(), 2)?;
    let triad_limit = combination_limit(storage.len(), 3)?;
    if storage.particle_contacts.len() > pair_limit || storage.pairs.len() > pair_limit {
        return Err(ParticleStorageError::CapacityExceeded { limit: pair_limit });
    }
    if storage.triads.len() > triad_limit {
        return Err(ParticleStorageError::CapacityExceeded { limit: triad_limit });
    }
    Ok(())
}

fn validate_generated_bounds(
    storage: &ParticleStorage,
    generated_pairs: &[crate::particle::storage::lanes::ParticlePair],
    generated_triads: &[crate::particle::storage::lanes::ParticleTriad],
) -> Result<(), ParticleStorageError> {
    let pair_limit = combination_limit(storage.len(), 2)?;
    let triad_limit = combination_limit(storage.len(), 3)?;
    let appended_pairs = generated_pairs
        .iter()
        .filter(|pair| {
            storage
                .pairs
                .iter()
                .all(|historical| historical.indices != pair.indices)
        })
        .count();
    let appended_triads = generated_triads
        .iter()
        .filter(|triad| {
            storage
                .triads
                .iter()
                .all(|historical| historical.indices != triad.indices)
        })
        .count();
    let pair_count = storage
        .pairs
        .len()
        .checked_add(appended_pairs)
        .ok_or(ParticleStorageError::InvalidLaneBundle)?;
    let triad_count = storage
        .triads
        .len()
        .checked_add(appended_triads)
        .ok_or(ParticleStorageError::InvalidLaneBundle)?;
    if pair_count > pair_limit {
        return Err(ParticleStorageError::CapacityExceeded { limit: pair_limit });
    }
    if triad_count > triad_limit {
        return Err(ParticleStorageError::CapacityExceeded { limit: triad_limit });
    }
    Ok(())
}

fn combination_limit(count: usize, width: usize) -> Result<usize, ParticleStorageError> {
    if count < width {
        return Ok(0);
    }
    match width {
        2 => count
            .checked_mul(count - 1)
            .map(|value| value / 2)
            .ok_or(ParticleStorageError::InvalidLaneBundle),
        3 => count
            .checked_mul(count - 1)
            .and_then(|value| value.checked_mul(count - 2))
            .map(|value| value / 6)
            .ok_or(ParticleStorageError::InvalidLaneBundle),
        _ => Err(ParticleStorageError::InvalidLaneBundle),
    }
}

fn validate_join_handles(
    storage: &ParticleStorage,
    group_a: ParticleGroupId,
    group_b: ParticleGroupId,
) -> Result<(GroupRecord, GroupRecord), ParticleStorageError> {
    if group_a == group_b {
        return Err(ParticleStorageError::InvalidGroupRange);
    }
    let record_a = live_group_record(storage, group_a)?;
    let record_b = live_group_record(storage, group_b)?;
    Ok((record_a, record_b))
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

fn join_mapping(
    storage: &ParticleStorage,
    record_a: GroupRecord,
    record_b: GroupRecord,
) -> Result<Vec<Option<usize>>, ParticleStorageError> {
    let mut source_order = (0..storage.len()).collect::<Vec<_>>();
    move_range_to_end(&mut source_order, record_b.range())?;
    let maybe_a_range = member_range(&source_order, record_a.range());
    let b_start = storage.len() - record_b.range().len();
    if let Some(a_range) = maybe_a_range {
        rotate_range_to(&mut source_order, a_range, b_start)?;
    }
    let mut old_to_new = vec![None; source_order.len()];
    for (new, old) in source_order.into_iter().enumerate() {
        old_to_new[old] = Some(new);
    }
    Ok(old_to_new)
}

fn move_range_to_end(
    order: &mut [usize],
    range: std::ops::Range<usize>,
) -> Result<(), ParticleStorageError> {
    if range.start > range.end || range.end > order.len() {
        return Err(ParticleStorageError::InvalidGroupRange);
    }
    if range.is_empty() {
        return Ok(());
    }
    order[range.start..].rotate_left(range.len());
    Ok(())
}

fn member_range(
    order: &[usize],
    original_range: std::ops::Range<usize>,
) -> Option<std::ops::Range<usize>> {
    if original_range.is_empty() {
        return None;
    }
    let start = order.iter().position(|old| original_range.contains(old))?;
    Some(start..start + original_range.len())
}

fn rotate_range_to(
    order: &mut [usize],
    source: std::ops::Range<usize>,
    destination_end: usize,
) -> Result<(), ParticleStorageError> {
    if source.end > destination_end || destination_end > order.len() {
        return Err(ParticleStorageError::InvalidPermutation);
    }
    order[source.start..destination_end].rotate_left(source.len());
    Ok(())
}

fn joined_range_and_threshold(
    storage: &ParticleStorage,
    group_a: ParticleGroupId,
    group_b: ParticleGroupId,
) -> Result<(std::ops::Range<usize>, usize), ParticleStorageError> {
    let a = live_group_record(storage, group_a)?;
    let b = live_group_record(storage, group_b)?;
    let threshold = if b.range().is_empty() {
        a.last
    } else {
        b.first
    };
    let start = if a.range().is_empty() {
        b.first
    } else {
        a.first
    };
    let end = if b.range().is_empty() { a.last } else { b.last };
    Ok((start..end, threshold))
}

struct CrossThresholdFilter {
    threshold: usize,
}

impl ConnectionFilter for CrossThresholdFilter {
    fn is_necessary(&self, _index: ParticleIndex) -> bool {
        true
    }

    fn should_create_pair(&self, [a, b]: [ParticleIndex; 2]) -> bool {
        (a.0 < self.threshold) != (b.0 < self.threshold)
    }

    fn should_create_triad(&self, indices: [ParticleIndex; 3]) -> bool {
        indices.iter().any(|index| index.0 < self.threshold)
            && indices.iter().any(|index| index.0 >= self.threshold)
    }
}

fn generate_cross_constraints(
    storage: &ParticleStorage,
    range: std::ops::Range<usize>,
    threshold: usize,
    topology: JoinTopologyParameters,
) -> Result<crate::particle::topology::constraints::GeneratedConstraints, ConstraintError> {
    let groups = storage
        .groups
        .iter()
        .copied()
        .map(|maybe_group| {
            maybe_group.and_then(|group| {
                storage
                    .group_records
                    .iter()
                    .copied()
                    .find(|record| record.id == group)
                    .map(TopologyGroup::from_record)
            })
        })
        .collect::<Vec<_>>();
    generate_pairs_and_triads(
        &TopologyInput {
            owner: storage.system,
            positions: &storage.positions,
            flags: &storage.flags,
            groups: &groups,
            contacts: &storage.particle_contacts,
            range,
            particle_diameter: topology.particle_diameter,
            voronoi_limits: topology.voronoi_limits,
        },
        &CrossThresholdFilter { threshold },
    )
}

fn finish_join(
    storage: &mut ParticleStorage,
    group_a: ParticleGroupId,
    group_b: ParticleGroupId,
) -> Result<(), ParticleStorageError> {
    let record_a = live_group_record(storage, group_a)?;
    let record_b = live_group_record(storage, group_b)?;
    let mut merged_a = record_a;
    merged_a.flags |= record_b.flags;
    merged_a.internal_flags.insert(record_b.internal_flags);
    if !record_a
        .flags
        .contains(crate::particle::ParticleGroupFlags::SOLID)
        && merged_a
            .flags
            .contains(crate::particle::ParticleGroupFlags::SOLID)
    {
        merged_a
            .internal_flags
            .insert(InternalGroupFlags::NEEDS_UPDATE_DEPTH);
    }
    merged_a.set_range(merged_range(record_a, record_b));
    merged_a.invalidate_statistics();

    let mut groups = storage.groups.clone();
    for maybe_group in &mut groups {
        if *maybe_group == Some(group_b) {
            *maybe_group = Some(group_a);
        }
    }
    let previous_records = storage
        .group_records
        .iter()
        .copied()
        .filter(|record| record.id != group_b)
        .map(|record| {
            if record.id == group_a {
                merged_a
            } else {
                record
            }
        })
        .collect::<Vec<_>>();
    let joined_records =
        rebuild_group_records_for_system(&previous_records, &groups, storage.system)?;
    let mut solver_state = storage.solver_state.clone();
    solver_state.refresh_group_flags(&joined_records);

    storage.groups = groups;
    storage.solver_state = solver_state;
    storage.group_records = joined_records;
    Ok(())
}

fn merged_range(a: GroupRecord, b: GroupRecord) -> std::ops::Range<usize> {
    match (a.range().is_empty(), b.range().is_empty()) {
        (true, true) => 0..0,
        (true, false) => b.range(),
        (false, true) => a.range(),
        (false, false) => a.first..b.last,
    }
}

#[cfg(test)]
mod tests;
