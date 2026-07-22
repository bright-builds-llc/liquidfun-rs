use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

mod phase10;
mod phase11;
mod phase9;

use super::{
    ApplicabilityStatus, CompatibilityEntry, CompatibilityKind, CompatibilityLedger,
    DiscoveryLedger, EVIDENCE_DIMENSIONS, EvidenceRecord, EvidenceStatus, InventoryError,
    discovery as scanner, require_schema_and_revision, validate_relative_path,
};

pub(super) fn compatibility(
    ledger: &CompatibilityLedger,
    oracle_revision: &str,
    repository_root: &Path,
) -> Result<(), InventoryError> {
    require_schema_and_revision(
        ledger.schema_version,
        &ledger.oracle_revision,
        oracle_revision,
    )?;
    if ledger.sort_contract != "entries are ordered lexicographically by id" {
        return Err(InventoryError::new(
            "schema",
            "unexpected compatibility sort contract",
        ));
    }
    if ledger.evidence_dimensions != EVIDENCE_DIMENSIONS {
        return Err(InventoryError::new(
            "schema",
            "compatibility evidence_dimensions must list all eight dimensions in canonical order",
        ));
    }
    let mut ids = BTreeSet::new();
    let mut mappings = BTreeMap::new();
    let mut maybe_previous_id: Option<&str> = None;
    for entry in &ledger.entries {
        if entry.id.is_empty()
            || !entry.id.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b".-".contains(&byte)
            })
        {
            return Err(InventoryError::new(
                "schema",
                format!("invalid stable id `{}`", entry.id),
            ));
        }
        if !ids.insert(entry.id.as_str()) {
            return Err(InventoryError::new(
                "duplicate-id",
                format!("duplicate compatibility id `{}`", entry.id),
            ));
        }
        if maybe_previous_id.is_some_and(|previous| previous >= entry.id.as_str()) {
            return Err(InventoryError::new(
                "ordering",
                format!("compatibility id `{}` is out of order", entry.id),
            ));
        }
        maybe_previous_id = Some(&entry.id);
        compatibility_entry(entry)?;
        if entry.kind != CompatibilityKind::Subsystem {
            let mapping = (
                entry.kind,
                entry.upstream_path.as_str(),
                entry.upstream_symbol.as_deref(),
            );
            if let Some(previous_id) = mappings.insert(mapping, entry.id.as_str()) {
                return Err(InventoryError::new(
                    "duplicate-mapping",
                    format!(
                        "compatibility entries `{previous_id}` and `{}` both map {} `{}` symbol `{}`",
                        entry.id,
                        entry.kind.as_str(),
                        entry.upstream_path,
                        entry.upstream_symbol.as_deref().unwrap_or("<none>")
                    ),
                ));
            }
        }
    }
    phase9::promotion(ledger)?;
    phase10::promotion(ledger)?;
    phase11::promotion(ledger, repository_root)
}

fn compatibility_entry(entry: &CompatibilityEntry) -> Result<(), InventoryError> {
    validate_relative_path(&entry.upstream_path, "upstream_path")?;
    if entry
        .upstream_symbol
        .as_ref()
        .is_some_and(|symbol| symbol.trim().is_empty())
    {
        return Err(InventoryError::new(
            "schema",
            "upstream_symbol cannot be empty",
        ));
    }
    for (field, value) in [
        (
            "applicability rationale",
            entry.applicability.rationale.as_str(),
        ),
        ("rust_target", entry.rust_target.as_str()),
        ("provenance_ref", entry.provenance_ref.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(InventoryError::new(
                "schema",
                format!("{field} cannot be empty"),
            ));
        }
    }
    if entry.notice_refs.is_empty()
        || entry
            .notice_refs
            .iter()
            .any(|value| value.trim().is_empty())
    {
        return Err(InventoryError::new(
            "schema",
            "notice_refs must contain nonempty references",
        ));
    }
    for (dimension, record) in entry.evidence.records() {
        let references_are_valid = match record.status {
            EvidenceStatus::Evidenced => !record.references.is_empty(),
            EvidenceStatus::NotEvidenced => record.references.is_empty(),
        } && record
            .references
            .iter()
            .all(|reference| !reference.trim().is_empty());
        if !references_are_valid {
            return Err(InventoryError::new(
                "evidence",
                format!("entry `{}` has invalid `{dimension}` references", entry.id),
            ));
        }
    }
    evidence_dependencies(entry)
}

