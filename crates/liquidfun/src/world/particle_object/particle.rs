use crate::collision::Shape;
use crate::math::Transform;
use crate::particle::ParticleQueryError;
use crate::world::query::{QueryDirective, WorldQueryOccurrence};

use super::{
    CreateObjectError, DestroyedId, DestructionCause, DestructionRecord, HandleError,
    HandleIdentity, ObjectSnapshot, ParticleCreationReceipt, ParticleDef, ParticleGroupId,
    ParticleId, ParticleInput, ParticleSnapshot, ParticleSystemId, StorageParticleSnapshot, World,
    particle_input, particle_lifecycle_creation_error, public_particle_snapshot, snapshot_system,
    storage_handle_error, storage_object_creation_error,
};

impl World {
    /// Creates one stable particle from a checked definition.
    ///
    /// Application association values remain application-owned and can be
    /// paired with the returned ID in [`crate::AssociationMap`].
    ///
    /// # Errors
    ///
    /// Returns a scoped owner error, capacity error, or identity exhaustion
    /// before any particle row is committed.
    pub fn create_particle_with_def<UserAssociation>(
        &mut self,
        system: ParticleSystemId,
        maybe_group: Option<ParticleGroupId>,
        definition: &ParticleDef<UserAssociation>,
    ) -> Result<ParticleCreationReceipt, CreateObjectError> {
        self.ensure_not_poisoned_for_handle()?;
        self.particle_systems.get(system)?;
        if let Some(group) = maybe_group {
            let group_record = self.particle_groups.get(group)?;
            if group_record.system != system {
                return Err(CreateObjectError::InvalidHandle(
                    HandleError::WrongParticleSystem,
                ));
            }
        }
        let input = particle_input(definition, maybe_group);
        let record = self.particle_systems.get(system)?;
        let occupied = record.storage.len();
        record
            .lifetime
            .reject_blocked_capacity(occupied)
            .map_err(particle_lifecycle_creation_error)?;
        let free_slots = usize::from(record.lifetime.creation_frees_a_slot(occupied));
        record
            .storage
            .validate_create_reserving(input, free_slots)
            .map_err(storage_object_creation_error)?;
        record
            .lifetime
            .validate_created_lifetime(&record.storage, definition.lifetime())?;
        let diagnostic_id = self.allocate_diagnostic_id()?;
        Ok(self.commit_preflighted_particle(system, input, definition.lifetime(), diagnostic_id))
    }

    /// Drops the oldest particles so `additional` creations fit, in one compaction.
    ///
    /// Emit loops should call this before creating that batch.
    ///
    /// # Errors
    ///
    /// Returns a scoped owner error, or a capacity error when the system cannot
    /// evict by age.
    pub fn reserve_particle_creations(
        &mut self,
        system: ParticleSystemId,
        additional: usize,
    ) -> Result<(), CreateObjectError> {
        self.ensure_not_poisoned_for_handle()?;
        self.particle_systems.get(system)?;
        if additional == 0 {
            return Ok(());
        }
        let record = self.system_mut_after_validation(system);
        record
            .lifetime
            .evict_oldest_for_creations(&mut record.storage, additional)
            .map_err(particle_lifecycle_creation_error)?;
        Ok(())
    }

    pub(super) fn commit_preflighted_particle(
        &mut self,
        system: ParticleSystemId,
        input: ParticleInput,
        lifetime: f32,
        diagnostic_id: u64,
    ) -> ParticleCreationReceipt {
        let record = self.system_mut_after_validation(system);
        let maybe_compaction = record
            .lifetime
            .prepare_capacity_for_creation(&mut record.storage)
            .expect("preflighted capacity decision remains valid until immediate commit");
        let particle = record
            .storage
            .create_with_diagnostic(input, diagnostic_id)
            .expect("preflighted particle candidate remains valid until immediate commit");
        record
            .lifetime
            .initialize_created_particle(&mut record.storage, particle, lifetime)
            .expect("preflighted lifetime remains valid until immediate commit");
        ParticleCreationReceipt {
            created_particle: particle,
            destruction_occurrences: maybe_compaction
                .map_or_else(Vec::new, |outcome| outcome.requested_listener_occurrences),
        }
    }

