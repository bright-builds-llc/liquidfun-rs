use super::{
    GroupRecord, ParticleGroupId, ParticleStorage, ParticleStorageError, PreparedPermutation,
    TopologyRemapPolicy, prepare_candidate, validate_basic_permutation,
};

pub(in crate::particle::storage) fn prepare_group_reassignment_permutation(
    storage: &ParticleStorage,
    old_to_new: &[Option<usize>],
    groups_by_old_row: &[Option<ParticleGroupId>],
    group_records: &[GroupRecord],
    topology_policy: TopologyRemapPolicy,
) -> Result<PreparedPermutation, ParticleStorageError> {
    storage.check_invariants()?;
    if groups_by_old_row.len() != storage.len() {
        return Err(ParticleStorageError::InvalidPermutation);
    }
    let new_count = validate_basic_permutation(old_to_new, storage.dense_to_id.len())?;
    prepare_candidate(
        storage,
        old_to_new,
        new_count,
        Some(groups_by_old_row),
        group_records,
        topology_policy,
    )
}
