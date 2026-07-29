use liquidfun_test_protocol::{BuildEvidenceTier, DivergenceHorizon, EvidenceTier};

use super::{report_evidence_tier, weakest_build_evidence_tier};

#[test]
fn report_evidence_tier_is_the_weakest_validated_build() {
    // Arrange / Act / Assert
    assert_eq!(
        weakest_build_evidence_tier(
            BuildEvidenceTier::D1Canonical,
            BuildEvidenceTier::D1Canonical
        ),
        EvidenceTier::D1Canonical
    );
    assert_eq!(
        weakest_build_evidence_tier(
            BuildEvidenceTier::D1Canonical,
            BuildEvidenceTier::D2Supported
        ),
        EvidenceTier::D2Supported
    );
    assert_eq!(
        weakest_build_evidence_tier(
            BuildEvidenceTier::D2Supported,
            BuildEvidenceTier::D3Exploratory
        ),
        EvidenceTier::D3Exploratory
    );
}

#[test]
fn unavailable_horizon_forces_exploratory_authority() {
    // Arrange / Act / Assert
    assert_eq!(
        report_evidence_tier(
            DivergenceHorizon::Unavailable,
            BuildEvidenceTier::D1Canonical,
            BuildEvidenceTier::D1Canonical,
        ),
        EvidenceTier::D3Exploratory
    );
    assert_eq!(
        report_evidence_tier(
            DivergenceHorizon::Unavailable,
            BuildEvidenceTier::D1Canonical,
            BuildEvidenceTier::D2Supported,
        ),
        EvidenceTier::D3Exploratory
    );
}
