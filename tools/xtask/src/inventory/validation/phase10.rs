use super::{
    ApplicabilityStatus, CompatibilityLedger, EvidenceRecord, EvidenceStatus, InventoryError,
};

pub(super) const PROMOTION_IDS: [&str; 5] = [
    "public-api.liquidfun-box2d-box2d-particle-b2particleassembly-h",
    "public-api.liquidfun-box2d-box2d-particle-b2particlegroup-h",
    "source-area.liquidfun-box2d-box2d-particle",
    "subsystem.particle-groups-pairs-and-triads",
    "subsystem.particle-solver-behaviors",
];

pub(super) const IMPLEMENTATION_REFERENCES: [&str; 12] = [
    "crates/liquidfun/src/particle/group.rs",
    "crates/liquidfun/src/particle/group/sampling.rs",
    "crates/liquidfun/src/particle/storage/group.rs",
    "crates/liquidfun/src/particle/storage/mutation.rs",
    "crates/liquidfun/src/particle/storage/permutation.rs",
    "crates/liquidfun/src/particle/topology.rs",
    "crates/liquidfun/src/particle/topology/connectivity.rs",
    "crates/liquidfun/src/particle/topology/constraints.rs",
    "crates/liquidfun/src/particle/topology/voronoi.rs",
    "crates/liquidfun/src/particle/solver.rs",
    "crates/liquidfun/src/world/particle_object.rs",
    "crates/liquidfun/src/world/step.rs",
];

pub(super) const TEST_REFERENCES: [&str; 9] = [
    "crates/liquidfun/src/particle/group/tests.rs",
    "crates/liquidfun/src/particle/solver/order_tests.rs",
    "crates/liquidfun/tests/particle_groups.rs",
    "crates/liquidfun/tests/particle_group_lifecycle.rs",
    "crates/liquidfun/tests/particle_group_mutation.rs",
    "crates/liquidfun/tests/particle_group_properties.rs",
    "crates/liquidfun/tests/particle_solver_baseline.rs",
    "crates/liquidfun/tests/particle_solver_flags.rs",
    "crates/liquidfun/tests/particle_solver_order.rs",
];

pub(super) const DIFFERENTIAL_REFERENCES: [&str; 10] = [
    "crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/phase10-v1.json",
    "crates/liquidfun-differential/src/rigid_world/phase10/evidence.rs",
    "crates/liquidfun-differential/src/rigid_world/phase10/comparator.rs",
    "crates/liquidfun-differential/tests/phase10_corpus.rs",
    "crates/liquidfun-differential/tests/phase10_protocol.rs",
    "crates/liquidfun-differential/tests/phase10_oracle.rs",
    "crates/liquidfun-differential/tests/phase10_comparator.rs",
    "tools/xtask/src/phase10_evidence.rs",
    ".planning/phases/10-particle-groups-solvers-and-compatibility-sign-off/10-31-SUMMARY.md",
    "TESTING.md#phase-10-closed-leaf-outcomes",
];

pub(super) const AUTHORITY_REFERENCES: [&str; 26] = [
    "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29832646127",
    "https://github.com/bright-builds-llc/liquidfun-rs/commit/b20328aec9697353e322e022cd289e65d5a31340",
    "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29832646127/job/88641473476",
    "https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8496062831/zip#api-sha256=7b04bdc6715eef0803b5e4ed84ecc8d755559622134715e2ababab491b7cc493",
    "phase10-canonical-29832646127-b20328aec9697353e322e022cd289e65d5a31340.zip#archive-sha256=7b04bdc6715eef0803b5e4ed84ecc8d755559622134715e2ababab491b7cc493",
    "phase10-canonical-29832646127-b20328aec9697353e322e022cd289e65d5a31340/identity.json#trace-sha256=a55c3ef5fa3235893d3222e9e76baffa709699fef1d27e71f638c6bcb07631d6",
    "phase10-canonical-29832646127-b20328aec9697353e322e022cd289e65d5a31340/identity.json#manifest-sha256=a55f0c0220fced9817a0d65cf12f4d999161809e75f31016f52237fbf8650d21",
    "phase10-canonical-29832646127-b20328aec9697353e322e022cd289e65d5a31340/identity.json#inventory-sha256=3453fd3f42f46c601f666a00360deafd3d5b3e8ebc5d7fd4aa729291f2e8ed62",
    "phase10-canonical-29832646127-b20328aec9697353e322e022cd289e65d5a31340/identity.json#provenance-sha256=870197e1b730903aa03c23cb34cf57a1a1cdb3fe26dfb0e2aa8c6c7de9a27de7",
    "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29832646127/job/88641473497",
    "https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8496084932/zip#api-sha256=a416aa078d02e743f4a0882947718f5352df9092a3e206a0b2959a6999a966d9",
    "phase10-sanitizer-29832646127-b20328aec9697353e322e022cd289e65d5a31340.zip#archive-sha256=a416aa078d02e743f4a0882947718f5352df9092a3e206a0b2959a6999a966d9",
    "phase10-sanitizer-29832646127-b20328aec9697353e322e022cd289e65d5a31340/identity.json#trace-sha256=3736687333ffdc909309f9d993470dcf034a917ff2007733288ced855093d941",
    "phase10-sanitizer-29832646127-b20328aec9697353e322e022cd289e65d5a31340/identity.json#manifest-sha256=a55f0c0220fced9817a0d65cf12f4d999161809e75f31016f52237fbf8650d21",
    "phase10-sanitizer-29832646127-b20328aec9697353e322e022cd289e65d5a31340/identity.json#inventory-sha256=3a31807ae93c4ee322122df12c17e5ebbadc29068ae2481910cbea4baa20b03a",
    "phase10-sanitizer-29832646127-b20328aec9697353e322e022cd289e65d5a31340/identity.json#provenance-sha256=427ef11ed969b3dcdefb4f3602b592594f10f8988b29c23ccf6ea16630bfe7ef",
    "phase10-manifest.json#fixture-manifest-sha256=49271756565607cfc391ccfd29ca1b38d6176bcf32490fdf5e3636c0db861b7f",
    "phase10-manifest.json#semantic-manifest-sha256=9f9fd558a6897a43c3fc9faecdce4879efebc7c7d706dc6a1d6577655fa9887b",
    "phase10-manifest.json#leaf-set-sha256=3c62c4b9b0aa6940eaad6b8fe073f56861ff90775cfb44c41ffbb553319b7a41",
    "phase10-manifest.json#policy-set-sha256=a336f95b245a6aa0bfd9a50fffd64f3b234fd91cf50339b5c637308f2599044c",
    "phase10-manifest.json#binding-set-sha256=fa40b0f32313fc92831f6ae023ebfa857d7743001363d7c25add7e5579156df5",
    "phase10-manifest.json#comparison-payload-set-sha256=168713a532c04cc06bfbc7f18be8c75db7b2028da260f3444362964afd32a125",
    "phase10-manifest.json#outcomes=80-supported-0-documented-difference-0-intentionally-unsupported",
    ".planning/phases/10-particle-groups-solvers-and-compatibility-sign-off/10-31-SUMMARY.md",
    "TESTING.md#validated-phase-10-evidence-set-2026-07-21",
    "TESTING.md#phase-10-closed-leaf-outcomes",
];

