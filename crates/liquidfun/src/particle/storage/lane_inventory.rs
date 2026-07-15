//! Closed Phase 9 particle-state and permutation inventory.
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AllocationKind {
    Unclassified,
    IdentityMapping,
    RequiredLane,
    LazyLane,
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
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        AllocationKind, ClearKind, INVENTORY, ParticleState, PermutationKind,
        REQUIRED_PARTICLE_STATE, RemapKind,
    };

    #[test]
    fn every_phase_nine_particle_state_has_one_inventory_entry() {
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
}
