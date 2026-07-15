//! Particle-system object lifecycle over one authoritative storage owner.

use crate::identity::HandleIdentity;
use crate::math::Vec2;
use crate::particle::lifetime::{ParticleLifecycleError, ParticleLifetimeState};
use crate::particle::storage::{
    ParticleInput, ParticleSnapshot as StorageParticleSnapshot, ParticleStorage,
    ParticleStorageError,
};
use crate::particle::{
    ParticleBufferAdoptionError, ParticleBufferAdoptionErrorKind, ParticleBufferBundle,
    ParticleCapacity, ParticleColor, ParticleDef, ParticleEditError, ParticleEditor, ParticleFlags,
    ParticleSystemDef, ParticleSystemView,
};
use crate::{
    ArenaInsertError, CreateObjectError, DestroyedId, DestructionCause, DestructionRecord,
    DestructionReport, HandleError, LifecycleEvent, MutationReport, ObjectSnapshot,
    ParticleGroupId, ParticleId, ParticleSystemId,
};

use super::object::{ParticleSystem, World};

const MAX_PARTICLE_COUNT: usize = i32::MAX as usize;

/// Owned configuration and membership state for one live particle system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleSystemSnapshot {
    definition: ParticleSystemDef,
    particle_count: usize,
    pending_particle_count: usize,
}

impl ParticleSystemSnapshot {
    /// Returns the checked system definition currently in force.
    #[must_use]
    pub const fn definition(self) -> ParticleSystemDef {
        self.definition
    }

    /// Returns whether stepping is paused for this system.
    #[must_use]
    pub const fn is_paused(self) -> bool {
        self.definition.is_paused()
    }

    /// Returns the number of live and pending rows owned by the system.
    #[must_use]
    pub const fn particle_count(self) -> usize {
        self.particle_count
    }

    /// Returns the number of rows marked for later destruction.
    #[must_use]
    pub const fn pending_particle_count(self) -> usize {
        self.pending_particle_count
    }
}

/// Owned semantic state for one live particle identity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleSnapshot {
    id: ParticleId,
    system: ParticleSystemId,
    maybe_group: Option<ParticleGroupId>,
    position: Vec2,
    velocity: Vec2,
    flags: ParticleFlags,
    color: ParticleColor,
}

impl ParticleSnapshot {
    /// Returns the stable particle identity.
    #[must_use]
    pub const fn id(self) -> ParticleId {
        self.id
    }

    /// Returns the owning particle system.
    #[must_use]
    pub const fn system(self) -> ParticleSystemId {
        self.system
    }

    /// Returns current group membership, when present.
    #[must_use]
    pub const fn maybe_group(self) -> Option<ParticleGroupId> {
        self.maybe_group
    }

    /// Returns position in meters.
    #[must_use]
    pub const fn position(self) -> Vec2 {
        self.position
    }

    /// Returns velocity in meters per second.
    #[must_use]
    pub const fn velocity(self) -> Vec2 {
        self.velocity
    }

    /// Returns exact retained particle flag bits.
    #[must_use]
    pub const fn flags(self) -> ParticleFlags {
        self.flags
    }

    /// Returns the exact particle color.
    #[must_use]
    pub const fn color(self) -> ParticleColor {
        self.color
    }
}

impl World {
    /// Creates a particle system that uniquely owns a validated consumer lane bundle.
    ///
    /// The bundle's fixed or growable mode replaces the definition's ordinary
    /// capacity policy. A configured maximum must still fit a fixed bundle.
    ///
    /// # Errors
    ///
    /// Returns the complete bundle with a typed definition or world allocation
    /// failure. No system, diagnostic identity, or lane ownership is committed.
    pub fn create_particle_system_with_buffers(
        &mut self,
        definition: &ParticleSystemDef,
        bundle: ParticleBufferBundle,
    ) -> Result<ParticleSystemId, ParticleBufferAdoptionError> {
        let mode = bundle.mode();
        let capacity = ParticleCapacity::from_buffer_mode(mode);
        let definition = match definition.with_capacity(capacity) {
            Ok(definition) => definition,
            Err(error) => {
                return Err(ParticleBufferAdoptionError::new(
                    ParticleBufferAdoptionErrorKind::Definition(error),
                    bundle,
                ));
            }
        };
        if let Err(error) = self.ensure_not_poisoned_for_insert() {
            return Err(ParticleBufferAdoptionError::new(
                ParticleBufferAdoptionErrorKind::World(error),
                bundle,
            ));
        }
        let system = match self.particle_systems.next_handle() {
            Ok(system) => system,
            Err(error) => {
                return Err(ParticleBufferAdoptionError::new(
                    ParticleBufferAdoptionErrorKind::World(error),
                    bundle,
                ));
            }
        };
        let declared_capacity = if capacity.is_fixed() {
            capacity.count()
        } else {
            definition.maximum_count().unwrap_or(MAX_PARTICLE_COUNT)
        };
        let mut storage = ParticleStorage::from_buffer_bundle(
            self.scope_key,
            system,
            0,
            declared_capacity,
            declared_capacity,
            bundle,
        );
        let lifetime = ParticleLifetimeState::new(definition, &mut storage);
        let diagnostic_id = match self.allocate_diagnostic_id() {
            Ok(diagnostic_id) => diagnostic_id,
            Err(error) => {
                return Err(ParticleBufferAdoptionError::new(
                    ParticleBufferAdoptionErrorKind::World(error),
                    storage.into_buffer_bundle(mode),
                ));
            }
        };
        self.insert_particle_system_after_preflight(
            system,
            ParticleSystem {
                diagnostic_id,
                definition,
                groups: Vec::new(),
                storage,
                lifetime,
            },
        );
        self.particle_system_order.insert(0, system);
        self.debug_assert_particle_system_order_invariant();
        Ok(system)
    }

