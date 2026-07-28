use std::collections::BTreeSet;
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

const REQUIRED_NOTICE: &str = "THIRD_PARTY_NOTICES.md";
const REQUIRED_PHASE13_FIELDS: [&str; 6] = [
    "record_class",
    "source_revision",
    "source_path",
    "derivation_kind",
    "alteration_summary",
    "notice_refs",
];
const REQUIRED_PHASE13_CLASSES: [(&str, &str, &str); 4] = [
    (
        "witness",
        "liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.cpp",
        "generated-semantic-oracle-witness",
    ),
    (
        "replay_evidence",
        ".",
        "repository-authored-replay-verification",
    ),
    (
        "staged_bundle",
        ".",
        "repository-authored-staged-evidence-bundle",
    ),
    (
        "promotion_receipt",
        ".",
        "repository-authored-promotion-receipt",
    ),
];
const REQUIRED_PHASE13_RECORDS: [(&str, &str, &str); 4] = [
    (
        "reference/artifacts/catalog/rigid-stack-v1.replay-evidence.json",
        "replay_evidence",
        "exact_bytes_sha256",
    ),
    (
        "reference/artifacts/phase13/promotion-receipt.json",
        "promotion_receipt",
        "phase13_receipt_semantic_v2",
    ),
    (
        "reference/artifacts/phase9/lifecycle-contact-witnesses.json",
        "witness",
        "exact_bytes_sha256",
    ),
    (
        "reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json",
        "witness",
        "exact_bytes_sha256",
    ),
];

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ArtifactSchemas {
    phase11_evidence: Phase11EvidenceSchema,
    phase13_evidence: Phase13EvidenceSchema,
}

impl ArtifactSchemas {
    #[cfg(test)]
    pub(super) fn current(oracle_revision: &str) -> Self {
        Self {
            phase11_evidence: Phase11EvidenceSchema {
                schema_version: 1,
                manifest_file: "phase11-v1.json".to_owned(),
                identity_file: "identity.json".to_owned(),
                protocol_version: "catalog-phase11-v1".to_owned(),
                generator_version: "phase11-evidence-v1".to_owned(),
                promotion: "exact-ref-same-run-only".to_owned(),
            },
            phase13_evidence: Phase13EvidenceSchema {
                schema_version: 1,
                required_fields: REQUIRED_PHASE13_FIELDS
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
                classes: REQUIRED_PHASE13_CLASSES
                    .into_iter()
                    .map(
                        |(record_class, source_path, derivation_kind)| Phase13EvidenceClass {
                            record_class: record_class.to_owned(),
                            source_revision: oracle_revision.to_owned(),
                            source_path: source_path.to_owned(),
                            derivation_kind: derivation_kind.to_owned(),
                            alteration_summary: "Repository-authored test schema identity."
                                .to_owned(),
                            notice_refs: vec![REQUIRED_NOTICE.to_owned()],
                        },
                    )
                    .collect(),
                records: Vec::new(),
            },
        }
    }

    pub(super) fn is_current(&self, oracle_revision: &str) -> bool {
        self.phase11_evidence.is_current() && self.phase13_evidence.is_current(oracle_revision)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Phase11EvidenceSchema {
    schema_version: u32,
    manifest_file: String,
    identity_file: String,
    protocol_version: String,
    generator_version: String,
    promotion: String,
}

impl Phase11EvidenceSchema {
    fn is_current(&self) -> bool {
        self.schema_version == 1
            && self.manifest_file == "phase11-v1.json"
            && self.identity_file == "identity.json"
            && self.protocol_version == "catalog-phase11-v1"
            && self.generator_version == "phase11-evidence-v1"
            && self.promotion == "exact-ref-same-run-only"
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Phase13EvidenceSchema {
    schema_version: u32,
    required_fields: Vec<String>,
    classes: Vec<Phase13EvidenceClass>,
    #[serde(default)]
    records: Vec<Phase13EvidenceRecord>,
}

impl Phase13EvidenceSchema {
    fn is_current(&self, oracle_revision: &str) -> bool {
        self.schema_version == 1
            && self.required_fields == REQUIRED_PHASE13_FIELDS
            && classes_are_current(&self.classes, oracle_revision)
            && records_are_current(&self.records, oracle_revision)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Phase13EvidenceClass {
    record_class: String,
    source_revision: String,
    source_path: String,
    derivation_kind: String,
    alteration_summary: String,
    notice_refs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Phase13EvidenceRecord {
    record_class: String,
    path: String,
    sha256: String,
    digest_mode: String,
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

fn classes_are_current(classes: &[Phase13EvidenceClass], oracle_revision: &str) -> bool {
    if classes.len() != REQUIRED_PHASE13_CLASSES.len() {
        return false;
    }
    let mut actual = BTreeSet::new();
    for class in classes {
        if class.source_revision != oracle_revision
            || class.alteration_summary.trim().is_empty()
            || class.notice_refs != [REQUIRED_NOTICE]
            || !confined_source_path(&class.source_path)
            || !actual.insert((
                class.record_class.as_str(),
                class.source_path.as_str(),
                class.derivation_kind.as_str(),
            ))
        {
            return false;
        }
    }
    actual
        == REQUIRED_PHASE13_CLASSES
            .into_iter()
            .collect::<BTreeSet<_>>()
}

fn records_are_current(records: &[Phase13EvidenceRecord], oracle_revision: &str) -> bool {
    if records.is_empty() {
        return true;
    }
    if records.len() != REQUIRED_PHASE13_RECORDS.len() {
        return false;
    }
    let mut actual = BTreeSet::new();
    for record in records {
        if !valid_revision(&record.generator_revision)
            || !valid_revision(&record.producer_sha)
            || !valid_digest(&record.bundle_sha256)
            || !valid_digest(&record.sha256)
            || record.source_revision != oracle_revision
            || record.derivation_kind.trim().is_empty()
            || record.alteration_summary.trim().is_empty()
            || record.notice_refs != [REQUIRED_NOTICE]
            || record.reviewer.trim().is_empty()
            || !confined_source_path(&record.path)
            || !confined_source_path(&record.source_path)
            || !actual.insert((
                record.path.as_str(),
                record.record_class.as_str(),
                record.digest_mode.as_str(),
            ))
        {
            return false;
        }
    }
    actual
        == REQUIRED_PHASE13_RECORDS
            .into_iter()
            .collect::<BTreeSet<_>>()
}

fn confined_source_path(value: &str) -> bool {
    value == "."
        || (!value.is_empty()
            && Path::new(value)
                .components()
                .all(|component| matches!(component, Component::Normal(_))))
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
