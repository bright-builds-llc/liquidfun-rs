use crate::identity::{HandleIdentity, Identity, ParticleGroupId, ParticleSystemId, WorldKey};
use crate::particle::ParticleBufferMode;

use super::super::{ParticleInput, ParticleStorage};
use super::*;

fn storage() -> ParticleStorage {
    let world = WorldKey::fresh().expect("test world key remains available");
    let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    ParticleStorage::new(world, system, 0, 8, 8).expect("test storage contract is valid")
}

fn input(value: f32, flags: ParticleFlags, maybe_group: Option<ParticleGroupId>) -> ParticleInput {
    ParticleInput {
        position: Vec2::new(value, -value),
        velocity: Vec2::new(value + 1.0, value + 2.0),
        flags,
        maybe_group,
        maybe_color: None,
        maybe_user_association: None,
        maybe_expiration_time: None,
    }
}

#[test]
fn exact_gates_allocate_and_append_aligned_zero_rows() {
    // Arrange
    let mut storage = storage();
    let group = ParticleGroupId::from_identity(Identity::new(storage.world, 1, 0));
    storage
        .create(input(
            1.0,
            ParticleFlags::STATIC_PRESSURE | ParticleFlags::TENSILE,
            Some(group),
        ))
        .expect("first grouped particle fits");
    storage
        .create(input(2.0, ParticleFlags::WATER, Some(group)))
        .expect("second grouped particle fits");
    storage.group_records[0].flags = ParticleGroupFlags::SOLID;
    storage.solver_state.mark_group_flags_dirty();
    assert!(storage.solver_state.maybe_static_pressures().is_none());
    assert!(storage.solver_state.maybe_tensile_accumulations().is_none());
    assert!(storage.solver_state.maybe_depths().is_none());

    // Act
    storage
        .ensure_static_pressures()
        .expect("static-pressure flag opens its allocation gate");
    storage
        .ensure_tensile_accumulations()
        .expect("tensile flag opens its allocation gate");
    storage
        .ensure_depths()
        .expect("solid group opens its allocation gate");
    storage
        .create(input(3.0, ParticleFlags::WATER, Some(group)))
        .expect("aligned append fits");

    // Assert
    assert_eq!(
        storage.solver_state.maybe_static_pressures(),
        Some([0.0, 0.0, 0.0].as_slice())
    );
    assert_eq!(
        storage.solver_state.maybe_tensile_accumulations(),
        Some([Vec2::ZERO, Vec2::ZERO, Vec2::ZERO].as_slice())
    );
    assert_eq!(
        storage.solver_state.maybe_depths(),
        Some([0.0, 0.0, 0.0].as_slice())
    );
    assert_eq!(storage.check_invariants(), Ok(()));
}

#[test]
fn non_finite_candidate_rejection_leaves_live_scratch_unchanged() {
    // Arrange
    let mut storage = storage();
    storage
        .create(input(1.0, ParticleFlags::STATIC_PRESSURE, None))
        .expect("particle fits");
    storage
        .ensure_static_pressures()
        .expect("static-pressure flag opens its allocation gate");
    let before = storage
        .solver_state
        .maybe_static_pressures()
        .expect("lane is allocated")
        .to_vec();

    // Act
    let result = storage.replace_static_pressures(vec![f32::NAN]);

    // Assert
    assert_eq!(result, Err(ParticleStorageError::InvalidLaneBundle));
    assert_eq!(
        storage.solver_state.maybe_static_pressures(),
        Some(before.as_slice())
    );
}

#[test]
fn oversized_allocation_candidate_leaves_optional_lane_absent() {
    // Arrange
    let mut state = SolverState::new();

    // Act
    let result = state.ensure_static_pressures(9, 8);

    // Assert
    assert_eq!(result, Err(ParticleStorageError::InvalidLaneBundle));
    assert!(state.maybe_static_pressures().is_none());
}

#[test]
fn mismatched_solver_lane_blocks_creation_before_live_replacement() {
    // Arrange
    let mut storage = storage();
    storage
        .create(input(1.0, ParticleFlags::STATIC_PRESSURE, None))
        .expect("particle fits");
    storage
        .ensure_static_pressures()
        .expect("static-pressure flag opens its allocation gate");
    storage
        .solver_state
        .maybe_static_pressures
        .as_mut()
        .expect("lane is allocated")
        .pop();
    let before = storage.clone();

    // Act
    let result = storage.create(input(2.0, ParticleFlags::WATER, None));

    // Assert
    assert_eq!(result, Err(ParticleStorageError::LaneLengthMismatch));
    assert!(storage == before);
}