    fn insert_particle_system_after_preflight(
        &mut self,
        expected: ParticleSystemId,
        record: ParticleSystem,
    ) {
        let inserted = self
            .particle_systems
            .insert(record)
            .expect("preflighted particle-system slot remains available until insertion");
        debug_assert_eq!(inserted, expected);
    }

    /// Creates a particle system from a checked reusable definition.
    ///
    /// # Errors
    ///
    /// Returns an allocation error before insertion when world identities or
    /// particle-system storage are exhausted.
    pub fn create_particle_system_with_def(
        &mut self,
        definition: &ParticleSystemDef,
    ) -> Result<ParticleSystemId, ArenaInsertError> {
        self.ensure_not_poisoned_for_insert()?;
        let system = self.particle_systems.next_handle()?;
        let capacity = definition.capacity();
        let declared_capacity = if capacity.is_fixed() {
            capacity.count()
        } else {
            definition.maximum_count().unwrap_or(MAX_PARTICLE_COUNT)
        };
        let mut storage = ParticleStorage::with_initial_capacity(
            self.scope_key,
            system,
            0,
            declared_capacity,
            capacity.count(),
            declared_capacity,
        )
        .map_err(storage_creation_error)?;
        let lifetime = ParticleLifetimeState::new(*definition, &mut storage);
        let diagnostic_id = self.allocate_diagnostic_id()?;
        let inserted = self.particle_systems.insert(ParticleSystem {
            diagnostic_id,
            definition: *definition,
            groups: Vec::new(),
            storage,
            lifetime,
        })?;
        debug_assert_eq!(inserted, system);
        self.particle_system_order.insert(0, system);
        self.debug_assert_particle_system_order_invariant();
        Ok(system)
    }

