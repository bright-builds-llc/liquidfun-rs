use crate::identity::{BodyId, FixtureId, Identity};

use super::*;

#[test]
fn position_edit_rebuilds_or_clears_every_spatially_derived_lane() {
    // Arrange
    let world = WorldKey::fresh().expect("test world key remains available");
    let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    let mut storage =
        ParticleStorage::new(world, system, 0, 4, 4).expect("test storage contract is valid");
    let first = storage
        .create(test_input(1.0))
        .expect("first particle fits");
    storage
        .create(test_input(2.0))
        .expect("second particle fits");
    storage
        .create(test_input(3.0))
        .expect("third particle fits");
    let indices = [ParticleIndex(0), ParticleIndex(1)];
    storage.weights.fill(2.0);
    storage.particle_contacts.push(ParticleContact {
        indices,
        flags: ParticleFlags::WATER,
        weight: 1.0,
        normal: Vec2::new(1.0, 0.0),
    });
    storage.body_contacts.push(ParticleBodyContact {
        index: ParticleIndex(0),
        body: BodyId::from_identity(Identity::new(world, 1, 0)),
        fixture: FixtureId::from_identity(Identity::new(world, 2, 0)),
        weight: 1.0,
        normal: Vec2::new(0.0, 1.0),
        mass: 1.0,
    });
    storage.pairs.push(ParticlePair {
        indices,
        flags: ParticleFlags::SPRING,
        strength: 1.0,
        distance: 1.0,
    });
    storage.triads.push(ParticleTriad {
        indices: [ParticleIndex(0), ParticleIndex(1), ParticleIndex(2)],
        flags: ParticleFlags::ELASTIC,
        strength: 1.0,
        pa: Vec2::new(1.0, 0.0),
        pb: Vec2::new(0.0, 1.0),
        pc: Vec2::new(-1.0, -1.0),
        ka: -1.0,
        kb: 1.0,
        kc: -1.0,
        s: 1.0,
    });
    storage.maybe_stuck = Some(StuckLanes {
        last_body_contact_steps: vec![1; 3],
        body_contact_counts: vec![1; 3],
        consecutive_contact_steps: vec![1; 3],
        candidates: vec![ParticleIndex(0)],
    });

    // Act
    storage
        .commit_kinematic_edit(first, Vec2::new(9.0, 8.0), Vec2::new(7.0, 6.0))
        .expect("live finite candidate commits");

    // Assert
    assert_eq!(storage.positions[0], Vec2::new(9.0, 8.0));
    assert_eq!(storage.velocities[0], Vec2::new(7.0, 6.0));
    assert_eq!(storage.weights, vec![0.0; 3]);
    assert_eq!(
        storage.proxies,
        (0..3)
            .map(|index| ParticleProxy::new(ParticleIndex(index)))
            .collect::<Vec<_>>()
    );
    assert!(storage.particle_contacts.is_empty());
    assert!(storage.body_contacts.is_empty());
    assert!(storage.pairs.is_empty());
    assert!(storage.triads.is_empty());
    let stuck = storage.maybe_stuck.expect("stuck lanes remain allocated");
    assert_eq!(stuck.last_body_contact_steps, vec![0; 3]);
    assert_eq!(stuck.body_contact_counts, vec![0; 3]);
    assert_eq!(stuck.consecutive_contact_steps, vec![0; 3]);
    assert!(stuck.candidates.is_empty());
}

fn test_input(value: f32) -> ParticleInput {
    ParticleInput {
        position: Vec2::new(value, 0.0),
        velocity: Vec2::ZERO,
        flags: ParticleFlags::WATER,
        maybe_group: None,
        maybe_color: None,
        maybe_user_association: None,
        maybe_expiration_time: None,
    }
}
