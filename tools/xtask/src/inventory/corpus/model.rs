//! Invariant-bearing types for semantic upstream corpus records.

use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter};
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

pub(crate) const CORPUS_SCHEMA_VERSION: u64 = 1;
pub(crate) const MAX_CORPUS_ITEMS: usize = 2_048;
pub(crate) const MAX_EVIDENCE_MAPPINGS: usize = 32;
const MAX_ITEM_ID_BYTES: usize = 160;
const MAX_SOURCE_PATH_BYTES: usize = 512;
const MAX_SOURCE_SYMBOL_BYTES: usize = 256;
const MAX_EVIDENCE_REFERENCE_BYTES: usize = 512;
const MAX_REVIEWER_BYTES: usize = 128;
const MIN_RATIONALE_BYTES: usize = 24;
const MAX_RATIONALE_BYTES: usize = 1_024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CorpusErrorKind {
    CollectionLimit,
    DepthLimit,
    DuplicateId,
    DuplicateSourceIdentity,
    Evidence,
    InputLimit,
    ItemId,
    Path,
    Rationale,
    Review,
    Revision,
    Schema,
    SourceIdentity,
    TerminalOutcome,
}

impl CorpusErrorKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::CollectionLimit => "collection-limit",
            Self::DepthLimit => "depth-limit",
            Self::DuplicateId => "duplicate-id",
            Self::DuplicateSourceIdentity => "duplicate-source-identity",
            Self::Evidence => "evidence",
            Self::InputLimit => "input-limit",
            Self::ItemId => "item-id",
            Self::Path => "path",
            Self::Rationale => "rationale",
            Self::Review => "review",
            Self::Revision => "revision",
            Self::Schema => "schema",
            Self::SourceIdentity => "source-identity",
            Self::TerminalOutcome => "terminal-outcome",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CorpusError {
    kind: CorpusErrorKind,
    item_id: Option<String>,
}

impl CorpusError {
    pub(crate) const fn new(kind: CorpusErrorKind) -> Self {
        Self {
            kind,
            item_id: None,
        }
    }

    fn for_item(kind: CorpusErrorKind, item_id: &CorpusItemId) -> Self {
        Self {
            kind,
            item_id: Some(item_id.0.clone()),
        }
    }

    pub(crate) const fn category(&self) -> &'static str {
        self.kind.as_str()
    }
}

impl Display for CorpusError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "corpus/{}", self.category())?;
        if let Some(item_id) = &self.item_id {
            write!(formatter, ": item `{item_id}`")?;
        }
        Ok(())
    }
}

impl std::error::Error for CorpusError {}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub(crate) struct CorpusSchemaVersion(u64);

