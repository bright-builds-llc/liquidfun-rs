//! The ungrouped creation fast path must leave storage equal to the full rebuild.

use super::*;

const CAPACITY: usize = 512;

/// Verbatim copy of `prepare_create` before the ungrouped fast path.
fn reference_prepare(
    storage: &ParticleStorage,
    input: ParticleInput,
    diagnostic_id: u64,
    free_slots: usize,
) -> Result<CreateCandidate, ParticleStorageError> {
    let occupied = storage.dense_to_id.len().saturating_sub(free_slots);
    if occupied >= storage.declared_capacity {
        return Err(ParticleStorageError::CapacityExceeded {
            limit: storage.declared_capacity,
        });
    }
    storage.validate_appended_group(input.maybe_group)?;
    let (local_slot, generation, append_identity) = storage.identity_slot_candidate(free_slots)?;
    let particle_slot = storage
        .identity_slot_base
        .checked_add(local_slot)
        .ok_or(ParticleStorageError::IdentityExhausted)?;
    let id = ParticleId::from_identity(Identity::new_particle(
        storage.world,
        particle_slot,
        generation,
        storage.system.identity(),
    ));
    let dense = ParticleIndex(storage.dense_to_id.len());
    let mut groups = storage.groups.clone();
    groups.push(input.maybe_group);
    let group_records =
        rebuild_group_records_for_system(&storage.group_records, &groups, storage.system)?;
    let solver_state = storage.solver_state.prepare_append(
        &storage.flags,
        input.flags,
        &group_records,
        storage.declared_capacity,
        free_slots,
    )?;
    Ok(CreateCandidate {
        input,
        diagnostic_id,
        id,
        local_slot,
        generation,
        append_identity,
        dense,
        group_records,
        solver_state,
    })
}

fn reference_create(
    storage: &mut ParticleStorage,
    input: ParticleInput,
    diagnostic_id: u64,
) -> Result<ParticleId, ParticleStorageError> {
    let candidate = reference_prepare(storage, input, diagnostic_id, 0)?;
    Ok(storage.commit_create(candidate))
}

fn reference_validate(
    storage: &ParticleStorage,
    input: ParticleInput,
    free_slots: usize,
) -> Result<(), ParticleStorageError> {
    reference_prepare(storage, input, 0, free_slots).map(|_candidate| ())
}

fn storage_with(identity_capacity: usize, declared_capacity: usize) -> ParticleStorage {
    let world = WorldKey::fresh().expect("test world key remains available");
    let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    ParticleStorage::new(world, system, 0, identity_capacity, declared_capacity)
        .expect("test storage contract is valid")
}

fn group_id(storage: &ParticleStorage, slot: usize) -> ParticleGroupId {
    ParticleGroupId::from_identity(Identity::new(storage.world, slot, 0))
}

#[allow(
    clippy::cast_precision_loss,
    reason = "test positions come from small row counts"
)]
fn input(row: usize, maybe_group: Option<ParticleGroupId>) -> ParticleInput {
    ParticleInput {
        position: Vec2::new(row as f32 * 0.25, 1.0),
        velocity: Vec2::new(0.5, -(row as f32)),
        flags: ParticleFlags::WATER,
        maybe_group,
        maybe_color: None,
        maybe_user_association: None,
        maybe_expiration_time: None,
    }
}

/// One ungrouped row, a three-row group, and a `CAN_BE_EMPTY` group emptied by
/// removals whose cached statistics carry a timestamp.
fn storage_with_groups() -> (ParticleStorage, ParticleGroupId, ParticleGroupId) {
    let mut storage = storage_with(CAPACITY, CAPACITY);
    let kept = group_id(&storage, 1);
    let emptied = group_id(&storage, 2);
    storage.create(input(0, None)).expect("ungrouped row fits");
    for row in 1..4 {
        storage
            .create(input(row, Some(kept)))
            .expect("kept group fits");
    }
    let emptied_members = (4..7)
        .map(|row| storage.create(input(row, Some(emptied))))
        .collect::<Result<Vec<_>, _>>()
        .expect("emptied group fits");
    storage
        .set_group_flags_internal(emptied, ParticleGroupFlags::CAN_BE_EMPTY)
        .expect("retained group flag applies");
    for member in emptied_members {
        storage.mark_delete(member).expect("member is live");
    }
    storage.compact_pending().expect("pending rows compact");
    storage
        .update_group_statistics(emptied, 1.0, 9)
        .expect("empty group statistics are cached");
    (storage, kept, emptied)
}

fn record(storage: &ParticleStorage, group: ParticleGroupId) -> GroupRecord {
    *storage
        .group_records
        .iter()
        .find(|record| record.id == group)
        .expect("group record exists")
}

