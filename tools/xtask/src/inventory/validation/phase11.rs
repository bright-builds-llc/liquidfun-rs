use std::{fs, path::Path};

use sha2::{Digest, Sha256};

use super::{
    ApplicabilityStatus, CompatibilityLedger, EvidenceRecord, EvidenceStatus, InventoryError,
};

pub(super) const PROMOTION_IDS: [&str; 4] = [
    "subsystem.headless-catalog-execution",
    "subsystem.headless-public-observation-and-debug-draw",
    "subsystem.headless-reviewed-upstream-equivalence",
    "subsystem.headless-semantic-checkpoints-and-comparison",
];

const INVESTIGATION_REFERENCES: [&str; 2] = [
    ".planning/phases/11-examples-headless-tooling-and-testbed/11-RESEARCH.md",
    "reference/artifacts/phase11/scenario-mappings.json",
];
const PLANNING_REFERENCES: [&str; 2] = [
    ".planning/ROADMAP.md#phase-11",
    ".planning/phases/11-examples-headless-tooling-and-testbed/11-23-PLAN.md",
];
const IMPLEMENTATION_REFERENCES: [&str; 8] = [
    "crates/liquidfun-test-protocol/src/catalog.rs",
    "crates/liquidfun-test-protocol/src/checkpoint.rs",
    "crates/liquidfun/src/world/observation.rs",
    "crates/liquidfun/src/debug_draw.rs",
    "crates/liquidfun-differential/src/catalog_native.rs",
    "crates/liquidfun-differential/src/comparison_model.rs",
    "crates/liquidfun-differential/src/runner/catalog.rs",
    "crates/liquidfun-differential/src/catalog_command.rs",
];
const TEST_REFERENCES: [&str; 8] = [
    "crates/liquidfun-test-protocol/tests/catalog.rs",
    "crates/liquidfun-test-protocol/tests/checkpoint_protocol.rs",
    "crates/liquidfun/tests/world_observations.rs",
    "crates/liquidfun/tests/debug_draw.rs",
    "crates/liquidfun-differential/tests/catalog_native.rs",
    "crates/liquidfun-differential/tests/comparison_model.rs",
    "crates/liquidfun-differential/tests/headless_catalog.rs",
    "crates/liquidfun-differential/tests/phase11_corpus.rs",
];
const DIFFERENTIAL_REFERENCES: [&str; 9] = [
    "reference/scenario-catalog.json",
    "reference/artifacts/phase11/scenario-mappings.json",
    "crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json",
    "crates/liquidfun-differential/tests/fixtures/catalog/cases/rigid-joint-rope.jsonl",
    "crates/liquidfun-differential/tests/fixtures/catalog/cases/particle-groups.jsonl",
    "crates/liquidfun-differential/tests/fixtures/catalog/cases/queries-callbacks-mutations.jsonl",
    "tools/xtask/src/phase11_evidence.rs",
    "tools/xtask/src/phase11_evidence/content.rs",
    ".planning/phases/11-examples-headless-tooling-and-testbed/11-21-SUMMARY.md",
];
const AUTHORITY_REFERENCES: [&str; 13] = [
    "reference/artifacts/phase11/exact-ref.json",
    "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29927362730",
    "https://github.com/bright-builds-llc/liquidfun-rs/commit/4ea1e1e65919619d8cd1155a5461c2cda16ab7b6",
    "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29927362730/job/88947879161",
    "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29927362730/job/88947879108",
    "https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8532642627/zip#live-and-archive-sha256=2bbf2dd14fdb3a8fbae119a150e0e2292dc36f6752c54ae3be466a23415c81e0",
    "https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8532662842/zip#live-and-archive-sha256=745b2e7fdeb730f8b40f68aea7f0f776b93465ed39a30e7832dcdeeaaa46ac3d",
    "phase11-canonical-29927362730-4ea1e1e65919619d8cd1155a5461c2cda16ab7b6/identity.json#sha256=3a2fe2c222103ca60501e448d0256821a1b3c21301773649a79a51f92f776303",
    "phase11-sanitizer-29927362730-4ea1e1e65919619d8cd1155a5461c2cda16ab7b6/identity.json#sha256=4beb3be3c802b7360adff2d90df20f51c8d74719bacb523ebfad0f84b5fd7437",
    "phase11-v1.json#sha256=ea5c1364ab3e2c50aafc2edb9aa09fe436e19f4b3fe8d48ff69ece5da1bd0860",
    "phase11-evidence#semantic-sha256=248b19ecdfa6f5cd202a5d6b07783c82e097927d78e4ad87f5a4fe4c772687eb",
    ".planning/phases/11-examples-headless-tooling-and-testbed/11-22-SUMMARY.md",
    "TESTING.md#validated-phase-11-evidence-set-2026-07-22",
];