const REJECTED_PHASE10_AUTHORITY_MARKERS: [&str; 4] = [
    "29831597090",
    "8495653581",
    "8495705068",
    "341fa70b50898d5bdf3f427240794f19210b881b",
];

pub(super) fn promotion(ledger: &CompatibilityLedger) -> Result<(), InventoryError> {
    reject_misplaced_or_rejected_authority(ledger)?;
    let promotion_started = ledger.entries.iter().any(|entry| {
        PROMOTION_IDS.contains(&entry.id.as_str())
            && entry.evidence.platform_validated.status == EvidenceStatus::Evidenced
    });
    if !promotion_started {
        return Ok(());
    }

    for id in PROMOTION_IDS {
        let Some(entry) = ledger.entries.iter().find(|entry| entry.id == id) else {
            return Err(evidence_error(format!(
                "incomplete Phase 10 promotion: missing scoped row `{id}`"
            )));
        };
        let required = [
            (&entry.evidence.implemented, &IMPLEMENTATION_REFERENCES[..]),
            (&entry.evidence.unit_tested, &TEST_REFERENCES[..]),
            (
                &entry.evidence.differentially_validated,
                &DIFFERENTIAL_REFERENCES[..],
            ),
            (
                &entry.evidence.platform_validated,
                &AUTHORITY_REFERENCES[..],
            ),
        ];
        for (record, expected) in required {
            if record.status != EvidenceStatus::Evidenced
                || !record
                    .references
                    .iter()
                    .map(String::as_str)
                    .eq(expected.iter().copied())
            {
                return Err(evidence_error(format!(
                    "incomplete or noncanonical Phase 10 promotion for scoped row `{id}`"
                )));
            }
        }
        if entry.applicability.status != ApplicabilityStatus::Applicable
            || entry.evidence.documented_difference.status != EvidenceStatus::NotEvidenced
            || entry.evidence.intentionally_unsupported.status != EvidenceStatus::NotEvidenced
        {
            return Err(evidence_error(format!(
                "Phase 10 scoped row `{id}` must have the exact supported outcome"
            )));
        }
    }
    Ok(())
}

fn reject_misplaced_or_rejected_authority(
    ledger: &CompatibilityLedger,
) -> Result<(), InventoryError> {
    for entry in &ledger.entries {
        for (_, record) in entry.evidence.records() {
            if contains_any(record, &REJECTED_PHASE10_AUTHORITY_MARKERS)
                || contains_any(record, &super::phase9::REJECTED_AUTHORITY_MARKERS)
                || contains_any(record, &super::phase9::SUPERSEDED_WR01_MARKERS)
            {
                return Err(evidence_error(format!(
                    "rejected Phase 10 authority in row `{}`",
                    entry.id
                )));
            }
            if !PROMOTION_IDS.contains(&entry.id.as_str())
                && contains_any(
                    record,
                    &["29832646127", "b20328aec9697353e322e022cd289e65d5a31340"],
                )
            {
                return Err(evidence_error(format!(
                    "Phase 10 authority cannot support out-of-scope row `{}`",
                    entry.id
                )));
            }
        }
    }
    Ok(())
}

fn contains_any(record: &EvidenceRecord, markers: &[&str]) -> bool {
    record
        .references
        .iter()
        .any(|reference| markers.iter().any(|marker| reference.contains(marker)))
}

fn evidence_error(message: String) -> InventoryError {
    InventoryError::new("evidence", message)
}
