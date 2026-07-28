//! Invariant-bearing types for semantic upstream corpus records.

use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter};
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

#[path = "model/validation.rs"]
mod validation;

#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use validation::*;

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

    // The standalone corpus-model test includes this file without inventory consumers.
    #[allow(dead_code)]
    pub(crate) const fn inventory_category(&self) -> &'static str {
        match self.kind {
            CorpusErrorKind::CollectionLimit => "corpus-collection-limit",
            CorpusErrorKind::DepthLimit => "corpus-depth-limit",
            CorpusErrorKind::DuplicateId => "corpus-duplicate-id",
            CorpusErrorKind::DuplicateSourceIdentity => "corpus-duplicate-source-identity",
            CorpusErrorKind::Evidence => "corpus-evidence",
            CorpusErrorKind::InputLimit => "corpus-input-limit",
            CorpusErrorKind::ItemId => "corpus-item-id",
            CorpusErrorKind::Path => "corpus-path",
            CorpusErrorKind::Rationale => "corpus-rationale",
            CorpusErrorKind::Review => "corpus-review",
            CorpusErrorKind::Revision => "corpus-revision",
            CorpusErrorKind::Schema => "corpus-schema",
            CorpusErrorKind::SourceIdentity => "corpus-source-identity",
            CorpusErrorKind::TerminalOutcome => "corpus-terminal-outcome",
        }
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

    #[allow(dead_code)]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Example => "example",
            Self::TestbedEntry => "testbed entry",
            Self::UpstreamTest => "upstream test",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Applicability {
    Applicable,
    ReviewedExclusion,
}

impl Applicability {
    #[allow(dead_code)]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Applicable => "applicable",
            Self::ReviewedExclusion => "reviewed exclusion",
        }
    }
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

    #[allow(dead_code)]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::DocumentedDifference => "documented difference",
            Self::EquivalentEvidence => "equivalent evidence",
            Self::IntentionalNonSupport => "intentional non-support",
            Self::NativePort => "native port",
            Self::ReviewedIrrelevance => "reviewed irrelevance",
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

impl CompatibilityImpact {
    #[allow(dead_code)]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Api => "API",
            Self::Behavioral => "behavioral",
            Self::None => "none",
            Self::Tooling => "tooling",
            Self::VisualOnly => "visual only",
        }
    }
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

impl EvidenceKind {
    #[allow(dead_code)]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::CompatibilityLedger => "compatibility ledger",
            Self::NativeScenario => "native scenario",
            Self::NativeTest => "native test",
            Self::RegressionFixture => "regression fixture",
            Self::Review => "review",
        }
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    applicability: Option<Applicability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disposition: Option<TerminalDisposition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compatibility_impact: Option<CompatibilityImpact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    evidence: Option<Vec<EvidenceMapping>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    review: Option<ReviewRecord>,
}

impl CorpusItem {
    fn from_raw(raw: RawCorpusItem) -> Result<Self, CorpusError> {
        let id = CorpusItemId::parse(raw.id, raw.kind)?;
        let source = SourceIdentity::from_raw(raw.source, raw.kind, &id)?;
        let (applicability, disposition, compatibility_impact, evidence, review) = match (
            raw.applicability,
            raw.disposition,
            raw.compatibility_impact,
            raw.evidence,
            raw.review,
        ) {
            (None, None, None, None, None) => (None, None, None, None, None),
            (
                Some(applicability),
                Some(disposition),
                Some(compatibility_impact),
                Some(raw_evidence),
                Some(raw_review),
            ) => {
                let review = ReviewRecord::from_raw(raw_review, &id)?;
                let evidence = checked_evidence(raw_evidence, &id)?;
                validate_terminal_outcome(
                    applicability,
                    disposition,
                    compatibility_impact,
                    &evidence,
                    &id,
                )?;
                (
                    Some(applicability),
                    Some(disposition),
                    Some(compatibility_impact),
                    Some(evidence),
                    Some(review),
                )
            }
            _ => {
                return Err(CorpusError::for_item(CorpusErrorKind::TerminalOutcome, &id));
            }
        };
        Ok(Self {
            id,
            kind: raw.kind,
            source,
            applicability,
            disposition,
            compatibility_impact,
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

#[allow(dead_code)]
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

    pub(crate) fn oracle_revision(&self) -> &str {
        &self.oracle_revision
    }

    pub(crate) fn items(&self) -> &[CorpusItem] {
        &self.items
    }
}

#[allow(dead_code)]
impl CorpusItem {
    pub(crate) fn id(&self) -> &str {
        &self.id.0
    }

    pub(crate) const fn kind(&self) -> CorpusKind {
        self.kind
    }

    pub(crate) fn source_path(&self) -> &str {
        &self.source.path
    }

    pub(crate) fn source_symbol(&self) -> &str {
        &self.source.symbol
    }

    pub(crate) const fn applicability(&self) -> Option<Applicability> {
        self.applicability
    }

    pub(crate) const fn disposition(&self) -> Option<TerminalDisposition> {
        self.disposition
    }

    pub(crate) const fn compatibility_impact(&self) -> Option<CompatibilityImpact> {
        self.compatibility_impact
    }

    pub(crate) fn evidence(&self) -> Option<&[EvidenceMapping]> {
        self.evidence.as_deref()
    }

    pub(crate) fn review(&self) -> Option<&ReviewRecord> {
        self.review.as_ref()
    }
}

#[allow(dead_code)]
impl EvidenceMapping {
    pub(crate) const fn kind(&self) -> EvidenceKind {
        self.kind
    }

    pub(crate) fn reference(&self) -> &str {
        &self.reference
    }
}

#[allow(dead_code)]
impl ReviewRecord {
    pub(crate) fn reviewer(&self) -> &str {
        &self.reviewer
    }

    pub(crate) fn reviewed_on(&self) -> &str {
        &self.reviewed_on
    }

    pub(crate) fn rationale(&self) -> &str {
        &self.rationale
    }
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
    #[serde(default)]
    applicability: Option<Applicability>,
    #[serde(default)]
    disposition: Option<TerminalDisposition>,
    #[serde(default)]
    compatibility_impact: Option<CompatibilityImpact>,
    #[serde(default)]
    evidence: Option<Vec<RawEvidenceMapping>>,
    #[serde(default)]
    review: Option<RawReviewRecord>,
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
