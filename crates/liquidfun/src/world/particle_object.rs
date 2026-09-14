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
    DestructionRecord, HandleError, MutationReport, ObjectSnapshot, ParticleGroupId, ParticleId,
    ParticleSystemId,
};

use super::object::{ParticleGroup, ParticleSystem, World};

mod group;
mod group_mutation;
mod particle;
mod system;
pub use group_mutation::ParticleGroupMutationError;

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

mod group_lifecycle;

#[cfg(test)]
#[path = "particle_object/group_lifecycle_tests.rs"]
mod group_lifecycle_tests;