fn evidence_dependencies(entry: &CompatibilityEntry) -> Result<(), InventoryError> {
    let evidenced = |record: &EvidenceRecord| record.status == EvidenceStatus::Evidenced;
    if evidenced(&entry.evidence.unit_tested) && !evidenced(&entry.evidence.implemented) {
        return Err(InventoryError::new(
            "evidence",
            format!(
                "entry `{}` is unit tested without implementation evidence",
                entry.id
            ),
        ));
    }
    if evidenced(&entry.evidence.differentially_validated)
        && (!evidenced(&entry.evidence.implemented) || !evidenced(&entry.evidence.unit_tested))
    {
        return Err(InventoryError::new(
            "evidence",
            format!(
                "entry `{}` has differential evidence without implementation and unit-test evidence",
                entry.id
            ),
        ));
    }
    if evidenced(&entry.evidence.platform_validated) && !evidenced(&entry.evidence.implemented) {
        return Err(InventoryError::new(
            "evidence",
            format!(
                "entry `{}` has platform evidence without implementation evidence",
                entry.id
            ),
        ));
    }
    let unsupported = evidenced(&entry.evidence.intentionally_unsupported);
    if unsupported != (entry.applicability.status == ApplicabilityStatus::ReviewedExclusion) {
        return Err(InventoryError::new(
            "applicability",
            format!(
                "entry `{}` must pair reviewed exclusion with intentionally unsupported evidence",
                entry.id
            ),
        ));
    }
    if unsupported && evidenced(&entry.evidence.implemented) {
        return Err(InventoryError::new(
            "evidence",
            format!(
                "entry `{}` cannot be implemented and intentionally unsupported",
                entry.id
            ),
        ));
    }
    Ok(())
}

pub(super) fn discovery(
    ledger: &DiscoveryLedger,
    oracle_revision: &str,
) -> Result<(), InventoryError> {
    require_schema_and_revision(
        ledger.schema_version,
        &ledger.oracle_revision,
        oracle_revision,
    )?;
    if ledger.sort_contract != scanner::SORT_CONTRACT {
        return Err(InventoryError::new(
            "schema",
            "unexpected discovery sort contract",
        ));
    }
    for scope in &ledger.scopes {
        validate_relative_path(&scope.root, "discovery scope root")?;
        if scope.matcher.trim().is_empty() {
            return Err(InventoryError::new(
                "schema",
                "discovery matcher cannot be empty",
            ));
        }
    }
    let mut entries = BTreeSet::new();
    for entry in &ledger.entries {
        validate_relative_path(&entry.upstream_path, "discovery upstream_path")?;
        if entry
            .upstream_symbol
            .as_ref()
            .is_some_and(|symbol| symbol.trim().is_empty())
        {
            return Err(InventoryError::new(
                "schema",
                "discovery symbol cannot be empty",
            ));
        }
        if !entries.insert(entry) {
            return Err(InventoryError::new(
                "duplicate-discovery",
                format!("duplicate discovery entry `{}`", entry.upstream_path),
            ));
        }
    }
    if ledger
        .entries
        .windows(2)
        .any(|pair| scanner::compare_entries(&pair[0], &pair[1]).is_ge())
    {
        return Err(InventoryError::new(
            "ordering",
            "discovery entries are out of order",
        ));
    }
    Ok(())
}

pub(super) fn coverage(
    compatibility: &CompatibilityLedger,
    discovery: &DiscoveryLedger,
) -> Result<(), InventoryError> {
    let mapped: BTreeSet<_> = compatibility
        .entries
        .iter()
        .filter(|entry| entry.kind != CompatibilityKind::Subsystem)
        .map(|entry| {
            (
                entry.kind,
                entry.upstream_path.as_str(),
                entry.upstream_symbol.as_deref(),
            )
        })
        .collect();
    let discovered: BTreeSet<_> = discovery
        .entries
        .iter()
        .map(|entry| {
            (
                entry.kind.compatibility_kind(),
                entry.upstream_path.as_str(),
                entry.upstream_symbol.as_deref(),
            )
        })
        .collect();
    if let Some((kind, path, symbol)) = discovered.difference(&mapped).next() {
        return Err(InventoryError::new(
            "unmapped-discovery",
            format!(
                "{} `{path}` symbol `{}` is not mapped",
                kind.as_str(),
                symbol.unwrap_or("<none>")
            ),
        ));
    }
    if let Some((kind, path, symbol)) = mapped.difference(&discovered).next() {
        return Err(InventoryError::new(
            "stale-mapping",
            format!(
                "{} `{path}` symbol `{}` is absent from discovery",
                kind.as_str(),
                symbol.unwrap_or("<none>")
            ),
        ));
    }
    Ok(())
}