    /// Returns owned semantic state after validating a particle's embedded owner.
    ///
    /// # Errors
    ///
    /// Wrong-world, stale-owner, pending-delete, and stale particle states are
    /// reported distinctly.
    pub fn particle_snapshot(&self, particle: ParticleId) -> Result<ParticleSnapshot, HandleError> {
        let system = self.particle_system_id_for_particle(particle)?;
        self.particle_snapshot_in_system(system, particle)
    }

    /// Returns owned semantic state while requiring one explicit owning system.
    ///
    /// # Errors
    ///
    /// In addition to ordinary scoped failures, returns
    /// [`HandleError::WrongParticleSystem`] when the particle belongs elsewhere.
    pub fn particle_snapshot_in_system(
        &self,
        system: ParticleSystemId,
        particle: ParticleId,
    ) -> Result<ParticleSnapshot, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.particle_systems.get(system)?;
        record
            .storage
            .snapshot(particle)
            .map(public_particle_snapshot)
            .map_err(storage_handle_error)
    }

    /// Marks a live particle pending-delete while preserving an owned snapshot.
    ///
    /// # Errors
    ///
    /// Returns a distinct pending-delete error for a repeated mark.
    pub fn mark_particle_for_destruction(
        &mut self,
        particle: ParticleId,
    ) -> Result<ParticleSnapshot, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let system = self.particle_system_id_for_particle(particle)?;
        self.system_mut_after_validation(system)
            .storage
            .mark_delete(particle)
            .map(public_particle_snapshot)
            .map_err(storage_handle_error)
    }

    /// Destroys particles of `system` whose positions pass `shape.test_point`.
    ///
    /// Queries the shape AABB, keeps only particles owned by `system` that pass
    /// the point test, marks them pending-delete, then compacts so the pocket is
    /// empty before the caller presents a frame.
    ///
    /// # Errors
    ///
    /// Returns a typed particle-query error when the system handle is invalid,
    /// AABB derivation fails, or compaction cannot proceed.
    pub fn destroy_particles_in_shape(
        &mut self,
        system: ParticleSystemId,
        shape: &Shape,
        transform: Transform,
    ) -> Result<usize, ParticleQueryError> {
        self.ensure_not_poisoned_for_handle()?;
        self.particle_systems.get(system)?;

        let mut candidates = Vec::new();
        for child_slot in 0..shape.child_count() {
            let child = shape
                .child_index(child_slot)
                .map_err(|_error| ParticleQueryError::NonFiniteDerivedGeometry)?;
            let aabb = shape
                .compute_aabb(transform, child)
                .map_err(|_error| ParticleQueryError::NonFiniteDerivedGeometry)?;
            self.query_aabb_with_particles(aabb, |occurrence| {
                if let WorldQueryOccurrence::Particle(hit) = occurrence {
                    if hit.system() == system {
                        candidates.push(hit.particle());
                    }
                }
                QueryDirective::Continue
            })?;
        }

        let mut marked = 0usize;
        for particle in candidates {
            let Ok(snapshot) = self.particle_snapshot(particle) else {
                continue;
            };
            let Ok(true) = shape.test_point(transform, snapshot.position()) else {
                continue;
            };
            if self.mark_particle_for_destruction(particle).is_ok() {
                marked += 1;
            }
        }

        if marked > 0 {
            self.compact_pending_particles(system)?;
        }
        Ok(marked)
    }

    pub(in crate::world) fn destroy_particle_now(
        &mut self,
        particle: ParticleId,
    ) -> Result<DestructionRecord, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let system = self.particle_system_id_for_particle(particle)?;
        let storage = &mut self.system_mut_after_validation(system).storage;
        storage
            .mark_delete(particle)
            .map_err(storage_handle_error)?;
        let snapshot = storage
            .compact_particle(particle)
            .expect("the particle marked by this operation remains pending until commit");
        Ok(Self::particle_destruction_record(
            snapshot,
            DestructionCause::Explicit,
        ))
    }

    pub(in crate::world) fn particle_destruction_record(
        snapshot: StorageParticleSnapshot,
        cause: DestructionCause,
    ) -> DestructionRecord {
        DestructionRecord {
            destroyed: DestroyedId::Particle(snapshot.id),
            diagnostic_id: snapshot.diagnostic_id,
            cause,
            snapshot: ObjectSnapshot::Particle {
                system: snapshot_system(snapshot),
                maybe_group: snapshot.input.maybe_group,
            },
        }
    }

    pub(super) fn particle_system_id_for_particle(
        &self,
        particle: ParticleId,
    ) -> Result<ParticleSystemId, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let identity = particle.identity();
        if identity.world() != self.scope_key {
            return Err(HandleError::WrongWorld);
        }
        let Some(system_scope) = identity.maybe_particle_system() else {
            return Err(HandleError::WrongParticleSystem);
        };
        self.particle_system_order
            .iter()
            .copied()
            .find(|system| system.identity().scope() == system_scope)
            .ok_or(HandleError::StaleOrDestroyed)
    }

    pub(super) fn debug_assert_particle_system_order_invariant(&self) {
        debug_assert_eq!(
            self.particle_system_order.len(),
            self.particle_systems.iter().count()
        );
        debug_assert!(
            self.particle_system_order
                .iter()
                .all(|system| self.particle_systems.get(*system).is_ok())
        );
    }
}

