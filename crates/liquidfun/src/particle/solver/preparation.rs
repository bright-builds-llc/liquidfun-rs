//! Exact S01-S06 and S13 particle-solver preparation kernels.

use crate::math::{Vec2, settings};
use crate::particle::body_contact::ParticleBodyContact;
use crate::particle::contact::ParticleContact;
use crate::particle::definition::ParticleSystemDef;
use crate::particle::storage::{ParticleStorage, ParticleStorageError};
use crate::particle::topology::VoronoiLimits;

/// S01 `ParticleContacts`: commit contacts prepared by the Phase 9 authority.
pub(super) fn particle_contacts(
    storage: &mut ParticleStorage,
    contacts: &[ParticleContact],
) -> Result<(), ParticleStorageError> {
    storage.replace_particle_contacts(contacts)
}

/// S02 `BodyContacts`: commit contacts prepared by the Phase 9 authority.
pub(super) fn body_contacts(
    storage: &mut ParticleStorage,
    contacts: &[ParticleBodyContact],
    timestamp: u32,
    stuck_threshold: u32,
) -> Result<(), ParticleStorageError> {
    storage.replace_body_contacts(contacts)?;
    storage.update_stuck_candidates(timestamp, stuck_threshold);
    Ok(())
}

/// S03 `Weight`: derive density weights in body-then-particle contact order.
pub(super) fn weight(storage: &mut ParticleStorage) {
    storage.refresh_solver_weights();
}

/// S04 `SolidDepth`: run the storage-owned, scheduled depth transaction.
pub(super) fn solid_depth(
    storage: &mut ParticleStorage,
    particle_diameter: f32,
) -> Result<(), ParticleStorageError> {
    storage.compute_solid_depth(particle_diameter)
}

/// S05 `ReactiveTopology`: run the storage-owned append-and-clear transaction.
pub(super) fn reactive_topology(
    storage: &mut ParticleStorage,
    particle_diameter: f32,
    voronoi_limits: VoronoiLimits,
) -> Result<(), ParticleStorageError> {
    storage.regenerate_reactive_topology(particle_diameter, voronoi_limits)
}

/// S06 `Force`: consume the pending-force marker after one complete update.
pub(super) fn force(
    storage: &mut ParticleStorage,
    definition: ParticleSystemDef,
    time_step: f32,
) -> Result<(), ParticleStorageError> {
    if !storage.has_pending_system_force() {
        return Ok(());
    }
    if !time_step.is_finite() {
        return Err(ParticleStorageError::InvalidLaneBundle);
    }

    let velocity_per_force = time_step * particle_inverse_mass(definition);
    if !velocity_per_force.is_finite() {
        return Err(ParticleStorageError::InvalidLaneBundle);
    }
    let mut velocities = Vec::new();
    velocities
        .try_reserve_exact(storage.len())
        .map_err(|_error| ParticleStorageError::InvalidLaneBundle)?;
    velocities.extend(
        storage
            .velocities()
            .iter()
            .copied()
            .zip(storage.forces())
            .map(|(velocity, accumulated_force)| {
                velocity + velocity_per_force * *accumulated_force
            }),
    );
    storage.replace_solver_velocities(velocities)?;
    storage.clear_pending_system_force();
    Ok(())
}

/// S13 `Gravity`: apply the source-ordered substep gravity increment.
pub(super) fn gravity(
    storage: &mut ParticleStorage,
    definition: ParticleSystemDef,
    time_step: f32,
    world_gravity: Vec2,
) -> Result<(), ParticleStorageError> {
    if storage.len() == 0 || time_step == 0.0 {
        return Ok(());
    }
    if !time_step.is_finite() || !world_gravity.is_valid() {
        return Err(ParticleStorageError::InvalidLaneBundle);
    }

    let gravity_delta = time_step * definition.gravity_scale() * world_gravity;
    if !gravity_delta.is_valid() {
        return Err(ParticleStorageError::InvalidLaneBundle);
    }
    let mut velocities = Vec::new();
    velocities
        .try_reserve_exact(storage.len())
        .map_err(|_error| ParticleStorageError::InvalidLaneBundle)?;
    velocities.extend(
        storage
            .velocities()
            .iter()
            .copied()
            .map(|velocity| velocity + gravity_delta),
    );
    storage.replace_solver_velocities(velocities)
}

fn particle_inverse_mass(definition: ParticleSystemDef) -> f32 {
    let inverse_diameter = 1.0 / (2.0 * definition.radius());
    let inverse_stride = inverse_diameter * (1.0 / settings::PARTICLE_STRIDE);
    (1.0 / definition.density()) * inverse_stride * inverse_stride
}

