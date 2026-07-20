//! Particle-system object lifecycle over one authoritative storage owner.

use crate::identity::HandleIdentity;
use crate::math::{Vec2, settings};
use crate::particle::VoronoiLimits;
use crate::particle::lifetime::{
    ParticleDestructionOccurrence, ParticleLifecycleError, ParticleLifetimeState,
};
use crate::particle::storage::{
    GroupPlan, GroupPlanError, GroupPlanInput, ParticleInput,
    ParticleSnapshot as StorageParticleSnapshot, ParticleStorage, ParticleStorageError,
};
use crate::particle::{
    ParticleBufferAdoptionError, ParticleBufferAdoptionErrorKind, ParticleBufferBundle,
    ParticleCapacity, ParticleColor, ParticleContactUpdate, ParticleDef, ParticleEditError,
    ParticleEditor, ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe,
    ParticleGroupView, ParticleNeighborhood, ParticleSystemDef, ParticleSystemStatistics,
    ParticleSystemView, ParticleWorldStatistics,
};
use crate::particle::{ParticleGroupSamplingError, SamplingLimits, force, plan_samples};
use crate::{
    ArenaInsertError, AssociationMap, CreateObjectError, DestroyedId, DestructionCause,
    DestructionRecord, DestructionReport, HandleError, LifecycleEvent, MutationReport,
    ObjectSnapshot, ParticleGroupId, ParticleId, ParticleSystemId,
};

use super::object::{ParticleGroup, ParticleSystem, World};

const MAX_PARTICLE_COUNT: usize = i32::MAX as usize;
const GROUP_SAMPLING_WORK_LIMIT: usize = 2_000_000;
const GROUP_SAMPLE_LIMIT: usize = 65_536;
const GROUP_TOPOLOGY_CELL_LIMIT: usize = 4_096;
const GROUP_TOPOLOGY_QUEUE_LIMIT: usize = 16_384;
const GROUP_TOPOLOGY_WORK_LIMIT: usize = 2_000_000;
const GROUP_TOPOLOGY_NODE_LIMIT: usize = 8_192;

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

/// Owned result of one committed particle-creation transaction.
///
/// Capacity eviction is synchronous. Any requested destruction-listener
/// occurrences therefore belong to this call and are returned before the
/// evicted identities become stale.
#[must_use = "particle creation can synchronously evict particles; inspect destruction_occurrences"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleCreationReceipt {
    created_particle: ParticleId,
    destruction_occurrences: Vec<ParticleDestructionOccurrence>,
}

struct ParticleGroupCreationPlan {
    system: ParticleSystemId,
    system_candidate: ParticleSystem,
    result_group: ParticleGroupId,
    maybe_shell: Option<(ParticleGroupId, u64)>,
    next_diagnostic_id: Option<u64>,
}

impl ParticleCreationReceipt {
    /// Returns the stable identity created by the committed transaction.
    #[must_use]
    pub const fn created_particle(&self) -> ParticleId {
        self.created_particle
    }

    /// Returns requested capacity-eviction occurrences in source order.
    #[must_use]
    pub fn destruction_occurrences(&self) -> &[ParticleDestructionOccurrence] {
        &self.destruction_occurrences
    }
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
                timestamp: 0,
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
            timestamp: 0,
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