#[test]
fn ungrouped_create_matches_reference_storage() {
    // Arrange
    let mut fast = storage_with(CAPACITY, CAPACITY);
    let mut reference = fast.clone();

    // Act
    for row in 0..300 {
        let diagnostic_id = u64::try_from(row).expect("row fits u64");
        assert_eq!(
            fast.validate_create_reserving(input(row, None), 1),
            reference_validate(&reference, input(row, None), 1)
        );
        let fast_id = fast.create_with_diagnostic(input(row, None), diagnostic_id);
        let reference_id = reference_create(&mut reference, input(row, None), diagnostic_id);
        assert_eq!(fast_id, reference_id);
    }

    // Assert
    assert!(fast == reference);
    assert_eq!(fast.group_records, reference.group_records);
}

#[test]
fn ungrouped_create_into_storage_with_groups_matches_reference() {
    // Arrange
    let (mut fast, kept, emptied) = storage_with_groups();
    let mut reference = fast.clone();
    assert_eq!(record(&fast, kept).range(), 1..4);
    assert_eq!(record(&fast, emptied).range(), 0..0);
    assert_eq!(
        record(&fast, emptied).statistics.maybe_source_timestamp,
        Some(9)
    );

    // Act
    for row in 4..304 {
        let fast_id = fast.create(input(row, None));
        let reference_id = reference_create(&mut reference, input(row, None), 0);
        assert_eq!(fast_id, reference_id);
    }

    // Assert
    assert!(fast == reference);
    assert_eq!(fast.group_records, reference.group_records);
    assert_eq!(
        record(&fast, emptied).statistics.maybe_source_timestamp,
        None
    );
}

#[test]
fn grouped_create_still_uses_full_rebuild() {
    // Arrange
    let (mut fast, kept, _emptied) = storage_with_groups();
    let mut reference = fast.clone();
    let appended = group_id(&fast, 3);

    // Act
    for (row, group) in [(4, kept), (5, kept), (6, appended), (7, appended)] {
        let fast_id = fast.create(input(row, Some(group)));
        let reference_id = reference_create(&mut reference, input(row, Some(group)), 0);
        assert_eq!(fast_id, reference_id);
    }

    // Assert
    assert!(fast == reference);
    assert_eq!(fast.group_records, reference.group_records);
    assert_eq!(record(&fast, appended).range(), 6..8);
}

#[test]
fn create_errors_keep_precedence() {
    // Arrange
    let mut full = storage_with(4, 2);
    full.create(input(0, None)).expect("first row fits");
    full.create(input(1, None)).expect("second row fits");
    let (mut split_group, kept, _emptied) = storage_with_groups();
    split_group
        .create(input(4, None))
        .expect("ungrouped tail fits");
    let mut exhausted = storage_with(2, 4);
    exhausted.create(input(0, None)).expect("first row fits");
    exhausted.identities.push(IdentityEntry {
        generation: u64::MAX,
        maybe_diagnostic_id: None,
        state: IdentityState::Retired,
    });
    exhausted.retired_identity_slots += 1;
    let exhausted_group = group_id(&exhausted, 1);
    let mut identities_full = storage_with(1, 4);
    identities_full
        .create(input(0, None))
        .expect("first row fits");
    let cases = [
        (&full, input(2, None), 0),
        (&full, input(2, None), 1),
        (&split_group, input(5, Some(kept)), 0),
        (&exhausted, input(1, None), 0),
        (&exhausted, input(1, Some(exhausted_group)), 0),
        (&identities_full, input(1, None), 0),
    ];

    // Act
    let results = cases
        .iter()
        .map(|(storage, input, free_slots)| {
            (
                storage.validate_create_reserving(*input, *free_slots),
                reference_validate(storage, *input, *free_slots),
            )
        })
        .collect::<Vec<_>>();
    let fast_created = full.create(input(2, None));

    // Assert
    let expected = [
        Err(ParticleStorageError::CapacityExceeded { limit: 2 }),
        Ok(()),
        Err(ParticleStorageError::InvalidGroupRange),
        Err(ParticleStorageError::IdentityExhausted),
        Err(ParticleStorageError::IdentityExhausted),
        Err(ParticleStorageError::CapacityExceeded { limit: 1 }),
    ];
    for ((fast, reference), expected) in results.into_iter().zip(expected) {
        assert_eq!(fast, reference);
        assert_eq!(fast, expected);
    }
    assert_eq!(
        fast_created,
        Err(ParticleStorageError::CapacityExceeded { limit: 2 })
    );
}
