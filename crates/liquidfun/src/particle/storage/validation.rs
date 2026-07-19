use std::collections::{HashMap, HashSet};

use super::{GroupRecord, ParticleGroupId, ParticleIndex, ParticleStorageError, ParticleSystemId};

pub(super) fn validate_references(
    references: &[ParticleIndex],
    count: usize,
) -> Result<(), ParticleStorageError> {
    if references.iter().any(|index| index.0 >= count) {
        return Err(ParticleStorageError::InvalidDerivedReference);
    }
    Ok(())
}

pub(super) fn validate_reference_sets<const N: usize>(
    references: &[[ParticleIndex; N]],
    count: usize,
) -> Result<(), ParticleStorageError> {
    if references.iter().flatten().any(|index| index.0 >= count) {
        return Err(ParticleStorageError::InvalidDerivedReference);
    }
    Ok(())
}

pub(super) fn rebuild_group_records_for_system(
    existing: &[GroupRecord],
    groups: &[Option<ParticleGroupId>],
    system: ParticleSystemId,
) -> Result<Vec<GroupRecord>, ParticleStorageError> {
    let ranges = membership_ranges(groups)?;
    let mut records_by_id = HashMap::with_capacity(existing.len());
    for record in existing {
        if records_by_id.insert(record.id, *record).is_some() {
            return Err(ParticleStorageError::InvalidGroupRange);
        }
    }
    let mut records = Vec::with_capacity(existing.len().max(ranges.len()));
    for (group, range) in &ranges {
        let mut record = records_by_id
            .remove(group)
            .unwrap_or_else(|| GroupRecord::new(*group, system, range.clone()));
        record.set_range(range.clone());
        records.push(record);
    }
    for existing_record in existing {
        let Some(mut record) = records_by_id.remove(&existing_record.id) else {
            continue;
        };
        record.retain_empty_after_member_removal();
        records.push(record);
    }
    validate_groups(system, groups, &records)?;
    Ok(records)
}

pub(super) fn validate_groups(
    system: ParticleSystemId,
    groups: &[Option<ParticleGroupId>],
    records: &[GroupRecord],
) -> Result<(), ParticleStorageError> {
    if records.len() > i32::MAX as usize || groups.len() > i32::MAX as usize {
        return Err(ParticleStorageError::InvalidGroupRange);
    }
    let mut seen = HashSet::with_capacity(records.len());
    let mut saw_empty = false;
    for record in records {
        record.validate(system, groups.len())?;
        if !seen.insert(record.id) || (saw_empty && record.first != record.last) {
            return Err(ParticleStorageError::InvalidGroupRange);
        }
        if record.first == record.last {
            saw_empty = true;
        }
    }
    let ranges = membership_ranges(groups)?;
    let nonempty_records = records
        .iter()
        .take_while(|record| record.first != record.last)
        .collect::<Vec<_>>();
    if nonempty_records.len() != ranges.len() {
        return Err(ParticleStorageError::InvalidGroupRange);
    }
    for (record, (group, range)) in nonempty_records.into_iter().zip(ranges) {
        if record.id != group || record.range() != range {
            return Err(ParticleStorageError::InvalidGroupRange);
        }
    }
    Ok(())
}

fn membership_ranges(
    groups: &[Option<ParticleGroupId>],
) -> Result<Vec<(ParticleGroupId, std::ops::Range<usize>)>, ParticleStorageError> {
    let mut ranges: Vec<(ParticleGroupId, std::ops::Range<usize>)> = Vec::new();
    let mut seen_groups = HashSet::new();
    for (dense, maybe_group) in groups.iter().copied().enumerate() {
        let Some(group) = maybe_group else {
            continue;
        };
        if let Some((last_group, last_range)) = ranges.last_mut()
            && *last_group == group
            && last_range.end == dense
        {
            last_range.end = dense + 1;
            continue;
        }
        if !seen_groups.insert(group) {
            return Err(ParticleStorageError::InvalidGroupRange);
        }
        ranges.push((group, dense..dense + 1));
    }
    Ok(ranges)
}

#[cfg(test)]
mod tests {
    use crate::identity::{HandleIdentity, Identity, WorldKey};
    use crate::particle::ParticleGroupFlags;

    use super::*;
    use crate::particle::storage::group::InternalGroupFlags;