const AUTHORITY_PATH: &str = "reference/artifacts/phase11/exact-ref.json";
const AUTHORITY_SHA256: &str = "251b7b58e56508ba3e4280c50c7821296d1d5d596bd6efe5f37f031a8573d3c7";
const MAX_AUTHORITY_BYTES: u64 = 128 * 1024;
const TRACKED_SEMANTIC_FILES: [(&str, &str); 11] = [
    (
        "reference/scenario-catalog.json",
        "e0b09f385c4df9ce2245ae35c37bd4a1b9ba5dc3e083e7bd793e98bc6cccf5d9",
    ),
    (
        "reference/artifacts/phase11/scenario-mappings.json",
        "66d6d4ef4a671a3213e936cdd5d5afd94add9f5d81e0ec1cf6e7b0487902222a",
    ),
    (
        "crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json",
        "ea5c1364ab3e2c50aafc2edb9aa09fe436e19f4b3fe8d48ff69ece5da1bd0860",
    ),
    (
        "crates/liquidfun-differential/tests/fixtures/catalog/cases/particle-groups.jsonl",
        "1e5cfeefc0d5b8dcc34682703a971ee3b2b705fef943a48815e38a48a87a801e",
    ),
    (
        "crates/liquidfun-differential/tests/fixtures/catalog/cases/queries-callbacks-mutations.jsonl",
        "b647a4d9aa7722b10b3b38090083e8917a8cf0593d9533bdefdf73d41c6d1f23",
    ),
    (
        "crates/liquidfun-differential/tests/fixtures/catalog/cases/rigid-joint-rope.jsonl",
        "c98dad94a3071d756cb721fea9b8013cacef5a176f14c71c8e723b29ae2d3bb5",
    ),
    (
        "protocol/tolerances/phase6-v1.toml",
        "7f10df148852866fd20d11b8d27adcddc0ad463ac3d3d716a8946ca5c8f1c63a",
    ),
    (
        "protocol/tolerances/phase7-v1.toml",
        "fd772b2cf523a6d40bf978bc4d0da18a4564181a93e6b2bdeb8e4d40d5613311",
    ),
    (
        "protocol/tolerances/phase8-v1.toml",
        "2843ca40bec5b1c680135664c58c12a8388a7a9e86ad77f8ef5a268f3f15a6bf",
    ),
    (
        "crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json",
        "e0936090c8b8453cd464e7e56e1fa09392265ffb1da1f81d8d692667956a3fcc",
    ),
    (
        "crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/phase10-v1.json",
        "49271756565607cfc391ccfd29ca1b38d6176bcf32490fdf5e3636c0db861b7f",
    ),
];
const AUTHORIZED_MARKERS: [&str; 15] = [
    "reference/artifacts/phase11/exact-ref.json",
    "29927362730",
    "4ea1e1e65919619d8cd1155a5461c2cda16ab7b6",
    "88947879161",
    "88947879108",
    "8532642627",
    "8532662842",
    "2bbf2dd14fdb3a8fbae119a150e0e2292dc36f6752c54ae3be466a23415c81e0",
    "745b2e7fdeb730f8b40f68aea7f0f776b93465ed39a30e7832dcdeeaaa46ac3d",
    "3a2fe2c222103ca60501e448d0256821a1b3c21301773649a79a51f92f776303",
    "4beb3be3c802b7360adff2d90df20f51c8d74719bacb523ebfad0f84b5fd7437",
    "248b19ecdfa6f5cd202a5d6b07783c82e097927d78e4ad87f5a4fe4c772687eb",
    "e0b09f385c4df9ce2245ae35c37bd4a1b9ba5dc3e083e7bd793e98bc6cccf5d9",
    "66d6d4ef4a671a3213e936cdd5d5afd94add9f5d81e0ec1cf6e7b0487902222a",
    "ea5c1364ab3e2c50aafc2edb9aa09fe436e19f4b3fe8d48ff69ece5da1bd0860",
];
const REJECTED_MARKERS: [&str; 3] = ["29899265024", "8521315244", "8521345417"];