    /// Returns an owned semantic statistics snapshot for one live system.
    ///
    /// Stable identities replace dense rows, and capacity values come only
    /// from explicit contracts rather than allocator internals.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle error when `system` is foreign or stale.
    pub fn particle_system_statistics(
        &self,
        system: ParticleSystemId,
    ) -> Result<ParticleSystemStatistics, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.particle_systems.get(system)?;
        Ok(ParticleSystemStatistics::from_storage(
            &record.storage,
            record.definition,
            record.groups.len(),
        ))
    }

    /// Returns owned aggregate semantic counts in system traversal order.
    ///
    /// # Panics
    ///
    /// Panics only if an internal particle-system order invariant was already
    /// violated; public operations maintain the bidirectional owner list.
    #[must_use]
    pub fn particle_world_statistics(&self) -> ParticleWorldStatistics {
        let mut statistics = ParticleWorldStatistics::default();
        for system in &self.particle_system_order {
            let record = self
                .particle_systems
                .get(*system)
                .expect("particle-system order contains only live systems");
            statistics.include(&ParticleSystemStatistics::from_storage(
                &record.storage,
                record.definition,
                record.groups.len(),
            ));
        }
        statistics
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

    /// Accumulates one checked world-space force for a stable particle.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle, non-finite vector, wall-particle, or derived
    /// distribution error without mutation.
    pub fn apply_particle_force(
        &mut self,
        particle: ParticleId,
        force: Vec2,
    ) -> Result<(), crate::ParticleForceError> {
        let system = self.particle_system_id_for_particle(particle)?;
        self.apply_particle_force_range(system, &[particle], force)
    }

    /// Distributes one checked force over contiguous stable identities.
    ///
    /// The identities must name every particle in one current source-ordered
    /// contiguous range. Validation completes before any force lane changes.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle, empty/non-contiguous range, non-finite vector,
    /// wall-particle, or derived distribution error without mutation.
    pub fn apply_particle_force_range(
        &mut self,
        system: ParticleSystemId,
        particles: &[ParticleId],
        force: Vec2,
    ) -> Result<(), crate::ParticleForceError> {
        self.ensure_not_poisoned_for_handle()?;
        let prepared = {
            let record = self.particle_systems.get(system)?;
            force::prepare_force(&record.storage, particles, force)?
        };
        force::apply_force(
            &mut self.system_mut_after_validation(system).storage,
            prepared,
        );
        Ok(())
    }

    /// Applies one checked world-space linear impulse to a stable particle.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle, non-finite vector, wall-particle, or derived
    /// mass/distribution error without mutation.
    pub fn apply_particle_linear_impulse(
        &mut self,
        particle: ParticleId,
        impulse: Vec2,
    ) -> Result<(), crate::ParticleForceError> {
        let system = self.particle_system_id_for_particle(particle)?;
        self.apply_particle_linear_impulse_range(system, &[particle], impulse)
    }

    /// Distributes one checked linear impulse over contiguous stable identities.
    ///
    /// The impulse targets the selected particles' total source-derived mass,
    /// so each selected velocity receives the same delta.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle, empty/non-contiguous range, non-finite vector,
    /// wall-particle, or derived mass/distribution error without mutation.
    pub fn apply_particle_linear_impulse_range(
        &mut self,
        system: ParticleSystemId,
        particles: &[ParticleId],
        impulse: Vec2,
    ) -> Result<(), crate::ParticleForceError> {
        self.ensure_not_poisoned_for_handle()?;
        let prepared = {
            let record = self.particle_systems.get(system)?;
            force::prepare_impulse(&record.storage, record.definition, particles, impulse)?
        };
        force::apply_impulse(
            &mut self.system_mut_after_validation(system).storage,
            prepared,
        );
        Ok(())
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

    /// Creates a complete particle group from one checked recipe.
    ///
    /// Sampling, particle identities, lifecycle capacity, contacts, and topology
    /// are prepared in owned storage before the world publishes the group shell.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error for a locked or poisoned world, an
    /// invalid owner or append target, exhausted capacity or identity space,
    /// invalid sampling output, or invalid topology.
    pub fn create_particle_group(
        &mut self,
        system: ParticleSystemId,
        recipe: &ParticleGroupRecipe<()>,
    ) -> Result<ParticleGroupId, CreateObjectError> {
        let plan = self.plan_particle_group(system, recipe)?;
        Ok(self.commit_particle_group(plan))
    }

    /// Creates a complete particle group and atomically installs its application association.
    ///
    /// A `New` recipe installs its carried association under the returned group
    /// identity. In pinned `AppendTo` semantics, the temporary group's
    /// association is discarded when that hidden group joins the target.
    ///
    /// # Errors
    ///
    /// Returns the same no-effect errors as [`Self::create_particle_group`],
    /// plus association-table reservation failure.
    pub fn create_particle_group_with_association<UserAssociation>(
        &mut self,
        system: ParticleSystemId,
        recipe: ParticleGroupRecipe<UserAssociation>,
        associations: &mut AssociationMap<ParticleGroupId, UserAssociation>,
    ) -> Result<ParticleGroupId, CreateObjectError> {
        let installs_association = matches!(recipe.destination(), ParticleGroupDestination::New)
            && recipe.maybe_user_association().is_some();
        if installs_association {
            associations
                .try_reserve_one()
                .map_err(|()| CreateObjectError::AssociationCapacityExceeded)?;
        }
        let plan = self.plan_particle_group(system, &recipe)?;
        let group = self.commit_particle_group(plan);
        let maybe_association = recipe.into_user_association();
        if installs_association && let Some(association) = maybe_association {
            let replaced = associations.insert(group, association);
            debug_assert!(replaced.is_none());
        }
        Ok(group)
    }

    /// Borrows complete semantic state for one live particle group.
    ///
    /// # Errors
    ///
    /// Returns a scoped error when the group or its owning system is foreign,
    /// stale, destroyed, or internally inconsistent.
    pub fn particle_group_view(
        &self,
        group: ParticleGroupId,
    ) -> Result<ParticleGroupView<'_>, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let shell = self.particle_groups.get(group)?;
        let system = self.particle_systems.get(shell.system)?;
        let diameter = 2.0 * system.definition.radius();
        let particle_mass =
            system.definition.density() * (settings::PARTICLE_STRIDE * diameter).powi(2);
        system
            .storage
            .group_view(group, particle_mass)
            .map_err(storage_handle_error)
    }

    fn plan_particle_group<UserAssociation>(
        &self,
        system: ParticleSystemId,
        recipe: &ParticleGroupRecipe<UserAssociation>,
    ) -> Result<ParticleGroupCreationPlan, CreateObjectError> {
        self.ensure_not_poisoned_for_handle()?;
        if self.step_state.is_locked() {
            return Err(CreateObjectError::WorldLocked);
        }
        let source_system = self.particle_systems.get(system)?;
        let maybe_append_target = match recipe.destination() {
            ParticleGroupDestination::New => None,
            ParticleGroupDestination::AppendTo(target) => {
                let target_shell = self.particle_groups.get(target)?;
                if target_shell.system != system {
                    return Err(CreateObjectError::InvalidHandle(
                        HandleError::WrongParticleSystem,
                    ));
                }
                Some(target)
            }
        };
        let temporary_group = self.particle_groups.next_handle()?;
        let maximum_samples = sampling_capacity(source_system).min(GROUP_SAMPLE_LIMIT);
        let samples = plan_samples(
            recipe,
            settings::PARTICLE_STRIDE * 2.0 * source_system.definition.radius(),
            SamplingLimits::new(GROUP_SAMPLING_WORK_LIMIT, maximum_samples),
        )
        .map_err(group_sampling_creation_error)?
        .into_samples();
        let creates_shell = maybe_append_target.is_none();
        let diagnostic_count = samples
            .len()
            .checked_add(usize::from(creates_shell))
            .ok_or(ArenaInsertError::DiagnosticIdExhausted)?;
        let (first_diagnostic_id, next_diagnostic_id) =
            self.preflight_diagnostic_ids(diagnostic_count)?;
        let particle_diagnostic_start = first_diagnostic_id + u64::from(creates_shell);

        let mut system_candidate = source_system.clone();
        for (ordinal, sample) in samples.iter().copied().enumerate() {
            append_group_particle(
                &mut system_candidate,
                recipe,
                temporary_group,
                sample.position(),
                sample.velocity(),
                particle_diagnostic_start
                    + u64::try_from(ordinal)
                        .map_err(|_error| ArenaInsertError::DiagnosticIdExhausted)?,
            )?;
        }
        refresh_candidate_contacts(&mut system_candidate)?;
        let topology: GroupPlan = system_candidate
            .storage
            .plan_group(GroupPlanInput {
                group: temporary_group,
                maybe_append_target,
                flags: recipe.group_flags(),
                strength: recipe.strength(),
                transform: recipe.transform(),
                particle_diameter: 2.0 * system_candidate.definition.radius(),
                voronoi_limits: group_topology_limits(),
            })
            .map_err(group_plan_creation_error)?;
        let result_group = topology.result_group();
        topology.commit_group(&mut system_candidate.storage);
        let maybe_shell = creates_shell.then_some((temporary_group, first_diagnostic_id));
        if creates_shell {
            system_candidate.groups.push(temporary_group);
        }
        Ok(ParticleGroupCreationPlan {
            system,
            system_candidate,
            result_group,
            maybe_shell,
            next_diagnostic_id,
        })
    }

    fn commit_particle_group(&mut self, plan: ParticleGroupCreationPlan) -> ParticleGroupId {
        if let Some((group, diagnostic_id)) = plan.maybe_shell {
            let inserted = self
                .particle_groups
                .insert(ParticleGroup {
                    diagnostic_id,
                    system: plan.system,
                })
                .expect("preflighted particle-group shell remains available until commit");
            debug_assert_eq!(inserted, group);
        }
        *self
            .particle_systems
            .get_mut(plan.system)
            .expect("validated particle system remains live until immediate commit") =
            plan.system_candidate;
        self.commit_next_diagnostic_id(plan.next_diagnostic_id);
        plan.result_group
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

fn sampling_capacity(system: &ParticleSystem) -> usize {
    if system.definition.destroys_by_age() {
        return system.storage.declared_capacity();
    }
    system
        .storage
        .declared_capacity()
        .saturating_sub(system.storage.len())
}

fn append_group_particle<UserAssociation>(
    system: &mut ParticleSystem,
    recipe: &ParticleGroupRecipe<UserAssociation>,
    group: ParticleGroupId,
    position: Vec2,
    velocity: Vec2,
    diagnostic_id: u64,
) -> Result<(), CreateObjectError> {
    system
        .lifetime
        .prepare_capacity_for_creation(&mut system.storage)
        .map_err(particle_lifecycle_creation_error)?;
    let input = ParticleInput {
        position,
        velocity,
        flags: recipe.particle_flags(),
        maybe_group: Some(group),
        maybe_color: (!recipe.color().is_zero()).then_some(recipe.color()),
        maybe_user_association: None,
        maybe_expiration_time: None,
    };
    system
        .storage
        .validate_create(input)
        .map_err(storage_object_creation_error)?;
    system
        .lifetime
        .validate_created_lifetime(&system.storage, recipe.lifetime())?;
    let particle = system
        .storage
        .create_with_diagnostic(input, diagnostic_id)
        .map_err(storage_object_creation_error)?;
    system
        .lifetime
        .initialize_created_particle(&mut system.storage, particle, recipe.lifetime())
        .map_err(particle_lifecycle_creation_error)
}

fn refresh_candidate_contacts(system: &mut ParticleSystem) -> Result<(), CreateObjectError> {
    let diameter = 2.0 * system.definition.radius();
    let view = ParticleSystemView::new(&system.storage);
    let neighborhood = ParticleNeighborhood::from_view(&view, diameter)
        .map_err(|_error| CreateObjectError::InvalidParticleGroupTopology)?;
    let previous = system.storage.semantic_particle_contacts();
    let update = ParticleContactUpdate::generate(&view, &neighborhood, &previous, |_contact| true)
        .map_err(|_error| CreateObjectError::InvalidParticleGroupTopology)?;
    system
        .storage
        .replace_particle_contacts(update.contacts())
        .map_err(storage_object_creation_error)
}

const fn group_topology_limits() -> VoronoiLimits {
    VoronoiLimits::new(
        GROUP_SAMPLE_LIMIT,
        GROUP_TOPOLOGY_CELL_LIMIT,
        GROUP_TOPOLOGY_QUEUE_LIMIT,
        GROUP_TOPOLOGY_WORK_LIMIT,
        GROUP_TOPOLOGY_NODE_LIMIT,
    )
}

fn group_sampling_creation_error(error: ParticleGroupSamplingError) -> CreateObjectError {
    match error {
        ParticleGroupSamplingError::CapacityExceeded { limit, .. } => {
            CreateObjectError::Arena(ArenaInsertError::CapacityExceeded { limit })
        }
        ParticleGroupSamplingError::WorkLimitExceeded { .. }
        | ParticleGroupSamplingError::NonFiniteDefaultStride
        | ParticleGroupSamplingError::NonPositiveDefaultStride
        | ParticleGroupSamplingError::ArithmeticOverflow
        | ParticleGroupSamplingError::NonFiniteDerivedGeometry
        | ParticleGroupSamplingError::NonFiniteDerivedPosition
        | ParticleGroupSamplingError::NonFiniteDerivedVelocity
        | ParticleGroupSamplingError::AllocationFailed
        | ParticleGroupSamplingError::Shape(_) => CreateObjectError::InvalidParticleGroupSampling,
    }
}

fn group_plan_creation_error(error: GroupPlanError) -> CreateObjectError {
    match error {
        GroupPlanError::Storage(error) => storage_object_creation_error(error),
        GroupPlanError::Topology => CreateObjectError::InvalidParticleGroupTopology,
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
