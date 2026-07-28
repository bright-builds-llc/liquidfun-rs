#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use super::*;

pub(super) fn supported_math_identity(adapter_digest_byte: &str) -> BuildIdentity {
    let phase4 = Phase4BuildIdentityFields::new(
        "33".repeat(32),
        "AppleClang",
        "21.0.0",
        "arm64-apple-darwin",
        "baseline",
        "<none>",
        "<none>",
        "O0",
        "precise",
        "off",
        "ieee",
        "scalar baseline",
        "macos",
        "libSystem",
        "libSystem",
        "nearest_ties_even",
        true,
    );
    BuildIdentity::new(
        BuildIdentityFields::new(
            ORACLE_REVISION,
            "adapter-v1",
            adapter_digest_byte.repeat(32),
            "oracle-debug",
            "AppleClang",
            "21.0.0",
            "arm64-apple-darwin",
            "Debug",
            "reviewed",
            "none",
            "none",
        )
        .with_phase4(phase4),
    )
    .expect("supported fixture identity should validate")
}

#[test]
fn request_horizon_must_exactly_match_field_policy() {
    // Arrange / Act / Assert
    assert!(horizons_match(
        MathProbeHorizon::ScenarioSteps { steps: 32 },
        DivergenceHorizon::ScenarioSteps { steps: 32 }
    ));
    assert!(!horizons_match(
        MathProbeHorizon::Operation,
        DivergenceHorizon::ScenarioSteps { steps: 32 }
    ));
    assert!(!horizons_match(
        MathProbeHorizon::ScenarioSteps { steps: 4 },
        DivergenceHorizon::ScenarioSteps { steps: 32 }
    ));
}

#[test]
fn exploratory_or_replay_tier_cannot_apply_authoritative_policy() {
    // Arrange / Act / Assert
    assert!(tier_authorizes(
        EvidenceTier::D2Supported,
        EvidenceTier::D1Canonical
    ));
    assert!(!tier_authorizes(
        EvidenceTier::D3Exploratory,
        EvidenceTier::D2Supported
    ));
    assert!(!tier_authorizes(
        EvidenceTier::D1Canonical,
        EvidenceTier::D0Replay
    ));
}
