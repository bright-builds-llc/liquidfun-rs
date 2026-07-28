use super::{
    ParticleBodyContact, ParticleContact, ParticleIndex, ParticlePair, ParticleProxy,
    ParticleStorageError, ParticleTriad, StuckLanes,
};

pub(super) fn replace_contents<T>(target: &mut Vec<T>, source: impl IntoIterator<Item = T>) {
    target.clear();
    target.extend(source);
}

pub(super) fn replace_optional_contents<T>(target: &mut Option<Vec<T>>, source: Option<Vec<T>>) {
    match (target.as_mut(), source) {
        (Some(target), Some(source)) => replace_contents(target, source),
        (None, Some(source)) => *target = Some(source),
        (Some(_), None) => *target = None,
        (None, None) => {}
    }
}

pub(super) fn copy_optional<T: Copy>(
    source: Option<&[T]>,
    destination: Option<&mut [T]>,
    old: usize,
    new: usize,
) {
    if let (Some(source), Some(destination)) = (source, destination) {
        destination[new] = source[old];
    }
}

pub(super) fn copy_stuck(
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

pub(super) fn validate_basic_permutation(
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

pub(super) fn remap_references(
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

pub(super) fn remap_proxies(
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

pub(super) fn remap_particle_contacts(
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

pub(super) fn remap_body_contacts(
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

pub(super) fn remap_pairs(
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

pub(super) fn remap_triads(
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
