use super::{CompatibilityLedger, EvidenceStatus, InventoryError};

const PHASE9_PROMOTION_IDS: [&str; 4] = [
    "public-api.liquidfun-box2d-box2d-particle-b2particle-h",
    "public-api.liquidfun-box2d-box2d-particle-b2particlesystem-h",
    "subsystem.particle-contacts-and-coupling",
    "subsystem.particle-storage-and-lifecycle",
];
const PHASE9_DEFERRED_IDS: [&str; 5] = [
    "public-api.liquidfun-box2d-box2d-particle-b2particleassembly-h",
    "public-api.liquidfun-box2d-box2d-particle-b2particlegroup-h",
    "source-area.liquidfun-box2d-box2d-particle",
    "subsystem.particle-groups-pairs-and-triads",
    "subsystem.particle-solver-behaviors",
];
const PHASE9_AUTHORITY_REFERENCES: [&str; 15] = [
    "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29661682074",
    "https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8434547024/zip#sha256=22a37f91965eaf494b3e1fea041e1c54da9be03c06da5e276a641ee6cf536084",
    "phase9-canonical-29661682074-9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce/identity.json#trace-sha256=eefec714082fc701fb6ec2cebd15ed9353114a8cc17f975b71c666b33fd3ccf7",
    "phase9-canonical-29661682074-9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce/identity.json#manifest-sha256=74998e953e79f5ed04a58097d43abbca3cc814bee4fc86d0fd552d2951b1ae7c",
    "https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8434557009/zip#sha256=849b8dba5b4c5a0f5e6ea4cddf10bf8243a71bdeec3b75676677358aa34d4316",
    "phase9-sanitizer-29661682074-9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce/identity.json#trace-sha256=3c697421472ee087d265cb9a6268ab04ef76dce37c39ed6b4202fa1a36c7dbdd",
    "phase9-sanitizer-29661682074-9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce/identity.json#manifest-sha256=74998e953e79f5ed04a58097d43abbca3cc814bee4fc86d0fd552d2951b1ae7c",
    "phase9-manifest.json#semantic-manifest-sha256=a319f771c5d9e952b9389160bb3ad19ce487da43271e62568828ce2ae22a33aa",
    "https://github.com/bright-builds-llc/liquidfun-rs/commit/9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce",
    ".planning/phases/09-particle-storage-lifecycle-and-coupling/09-31-SUMMARY.md",
    "TESTING.md#approved-phase-9-evidence-run-2026-07-18-wr-02",
    "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29661682074/job/88125511292",
    "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29661682074/job/88125511305",
    "phase9-manifest.json#payload-digest-set-sha256=72797909ebb807c4c7dc591b4fa8987b26f3f26e43b967e080db4363f26b509d",
    "phase9-manifest.json#binding-digest-set-sha256=2e0e4212a62aec27b371bcd8dc9301966e0f712b0d28736e39f3993cc3ab3134",
];
const PHASE9_DIFFERENTIAL_REFERENCES: [&str; 7] = [
    "crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json",
    "crates/liquidfun-differential/tests/phase9_corpus.rs",
    "crates/liquidfun-differential/src/rigid_world/phase9.rs",
    "crates/liquidfun-differential/src/rigid_world/phase9/comparator.rs",
    "tools/xtask/src/phase9_evidence.rs",
    ".planning/phases/09-particle-storage-lifecycle-and-coupling/09-27-SUMMARY.md",
    ".planning/phases/09-particle-storage-lifecycle-and-coupling/09-28-SUMMARY.md",
];
pub(super) const REJECTED_AUTHORITY_MARKERS: [&str; 25] = [
    "29439515367",
    "8352859391",
    "8352881868",
    "a87f84bbdbfe55fb732d74c481c4a4bda9eec958",
    "f237d6f1ebe0e59f65a5ae0609140eecdd8b32247e9d2064c83748be1ab9f5ea",
    "95ad57e5d5711ae6aa93847ad1efd4a04025bd2956b4996535fa0e5f45a5893f",
    "3a339387b4c4acccc15b5fc4944d6bec9c7e1d315f4753034ae52a5ff97f2e64",
    "ee75462d49275c5b7d02b8677eb6f9bf82c241c6b993c16d6df08a2ae231a070",
    "09-16-SUMMARY.md",
    "phase-9-canonical-evidence-2026-07-15",
    "29583793056",
    "8408156562",
    "8408174081",
    "b27fc14f6b29fb82ca815fa1effba71bae09d424",
    "faaf24c870826251f0dd1d507ba9c335269b78433ba1ce2ee0e1995336f0139a",
    "f4b30cebed7b81a41282a33d45b81231485a2fa0c3a958c7b68a3ecbad086e7c",
    "5c6805e0e998394947439bf6eb295526130cb4db81a67e0f560c6d6bc3f33545",
    "4261dbe8993155dd7ab9e7992f90bef60e57762c36a66ea97b3bc9131804508a",
    "b7bb43a6ce083fe543bf6eb3f92b1b4f663d4bd6520dbb83c3a00072a3010a8b",
    "09-23-SUMMARY.md",
    "approved-phase-9-evidence-run-2026-07-17",
    "29625083184",
    "8423580554",
    "7ed430c497efbaa8585ee9ef3862be1abda29ef5",
    "f7478565688e7250257bc8c1d066456853604394c61e7cbe38ffcc11e73c5c5b",
];
pub(super) const SUPERSEDED_WR01_MARKERS: [&str; 8] = [
    "29652578231",
    "8431920189",
    "8431922578",
    "22b31c0e1be8896df622b1decd58ba2853a60b04",
    "ea333de6ac32d64c1c5b4e80738275451f0e51994b7f78e70961597d48e77500",
    "99fa817d3b891a8942709e4b4af2bd4fa0aedbde0fc4c19b398829f02128a6c6",
    "662b9514472c1d6d8186115577f43c5987870a2a24592156b46631f1c28b4a3e",
    "671d16f1c7af0f948760b9cdc62b3ed1fefb7307889a46334230605365aefe80",
];

