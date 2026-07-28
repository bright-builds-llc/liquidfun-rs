#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use super::*;

pub(super) fn checked_evidence(
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

pub(super) fn validate_terminal_outcome(
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

pub(super) fn validate_revision(actual: &str, expected: &str) -> Result<(), CorpusError> {
    let valid = actual.len() == 40
        && actual
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
    if !valid || actual != expected {
        return Err(CorpusError::new(CorpusErrorKind::Revision));
    }
    Ok(())
}

pub(super) fn validate_relative_path(value: &str) -> Result<(), ()> {
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

pub(super) fn valid_evidence_reference(value: &str) -> bool {
    if value.starts_with("https://") {
        return !value.contains(char::is_whitespace);
    }
    let (path, _) = value.split_once('#').unwrap_or((value, ""));
    validate_relative_path(path).is_ok()
}

pub(super) fn valid_review_date(value: &str) -> bool {
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

pub(super) fn validate_rationale(value: &str, item_id: &CorpusItemId) -> Result<(), CorpusError> {
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
