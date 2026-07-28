use std::collections::HashSet;

use super::{RigidWorldWitness, RigidWorldWitnessFamily};

#[test]
fn witness_registry_covers_every_bounded_phase7_family_without_overlap() {
    // Arrange
    let mut witnesses = HashSet::new();

    // Act
    for family in RigidWorldWitnessFamily::ALL {
        assert!(!family.required_witnesses().is_empty());
        for witness in family.required_witnesses() {
            assert!(witnesses.insert(*witness), "duplicate witness: {witness:?}");
        }
    }

    // Assert
    assert_eq!(RigidWorldWitnessFamily::PHASE7_REQUIRED.len(), 7);
    assert!(witnesses.contains(&RigidWorldWitness::RayEqualFractionTieSet));
    assert!(witnesses.contains(&RigidWorldWitness::ContinuousBudgetResumeCompleted));
    assert!(witnesses.contains(&RigidWorldWitness::OriginShiftPreservedTopology));
    assert_eq!(RigidWorldWitnessFamily::ALL.len(), 19);
    assert_eq!(RigidWorldWitnessFamily::PHASE8_REQUIRED.len(), 10);
    assert!(witnesses.contains(&RigidWorldWitness::StandaloneRopePositiveStep));
    assert!(witnesses.contains(&RigidWorldWitness::DiagnosticsCountsObserved));
}
