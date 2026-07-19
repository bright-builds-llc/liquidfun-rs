//! Closed particle-state, solver-state, and permutation inventory.
//!
//! This table is deliberately independent from the current Phase 3 storage
//! spike. Later production storage fields must select one entry here instead
//! of creating another unchecked permutation path.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ParticleState {
    StableIdentityMap,
    Flags,
    Positions,
    Velocities,
    Colors,
    UserAssociations,
    Groups,
    Weights,
    Forces,
    LastBodyContactStep,
    BodyContactCount,
    ConsecutiveContactSteps,
    StuckCandidates,
    Proxies,
    ParticleContacts,
    BodyContacts,
    ExpirationTimes,
    ExpirationOrder,
    Pairs,
    Triads,
    GroupRanges,
    StaticPressures,
    TensileAccumulations,
    Depths,
    AggregateParticleFlags,
    AggregateGroupFlags,
    PendingSystemForce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AllocationKind {
    Unclassified,
    IdentityMapping,
    RequiredLane,
    LazyLane,
    LazySolverLane,
    AggregateState,
    MarkerState,
    DerivedCollection,
    DeferredReferenceCollection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClearKind {
    Unclassified,
    RetireIdentity,
    CompactRow,
    CompactActiveRow,
    Recompute,
    RemapAndDropInvalid,
    AdjustGroupRanges,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PermutationKind {
    Unclassified,
    RebuildStableLookup,
    PermuteRows,
    PermuteActiveRows,
    RecomputeBeforeUse,
    RemapReferences,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RemapKind {
    None,
    StableIdentityLookup,
    SingleIndex,
    PairIndices,
    TriadIndices,
    ExpirationOrder,
    ContiguousGroupRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BufferOwnership {
    ConsumerTransfer,
    StorageSource,
    StorageDerived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LaneInventoryEntry {
    state: ParticleState,
    allocation: AllocationKind,
    clear: ClearKind,
    permutation: PermutationKind,
    remap: RemapKind,
    generated_in_phase_nine: bool,
    allocation_source: &'static str,
    permutation_source: &'static str,
}

const BUFFER_DECLARATIONS: &str = "b2ParticleSystem.h:1081-1134";
const BUFFER_ALLOCATION: &str = "b2ParticleSystem.cpp:575-631";
const PARTICLE_CREATION: &str = "b2ParticleSystem.cpp:637-727";
const ZOMBIE_AND_ROTATE: &str = "b2ParticleSystem.cpp:3798-4038,4078-4238";
const GROUP_REMAP: &str = "b2ParticleSystem.cpp:3980-4029,4229-4238";
const WEIGHT_RECOMPUTE: &str = "b2ParticleSystem.cpp:1664-1681";
const STUCK_RECOMPUTE: &str = "b2ParticleSystem.cpp:2290-2339,2608-2739";
const SOLVER_STATE_DECLARATIONS: &str = "b2ParticleSystem.h:1070-1120";
const SOLVER_STATE_ALLOCATION: &str = "b2ParticleSystem.cpp:541-620,3141-3189,3612-3659,4390-4434";
const SOLVER_STATE_PERMUTATION: &str = "b2ParticleSystem.cpp:1395-1430,3840-3880,4125-4155";
const AGGREGATE_REFRESH: &str = "b2ParticleSystem.cpp:2984-3014,3097-3116,4000-4024";
const FORCE_MARKER: &str = "b2ParticleSystem.cpp:3019-3021,3763-3772,4442-4455";

const REQUIRED_PARTICLE_STATE: &[ParticleState] = &[
    ParticleState::StableIdentityMap,
    ParticleState::Flags,
    ParticleState::Positions,
    ParticleState::Velocities,
    ParticleState::Colors,
    ParticleState::UserAssociations,
    ParticleState::Groups,
    ParticleState::Weights,
    ParticleState::Forces,
    ParticleState::LastBodyContactStep,
    ParticleState::BodyContactCount,
    ParticleState::ConsecutiveContactSteps,
    ParticleState::StuckCandidates,
    ParticleState::Proxies,
    ParticleState::ParticleContacts,
    ParticleState::BodyContacts,
    ParticleState::ExpirationTimes,
    ParticleState::ExpirationOrder,
    ParticleState::Pairs,
    ParticleState::Triads,
    ParticleState::GroupRanges,
    ParticleState::StaticPressures,
    ParticleState::TensileAccumulations,
    ParticleState::Depths,
    ParticleState::AggregateParticleFlags,
    ParticleState::AggregateGroupFlags,
    ParticleState::PendingSystemForce,
];

macro_rules! entry {
    (
        $state:ident,
        $allocation:ident,
        $clear:ident,
        $permutation:ident,
        $remap:ident,
        $generated:expr,
        $allocation_source:expr,
        $permutation_source:expr
    ) => {
        LaneInventoryEntry {
            state: ParticleState::$state,
            allocation: AllocationKind::$allocation,
            clear: ClearKind::$clear,
            permutation: PermutationKind::$permutation,
            remap: RemapKind::$remap,
            generated_in_phase_nine: $generated,
            allocation_source: $allocation_source,
            permutation_source: $permutation_source,
        }
    };
}

const INVENTORY: &[LaneInventoryEntry] = &[
    entry!(
        StableIdentityMap,
        IdentityMapping,
        RetireIdentity,
        RebuildStableLookup,
        StableIdentityLookup,
        true,
        BUFFER_DECLARATIONS,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        Flags,
        RequiredLane,
        CompactRow,
        PermuteRows,
        None,
        true,
        BUFFER_ALLOCATION,
        WEIGHT_RECOMPUTE
    ),
    entry!(
        Positions,
        RequiredLane,
        CompactRow,
        PermuteRows,
        None,
        true,
        BUFFER_ALLOCATION,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        Velocities,
        RequiredLane,
        CompactRow,
        PermuteRows,
        None,
        true,
        BUFFER_ALLOCATION,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        Colors,
        LazyLane,
        CompactRow,
        PermuteRows,
        None,
        true,
        PARTICLE_CREATION,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        UserAssociations,
        LazyLane,
        CompactRow,
        PermuteRows,
        None,
        true,
        PARTICLE_CREATION,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        Groups,
        RequiredLane,
        CompactRow,
        PermuteRows,
        None,
        true,
        PARTICLE_CREATION,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        Weights,
        RequiredLane,
        Recompute,
        RecomputeBeforeUse,
        None,
        true,
        BUFFER_ALLOCATION,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        Forces,
        RequiredLane,
        CompactActiveRow,
        PermuteActiveRows,
        None,
        true,
        BUFFER_ALLOCATION,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        LastBodyContactStep,
        LazyLane,
        CompactRow,
        PermuteRows,
        None,
        true,
        BUFFER_ALLOCATION,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        BodyContactCount,
        LazyLane,
        CompactRow,
        PermuteRows,
        None,
        true,
        BUFFER_ALLOCATION,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        ConsecutiveContactSteps,
        LazyLane,
        CompactRow,
        PermuteRows,
        None,
        true,
        BUFFER_ALLOCATION,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        StuckCandidates,
        DerivedCollection,
        Recompute,
        RecomputeBeforeUse,
        None,
        true,
        BUFFER_DECLARATIONS,
        STUCK_RECOMPUTE
    ),
    entry!(
        Proxies,
        DerivedCollection,
        RemapAndDropInvalid,
        RemapReferences,
        SingleIndex,
        true,
        PARTICLE_CREATION,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        ParticleContacts,
        DerivedCollection,
        RemapAndDropInvalid,
        RemapReferences,
        PairIndices,
        true,
        BUFFER_DECLARATIONS,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        BodyContacts,
        DerivedCollection,
        RemapAndDropInvalid,
        RemapReferences,
        SingleIndex,
        true,
        BUFFER_DECLARATIONS,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        ExpirationTimes,
        LazyLane,
        CompactRow,
        PermuteRows,
        None,
        true,
        PARTICLE_CREATION,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        ExpirationOrder,
        LazyLane,
        RemapAndDropInvalid,
        RemapReferences,
        ExpirationOrder,
        true,
        PARTICLE_CREATION,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        Pairs,
        DeferredReferenceCollection,
        RemapAndDropInvalid,
        RemapReferences,
        PairIndices,
        false,
        BUFFER_DECLARATIONS,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        Triads,
        DeferredReferenceCollection,
        RemapAndDropInvalid,
        RemapReferences,
        TriadIndices,
        false,
        BUFFER_DECLARATIONS,
        ZOMBIE_AND_ROTATE
    ),
    entry!(
        GroupRanges,
        DerivedCollection,
        AdjustGroupRanges,
        RemapReferences,
        ContiguousGroupRange,
        false,
        BUFFER_DECLARATIONS,
        GROUP_REMAP
    ),
    entry!(
        StaticPressures,
        LazySolverLane,
        CompactRow,
        PermuteRows,
        None,
        false,
        SOLVER_STATE_ALLOCATION,
        SOLVER_STATE_PERMUTATION
    ),
    entry!(
        TensileAccumulations,
        LazySolverLane,
        CompactRow,
        PermuteRows,
        None,
        false,
        SOLVER_STATE_ALLOCATION,
        SOLVER_STATE_PERMUTATION
    ),
    entry!(
        Depths,
        LazySolverLane,
        CompactRow,
        PermuteRows,
        None,
        false,
        SOLVER_STATE_ALLOCATION,
        SOLVER_STATE_PERMUTATION
    ),
    entry!(
        AggregateParticleFlags,
        AggregateState,
        Recompute,
        RecomputeBeforeUse,
        None,
        false,
        SOLVER_STATE_DECLARATIONS,
        AGGREGATE_REFRESH
    ),
    entry!(
        AggregateGroupFlags,
        AggregateState,
        Recompute,
        RecomputeBeforeUse,
        None,
        false,
        SOLVER_STATE_DECLARATIONS,
        AGGREGATE_REFRESH
    ),
    entry!(
        PendingSystemForce,
        MarkerState,
        CompactActiveRow,
        PermuteActiveRows,
        None,
        false,
        SOLVER_STATE_DECLARATIONS,
        FORCE_MARKER
    ),
];

const fn buffer_ownership(state: ParticleState) -> BufferOwnership {
    match state {
        ParticleState::Flags
        | ParticleState::Positions
        | ParticleState::Velocities
        | ParticleState::Colors => BufferOwnership::ConsumerTransfer,
        ParticleState::Groups
        | ParticleState::UserAssociations
        | ParticleState::ExpirationTimes => BufferOwnership::StorageSource,
        ParticleState::StableIdentityMap
        | ParticleState::Weights
        | ParticleState::Forces
        | ParticleState::LastBodyContactStep
        | ParticleState::BodyContactCount
        | ParticleState::ConsecutiveContactSteps
        | ParticleState::StuckCandidates
        | ParticleState::Proxies
        | ParticleState::ParticleContacts
        | ParticleState::BodyContacts
        | ParticleState::ExpirationOrder
        | ParticleState::Pairs
        | ParticleState::Triads
        | ParticleState::GroupRanges
        | ParticleState::StaticPressures
        | ParticleState::TensileAccumulations
        | ParticleState::Depths
        | ParticleState::AggregateParticleFlags
        | ParticleState::AggregateGroupFlags
        | ParticleState::PendingSystemForce => BufferOwnership::StorageDerived,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        AllocationKind, BufferOwnership, ClearKind, INVENTORY, ParticleState, PermutationKind,
        REQUIRED_PARTICLE_STATE, RemapKind, buffer_ownership,
    };

    #[test]
    fn every_particle_state_has_one_inventory_entry() {
        // Arrange
        let inventoried = INVENTORY
            .iter()
            .map(|entry| entry.state)
            .collect::<HashSet<_>>();

        // Act
        let unique_entry_count = inventoried.len();

        // Assert
        assert_eq!(INVENTORY.len(), REQUIRED_PARTICLE_STATE.len());
        assert_eq!(unique_entry_count, REQUIRED_PARTICLE_STATE.len());
        for state in REQUIRED_PARTICLE_STATE {
            assert!(
                inventoried.contains(state),
                "missing inventory for {state:?}"
            );
        }
    }

    #[test]
    fn every_entry_has_explicit_lifecycle_and_source_obligations() {
        // Arrange / Act / Assert
        for entry in INVENTORY {
            assert_ne!(entry.allocation, AllocationKind::Unclassified);
            assert_ne!(entry.clear, ClearKind::Unclassified);
            assert_ne!(entry.permutation, PermutationKind::Unclassified);
            assert!(!entry.allocation_source.is_empty());
            assert!(!entry.permutation_source.is_empty());
        }
    }

    #[test]
    fn deferred_topology_state_is_permutation_safe_but_not_generated() {
        // Arrange
        let deferred = [ParticleState::Pairs, ParticleState::Triads];

        // Act / Assert
        for state in deferred {
            let entry = INVENTORY
                .iter()
                .find(|entry| entry.state == state)
                .expect("required state should be inventoried");
            assert_eq!(
                entry.allocation,
                AllocationKind::DeferredReferenceCollection
            );
            assert_eq!(entry.clear, ClearKind::RemapAndDropInvalid);
            assert_eq!(entry.permutation, PermutationKind::RemapReferences);
            assert!(matches!(
                entry.remap,
                RemapKind::PairIndices | RemapKind::TriadIndices
            ));
            assert!(!entry.generated_in_phase_nine);
        }
    }

    #[test]
    fn derived_state_never_masquerades_as_a_permuted_row_lane() {
        // Arrange
        let derived = [ParticleState::Weights, ParticleState::StuckCandidates];

        // Act / Assert
        for state in derived {
            let entry = INVENTORY
                .iter()
                .find(|entry| entry.state == state)
                .expect("required state should be inventoried");
            assert_eq!(entry.clear, ClearKind::Recompute);
            assert_eq!(entry.permutation, PermutationKind::RecomputeBeforeUse);
            assert_eq!(entry.remap, RemapKind::None);
        }
    }

    #[test]
    fn solver_state_is_complete_permutation_safe_and_storage_owned() {
        // Arrange
        let aligned = [
            ParticleState::StaticPressures,
            ParticleState::TensileAccumulations,
            ParticleState::Depths,
        ];
        let aggregates = [
            ParticleState::AggregateParticleFlags,
            ParticleState::AggregateGroupFlags,
        ];

        // Act / Assert
        for state in aligned {
            let entry = INVENTORY
                .iter()
                .find(|entry| entry.state == state)
                .expect("required solver lane should be inventoried");
            assert_eq!(entry.allocation, AllocationKind::LazySolverLane);
            assert_eq!(entry.clear, ClearKind::CompactRow);
            assert_eq!(entry.permutation, PermutationKind::PermuteRows);
            assert_eq!(buffer_ownership(state), BufferOwnership::StorageDerived);
        }
        for state in aggregates {
            let entry = INVENTORY
                .iter()
                .find(|entry| entry.state == state)
                .expect("required aggregate should be inventoried");
            assert_eq!(entry.allocation, AllocationKind::AggregateState);
            assert_eq!(entry.clear, ClearKind::Recompute);
            assert_eq!(entry.permutation, PermutationKind::RecomputeBeforeUse);
            assert_eq!(buffer_ownership(state), BufferOwnership::StorageDerived);
        }
        assert_eq!(
            buffer_ownership(ParticleState::PendingSystemForce),
            BufferOwnership::StorageDerived
        );
    }
}