#[cfg(test)]
mod tests {
    use crate::identity::{
        BodyId, FixtureId, HandleIdentity, Identity, ParticleId, ParticleSystemId, WorldKey,
    };
    use crate::particle::storage::ParticleInput;
    use crate::particle::{ParticleFlags, ParticleGroupFlags};

    use super::*;
    use crate::particle::solver::PassId;
    use crate::particle::solver::manifest::PASS_GRAPH;

    struct Fixture {
        world: WorldKey,
        storage: ParticleStorage,
        particles: Vec<ParticleId>,
    }

    impl Fixture {
        fn new(inputs: &[(Vec2, Vec2, ParticleFlags, Option<usize>)]) -> Self {
            let world = WorldKey::fresh().expect("test world key remains available");
            let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
            let capacity = inputs.len().max(8);
            let mut storage = ParticleStorage::new(world, system, 32, capacity, capacity)
                .expect("test storage contract is valid");
            let particles = inputs
                .iter()
                .copied()
                .map(|(position, velocity, flags, maybe_group_slot)| {
                    let maybe_group = maybe_group_slot.map(|slot| {
                        crate::ParticleGroupId::from_identity(Identity::new(world, slot, 0))
                    });
                    storage
                        .create(ParticleInput {
                            position,
                            velocity,
                            flags,
                            maybe_group,
                            maybe_color: None,
                            maybe_user_association: None,
                            maybe_expiration_time: None,
                        })
                        .expect("test particle fits")
                })
                .collect();
            Self {
                world,
                storage,
                particles,
            }
        }

        fn particle_contact(&self, indices: [usize; 2], weight: f32) -> ParticleContact {
            ParticleContact::new_internal(
                indices.map(|index| self.particles[index]),
                indices.iter().fold(ParticleFlags::empty(), |flags, index| {
                    flags | self.storage.flags()[*index]
                }),
                weight,
                Vec2::new(1.0, 0.0),
            )
        }

        fn body_contact(
            &self,
            particle: usize,
            body_slot: usize,
            weight: f32,
        ) -> ParticleBodyContact {
            ParticleBodyContact::new_internal(
                self.particles[particle],
                BodyId::from_identity(Identity::new(self.world, body_slot, 0)),
                FixtureId::from_identity(Identity::new(self.world, body_slot + 8, 0)),
                weight,
                Vec2::new(0.0, 1.0),
                2.0,
            )
        }
    }

    fn water(position_x: f32) -> (Vec2, Vec2, ParticleFlags, Option<usize>) {
        (
            Vec2::new(position_x, 0.0),
            Vec2::ZERO,
            ParticleFlags::WATER,
            None,
        )
    }

    fn limits() -> VoronoiLimits {
        VoronoiLimits::new(64, 4_096, 16_384, 2_000_000, 8_192)
    }

    fn vec_bits(values: &[Vec2]) -> Vec<[u32; 2]> {
        values
            .iter()
            .map(|value| [value.x.to_bits(), value.y.to_bits()])
            .collect()
    }

    #[test]
    fn manifest_admits_each_preparation_kernel_exactly_once() {
        // Arrange
        let preparation_ids = [
            PassId::ParticleContacts,
            PassId::BodyContacts,
            PassId::Weight,
            PassId::SolidDepth,
            PassId::ReactiveTopology,
            PassId::Force,
            PassId::Gravity,
        ];

        // Act
        let counts = preparation_ids.map(|id| {
            PASS_GRAPH
                .iter()
                .filter(|descriptor| descriptor.id == id)
                .count()
        });

        // Assert
        assert_eq!(counts, [1; 7]);
    }

    #[test]
    fn particle_contacts_replace_in_candidate_order_and_empty_is_a_control() {
        // Arrange
        let mut fixture = Fixture::new(&[water(0.0), water(1.0), water(2.0)]);
        let ordered = [
            fixture.particle_contact([2, 1], 0.25),
            fixture.particle_contact([0, 1], 0.75),
        ];

        // Act
        particle_contacts(&mut fixture.storage, &ordered)
            .expect("source-ordered live contacts should commit");
        let committed = fixture.storage.semantic_particle_contacts();
        particle_contacts(&mut fixture.storage, &[]).expect("empty contact refresh should commit");

        // Assert
        assert_eq!(committed, ordered);
        assert!(fixture.storage.particle_contacts().is_empty());
    }