impl CorpusSchemaVersion {
    fn parse(value: u64) -> Result<Self, CorpusError> {
        if value != CORPUS_SCHEMA_VERSION {
            return Err(CorpusError::new(CorpusErrorKind::Schema));
        }
        Ok(Self(value))
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub(crate) struct CorpusItemId(String);

impl CorpusItemId {
    fn parse(value: String, kind: CorpusKind) -> Result<Self, CorpusError> {
        let expected_prefix = kind.id_prefix();
        let valid_bytes = value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        });
        if value.len() > MAX_ITEM_ID_BYTES
            || !value.starts_with(expected_prefix)
            || value.len() == expected_prefix.len()
            || !valid_bytes
        {
            return Err(CorpusError::new(CorpusErrorKind::ItemId));
        }
        Ok(Self(value))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorpusKind {
    Example,
    TestbedEntry,
    UpstreamTest,
}

impl CorpusKind {
    const fn id_prefix(self) -> &'static str {
        match self {
            Self::Example => "example.",
            Self::TestbedEntry => "testbed.",
            Self::UpstreamTest => "upstream-test.",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Applicability {
    Applicable,
    ReviewedExclusion,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TerminalDisposition {
    DocumentedDifference,
    EquivalentEvidence,
    IntentionalNonSupport,
    NativePort,
    ReviewedIrrelevance,
}

impl TerminalDisposition {
    const fn expected_applicability(self) -> Applicability {
        match self {
            Self::DocumentedDifference | Self::EquivalentEvidence | Self::NativePort => {
                Applicability::Applicable
            }
            Self::IntentionalNonSupport | Self::ReviewedIrrelevance => {
                Applicability::ReviewedExclusion
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CompatibilityImpact {
    Api,
    Behavioral,
    None,
    Tooling,
    VisualOnly,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EvidenceKind {
    CompatibilityLedger,
    NativeScenario,
    NativeTest,
    RegressionFixture,
    Review,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct EvidenceMapping {
    kind: EvidenceKind,
    reference: String,
}

impl EvidenceMapping {
    fn from_raw(raw: RawEvidenceMapping, item_id: &CorpusItemId) -> Result<Self, CorpusError> {
        if raw.reference.is_empty()
            || raw.reference.len() > MAX_EVIDENCE_REFERENCE_BYTES
            || raw.reference.trim() != raw.reference
            || raw.reference == item_id.0
            || !valid_evidence_reference(&raw.reference)
        {
            return Err(CorpusError::for_item(CorpusErrorKind::Evidence, item_id));
        }
        Ok(Self {
            kind: raw.kind,
            reference: raw.reference,
        })
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SourceIdentity {
    path: String,
    symbol: String,
}

impl SourceIdentity {
    fn from_raw(
        raw: RawSourceIdentity,
        kind: CorpusKind,
        item_id: &CorpusItemId,
    ) -> Result<Self, CorpusError> {
        validate_relative_path(&raw.path)
            .map_err(|()| CorpusError::for_item(CorpusErrorKind::Path, item_id))?;
        if raw.path.len() > MAX_SOURCE_PATH_BYTES
            || raw.symbol.is_empty()
            || raw.symbol.len() > MAX_SOURCE_SYMBOL_BYTES
            || raw.symbol.trim() != raw.symbol
            || !raw.symbol.is_ascii()
        {
            return Err(CorpusError::for_item(
                CorpusErrorKind::SourceIdentity,
                item_id,
            ));
        }
        if kind == CorpusKind::UpstreamTest && !raw.symbol.contains('.') {
            return Err(CorpusError::for_item(
                CorpusErrorKind::SourceIdentity,
                item_id,
            ));
        }
        Ok(Self {
            path: raw.path,
            symbol: raw.symbol,
        })
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct ReviewRecord {
    reviewer: String,
    reviewed_on: String,
    rationale: String,
}

impl ReviewRecord {
    fn from_raw(raw: RawReviewRecord, item_id: &CorpusItemId) -> Result<Self, CorpusError> {
        if raw.reviewer.is_empty()
            || raw.reviewer.len() > MAX_REVIEWER_BYTES
            || raw.reviewer.trim() != raw.reviewer
            || !valid_review_date(&raw.reviewed_on)
        {
            return Err(CorpusError::for_item(CorpusErrorKind::Review, item_id));
        }
        validate_rationale(&raw.rationale, item_id)?;
        Ok(Self {
            reviewer: raw.reviewer,
            reviewed_on: raw.reviewed_on,
            rationale: raw.rationale,
        })
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct CorpusItem {
    id: CorpusItemId,
    kind: CorpusKind,
    source: SourceIdentity,
    applicability: Applicability,
    disposition: TerminalDisposition,
    compatibility_impact: CompatibilityImpact,
    evidence: Vec<EvidenceMapping>,
    review: ReviewRecord,
}

impl CorpusItem {
    fn from_raw(raw: RawCorpusItem) -> Result<Self, CorpusError> {
        let id = CorpusItemId::parse(raw.id, raw.kind)?;
        let source = SourceIdentity::from_raw(raw.source, raw.kind, &id)?;
        let review = ReviewRecord::from_raw(raw.review, &id)?;
        let evidence = checked_evidence(raw.evidence, &id)?;
        validate_terminal_outcome(
            raw.applicability,
            raw.disposition,
            raw.compatibility_impact,
            &evidence,
            &id,
        )?;
        Ok(Self {
            id,
            kind: raw.kind,
            source,
            applicability: raw.applicability,
            disposition: raw.disposition,
            compatibility_impact: raw.compatibility_impact,
            evidence,
            review,
        })
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct CorpusManifest {
    schema_version: CorpusSchemaVersion,
    oracle_revision: String,
    items: Vec<CorpusItem>,
}

impl CorpusManifest {
    pub(super) fn from_raw(
        raw: RawCorpusManifest,
        expected_revision: &str,
    ) -> Result<Self, CorpusError> {
        let schema_version = CorpusSchemaVersion::parse(raw.schema_version)?;
        validate_revision(&raw.oracle_revision, expected_revision)?;
        if raw.items.is_empty() || raw.items.len() > MAX_CORPUS_ITEMS {
            return Err(CorpusError::new(CorpusErrorKind::CollectionLimit));
        }

        let mut ids = BTreeSet::new();
        let mut source_identities = BTreeSet::new();
        let mut items = Vec::with_capacity(raw.items.len());
        for raw_item in raw.items {
            let item = CorpusItem::from_raw(raw_item)?;
            if !ids.insert(item.id.clone()) {
                return Err(CorpusError::for_item(
                    CorpusErrorKind::DuplicateId,
                    &item.id,
                ));
            }
            if !source_identities.insert(item.source.clone()) {
                return Err(CorpusError::for_item(
                    CorpusErrorKind::DuplicateSourceIdentity,
                    &item.id,
                ));
            }
            items.push(item);
        }
        Ok(Self {
            schema_version,
            oracle_revision: raw.oracle_revision,
            items,
        })
    }
}

fn checked_evidence(
    raw_evidence: Vec<RawEvidenceMapping>,
    item_id: &CorpusItemId,
) -> Result<Vec<EvidenceMapping>, CorpusError> {
    if raw_evidence.is_empty() || raw_evidence.len() > MAX_EVIDENCE_MAPPINGS {
        return Err(CorpusError::for_item(
            CorpusErrorKind::CollectionLimit,
            item_id,
        ));
    }
    let mut unique_references = BTreeSet::new();
    let mut evidence = Vec::with_capacity(raw_evidence.len());
    for raw in raw_evidence {
        let mapping = EvidenceMapping::from_raw(raw, item_id)?;
        if !unique_references.insert(mapping.reference.clone()) {
            return Err(CorpusError::for_item(CorpusErrorKind::Evidence, item_id));
        }
        evidence.push(mapping);
    }
    Ok(evidence)
}

fn validate_terminal_outcome(
    applicability: Applicability,
    disposition: TerminalDisposition,
    impact: CompatibilityImpact,
    evidence: &[EvidenceMapping],
    item_id: &CorpusItemId,
) -> Result<(), CorpusError> {
    if applicability != disposition.expected_applicability() {
        return Err(CorpusError::for_item(
            CorpusErrorKind::TerminalOutcome,
            item_id,
        ));
    }
    let has_kind = |kind| evidence.iter().any(|mapping| mapping.kind == kind);
    let valid_evidence = match disposition {
        TerminalDisposition::NativePort => {
            has_kind(EvidenceKind::NativeScenario) || has_kind(EvidenceKind::NativeTest)
        }
        TerminalDisposition::EquivalentEvidence => evidence.iter().any(|mapping| {
            matches!(
                mapping.kind,
                EvidenceKind::CompatibilityLedger
                    | EvidenceKind::NativeScenario
                    | EvidenceKind::NativeTest
                    | EvidenceKind::RegressionFixture
            )
        }),
        TerminalDisposition::DocumentedDifference => has_kind(EvidenceKind::CompatibilityLedger),
        TerminalDisposition::ReviewedIrrelevance | TerminalDisposition::IntentionalNonSupport => {
            has_kind(EvidenceKind::Review)
        }
    };
    let valid_impact = match disposition {
        TerminalDisposition::ReviewedIrrelevance => impact == CompatibilityImpact::None,
        TerminalDisposition::IntentionalNonSupport => impact != CompatibilityImpact::None,
        _ => true,
    };
    if !valid_evidence || !valid_impact {
        return Err(CorpusError::for_item(
            CorpusErrorKind::TerminalOutcome,
            item_id,
        ));
    }
    Ok(())
}

fn validate_revision(actual: &str, expected: &str) -> Result<(), CorpusError> {
    let valid = actual.len() == 40
        && actual
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
    if !valid || actual != expected {
        return Err(CorpusError::new(CorpusErrorKind::Revision));
    }
    Ok(())
}

fn validate_relative_path(value: &str) -> Result<(), ()> {
    if value.is_empty() || value.contains('\\') {
        return Err(());
    }
    let path = Path::new(value);
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(());
    }
    Ok(())
}

fn valid_evidence_reference(value: &str) -> bool {
    if value.starts_with("https://") {
        return !value.contains(char::is_whitespace);
    }
    let (path, _) = value.split_once('#').unwrap_or((value, ""));
    validate_relative_path(path).is_ok()
}

fn valid_review_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
    {
        return false;
    }
    let month = &value[5..7];
    let day = &value[8..10];
    ("01"..="12").contains(&month) && ("01"..="31").contains(&day)
}

fn validate_rationale(value: &str, item_id: &CorpusItemId) -> Result<(), CorpusError> {
    let normalized = value.trim().to_ascii_lowercase();
    let vague_rationales = ["n/a", "none", "not applicable", "todo", "see item"];
    if value.len() < MIN_RATIONALE_BYTES
        || value.len() > MAX_RATIONALE_BYTES
        || value.trim() != value
        || vague_rationales.contains(&normalized.as_str())
        || normalized.contains(&item_id.0)
    {
        return Err(CorpusError::for_item(CorpusErrorKind::Rationale, item_id));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawCorpusManifest {
    schema_version: u64,
    oracle_revision: String,
    items: Vec<RawCorpusItem>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCorpusItem {
    id: String,
    kind: CorpusKind,
    source: RawSourceIdentity,
    applicability: Applicability,
    disposition: TerminalDisposition,
    compatibility_impact: CompatibilityImpact,
    evidence: Vec<RawEvidenceMapping>,
    review: RawReviewRecord,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSourceIdentity {
    path: String,
    symbol: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEvidenceMapping {
    kind: EvidenceKind,
    reference: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawReviewRecord {
    reviewer: String,
    reviewed_on: String,
    rationale: String,
}
