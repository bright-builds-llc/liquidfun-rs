//! Phase 13 evidence-class provenance contracts.

use std::collections::BTreeSet;
use std::path::{Component, Path};

use serde::Deserialize;

use super::ProvenanceError;

const EXPECTED_CLASSES: [&str; 4] = [
    "witness",
    "replay_evidence",
    "staged_bundle",
    "promotion_receipt",
];
const EXPECTED_FIELDS: [&str; 6] = [
    "record_class",
    "source_revision",
    "source_path",
    "derivation_kind",
    "alteration_summary",
    "notice_refs",
];
const REQUIRED_NOTICE: &str = "THIRD_PARTY_NOTICES.md";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Phase13EvidenceSchema {
    schema_version: u64,
    required_fields: Vec<String>,
    classes: Vec<Phase13EvidenceClass>,
    #[serde(default)]
    records: Vec<Phase13EvidenceRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Phase13EvidenceClass {
    record_class: String,
    source_revision: String,
    source_path: String,
    derivation_kind: String,
    alteration_summary: String,
    notice_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Phase13EvidenceRecord {
    record_class: String,
    path: String,
    sha256: String,
    generator_revision: String,
    producer_sha: String,
    bundle_sha256: String,
    source_revision: String,
    source_path: String,
    derivation_kind: String,
    alteration_summary: String,
    notice_refs: Vec<String>,
    reviewer: String,
}

#[cfg(test)]
#[allow(dead_code)]
pub(super) fn parse_and_validate(
    contents: &str,
    oracle_revision: &str,
) -> Result<Phase13EvidenceSchema, ProvenanceError> {
    #[derive(Deserialize)]
    struct Document {
        artifact_schemas: ArtifactSchemas,
    }

    #[derive(Deserialize)]
    struct ArtifactSchemas {
        phase13_evidence: Phase13EvidenceSchema,
    }

    let document: Document = toml::from_str(contents).map_err(|error| {
        ProvenanceError::new(
            "schema",
            format!("invalid Phase 13 evidence schema: {error}"),
        )
    })?;
    validate(&document.artifact_schemas.phase13_evidence, oracle_revision)?;
    Ok(document.artifact_schemas.phase13_evidence)
}

pub(super) fn validate(
    schema: &Phase13EvidenceSchema,
    oracle_revision: &str,
) -> Result<(), ProvenanceError> {
    if schema.schema_version != 1 || schema.required_fields != EXPECTED_FIELDS {
        return Err(ProvenanceError::new(
            "schema",
            "Phase 13 evidence must use schema version 1 and the exact required field contract",
        ));
    }

    let mut classes = BTreeSet::new();
    for class in &schema.classes {
        if !classes.insert(class.record_class.as_str()) {
            return Err(ProvenanceError::new(
                "schema",
                format!("duplicate Phase 13 evidence class `{}`", class.record_class),
            ));
        }
        validate_class(class, oracle_revision)?;
    }
    let actual = classes.into_iter().collect::<Vec<_>>();
    let expected = EXPECTED_CLASSES.into_iter().collect::<BTreeSet<_>>();
    let expected = expected.into_iter().collect::<Vec<_>>();
    if actual != expected {
        return Err(ProvenanceError::new(
            "schema",
            "Phase 13 evidence classes must define witness, replay_evidence, staged_bundle, and promotion_receipt exactly once",
        ));
    }
    validate_records(&schema.records, oracle_revision)?;
    Ok(())
}

fn validate_records(
    records: &[Phase13EvidenceRecord],
    oracle_revision: &str,
) -> Result<(), ProvenanceError> {
    if records.is_empty() {
        return Ok(());
    }
    let expected_paths = [
        "reference/artifacts/catalog/rigid-stack-v1.replay-evidence.json",
        "reference/artifacts/phase13/promotion-receipt.json",
        "reference/artifacts/phase9/lifecycle-contact-witnesses.json",
        "reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let mut actual_paths = BTreeSet::new();
    for record in records {
        if !actual_paths.insert(record.path.as_str())
            || !EXPECTED_CLASSES.contains(&record.record_class.as_str())
            || record.record_class == "staged_bundle"
            || !valid_revision(&record.generator_revision)
            || !valid_revision(&record.producer_sha)
            || !valid_digest(&record.bundle_sha256)
            || !valid_digest(&record.sha256)
            || record.source_revision != oracle_revision
            || record.derivation_kind.trim().is_empty()
            || record.alteration_summary.trim().is_empty()
            || record.notice_refs != [REQUIRED_NOTICE]
            || record.reviewer.trim().is_empty()
        {
            return Err(ProvenanceError::new(
                "schema",
                "Phase 13 promoted evidence record is incomplete or malformed",
            ));
        }
        validate_source_path(&record.path, &record.record_class)?;
        validate_source_path(&record.source_path, &record.record_class)?;
    }
    if actual_paths != expected_paths {
        return Err(ProvenanceError::new(
            "schema",
            "Phase 13 promoted evidence records must cover the exact reviewed artifact set",
        ));
    }
    Ok(())
}

fn valid_revision(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(lower_hex)
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(lower_hex)
}

fn lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

fn validate_class(
    class: &Phase13EvidenceClass,
    oracle_revision: &str,
) -> Result<(), ProvenanceError> {
    if class.source_revision != oracle_revision {
        return Err(ProvenanceError::new(
            "revision",
            format!(
                "Phase 13 `{}` source revision mismatch: expected `{oracle_revision}`, actual `{}`",
                class.record_class, class.source_revision
            ),
        ));
    }
    validate_source_path(&class.source_path, &class.record_class)?;
    for (field, value) in [
        ("derivation_kind", class.derivation_kind.as_str()),
        ("alteration_summary", class.alteration_summary.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(ProvenanceError::new(
                "schema",
                format!("Phase 13 `{}` has empty {field}", class.record_class),
            ));
        }
    }
    if class.notice_refs != [REQUIRED_NOTICE] {
        return Err(ProvenanceError::new(
            "notice",
            format!(
                "Phase 13 `{}` must reference {REQUIRED_NOTICE}",
                class.record_class
            ),
        ));
    }
    Ok(())
}

fn validate_source_path(value: &str, record_class: &str) -> Result<(), ProvenanceError> {
    if value.is_empty()
        || value.contains('*')
        || value.contains('\\')
        || Path::new(value)
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir))
    {
        return Err(ProvenanceError::new(
            "path",
            format!("Phase 13 `{record_class}` has invalid source_path `{value}`"),
        ));
    }
    Ok(())
}