    #[test]
    fn particle_contact_failure_is_an_exact_no_diff_with_relative_slots() {
        // Arrange
        let mut fixture = Fixture::new(&[water(0.0), water(1.0)]);
        let foreign_world = WorldKey::fresh().expect("foreign test world remains available");
        let foreign = ParticleId::from_identity(Identity::new(foreign_world, 32, 0));
        let invalid = ParticleContact::new_internal(
            [fixture.particles[0], foreign],
            ParticleFlags::WATER,
            0.5,
            Vec2::new(1.0, 0.0),
        );
        let before = fixture.storage.clone();

        // Act
        let result = particle_contacts(&mut fixture.storage, &[invalid]);

        // Assert
        assert_eq!(result, Err(ParticleStorageError::WrongWorld));
        assert!(fixture.storage == before);
        assert_eq!(fixture.particles[0].identity().slot(), 32);
    }

    #[test]
    fn body_contacts_update_stuck_state_without_reordering_contacts() {
        // Arrange
        let mut fixture = Fixture::new(&[water(0.0), water(1.0)]);
        let ordered = [
            fixture.body_contact(1, 3, 0.25),
            fixture.body_contact(0, 2, 0.75),
        ];

        // Act
        body_contacts(&mut fixture.storage, &ordered, 1, 1)
            .expect("source-ordered live body contacts should commit");

        // Assert
        assert_eq!(fixture.storage.semantic_body_contacts(), ordered);
        assert_eq!(fixture.storage.stuck_candidates().count(), 0);
    }

    #[test]
    fn weight_accumulates_body_rows_before_particle_rows() {
        // Arrange
        let mut fixture = Fixture::new(&[water(0.0), water(1.0)]);
        let particle = [fixture.particle_contact([0, 1], 0.75)];
        let body = [fixture.body_contact(0, 2, 0.5)];
        particle_contacts(&mut fixture.storage, &particle).expect("particle contact commits");
        body_contacts(&mut fixture.storage, &body, 1, 0).expect("body contact commits");

        // Act
        weight(&mut fixture.storage);

        // Assert
        assert_eq!(
            fixture
                .storage
                .weights()
                .iter()
                .map(|weight| weight.to_bits())
                .collect::<Vec<_>>(),
            [1.25_f32.to_bits(), 0.75_f32.to_bits()]
        );
    }

    #[test]
    fn solid_depth_mixed_groups_update_only_scheduled_members() {
        // Arrange
        let mut fixture = Fixture::new(&[
            (
                Vec2::new(0.0, 0.0),
                Vec2::ZERO,
                ParticleFlags::WATER,
                Some(1),
            ),
            (
                Vec2::new(1.0, 0.0),
                Vec2::ZERO,
                ParticleFlags::WATER,
                Some(1),
            ),
            (
                Vec2::new(2.0, 0.0),
                Vec2::ZERO,
                ParticleFlags::WATER,
                Some(2),
            ),
        ]);
        let actual_first =
            fixture.storage.groups()[0].expect("first test particle belongs to a group");
        fixture
            .storage
            .set_group_flags_internal(actual_first, ParticleGroupFlags::SOLID)
            .expect("solid transition schedules depth");
        let contact = [fixture.particle_contact([0, 1], 0.9)];
        particle_contacts(&mut fixture.storage, &contact).expect("group contact commits");

        // Act
        solid_depth(&mut fixture.storage, 1.0).expect("scheduled depth should compute");

        // Assert
        assert_eq!(
            fixture.storage.maybe_depths(),
            Some([0.0, 0.0, 0.0].as_slice())
        );
    }

    #[test]
    fn reactive_topology_activates_once_and_clears_only_marked_flags() {
        // Arrange
        let mut fixture = Fixture::new(&[
            (
                Vec2::new(0.0, 0.0),
                Vec2::ZERO,
                ParticleFlags::SPRING | ParticleFlags::REACTIVE,
                None,
            ),
            (Vec2::new(1.0, 0.0), Vec2::ZERO, ParticleFlags::SPRING, None),
        ]);
        let contacts = [fixture.particle_contact([0, 1], 0.5)];
        particle_contacts(&mut fixture.storage, &contacts).expect("reactive contact commits");

        // Act
        reactive_topology(&mut fixture.storage, 1.0, limits())
            .expect("reactive topology should commit");

        // Assert
        assert_eq!(fixture.storage.pairs().len(), 1);
        assert!(
            fixture
                .storage
                .flags()
                .iter()
                .all(|flags| !flags.contains(ParticleFlags::REACTIVE))
        );
    }