pub(super) fn promotion(
    ledger: &CompatibilityLedger,
    repository_root: &Path,
) -> Result<(), InventoryError> {
    reject_misplaced_or_rejected_authority(ledger)?;
    let promotion_started = ledger.entries.iter().any(|entry| {
        PROMOTION_IDS.contains(&entry.id.as_str())
            && entry.evidence.platform_validated.status == EvidenceStatus::Evidenced
    });
    if !promotion_started {
        return Ok(());
    }

    require_exact_authority(repository_root)?;
    for id in PROMOTION_IDS {
        let Some(entry) = ledger.entries.iter().find(|entry| entry.id == id) else {
            return Err(evidence_error(format!(
                "incomplete Phase 11 promotion: missing scoped row `{id}`"
            )));
        };
        let required = [
            (&entry.evidence.investigated, &INVESTIGATION_REFERENCES[..]),
            (&entry.evidence.planned, &PLANNING_REFERENCES[..]),
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
            if !matches_exact_evidence(record, expected) {
                return Err(evidence_error(format!(
                    "incomplete or noncanonical Phase 11 promotion for scoped row `{id}`"
                )));
            }
        }
        if entry.applicability.status != ApplicabilityStatus::Applicable
            || entry.evidence.documented_difference.status != EvidenceStatus::NotEvidenced
            || entry.evidence.intentionally_unsupported.status != EvidenceStatus::NotEvidenced
        {
            return Err(evidence_error(format!(
                "Phase 11 scoped row `{id}` must have the exact supported outcome"
            )));
        }
    }
    Ok(())
}

fn require_exact_authority(repository_root: &Path) -> Result<(), InventoryError> {
    require_regular_digest(
        repository_root,
        AUTHORITY_PATH,
        AUTHORITY_SHA256,
        MAX_AUTHORITY_BYTES,
    )?;
    let authority_bytes = fs::read(repository_root.join(AUTHORITY_PATH))
        .map_err(|error| evidence_error(format!("failed to read Phase 11 authority: {error}")))?;
    let _: serde_json::Value = serde_json::from_slice(&authority_bytes)
        .map_err(|error| evidence_error(format!("invalid Phase 11 authority JSON: {error}")))?;

    for (relative, expected_sha256) in TRACKED_SEMANTIC_FILES {
        require_regular_digest(repository_root, relative, expected_sha256, 4 * 1024 * 1024)?;
    }
    Ok(())
}

fn require_regular_digest(
    repository_root: &Path,
    relative: &str,
    expected_sha256: &str,
    max_bytes: u64,
) -> Result<(), InventoryError> {
    let path = repository_root.join(relative);
    let metadata = fs::symlink_metadata(&path).map_err(|error| {
        evidence_error(format!(
            "failed to inspect Phase 11 authority input `{relative}`: {error}"
        ))
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(evidence_error(format!(
            "Phase 11 authority input `{relative}` must be a regular non-symlink file"
        )));
    }
    if metadata.len() > max_bytes {
        return Err(evidence_error(format!(
            "Phase 11 authority input `{relative}` exceeds its reviewed bound"
        )));
    }
    let bytes = fs::read(&path).map_err(|error| {
        evidence_error(format!(
            "failed to read Phase 11 authority input `{relative}`: {error}"
        ))
    })?;
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual != expected_sha256 {
        return Err(evidence_error(format!(
            "Phase 11 authority input `{relative}` has a stale or unreviewed digest"
        )));
    }
    Ok(())
}

fn reject_misplaced_or_rejected_authority(
    ledger: &CompatibilityLedger,
) -> Result<(), InventoryError> {
    for entry in &ledger.entries {
        for (_, record) in entry.evidence.records() {
            if contains_any(record, &REJECTED_MARKERS) {
                return Err(evidence_error(format!(
                    "rejected Phase 11 authority in row `{}`",
                    entry.id
                )));
            }
            if !PROMOTION_IDS.contains(&entry.id.as_str())
                && contains_any(record, &AUTHORIZED_MARKERS)
            {
                return Err(evidence_error(format!(
                    "Phase 11 authority cannot support out-of-scope row `{}`",
                    entry.id
                )));
            }
        }
    }
    Ok(())
}

fn matches_exact_evidence(record: &EvidenceRecord, expected: &[&str]) -> bool {
    record.status == EvidenceStatus::Evidenced
        && record
            .references
            .iter()
            .map(String::as_str)
            .eq(expected.iter().copied())
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