    /// Enumerates live systems in pinned newest-first world order.
    #[must_use]
    pub fn particle_system_ids(
        &self,
    ) -> impl ExactSizeIterator<Item = ParticleSystemId> + DoubleEndedIterator + '_ {
        self.particle_system_order.iter().copied()
    }

    /// Returns owned configuration and membership state for a live system.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle error when `system` is foreign or stale.
    pub fn particle_system_snapshot(
        &self,
        system: ParticleSystemId,
    ) -> Result<ParticleSystemSnapshot, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.particle_systems.get(system)?;
        Ok(ParticleSystemSnapshot {
            definition: record.definition,
            particle_count: record.storage.len(),
            pending_particle_count: record.storage.pending_count(),
        })
    }

    /// Borrows every supported semantic lane for one live particle system.
    ///
    /// The returned view keeps this world immutably borrowed, preventing
    /// structural particle mutation while any lane or derived record is in use.
    /// Dense rows, capacities, scratch state, and mutable slices remain private.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle error when `system` is foreign or stale.
    pub fn particle_system_view(
        &self,
        system: ParticleSystemId,
    ) -> Result<ParticleSystemView<'_>, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.particle_systems.get(system)?;
        Ok(ParticleSystemView::new(&record.storage))
    }

    /// Applies one closure-scoped edit after validating the complete candidate.
    ///
    /// Position changes synchronously rebuild proxies and clear contact- and
    /// spatially-derived records. A closure panic occurs before storage mutation.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle or non-finite candidate error without mutation.
    pub fn edit_particle<R>(
        &mut self,
        particle: ParticleId,
        edit: impl for<'edit> FnOnce(&mut ParticleEditor<'edit>) -> Result<R, ParticleEditError>,
    ) -> Result<R, ParticleEditError> {
        self.ensure_not_poisoned_for_handle()?;
        let system = self.particle_system_id_for_particle(particle)?;
        let input = self
            .particle_systems
            .get(system)?
            .storage
            .input(particle)
            .map_err(storage_handle_error)?;
        let mut editor = ParticleEditor::new(input.position, input.velocity);
        let output = edit(&mut editor)?;
        let (position, velocity) = editor.into_parts();
        self.system_mut_after_validation(system)
            .storage
            .commit_kinematic_edit(particle, position, velocity)
            .map_err(storage_handle_error)?;
        Ok(output)
    }

    /// Sets a stable particle position and repairs spatially derived state.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle or non-finite position error without mutation.
    pub fn set_particle_position(
        &mut self,
        particle: ParticleId,
        position: Vec2,
    ) -> Result<(), ParticleEditError> {
        self.edit_particle(particle, |editor| editor.set_position(position))
    }

    /// Sets a stable particle velocity without exposing a mutable lane.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle or non-finite velocity error without mutation.
    pub fn set_particle_velocity(
        &mut self,
        particle: ParticleId,
        velocity: Vec2,
    ) -> Result<(), ParticleEditError> {
        self.edit_particle(particle, |editor| editor.set_velocity(velocity))
    }

    /// Changes only the paused state of a live system.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle error without mutation for a foreign or stale system.
    pub fn set_particle_system_paused(
        &mut self,
        system: ParticleSystemId,
        paused: bool,
    ) -> Result<(), HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let definition = self
            .particle_systems
            .get(system)?
            .definition
            .with_paused(paused);
        self.system_mut_after_validation(system).definition = definition;
        Ok(())
    }

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
    ) -> Result<ParticleId, CreateObjectError> {
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
        let mut preflight = self.particle_systems.get(system)?.clone();
        preflight
            .lifetime
            .prepare_capacity_for_creation(&mut preflight.storage)
            .map_err(particle_lifecycle_creation_error)?;
        preflight
            .storage
            .validate_create(input)
            .map_err(storage_object_creation_error)?;
        preflight
            .lifetime
            .validate_created_lifetime(&preflight.storage, definition.lifetime())?;
        let diagnostic_id = self.allocate_diagnostic_id()?;
        Ok(self.commit_preflighted_particle(system, input, definition.lifetime(), diagnostic_id))
    }

    fn commit_preflighted_particle(
        &mut self,
        system: ParticleSystemId,
        input: ParticleInput,
        lifetime: f32,
        diagnostic_id: u64,
    ) -> ParticleId {
        let record = self.system_mut_after_validation(system);
        record
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
        particle
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

    /// Invalidates every pending identity in ascending old-row order.
    ///
    /// # Errors
    ///
    /// Returns a scoped system error without mutation when `system` is invalid.
    ///
    /// # Panics
    ///
    /// Panics only if an internal storage invariant was violated before this
    /// call; public operations cannot construct such a state.
    pub fn compact_pending_particles(
        &mut self,
        system: ParticleSystemId,
    ) -> Result<DestructionReport, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        self.particle_systems.get(system)?;
        let outcome = crate::particle::lifetime::compact_pending_with_occurrences(
            &mut self.system_mut_after_validation(system).storage,
        )
        .expect("validated authoritative storage compacts transactionally");
        let records = outcome
            .destroyed
            .into_iter()
            .map(|snapshot| Self::particle_destruction_record(snapshot, DestructionCause::Explicit))
            .collect::<Vec<_>>();
        let lifecycle = outcome
            .requested_listener_occurrences
            .into_iter()
            .map(|occurrence| {
                let particle = occurrence.particle();
                let record = records
                    .iter()
                    .find(|record| record.destroyed() == DestroyedId::Particle(particle))
                    .cloned()
                    .expect("a requested occurrence always names one compacted particle");
                LifecycleEvent::ParticleDestruction(record)
            })
            .collect();
        Ok(MutationReport::new(records, lifecycle))
    }

    pub(super) fn destroy_particle_now(
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

    pub(super) fn particle_destruction_record(
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

    fn particle_system_id_for_particle(
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

    fn debug_assert_particle_system_order_invariant(&self) {
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

fn particle_input<UserAssociation>(
    definition: &ParticleDef<UserAssociation>,
    maybe_group: Option<ParticleGroupId>,
) -> ParticleInput {
    ParticleInput {
        position: definition.position(),
        velocity: definition.velocity(),
        flags: definition.flags(),
        maybe_group,
        maybe_color: (!definition.color().is_zero()).then_some(definition.color()),
        maybe_user_association: None,
        maybe_expiration_time: None,
    }
}

fn public_particle_snapshot(snapshot: StorageParticleSnapshot) -> ParticleSnapshot {
    ParticleSnapshot {
        id: snapshot.id,
        system: snapshot_system(snapshot),
        maybe_group: snapshot.input.maybe_group,
        position: snapshot.input.position,
        velocity: snapshot.input.velocity,
        flags: snapshot.input.flags,
        color: snapshot.input.maybe_color.unwrap_or(ParticleColor::ZERO),
    }
}

fn snapshot_system(snapshot: StorageParticleSnapshot) -> ParticleSystemId {
    let identity = snapshot
        .id
        .identity()
        .maybe_particle_system()
        .expect("storage particle snapshots always retain an owning system")
        .identity();
    ParticleSystemId::from_identity(identity)
}

fn storage_creation_error(error: ParticleStorageError) -> ArenaInsertError {
    match error {
        ParticleStorageError::CapacityExceeded { limit } => {
            ArenaInsertError::CapacityExceeded { limit }
        }
        ParticleStorageError::IdentityExhausted => ArenaInsertError::GenerationExhausted,
        ParticleStorageError::WrongWorld
        | ParticleStorageError::WrongParticleSystem
        | ParticleStorageError::StaleOrDestroyed
        | ParticleStorageError::PendingDelete
        | ParticleStorageError::InvalidPermutation
        | ParticleStorageError::LaneLengthMismatch
        | ParticleStorageError::InvalidDerivedReference
        | ParticleStorageError::InvalidGroupRange
        | ParticleStorageError::InvalidLaneBundle => {
            unreachable!("checked particle-system definitions construct valid empty storage")
        }
    }
}

fn storage_object_creation_error(error: ParticleStorageError) -> CreateObjectError {
    match error {
        ParticleStorageError::WrongWorld => {
            CreateObjectError::InvalidHandle(HandleError::WrongWorld)
        }
        ParticleStorageError::WrongParticleSystem | ParticleStorageError::InvalidGroupRange => {
            CreateObjectError::InvalidHandle(HandleError::WrongParticleSystem)
        }
        ParticleStorageError::StaleOrDestroyed => {
            CreateObjectError::InvalidHandle(HandleError::StaleOrDestroyed)
        }
        ParticleStorageError::PendingDelete => {
            CreateObjectError::InvalidHandle(HandleError::PendingDelete)
        }
        ParticleStorageError::CapacityExceeded { limit } => {
            CreateObjectError::Arena(ArenaInsertError::CapacityExceeded { limit })
        }
        ParticleStorageError::IdentityExhausted => {
            CreateObjectError::Arena(ArenaInsertError::GenerationExhausted)
        }
        ParticleStorageError::InvalidPermutation
        | ParticleStorageError::LaneLengthMismatch
        | ParticleStorageError::InvalidDerivedReference
        | ParticleStorageError::InvalidLaneBundle => {
            unreachable!("checked creation cannot invalidate authoritative storage")
        }
    }
}

fn particle_lifecycle_creation_error(error: ParticleLifecycleError) -> CreateObjectError {
    match error {
        ParticleLifecycleError::Lifetime(error) => {
            CreateObjectError::InvalidParticleLifetime(error)
        }
        ParticleLifecycleError::CapacityExceeded { limit } => {
            CreateObjectError::Arena(ArenaInsertError::CapacityExceeded { limit })
        }
        ParticleLifecycleError::Storage(error) => storage_object_creation_error(error),
        ParticleLifecycleError::OldestRankOutOfRange => {
            unreachable!("a full non-empty system always has an oldest particle")
        }
    }
}

fn storage_handle_error(error: ParticleStorageError) -> HandleError {
    match error {
        ParticleStorageError::WrongWorld => HandleError::WrongWorld,
        ParticleStorageError::WrongParticleSystem => HandleError::WrongParticleSystem,
        ParticleStorageError::StaleOrDestroyed => HandleError::StaleOrDestroyed,
        ParticleStorageError::PendingDelete => HandleError::PendingDelete,
        ParticleStorageError::CapacityExceeded { .. }
        | ParticleStorageError::IdentityExhausted
        | ParticleStorageError::InvalidPermutation
        | ParticleStorageError::LaneLengthMismatch
        | ParticleStorageError::InvalidDerivedReference
        | ParticleStorageError::InvalidGroupRange
        | ParticleStorageError::InvalidLaneBundle => {
            unreachable!("handle resolution cannot produce a storage invariant error")
        }
    }
}