    fn identities() -> (
        ParticleSystemId,
        ParticleSystemId,
        ParticleGroupId,
        ParticleGroupId,
    ) {
        let world = WorldKey::fresh().expect("test world key remains available");
        (
            ParticleSystemId::from_identity(Identity::new(world, 0, 0)),
            ParticleSystemId::from_identity(Identity::new(world, 1, 0)),
            ParticleGroupId::from_identity(Identity::new(world, 2, 0)),
            ParticleGroupId::from_identity(Identity::new(world, 3, 0)),
        )
    }

    #[test]
    fn contiguous_source_membership_validates_against_complete_records() {
        // Arrange
        let (system, _other_system, first, second) = identities();
        let groups = [Some(first), Some(first), None, Some(second)];
        let records = [
            GroupRecord::new(first, system, 0..2),
            GroupRecord::new(second, system, 3..4),
        ];

        // Act
        let result = validate_groups(system, &groups, &records);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn wrong_system_overlap_gap_and_membership_disagreement_are_rejected() {
        // Arrange
        let (system, other_system, first, second) = identities();
        let groups = [Some(first), Some(first), Some(second), Some(second)];
        let wrong_system = [
            GroupRecord::new(first, other_system, 0..2),
            GroupRecord::new(second, system, 2..4),
        ];
        let overlap = [
            GroupRecord::new(first, system, 0..3),
            GroupRecord::new(second, system, 2..4),
        ];
        let gap = [
            GroupRecord::new(first, system, 0..2),
            GroupRecord::new(second, system, 3..4),
        ];

        // Act
        let wrong_system_result = validate_groups(system, &groups, &wrong_system);
        let overlap_result = validate_groups(system, &groups, &overlap);
        let gap_result = validate_groups(system, &groups, &gap);

        // Assert
        assert_eq!(
            wrong_system_result,
            Err(ParticleStorageError::InvalidGroupRange)
        );
        assert_eq!(overlap_result, Err(ParticleStorageError::InvalidGroupRange));
        assert_eq!(gap_result, Err(ParticleStorageError::InvalidGroupRange));
    }

    #[test]
    fn retained_or_deferred_empty_records_require_exact_zero_statistics() {
        // Arrange
        let (system, _other_system, first, second) = identities();
        let groups = [];
        let mut retained = GroupRecord::new(first, system, 0..0);
        retained.flags = ParticleGroupFlags::CAN_BE_EMPTY;
        let mut deferred = GroupRecord::new(second, system, 0..0);
        deferred
            .internal_flags
            .insert(InternalGroupFlags::WILL_BE_DESTROYED);
        let bare = GroupRecord::new(first, system, 0..0);
        let mut nonzero = retained;
        nonzero.statistics.mass = 1.0;

        // Act
        let valid_result = validate_groups(system, &groups, &[retained, deferred]);
        let bare_result = validate_groups(system, &groups, &[bare]);
        let nonzero_result = validate_groups(system, &groups, &[nonzero]);

        // Assert
        assert_eq!(valid_result, Ok(()));
        assert_eq!(bare_result, Err(ParticleStorageError::InvalidGroupRange));
        assert_eq!(nonzero_result, Err(ParticleStorageError::InvalidGroupRange));
    }

    #[test]
    fn rebuilt_records_expose_flags_in_stable_source_order() {
        // Arrange
        let (system, _other_system, first, second) = identities();
        let mut first_record = GroupRecord::new(first, system, 0..1);
        first_record.flags = ParticleGroupFlags::RIGID;
        let mut second_record = GroupRecord::new(second, system, 1..2);
        second_record.flags = ParticleGroupFlags::SOLID;
        let groups = [Some(second), Some(second), Some(first)];

        // Act
        let rebuilt =
            rebuild_group_records_for_system(&[first_record, second_record], &groups, system)
                .expect("source-contiguous groups rebuild");
        let repeated =
            rebuild_group_records_for_system(&rebuilt, &groups, system).expect("rebuild is stable");

        // Assert
        assert_eq!(
            rebuilt
                .iter()
                .map(|record| record.flags)
                .collect::<Vec<_>>(),
            vec![ParticleGroupFlags::SOLID, ParticleGroupFlags::RIGID]
        );
        assert_eq!(repeated, rebuilt);
    }
}