#[test]
fn rotation_and_compaction_preserve_every_solver_lane_by_stable_identity() {
    // Arrange
    let mut storage = storage();
    let ids = [
        storage
            .create(input(
                1.0,
                ParticleFlags::STATIC_PRESSURE | ParticleFlags::TENSILE,
                None,
            ))
            .expect("first particle fits"),
        storage
            .create(input(2.0, ParticleFlags::WATER, None))
            .expect("second particle fits"),
        storage
            .create(input(3.0, ParticleFlags::WATER, None))
            .expect("third particle fits"),
        storage
            .create(input(4.0, ParticleFlags::WATER, None))
            .expect("fourth particle fits"),
    ];
    storage
        .ensure_static_pressures()
        .expect("static-pressure flag opens its allocation gate");
    storage
        .ensure_tensile_accumulations()
        .expect("tensile flag opens its allocation gate");
    let particle_count = storage.len();
    let declared_capacity = storage.declared_capacity();
    storage
        .solver_state
        .ensure_depths(particle_count, declared_capacity)
        .expect("test directly opens the depth source point");
    storage
        .replace_static_pressures(vec![1.0, 2.0, 3.0, 4.0])
        .expect("finite static-pressure candidate is aligned");
    storage
        .replace_tensile_accumulations(vec![
            Vec2::new(1.0, 10.0),
            Vec2::new(2.0, 20.0),
            Vec2::new(3.0, 30.0),
            Vec2::new(4.0, 40.0),
        ])
        .expect("finite tensile candidate is aligned");
    storage
        .replace_depths(vec![10.0, 20.0, 30.0, 40.0])
        .expect("finite depth candidate is aligned");

    // Act
    storage
        .rotate_rows(0, 2, 4)
        .expect("whole storage rotation is valid");
    storage
        .mark_delete(ids[0])
        .expect("rotated particle is live");
    storage
        .compact_pending()
        .expect("selected particle compacts");

    // Assert
    assert_eq!(
        storage.solver_state.maybe_static_pressures(),
        Some([3.0, 4.0, 2.0].as_slice())
    );
    assert_eq!(
        storage.solver_state.maybe_tensile_accumulations(),
        Some(
            [
                Vec2::new(3.0, 30.0),
                Vec2::new(4.0, 40.0),
                Vec2::new(2.0, 20.0),
            ]
            .as_slice()
        )
    );
    assert_eq!(
        storage.solver_state.maybe_depths(),
        Some([30.0, 40.0, 20.0].as_slice())
    );
    assert_eq!(storage.check_invariants(), Ok(()));
}

#[test]
fn stable_scans_refresh_the_only_group_aggregate_authority() {
    // Arrange
    let mut storage = storage();
    let first = ParticleGroupId::from_identity(Identity::new(storage.world, 1, 0));
    let second = ParticleGroupId::from_identity(Identity::new(storage.world, 2, 0));
    storage
        .create(input(1.0, ParticleFlags::VISCOUS, Some(first)))
        .expect("first grouped particle fits");
    storage
        .create(input(2.0, ParticleFlags::TENSILE, Some(second)))
        .expect("second grouped particle fits");
    storage.group_records[0].flags = ParticleGroupFlags::RIGID;
    storage.group_records[1].flags = ParticleGroupFlags::SOLID;
    storage.solver_state.mark_group_flags_dirty();

    // Act
    let particle_flags = storage.aggregate_particle_flags();
    let group_flags = storage.aggregate_group_flags();

    // Assert
    assert_eq!(
        particle_flags,
        ParticleFlags::VISCOUS | ParticleFlags::TENSILE
    );
    assert_eq!(
        group_flags.public,
        ParticleGroupFlags::RIGID | ParticleGroupFlags::SOLID
    );
    let group_source = include_str!("../group.rs");
    assert!(!group_source.contains("aggregate_group_flags:"));
    assert!(!group_source.contains("group_flags_dirty:"));
}

#[test]
fn pending_force_is_storage_state_and_external_teardown_omits_solver_lanes() {
    // Arrange
    let mut storage = storage();
    storage
        .create(input(1.0, ParticleFlags::STATIC_PRESSURE, None))
        .expect("particle fits");
    storage
        .ensure_static_pressures()
        .expect("static-pressure flag opens its allocation gate");

    // Act
    storage.replace_force_range(0..1, &[Vec2::new(2.0, 3.0)]);
    let has_pending_force = storage.solver_state.has_pending_system_force();
    let bundle = storage.into_buffer_bundle(ParticleBufferMode::Fixed { capacity: 8 });
    let lanes = bundle.into_lanes();

    // Assert
    assert!(has_pending_force);
    assert_eq!(lanes.positions(), &[Vec2::new(1.0, -1.0)]);
    let consumer_source = include_str!("../../buffer.rs");
    for derived in [
        "static_pressures",
        "tensile_accumulations",
        "depths",
        "aggregate_group_flags",
        "pending_system_force",
    ] {
        assert!(
            !consumer_source.contains(derived),
            "{derived} must remain storage-owned"
        );
    }
}