pub(super) fn promotion(ledger: &CompatibilityLedger) -> Result<(), InventoryError> {
    let promotion_started = ledger.entries.iter().any(|entry| {
        PHASE9_PROMOTION_IDS.contains(&entry.id.as_str())
            && entry.evidence.platform_validated.status == EvidenceStatus::Evidenced
    });
    if !promotion_started {
        return Ok(());
    }

    validate_promoted_rows(ledger)?;
    validate_deferred_rows(ledger)?;
    validate_fresh_authority(ledger)
}

fn validate_promoted_rows(ledger: &CompatibilityLedger) -> Result<(), InventoryError> {
    for id in PHASE9_PROMOTION_IDS {
        let maybe_entry = ledger.entries.iter().find(|entry| entry.id == id);
        let Some(entry) = maybe_entry else {
            return Err(InventoryError::new(
                "evidence",
                format!("incomplete Phase 9 promotion: missing scoped row `{id}`"),
            ));
        };
        let complete = [
            &entry.evidence.implemented,
            &entry.evidence.unit_tested,
            &entry.evidence.differentially_validated,
            &entry.evidence.platform_validated,
        ]
        .into_iter()
        .all(|record| record.status == EvidenceStatus::Evidenced);
        if !complete {
            return Err(InventoryError::new(
                "evidence",
                format!("incomplete Phase 9 promotion for scoped row `{id}`"),
            ));
        }
        if !entry
            .evidence
            .platform_validated
            .references
            .iter()
            .map(String::as_str)
            .eq(PHASE9_AUTHORITY_REFERENCES)
        {
            return Err(InventoryError::new(
                "evidence",
                format!("noncanonical Phase 9 authority for scoped row `{id}`"),
            ));
        }
        for record in [
            &entry.evidence.implemented,
            &entry.evidence.unit_tested,
            &entry.evidence.differentially_validated,
            &entry.evidence.platform_validated,
        ] {
            if record.references.iter().any(|reference| {
                REJECTED_AUTHORITY_MARKERS
                    .iter()
                    .any(|marker| reference.contains(marker))
            }) {
                return Err(InventoryError::new(
                    "evidence",
                    format!("superseded Phase 9 authority for scoped row `{id}`"),
                ));
            }
        }
        if !entry
            .evidence
            .differentially_validated
            .references
            .iter()
            .map(String::as_str)
            .eq(PHASE9_DIFFERENTIAL_REFERENCES)
        {
            return Err(InventoryError::new(
                "evidence",
                format!("incomplete Phase 9 semantic evidence for scoped row `{id}`"),
            ));
        }
    }

    Ok(())
}

fn validate_deferred_rows(ledger: &CompatibilityLedger) -> Result<(), InventoryError> {
    let phase10_promotion_started = ledger.entries.iter().any(|entry| {
        super::phase10::PROMOTION_IDS.contains(&entry.id.as_str())
            && entry.evidence.platform_validated.status == EvidenceStatus::Evidenced
    });
    if !phase10_promotion_started {
        for id in PHASE9_DEFERRED_IDS {
            let Some(entry) = ledger.entries.iter().find(|entry| entry.id == id) else {
                continue;
            };
            let deferred = [
                &entry.evidence.implemented,
                &entry.evidence.unit_tested,
                &entry.evidence.differentially_validated,
                &entry.evidence.platform_validated,
            ]
            .into_iter()
            .all(|record| record.status == EvidenceStatus::NotEvidenced);
            if !deferred {
                return Err(InventoryError::new(
                    "evidence",
                    format!("Phase 9 promotion cannot claim deferred Phase 10 row `{id}`"),
                ));
            }
        }
    }

    Ok(())
}

fn validate_fresh_authority(ledger: &CompatibilityLedger) -> Result<(), InventoryError> {
    for id in PHASE9_PROMOTION_IDS {
        let entry = ledger
            .entries
            .iter()
            .find(|entry| entry.id == id)
            .expect("promotion rows were validated above");
        if entry
            .evidence
            .platform_validated
            .references
            .iter()
            .any(|reference| {
                SUPERSEDED_WR01_MARKERS
                    .iter()
                    .any(|marker| reference.contains(marker))
            })
        {
            return Err(InventoryError::new(
                "evidence",
                format!(
                    "superseded pre-WR-01 Phase 9 authority for scoped row `{id}`; fresh exact-ref evidence is required"
                ),
            ));
        }
    }

    Ok(())
}
