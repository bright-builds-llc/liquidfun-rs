use crate::identity::{HandleIdentity, Identity, ParticleSystemId, WorldKey};
use crate::math::Vec2;
use crate::particle::ParticleFlags;
use crate::particle::storage::{ParticleInput, ParticleStorage};

use super::*;

fn system_definition(maximum: usize) -> ParticleSystemDef {
    ParticleSystemDef::default()
        .with_lifetime_granularity(1.0)
        .expect("test granularity is positive")
        .with_maximum_count(maximum)
        .expect("test maximum fits")
}

fn storage(capacity: usize) -> ParticleStorage {
    let world = WorldKey::fresh().expect("test world key remains available");
    let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    ParticleStorage::new(world, system, 0, capacity, capacity).expect("test storage is valid")
}

fn input() -> ParticleInput {
    ParticleInput {
        position: Vec2::ZERO,
        velocity: Vec2::ZERO,
        flags: ParticleFlags::WATER,
        maybe_group: None,
        maybe_color: None,
        maybe_user_association: None,
        maybe_expiration_time: None,
    }
}

#[test]
fn solve_marks_only_expired_finite_particles() {
    // Arrange
    let definition = system_definition(3);
    let mut storage = storage(3);
    let first = storage.create(input()).expect("first particle fits");
    let second = storage.create(input()).expect("second particle fits");
    let third = storage.create(input()).expect("third particle fits");
    let mut state = ParticleLifetimeState::new(definition, &mut storage);
    state
        .initialize_created_particle(&mut storage, first, 2.0)
        .expect("finite lifetime is valid");
    state
        .initialize_created_particle(&mut storage, second, 0.0)
        .expect("infinite lifetime is valid");
    state
        .initialize_created_particle(&mut storage, third, 4.0)
        .expect("finite lifetime is valid");

    // Act
    let marked = state
        .solve_lifetimes(&mut storage, 2.0)
        .expect("clock and storage remain valid");

    // Assert
    assert_eq!(
        marked
            .into_iter()
            .map(|snapshot| snapshot.id)
            .collect::<Vec<_>>(),
        vec![first]
    );
    assert_eq!(
        storage.input(first),
        Err(ParticleStorageError::PendingDelete)
    );
    assert!(storage.input(second).is_ok());
    assert!(storage.input(third).is_ok());
}

#[test]
fn tracked_infinite_insertion_dirties_a_previously_sorted_finite_order() {
    // Arrange
    let definition = system_definition(2);
    let mut storage = storage(2);
    let mut state = ParticleLifetimeState::new(definition, &mut storage);
    let finite = storage.create(input()).expect("finite particle fits");
    state
        .initialize_created_particle(&mut storage, finite, 1.0)
        .expect("finite lifetime is valid");
    state
        .solve_lifetimes(&mut storage, 0.0)
        .expect("zero tick sorts the initial order");
    let infinite = storage.create(input()).expect("infinite particle fits");
    state
        .initialize_created_particle(&mut storage, infinite, -2.0)
        .expect("infinite lifetime is valid");

    // Act
    let marked = state
        .solve_lifetimes(&mut storage, 1.0)
        .expect("finite particle should expire past the new infinite row");

    // Assert
    assert_eq!(
        marked
            .into_iter()
            .map(|snapshot| snapshot.id)
            .collect::<Vec<_>>(),
        vec![finite]
    );
    assert_eq!(storage.is_pending(finite), Ok(true));
    assert_eq!(storage.is_pending(infinite), Ok(false));
}

#[test]
fn full_capacity_evicts_canonical_tie_without_listener_request() {
    // Arrange
    let definition = system_definition(2);
    let mut storage = storage(2);
    let first = storage.create(input()).expect("first particle fits");
    let second = storage.create(input()).expect("second particle fits");
    let mut state = ParticleLifetimeState::new(definition, &mut storage);
    state
        .initialize_created_particle(&mut storage, first, 2.5)
        .expect("finite lifetime is valid");
    state
        .initialize_created_particle(&mut storage, second, 2.5)
        .expect("finite lifetime is valid");

    // Act
    let outcome = state
        .prepare_capacity_for_creation(&mut storage)
        .expect("destroy-by-age frees capacity")
        .expect("full capacity evicts one particle");
    let evicted = outcome
        .destroyed
        .first()
        .copied()
        .expect("one full-capacity particle is destroyed");

    // Assert
    assert_eq!(evicted.id, second);
    assert!(evicted.input.flags.contains(ParticleFlags::ZOMBIE));
    assert!(
        !evicted
            .input
            .flags
            .contains(ParticleFlags::DESTRUCTION_LISTENER)
    );
    assert_eq!(storage.particle_ids(), &[first]);
    assert_eq!(
        storage.input(second),
        Err(ParticleStorageError::StaleOrDestroyed)
    );
}

#[test]
fn compaction_emits_requested_occurrences_before_ascending_invalidation() {
    // Arrange
    let mut storage = storage(4);
    let ids = (0..4)
        .map(|_| storage.create(input()).expect("particle fits"))
        .collect::<Vec<_>>();
    storage
        .mark_delete_for_lifecycle(ids[2], true)
        .expect("third particle becomes pending");
    storage
        .mark_delete_for_lifecycle(ids[0], true)
        .expect("first particle becomes pending");

    // Act
    let outcome = compact_pending_with_occurrences(&mut storage)
        .expect("authoritative permutation remains valid");

    // Assert
    assert_eq!(
        outcome
            .requested_listener_occurrences
            .iter()
            .map(|occurrence| occurrence.particle())
            .collect::<Vec<_>>(),
        vec![ids[0], ids[2]]
    );
    assert_eq!(
        outcome
            .destroyed
            .iter()
            .map(|snapshot| snapshot.id)
            .collect::<Vec<_>>(),
        vec![ids[0], ids[2]]
    );
    assert_eq!(storage.particle_ids(), &[ids[1], ids[3]]);
    assert_eq!(
        storage.input(ids[0]),
        Err(ParticleStorageError::StaleOrDestroyed)
    );
}

#[test]
fn oldest_selection_tolerates_an_already_pending_only_particle() {
    // Arrange
    let definition = system_definition(1);
    let mut storage = storage(1);
    let particle = storage.create(input()).expect("particle fits");
    let mut state = ParticleLifetimeState::new(definition, &mut storage);
    storage
        .mark_delete_for_lifecycle(particle, false)
        .expect("particle becomes pending");

    // Act
    let selected = state
        .destroy_oldest_particle(&mut storage, 0, false)
        .expect("reselecting a pending oldest particle is idempotent");
    let outcome =
        compact_pending_with_occurrences(&mut storage).expect("pending particle compacts");

    // Assert
    assert_eq!(selected.id, particle);
    assert_eq!(
        outcome
            .destroyed
            .iter()
            .map(|snapshot| snapshot.id)
            .collect::<Vec<_>>(),
        vec![particle]
    );
}