#[cfg(test)]
mod destroy_in_shape_tests {
    use crate::collision::{CircleShape, Shape};
    use crate::math::{Transform, Vec2};
    use crate::particle::{ParticleDef, ParticleSystemDef};
    use crate::world::object::World;

    fn count_particles_inside(world: &World, shape: &Shape, transform: Transform) -> usize {
        let Ok(view) = world.particle_system_view(
            world
                .particle_system_ids()
                .into_iter()
                .next()
                .expect("test world has one system"),
        ) else {
            return 0;
        };
        view.positions()
            .iter()
            .filter(|position| {
                shape
                    .test_point(transform, **position)
                    .expect("finite test point")
            })
            .count()
    }

    #[test]
    fn destroy_particles_in_shape_clears_pocket_under_circle() {
        // Arrange
        let mut world = World::new().expect("world key remains available");
        let system = world
            .create_particle_system_with_def(
                &ParticleSystemDef::default()
                    .with_radius(0.05)
                    .expect("radius is valid"),
            )
            .expect("particle system fits");
        for x in [-0.3, -0.15, 0.0, 0.15, 0.3] {
            for y in [-0.3, -0.15, 0.0, 0.15, 0.3] {
                let definition = ParticleDef::default()
                    .with_position(Vec2::new(x, y))
                    .expect("finite position");
                let _ = world
                    .create_particle_with_def(system, None, &definition)
                    .expect("particle fits");
            }
        }
        let shape = Shape::from(
            CircleShape::new(Vec2::ZERO, 0.2).expect("circle geometry is valid"),
        );
        let transform = Transform::IDENTITY;
        let inside_before = count_particles_inside(&world, &shape, transform);
        assert!(
            inside_before > 0,
            "fixture must overlap at least one particle before carve"
        );

        // Act
        let destroyed = world
            .destroy_particles_in_shape(system, &shape, transform)
            .expect("carve should succeed");

        // Assert
        assert!(destroyed > 0, "carve must remove at least one particle");
        assert_eq!(
            count_particles_inside(&world, &shape, transform),
            0,
            "pocket under the circle must be empty after destroy_particles_in_shape"
        );
    }
}
