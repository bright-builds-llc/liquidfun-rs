use super::{
    GroupRecord, IdentityEntry, IdentityState, ParticleBodyContact, ParticleColor, ParticleContact,
    ParticleFlags, ParticleGroupId, ParticleId, ParticleIndex, ParticlePair, ParticleProxy,
    ParticleSnapshot, ParticleStorage, ParticleStorageError, ParticleTriad, SolverState,
    StuckLanes, UserAssociationKey, Vec2, rebuild_group_records_for_system,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TopologyRemapMode {
    PreserveHistoricalOrder,
    AppendStableSortFirstDuplicate,
}

pub(super) enum TopologyRemapPolicy {
    PreserveHistoricalOrder,
    AppendStableSortFirstDuplicate {
        pairs: Vec<ParticlePair>,
        triads: Vec<ParticleTriad>,
    },
}

impl TopologyRemapPolicy {
    pub(super) const fn mode(&self) -> TopologyRemapMode {
        match self {
            Self::PreserveHistoricalOrder => TopologyRemapMode::PreserveHistoricalOrder,
            Self::AppendStableSortFirstDuplicate { .. } => {
                TopologyRemapMode::AppendStableSortFirstDuplicate
            }
        }
    }
}

pub(super) struct PreparedPermutation {
    identities: Vec<IdentityEntry>,
    freed_slots: Vec<usize>,
    dense_to_id: Vec<Option<ParticleId>>,
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    flags: Vec<ParticleFlags>,
    groups: Vec<Option<ParticleGroupId>>,
    weights: Vec<f32>,
    forces: Vec<Vec2>,
    maybe_colors: Option<Vec<ParticleColor>>,
    maybe_user_associations: Option<Vec<Option<UserAssociationKey>>>,
    maybe_stuck: Option<StuckLanes>,
    maybe_expiration_times: Option<Vec<i32>>,
    maybe_expiration_order: Option<Vec<ParticleIndex>>,
    proxies: Vec<ParticleProxy>,
    particle_contacts: Vec<ParticleContact>,
    body_contacts: Vec<ParticleBodyContact>,
    pairs: Vec<ParticlePair>,
    triads: Vec<ParticleTriad>,
    group_records: Vec<GroupRecord>,
    solver_state: SolverState,
    destroyed: Vec<ParticleSnapshot>,
}

struct DerivedPermutation {
    proxies: Vec<ParticleProxy>,
    particle_contacts: Vec<ParticleContact>,
    body_contacts: Vec<ParticleBodyContact>,
    pairs: Vec<ParticlePair>,
    triads: Vec<ParticleTriad>,
    maybe_expiration_order: Option<Vec<ParticleIndex>>,
}

struct RowPermutationCandidate {
    identities: Vec<IdentityEntry>,
    freed_slots: Vec<usize>,
    dense_to_id: Vec<Option<ParticleId>>,
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    flags: Vec<ParticleFlags>,
    groups: Vec<Option<ParticleGroupId>>,
    forces: Vec<Vec2>,
    maybe_colors: Option<Vec<ParticleColor>>,
    maybe_user_associations: Option<Vec<Option<UserAssociationKey>>>,
    maybe_stuck: Option<StuckLanes>,
    maybe_expiration_times: Option<Vec<i32>>,
    destroyed: Vec<ParticleSnapshot>,
}

pub(super) fn prepare_permutation(
    storage: &ParticleStorage,
    old_to_new: &[Option<usize>],
    topology_policy: TopologyRemapPolicy,
) -> Result<PreparedPermutation, ParticleStorageError> {
    storage.check_invariants()?;
    let new_count = validate_basic_permutation(old_to_new, storage.dense_to_id.len())?;
    prepare_candidate(storage, old_to_new, new_count, topology_policy)
}

pub(in crate::particle) fn apply_preserving_historical_order(
    storage: &mut ParticleStorage,
    old_to_new: &[Option<usize>],
) -> Result<Vec<ParticleSnapshot>, ParticleStorageError> {
    let candidate = prepare_permutation(
        storage,
        old_to_new,
        TopologyRemapPolicy::PreserveHistoricalOrder,
    )?;
    Ok(commit_prepared(storage, candidate))
}

fn prepare_candidate(
    storage: &ParticleStorage,
    old_to_new: &[Option<usize>],
    new_count: usize,
    topology_policy: TopologyRemapPolicy,
) -> Result<PreparedPermutation, ParticleStorageError> {
    let rows = prepare_rows(storage, old_to_new, new_count)?;
    let mut derived = remap_derived(storage, old_to_new)?;
    apply_topology_policy(&mut derived, topology_policy, new_count)?;
    let group_records =
        rebuild_group_records_for_system(&storage.group_records, &rows.groups, storage.system)?;
    let solver_state = storage.solver_state.prepare_permutation(
        old_to_new,
        new_count,
        &rows.flags,
        &group_records,
        storage.declared_capacity,
    )?;
    let weights = recompute_weights(
        new_count,
        &derived.body_contacts,
        &derived.particle_contacts,
    );
    Ok(PreparedPermutation {
        identities: rows.identities,
        freed_slots: rows.freed_slots,
        dense_to_id: rows.dense_to_id,
        positions: rows.positions,
        velocities: rows.velocities,
        flags: rows.flags,
        groups: rows.groups,
        weights,
        forces: rows.forces,
        maybe_colors: rows.maybe_colors,
        maybe_user_associations: rows.maybe_user_associations,
        maybe_stuck: rows.maybe_stuck,
        maybe_expiration_times: rows.maybe_expiration_times,
        maybe_expiration_order: derived.maybe_expiration_order,
        proxies: derived.proxies,
        particle_contacts: derived.particle_contacts,
        body_contacts: derived.body_contacts,
        pairs: derived.pairs,
        triads: derived.triads,
        group_records,
        solver_state,
        destroyed: rows.destroyed,
    })
}

fn prepare_rows(
    storage: &ParticleStorage,
    old_to_new: &[Option<usize>],
    new_count: usize,
) -> Result<RowPermutationCandidate, ParticleStorageError> {
    let mut rows = empty_rows(storage, new_count);
    for (old, maybe_new) in old_to_new.iter().copied().enumerate() {
        let id = storage.dense_to_id[old];
        let local_slot = storage.local_slot(id)?;
        let Some(new) = maybe_new else {
            remove_pending(local_slot, &mut rows)?;
            continue;
        };
        let entry = &mut rows.identities[local_slot];
        rows.dense_to_id[new] = Some(id);
        rows.positions[new] = storage.positions[old];
        rows.velocities[new] = storage.velocities[old];
        rows.flags[new] = storage.flags[old];
        rows.groups[new] = storage.groups[old];
        rows.forces[new] = storage.forces[old];
        copy_optional(
            storage.maybe_colors.as_deref(),
            rows.maybe_colors.as_deref_mut(),
            old,
            new,
        );
        copy_optional(
            storage.maybe_user_associations.as_deref(),
            rows.maybe_user_associations.as_deref_mut(),
            old,
            new,
        );
        copy_stuck(
            storage.maybe_stuck.as_ref(),
            rows.maybe_stuck.as_mut(),
            old,
            new,
        );
        copy_optional(
            storage.maybe_expiration_times.as_deref(),
            rows.maybe_expiration_times.as_deref_mut(),
            old,
            new,
        );
        entry.state = remap_identity_state(entry.state, new)?;
    }
    Ok(rows)
}

fn remove_pending(
    local_slot: usize,
    rows: &mut RowPermutationCandidate,
) -> Result<(), ParticleStorageError> {
    let entry = &mut rows.identities[local_slot];
    let IdentityState::PendingDelete { snapshot, .. } = entry.state else {
        return Err(ParticleStorageError::InvalidPermutation);
    };
    rows.destroyed.push(snapshot);
    let Some(next_generation) = entry.generation.checked_add(1) else {
        entry.state = IdentityState::Retired;
        return Ok(());
    };
    entry.generation = next_generation;
    entry.state = IdentityState::Vacant;
    rows.freed_slots.push(local_slot);
    Ok(())
}

fn remap_identity_state(
    state: IdentityState,
    new: usize,
) -> Result<IdentityState, ParticleStorageError> {
    match state {
        IdentityState::Live(_) => Ok(IdentityState::Live(ParticleIndex(new))),
        IdentityState::PendingDelete { snapshot, .. } => Ok(IdentityState::PendingDelete {
            dense: ParticleIndex(new),
            snapshot,
        }),
        IdentityState::Vacant | IdentityState::Retired => {
            Err(ParticleStorageError::InvalidPermutation)
        }
    }
}

fn empty_rows(storage: &ParticleStorage, new_count: usize) -> RowPermutationCandidate {
    RowPermutationCandidate {
        identities: storage.identities.clone(),
        freed_slots: Vec::new(),
        dense_to_id: vec![None; new_count],
        positions: vec![Vec2::ZERO; new_count],
        velocities: vec![Vec2::ZERO; new_count],
        flags: vec![ParticleFlags::WATER; new_count],
        groups: vec![None; new_count],
        forces: vec![Vec2::ZERO; new_count],
        maybe_colors: storage
            .maybe_colors
            .as_ref()
            .map(|_| vec![ParticleColor::ZERO; new_count]),
        maybe_user_associations: storage
            .maybe_user_associations
            .as_ref()
            .map(|_| vec![None; new_count]),
        maybe_stuck: storage.maybe_stuck.as_ref().map(|_| StuckLanes {
            last_body_contact_steps: vec![0; new_count],
            body_contact_counts: vec![0; new_count],
            consecutive_contact_steps: vec![0; new_count],
            candidates: Vec::new(),
        }),
        maybe_expiration_times: storage
            .maybe_expiration_times
            .as_ref()
            .map(|_| vec![0; new_count]),
        destroyed: Vec::new(),
    }
}

fn recompute_weights(
    particle_count: usize,
    body_contacts: &[ParticleBodyContact],
    particle_contacts: &[ParticleContact],
) -> Vec<f32> {
    let mut weights = vec![0.0; particle_count];
    ParticleStorage::recompute_contact_weights(&mut weights, body_contacts, particle_contacts);
    weights
}

fn remap_derived(
    storage: &ParticleStorage,
    old_to_new: &[Option<usize>],
) -> Result<DerivedPermutation, ParticleStorageError> {
    Ok(DerivedPermutation {
        proxies: remap_proxies(&storage.proxies, old_to_new)?,
        particle_contacts: remap_particle_contacts(&storage.particle_contacts, old_to_new)?,
        body_contacts: remap_body_contacts(&storage.body_contacts, old_to_new)?,
        pairs: remap_pairs(&storage.pairs, old_to_new)?,
        triads: remap_triads(&storage.triads, old_to_new)?,
        maybe_expiration_order: storage
            .maybe_expiration_order
            .as_ref()
            .map(|order| remap_references(order, old_to_new))
            .transpose()?,
    })
}

fn apply_topology_policy(
    derived: &mut DerivedPermutation,
    topology_policy: TopologyRemapPolicy,
    particle_count: usize,
) -> Result<(), ParticleStorageError> {
    validate_topology(&derived.pairs, &derived.triads, particle_count)?;
    let TopologyRemapPolicy::AppendStableSortFirstDuplicate { pairs, triads } = topology_policy
    else {
        return Ok(());
    };
    validate_topology(&pairs, &triads, particle_count)?;
    derived
        .pairs
        .try_reserve_exact(pairs.len())
        .map_err(|_error| ParticleStorageError::InvalidLaneBundle)?;
    derived
        .triads
        .try_reserve_exact(triads.len())
        .map_err(|_error| ParticleStorageError::InvalidLaneBundle)?;
    derived.pairs.extend(pairs);
    derived.triads.extend(triads);
    stable_sort_first_pair_duplicate(&mut derived.pairs);
    stable_sort_first_triad_duplicate(&mut derived.triads);
    Ok(())
}

fn validate_topology(
    pairs: &[ParticlePair],
    triads: &[ParticleTriad],
    particle_count: usize,
) -> Result<(), ParticleStorageError> {
    for pair in pairs {
        pair.validate(particle_count)?;
    }
    for triad in triads {
        triad.validate(particle_count)?;
    }
    Ok(())
}

fn stable_sort_first_pair_duplicate(pairs: &mut Vec<ParticlePair>) {
    pairs.sort_by_key(|pair| pair.indices.map(|index| index.0));
    retain_first_by_indices(pairs, |pair| pair.indices);
}

fn stable_sort_first_triad_duplicate(triads: &mut Vec<ParticleTriad>) {
    triads.sort_by_key(|triad| triad.indices.map(|index| index.0));
    retain_first_by_indices(triads, |triad| triad.indices);
}

fn retain_first_by_indices<T: Copy, const N: usize>(
    records: &mut Vec<T>,
    indices: impl Fn(T) -> [ParticleIndex; N],
) {
    let mut retained = Vec::with_capacity(records.len());
    for record in records.iter().copied() {
        let is_duplicate = retained
            .last()
            .is_some_and(|previous| indices(*previous) == indices(record));
        if !is_duplicate {
            retained.push(record);
        }
    }
    *records = retained;
}

pub(super) fn commit_prepared(
    storage: &mut ParticleStorage,
    candidate: PreparedPermutation,
) -> Vec<ParticleSnapshot> {
    storage.identities = candidate.identities;
    storage.free_identity_slots.extend(candidate.freed_slots);
    storage.retired_identity_slots = storage
        .identities
        .iter()
        .filter(|entry| entry.state == IdentityState::Retired)
        .count();
    replace_contents(
        &mut storage.dense_to_id,
        candidate
            .dense_to_id
            .into_iter()
            .map(|maybe_id| maybe_id.expect("validated permutations fill every destination")),
    );
    replace_contents(&mut storage.positions, candidate.positions);
    replace_contents(&mut storage.velocities, candidate.velocities);
    replace_contents(&mut storage.flags, candidate.flags);
    replace_contents(&mut storage.groups, candidate.groups);
    replace_contents(&mut storage.weights, candidate.weights);
    replace_contents(&mut storage.forces, candidate.forces);
    replace_optional_contents(&mut storage.maybe_colors, candidate.maybe_colors);
    replace_optional_contents(
        &mut storage.maybe_user_associations,
        candidate.maybe_user_associations,
    );
    storage.maybe_stuck = candidate.maybe_stuck;
    storage.maybe_expiration_times = candidate.maybe_expiration_times;
    storage.maybe_expiration_order = candidate.maybe_expiration_order;
    storage.proxies = candidate.proxies;
    storage.particle_contacts = candidate.particle_contacts;
    storage.body_contacts = candidate.body_contacts;
    storage.pairs = candidate.pairs;
    storage.triads = candidate.triads;
    storage.group_records = candidate.group_records;
    storage.solver_state = candidate.solver_state;
    debug_assert_eq!(storage.check_invariants(), Ok(()));
    candidate.destroyed
}

impl PreparedPermutation {
    pub(super) fn destroyed(&self) -> &[ParticleSnapshot] {
        &self.destroyed
    }
}

fn replace_contents<T>(target: &mut Vec<T>, source: impl IntoIterator<Item = T>) {
    target.clear();
    target.extend(source);
}

fn replace_optional_contents<T>(target: &mut Option<Vec<T>>, source: Option<Vec<T>>) {
    match (target.as_mut(), source) {
        (Some(target), Some(source)) => replace_contents(target, source),
        (None, Some(source)) => *target = Some(source),
        (Some(_), None) => *target = None,
        (None, None) => {}
    }
}

fn copy_optional<T: Copy>(
    source: Option<&[T]>,
    destination: Option<&mut [T]>,
    old: usize,
    new: usize,
) {
    if let (Some(source), Some(destination)) = (source, destination) {
        destination[new] = source[old];
    }
}

fn copy_stuck(
    source: Option<&StuckLanes>,
    destination: Option<&mut StuckLanes>,
    old: usize,
    new: usize,
) {
    let (Some(source), Some(destination)) = (source, destination) else {
        return;
    };
    destination.last_body_contact_steps[new] = source.last_body_contact_steps[old];
    destination.body_contact_counts[new] = source.body_contact_counts[old];
    destination.consecutive_contact_steps[new] = source.consecutive_contact_steps[old];
}

fn validate_basic_permutation(
    old_to_new: &[Option<usize>],
    old_count: usize,
) -> Result<usize, ParticleStorageError> {
    if old_to_new.len() != old_count {
        return Err(ParticleStorageError::InvalidPermutation);
    }
    let new_count = old_to_new.iter().filter(|value| value.is_some()).count();
    let mut seen = vec![false; new_count];
    for destination in old_to_new.iter().flatten().copied() {
        let Some(was_seen) = seen.get_mut(destination) else {
            return Err(ParticleStorageError::InvalidPermutation);
        };
        if *was_seen {
            return Err(ParticleStorageError::InvalidPermutation);
        }
        *was_seen = true;
    }
    if seen.iter().any(|was_seen| !was_seen) {
        return Err(ParticleStorageError::InvalidPermutation);
    }
    Ok(new_count)
}

fn remap_references(
    references: &[ParticleIndex],
    old_to_new: &[Option<usize>],
) -> Result<Vec<ParticleIndex>, ParticleStorageError> {
    references
        .iter()
        .filter_map(|index| match old_to_new.get(index.0) {
            Some(Some(destination)) => Some(Ok(ParticleIndex(*destination))),
            Some(None) => None,
            None => Some(Err(ParticleStorageError::InvalidDerivedReference)),
        })
        .collect()
}

fn remap_proxies(
    proxies: &[ParticleProxy],
    old_to_new: &[Option<usize>],
) -> Result<Vec<ParticleProxy>, ParticleStorageError> {
    proxies
        .iter()
        .filter_map(|proxy| match remap_index(proxy.index, old_to_new) {
            Ok(Some(index)) => Some(Ok(ParticleProxy { index, ..*proxy })),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn remap_particle_contacts(
    contacts: &[ParticleContact],
    old_to_new: &[Option<usize>],
) -> Result<Vec<ParticleContact>, ParticleStorageError> {
    remap_records(
        contacts,
        old_to_new,
        |contact| contact.indices,
        |contact, indices| ParticleContact { indices, ..contact },
    )
}

fn remap_body_contacts(
    contacts: &[ParticleBodyContact],
    old_to_new: &[Option<usize>],
) -> Result<Vec<ParticleBodyContact>, ParticleStorageError> {
    contacts
        .iter()
        .filter_map(|contact| match remap_index(contact.index, old_to_new) {
            Ok(Some(index)) => Some(Ok(ParticleBodyContact { index, ..*contact })),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn remap_pairs(
    pairs: &[ParticlePair],
    old_to_new: &[Option<usize>],
) -> Result<Vec<ParticlePair>, ParticleStorageError> {
    remap_records(
        pairs,
        old_to_new,
        |pair| pair.indices,
        |pair, indices| ParticlePair { indices, ..pair },
    )
}

fn remap_triads(
    triads: &[ParticleTriad],
    old_to_new: &[Option<usize>],
) -> Result<Vec<ParticleTriad>, ParticleStorageError> {
    remap_records(
        triads,
        old_to_new,
        |triad| triad.indices,
        |triad, indices| ParticleTriad { indices, ..triad },
    )
}

fn remap_records<T: Copy, const N: usize>(
    records: &[T],
    old_to_new: &[Option<usize>],
    indices: impl Fn(T) -> [ParticleIndex; N],
    rebuild: impl Fn(T, [ParticleIndex; N]) -> T,
) -> Result<Vec<T>, ParticleStorageError> {
    let mut remapped = Vec::with_capacity(records.len());
    for record in records.iter().copied() {
        let old_indices = indices(record);
        let mut new_indices = [ParticleIndex(0); N];
        let mut removed = false;
        for (destination, old) in new_indices.iter_mut().zip(old_indices) {
            match remap_index(old, old_to_new)? {
                Some(new) => *destination = new,
                None => removed = true,
            }
        }
        if !removed {
            remapped.push(rebuild(record, new_indices));
        }
    }
    Ok(remapped)
}

fn remap_index(
    old: ParticleIndex,
    old_to_new: &[Option<usize>],
) -> Result<Option<ParticleIndex>, ParticleStorageError> {
    old_to_new
        .get(old.0)
        .copied()
        .map(|maybe_new| maybe_new.map(ParticleIndex))
        .ok_or(ParticleStorageError::InvalidDerivedReference)
}

#[cfg(test)]
mod tests;
