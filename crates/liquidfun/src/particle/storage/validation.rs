use super::{GroupRange, ParticleGroupId, ParticleIndex, ParticleStorageError};

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

pub(super) fn build_group_ranges(
    groups: &[Option<ParticleGroupId>],
) -> Result<Vec<GroupRange>, ParticleStorageError> {
    let mut ranges: Vec<GroupRange> = Vec::new();
    for (dense, maybe_group) in groups.iter().copied().enumerate() {
        let Some(group) = maybe_group else {
            continue;
        };
        if let Some(last) = ranges.last_mut()
            && last.maybe_group == Some(group)
            && last.end.0 == dense
        {
            last.end = ParticleIndex(dense + 1);
            continue;
        }
        if ranges.iter().any(|range| range.maybe_group == Some(group)) {
            return Err(ParticleStorageError::InvalidGroupRange);
        }
        ranges.push(GroupRange {
            maybe_group: Some(group),
            start: ParticleIndex(dense),
            end: ParticleIndex(dense + 1),
        });
    }
    Ok(ranges)
}