    #[test]
    fn force_applies_once_and_inactive_second_call_is_an_exact_no_diff() {
        // Arrange
        let mut fixture = Fixture::new(&[water(0.0), water(1.0)]);
        fixture
            .storage
            .replace_force_range(0..2, &[Vec2::new(2.0, -1.0), Vec2::new(-3.0, 4.0)]);
        let definition = ParticleSystemDef::default();

        // Act
        force(&mut fixture.storage, definition, 0.25).expect("finite pending force should solve");
        let after_first = fixture.storage.clone();
        force(&mut fixture.storage, definition, 0.25).expect("consumed force should be inactive");

        // Assert
        assert!(fixture.storage == after_first);
        assert!(!fixture.storage.has_pending_system_force());
        assert_ne!(
            vec_bits(fixture.storage.velocities()),
            vec_bits(&[Vec2::ZERO; 2])
        );
    }

    #[test]
    fn invalid_force_candidate_preserves_pending_state_and_velocities() {
        // Arrange
        let mut fixture = Fixture::new(&[water(0.0)]);
        fixture
            .storage
            .replace_force_range(0..1, &[Vec2::new(f32::MAX, 0.0)]);
        let before = fixture.storage.clone();

        // Act
        let result = force(&mut fixture.storage, ParticleSystemDef::default(), f32::MAX);

        // Assert
        assert_eq!(result, Err(ParticleStorageError::InvalidLaneBundle));
        assert!(fixture.storage == before);
    }

    #[test]
    fn zero_timestep_consumes_force_without_changing_velocity_bits() {
        // Arrange
        let mut fixture = Fixture::new(&[water(0.0)]);
        fixture
            .storage
            .replace_force_range(0..1, &[Vec2::new(3.0, -4.0)]);
        let before_velocity = vec_bits(fixture.storage.velocities());

        // Act
        force(&mut fixture.storage, ParticleSystemDef::default(), 0.0)
            .expect("zero-step force still consumes the pending marker");

        // Assert
        assert_eq!(vec_bits(fixture.storage.velocities()), before_velocity);
        assert!(!fixture.storage.has_pending_system_force());
    }

    #[test]
    fn gravity_uses_substep_scale_for_all_mixed_flags() {
        // Arrange
        let mut fixture = Fixture::new(&[
            water(0.0),
            (
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, -2.0),
                ParticleFlags::WALL,
                None,
            ),
        ]);
        let definition = ParticleSystemDef::default()
            .with_gravity_scale(0.5)
            .expect("finite gravity scale is valid");

        // Act
        gravity(&mut fixture.storage, definition, 0.25, Vec2::new(0.0, -8.0))
            .expect("finite gravity should solve");

        // Assert
        assert_eq!(
            fixture.storage.velocities(),
            &[Vec2::new(0.0, -1.0), Vec2::new(1.0, -3.0)]
        );
    }

    #[test]
    fn zero_timestep_and_empty_gravity_are_exact_controls() {
        // Arrange
        let mut fixture =
            Fixture::new(&[(Vec2::ZERO, Vec2::new(-0.0, 3.0), ParticleFlags::WATER, None)]);
        let before = fixture.storage.clone();
        let mut empty = Fixture::new(&[]);
        let empty_before = empty.storage.clone();

        // Act
        gravity(
            &mut fixture.storage,
            ParticleSystemDef::default(),
            0.0,
            Vec2::new(0.0, -10.0),
        )
        .expect("zero time step is inactive");
        particle_contacts(&mut empty.storage, &[])
            .expect("empty particle-contact refresh is inactive");
        body_contacts(&mut empty.storage, &[], 1, 0)
            .expect("empty body-contact refresh is inactive");
        weight(&mut empty.storage);
        gravity(
            &mut empty.storage,
            ParticleSystemDef::default(),
            1.0,
            Vec2::new(0.0, -10.0),
        )
        .expect("empty storage is inactive");

        // Assert
        assert!(fixture.storage == before);
        assert!(empty.storage == empty_before);
    }

    #[test]
    fn retained_empty_group_does_not_activate_depth() {
        // Arrange
        let mut fixture = Fixture::new(&[(Vec2::ZERO, Vec2::ZERO, ParticleFlags::WATER, Some(1))]);
        let group = fixture.storage.groups()[0].expect("test particle belongs to its group");
        fixture
            .storage
            .set_group_flags_internal(group, ParticleGroupFlags::CAN_BE_EMPTY)
            .expect("retained-empty policy is valid");
        fixture
            .storage
            .mark_delete(fixture.particles[0])
            .expect("test particle is live");
        fixture
            .storage
            .compact_pending()
            .expect("retained group member compacts");
        assert_eq!(fixture.storage.len(), 0);
        let before = fixture.storage.clone();

        // Act
        solid_depth(&mut fixture.storage, 1.0)
            .expect("storage without a scheduled nonempty group is inactive");

        // Assert
        assert!(fixture.storage == before);
    }
}
