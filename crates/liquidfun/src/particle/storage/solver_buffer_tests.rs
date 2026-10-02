//! Validated swaps preserve failure state and return displaced ownership.
use super::*;

#[test]
fn invalid_lane_swap_is_atomic_and_valid_swap_returns_old_vectors() {
    // Arrange
    let world = WorldKey::fresh().expect("world");
    let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    let mut storage = ParticleStorage::new(world, system, 0, 4, 4).expect("storage");
    storage
        .create(ParticleInput {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            flags: ParticleFlags::WATER,
            maybe_group: None,
            maybe_color: None,
            maybe_user_association: None,
            maybe_expiration_time: None,
        })
        .expect("particle");
    let ids = storage.particle_ids().to_vec();
    let old_pointer = storage.positions().as_ptr();
    let mut positions = vec![Vec2::new(2.0, 3.0)];
    let mut velocities = Vec::new();
    let mut forces = vec![Vec2::ZERO];
    let mut groups = Vec::new();

    // Act
    let failed = storage.swap_solver_candidate(
        &ids,
        &mut positions,
        &mut velocities,
        &mut forces,
        &mut groups,
        false,
    );

    // Assert
    assert_eq!(failed, Err(ParticleStorageError::InvalidLaneBundle));
    assert_eq!(storage.positions(), [Vec2::ZERO]);
    assert_eq!(storage.positions().as_ptr(), old_pointer);
    assert_eq!(positions, [Vec2::new(2.0, 3.0)]);
    velocities.push(Vec2::new(1.0, 0.0));
    storage
        .swap_solver_candidate(
            &ids,
            &mut positions,
            &mut velocities,
            &mut forces,
            &mut groups,
            false,
        )
        .expect("validated commit");
    assert_eq!(storage.positions(), [Vec2::new(2.0, 3.0)]);
    assert_eq!(positions, [Vec2::ZERO]);
    assert_eq!(positions.as_ptr(), old_pointer);
}
