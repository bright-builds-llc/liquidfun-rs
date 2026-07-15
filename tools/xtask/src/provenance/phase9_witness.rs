//! Fail-closed validation for the pre-implementation Phase 9 oracle witnesses.

use std::collections::BTreeSet;
use std::path::Path;

use serde::Deserialize;

use super::{ProvenanceError, read_json, require_revision, sha256};

const WITNESS_PATH: &str = "reference/artifacts/phase9/lifecycle-contact-witnesses.json";
const PROVENANCE_PATH: &str =
    "reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json";
const PROBE_SOURCE_PATH: &str = "tools/reference/src/phase9_lifecycle_contact_witness.cpp";
const EXPECTED_TARGET: &str = "phase9-lifecycle-contact-witness";
const EXPECTED_PRESET: &str = "oracle-debug";
const EXPECTED_ARGUMENTS: [&str; 5] = [
    "target/reference/oracle-debug/phase9-lifecycle-contact-witness",
    "--output",
    WITNESS_PATH,
    "--provenance",
    PROVENANCE_PATH,
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WitnessDocument {
    schema_version: u64,
    oracle_revision: String,
    witnesses: Vec<Witness>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "scenario_id", deny_unknown_fields)]
enum Witness {
    #[serde(rename = "equal_quantized_expiration")]
    EqualExpiration {
        particle_count: usize,
        quantized_expiration: i32,
        creation_order: Vec<String>,
        expiration_order: Vec<String>,
        oldest_selection_order: Vec<String>,
    },
    #[serde(rename = "strict_contact_pruning")]
    StrictContactPruning {
        fixture_count: usize,
        equal_weight_bits: String,
        candidate_order: Vec<String>,
        strict_order: Vec<String>,
        outcomes: Vec<ContactOutcome>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContactOutcome {
    fixture_id: String,
    result: ContactResult,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ContactResult {
    Kept,
    Removed,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WitnessProvenance {
    schema_version: u64,
    oracle_revision: String,
    adapter_content_sha256: String,
    probe_source_sha256: String,
    compiler_id: String,
    compiler_version: String,
    target: String,
    cmake_preset: String,
    cmake_target: String,
    exact_argv: Vec<String>,
    generation_timestamp: String,
    witness_sha256: String,
}

pub(super) fn validate(
    repository_root: &Path,
    oracle_revision: &str,
) -> Result<(), ProvenanceError> {
    let witness_path = repository_root.join(WITNESS_PATH);
    let provenance_path = repository_root.join(PROVENANCE_PATH);
    match (witness_path.is_file(), provenance_path.is_file()) {
        (false, false) => return Ok(()),
        (true, true) => {}
        _ => {
            return Err(ProvenanceError::new(
                "phase9-witness",
                "Phase 9 lifecycle/contact witness and provenance must exist together",
            ));
        }
    }

    let witnesses: WitnessDocument = read_json(&witness_path, WITNESS_PATH)?;
    if witnesses.schema_version != 1 {
        return Err(ProvenanceError::new(
            "phase9-witness",
            "Phase 9 lifecycle/contact witness must use schema version 1",
        ));
    }
    require_revision(
        "Phase 9 lifecycle/contact witness",
        oracle_revision,
        &witnesses.oracle_revision,
    )?;
    validate_witnesses(&witnesses.witnesses)?;

    let provenance: WitnessProvenance = read_json(&provenance_path, PROVENANCE_PATH)?;
    if provenance.schema_version != 1 {
        return Err(ProvenanceError::new(
            "phase9-witness",
            "Phase 9 lifecycle/contact provenance must use schema version 1",
        ));
    }
    require_revision(
        "Phase 9 lifecycle/contact provenance",
        oracle_revision,
        &provenance.oracle_revision,
    )?;
    validate_provenance(repository_root, &provenance)?;

    println!(
        "Phase 9 lifecycle/contact witness verified: {}",
        provenance.witness_sha256
    );
    Ok(())
}

fn validate_witnesses(witnesses: &[Witness]) -> Result<(), ProvenanceError> {
    let [
        Witness::EqualExpiration {
            particle_count,
            quantized_expiration,
            creation_order,
            expiration_order,
            oldest_selection_order,
        },
        Witness::StrictContactPruning {
            fixture_count,
            equal_weight_bits,
            candidate_order,
            strict_order,
            outcomes,
        },
    ] = witnesses
    else {
        return Err(ProvenanceError::new(
            "phase9-witness",
            "expected ordered equal-expiration and strict-contact witness records",
        ));
    };

    validate_equal_expiration(
        *particle_count,
        *quantized_expiration,
        creation_order,
        expiration_order,
        oldest_selection_order,
    )?;
    validate_strict_contacts(
        *fixture_count,
        equal_weight_bits,
        candidate_order,
        strict_order,
        outcomes,
    )
}

fn validate_equal_expiration(
    particle_count: usize,
    quantized_expiration: i32,
    creation_order: &[String],
    expiration_order: &[String],
    oldest_selection_order: &[String],
) -> Result<(), ProvenanceError> {
    if particle_count == 0
        || quantized_expiration <= 0
        || creation_order.len() != particle_count
        || expiration_order.len() != particle_count
        || oldest_selection_order.len() != particle_count
    {
        return Err(ProvenanceError::new(
            "phase9-witness",
            "equal-expiration orders must all cover the declared nonzero particle count",
        ));
    }
    let created = unique_ids(creation_order, "equal-expiration creation order")?;
    let expiration = unique_ids(expiration_order, "equal-expiration sorted order")?;
    let oldest = unique_ids(oldest_selection_order, "equal-expiration oldest order")?;
    if created != expiration || created != oldest {
        return Err(ProvenanceError::new(
            "phase9-witness",
            "equal-expiration orders must contain the same semantic particle IDs",
        ));
    }
    Ok(())
}

fn validate_strict_contacts(
    fixture_count: usize,
    equal_weight_bits: &str,
    candidate_order: &[String],
    strict_order: &[String],
    outcomes: &[ContactOutcome],
) -> Result<(), ProvenanceError> {
    if fixture_count == 0
        || candidate_order.len() != fixture_count
        || strict_order.is_empty()
        || strict_order.len() >= fixture_count
        || outcomes.len() != fixture_count
        || !valid_u32_bits(equal_weight_bits)
    {
        return Err(ProvenanceError::new(
            "phase9-witness",
            "strict-contact witness must declare a tied candidate set with real pruning",
        ));
    }
    let candidates = unique_ids(candidate_order, "strict-contact candidates")?;
    let kept = unique_ids(strict_order, "strict-contact kept order")?;
    if !kept.is_subset(&candidates) {
        return Err(ProvenanceError::new(
            "phase9-witness",
            "strict-contact kept IDs must be a subset of candidates",
        ));
    }

    let mut outcome_ids = BTreeSet::new();
    for outcome in outcomes {
        if outcome.fixture_id.is_empty() || !outcome_ids.insert(outcome.fixture_id.as_str()) {
            return Err(ProvenanceError::new(
                "phase9-witness",
                "strict-contact outcomes contain an empty or duplicate fixture ID",
            ));
        }
        let expected = if kept.contains(outcome.fixture_id.as_str()) {
            ContactResult::Kept
        } else {
            ContactResult::Removed
        };
        if outcome.result != expected {
            return Err(ProvenanceError::new(
                "phase9-witness",
                format!(
                    "strict-contact outcome for `{}` disagrees with kept order",
                    outcome.fixture_id
                ),
            ));
        }
    }
    if outcome_ids != candidates {
        return Err(ProvenanceError::new(
            "phase9-witness",
            "strict-contact outcomes must cover every candidate fixture exactly once",
        ));
    }
    Ok(())
}

fn validate_provenance(
    repository_root: &Path,
    provenance: &WitnessProvenance,
) -> Result<(), ProvenanceError> {
    let actual_witness_sha256 = sha256(&repository_root.join(WITNESS_PATH))?;
    if provenance.witness_sha256 != actual_witness_sha256 {
        return Err(ProvenanceError::new(
            "hash",
            format!(
                "Phase 9 lifecycle/contact witness SHA-256 mismatch: expected `{}`, actual `{actual_witness_sha256}`",
                provenance.witness_sha256
            ),
        ));
    }
    let actual_probe_sha256 = sha256(&repository_root.join(PROBE_SOURCE_PATH))?;
    if provenance.probe_source_sha256 != actual_probe_sha256 {
        return Err(ProvenanceError::new(
            "hash",
            "Phase 9 lifecycle/contact probe source SHA-256 mismatch",
        ));
    }
    let actual_adapter_sha256 = liquidfun_differential::adapter_source_digest(repository_root)
        .map_err(|error| ProvenanceError::new("hash", error.to_string()))?;
    if provenance.adapter_content_sha256 != actual_adapter_sha256 {
        return Err(ProvenanceError::new(
            "hash",
            "Phase 9 lifecycle/contact adapter content SHA-256 mismatch",
        ));
    }
    if provenance.compiler_id.is_empty()
        || provenance.compiler_version.is_empty()
        || provenance.target.is_empty()
        || provenance.cmake_preset != EXPECTED_PRESET
        || provenance.cmake_target != EXPECTED_TARGET
        || provenance.exact_argv != EXPECTED_ARGUMENTS
        || !valid_utc_timestamp(&provenance.generation_timestamp)
    {
        return Err(ProvenanceError::new(
            "identity",
            "Phase 9 lifecycle/contact provenance has invalid tool, target, command, or timestamp identity",
        ));
    }
    Ok(())
}

fn unique_ids<'a>(ids: &'a [String], label: &str) -> Result<BTreeSet<&'a str>, ProvenanceError> {
    let unique = ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if unique.len() != ids.len() || unique.contains("") {
        return Err(ProvenanceError::new(
            "phase9-witness",
            format!("{label} contains an empty or duplicate semantic ID"),
        ));
    }
    Ok(unique)
}

fn valid_u32_bits(value: &str) -> bool {
    value.len() == 10
        && value.starts_with("0x")
        && value[2..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_utc_timestamp(value: &str) -> bool {
    value.len() == 20
        && value.as_bytes().get(4) == Some(&b'-')
        && value.as_bytes().get(7) == Some(&b'-')
        && value.as_bytes().get(10) == Some(&b'T')
        && value.as_bytes().get(13) == Some(&b':')
        && value.as_bytes().get(16) == Some(&b':')
        && value.ends_with('Z')
        && value
            .bytes()
            .enumerate()
            .filter(|(index, _)| ![4, 7, 10, 13, 16, 19].contains(index))
            .all(|(_, byte)| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    use serde_json::json;

    use super::{PROBE_SOURCE_PATH, PROVENANCE_PATH, WITNESS_PATH, sha256, validate};

    const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
    const ADAPTER_PATH: &str = "tools/reference/src/adapter.cpp";
    static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn adapter_input_change_requires_refreshed_witness_provenance() -> Result<(), Box<dyn Error>> {
        // Arrange
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "liquidfun-phase9-witness-provenance-{}-{id}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(root.join("reference/artifacts/phase9"))?;
        fs::create_dir_all(root.join("tools/reference/src"))?;
        fs::write(
            root.join("tools/reference/adapter-inputs.txt"),
            format!("{ADAPTER_PATH}\n"),
        )?;
        fs::write(root.join(ADAPTER_PATH), "adapter-v1\n")?;
        fs::write(root.join(PROBE_SOURCE_PATH), "probe-v1\n")?;
        fs::write(
            root.join(WITNESS_PATH),
            serde_json::to_string_pretty(&json!({
                "schema_version": 1,
                "oracle_revision": REVISION,
                "witnesses": [
                    {
                        "scenario_id": "equal_quantized_expiration",
                        "particle_count": 1,
                        "quantized_expiration": 1,
                        "creation_order": ["particle-0"],
                        "expiration_order": ["particle-0"],
                        "oldest_selection_order": ["particle-0"]
                    },
                    {
                        "scenario_id": "strict_contact_pruning",
                        "fixture_count": 2,
                        "equal_weight_bits": "0x3f800000",
                        "candidate_order": ["fixture-0", "fixture-1"],
                        "strict_order": ["fixture-0"],
                        "outcomes": [
                            {"fixture_id": "fixture-0", "result": "kept"},
                            {"fixture_id": "fixture-1", "result": "removed"}
                        ]
                    }
                ]
            }))? + "\n",
        )?;
        let witness_sha256 = sha256(&root.join(WITNESS_PATH))?;
        let probe_source_sha256 = sha256(&root.join(PROBE_SOURCE_PATH))?;
        let adapter_content_sha256 = liquidfun_differential::adapter_source_digest(&root)?;
        fs::write(
            root.join(PROVENANCE_PATH),
            serde_json::to_string_pretty(&json!({
                "schema_version": 1,
                "oracle_revision": REVISION,
                "adapter_content_sha256": adapter_content_sha256,
                "probe_source_sha256": probe_source_sha256,
                "compiler_id": "fixture-compiler",
                "compiler_version": "1.0.0",
                "target": "fixture-target",
                "cmake_preset": "oracle-debug",
                "cmake_target": "phase9-lifecycle-contact-witness",
                "exact_argv": [
                    "target/reference/oracle-debug/phase9-lifecycle-contact-witness",
                    "--output",
                    WITNESS_PATH,
                    "--provenance",
                    PROVENANCE_PATH
                ],
                "generation_timestamp": "2026-07-15T00:00:00Z",
                "witness_sha256": witness_sha256
            }))? + "\n",
        )?;
        validate(&root, REVISION)?;

        // Act
        fs::write(root.join(ADAPTER_PATH), "adapter-v2\n")?;
        let error = validate(&root, REVISION).expect_err("stale provenance must fail closed");

        // Assert
        assert_eq!(error.category, "hash");
        assert!(error.message.contains("adapter content SHA-256 mismatch"));
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
