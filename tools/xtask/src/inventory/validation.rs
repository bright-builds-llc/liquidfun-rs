use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

mod phase10;
mod phase11;
mod phase9;

use super::{
    ApplicabilityStatus, CompatibilityEntry, CompatibilityKind, CompatibilityLedger,
    DiscoveryLedger, EVIDENCE_DIMENSIONS, EvidenceRecord, EvidenceStatus, InventoryError,
    ReleaseOutcome, ReleaseReadiness, corpus::CorpusManifest, discovery as scanner,
    require_schema_and_revision, validate_relative_path,
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

pub(super) fn release_readiness(
    ledger: &CompatibilityLedger,
    corpus: &CorpusManifest,
    repository_root: &Path,
) -> Result<ReleaseReadiness, InventoryError> {
    validate_machine_authorities(repository_root)?;
    validate_release_join(ledger)?;
    if let Some(item) = corpus
        .items()
        .iter()
        .find(|item| item.disposition().is_none())
    {
        return Err(InventoryError::new(
            "corpus-terminal-outcome",
            format!("corpus item `{}` has no terminal outcome", item.id()),
        ));
    }

    let terminal_corpus_paths = corpus
        .items()
        .iter()
        .filter(|item| item.disposition().is_some())
        .fold(BTreeMap::<_, BTreeSet<_>>::new(), |mut paths, item| {
            paths
                .entry(item.source_path())
                .or_default()
                .insert(item.id());
            paths
        });
    let mut d1_rows = 0;
    let mut corpus_terminal_rows = 0;
    let mut reviewed_difference_rows = 0;
    let mut intentional_unsupported_rows = 0;

    for (entry, disposition) in ledger.entries.iter().zip(&ledger.release_dispositions) {
        validate_release_disposition(entry, disposition, &terminal_corpus_paths, repository_root)?;
        match disposition.outcome {
            ReleaseOutcome::D1Canonical => d1_rows += 1,
            ReleaseOutcome::CorpusTerminal => corpus_terminal_rows += 1,
            ReleaseOutcome::ReviewedDifference => reviewed_difference_rows += 1,
            ReleaseOutcome::IntentionalUnsupported => intentional_unsupported_rows += 1,
        }
    }

    Ok(ReleaseReadiness {
        d1_rows,
        d2_rows: ledger
            .entries
            .iter()
            .filter(|entry| entry.evidence.platform_validated.status == EvidenceStatus::Evidenced)
            .count(),
        corpus_terminal_rows,
        reviewed_difference_rows,
        intentional_unsupported_rows,
        corpus_items: corpus.items().len(),
    })
}

fn validate_release_join(ledger: &CompatibilityLedger) -> Result<(), InventoryError> {
    let ledger_ids = ledger
        .entries
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut release_ids = BTreeSet::new();
    let mut maybe_previous_id: Option<&str> = None;
    for disposition in &ledger.release_dispositions {
        if !release_ids.insert(disposition.id.as_str())
            || maybe_previous_id.is_some_and(|previous| previous >= disposition.id.as_str())
        {
            return Err(InventoryError::new(
                "release-join",
                format!(
                    "release disposition identity `{}` is duplicated or out of order",
                    disposition.id
                ),
            ));
        }
        maybe_previous_id = Some(&disposition.id);
    }
    if ledger_ids != release_ids {
        let missing = ledger_ids
            .difference(&release_ids)
            .next()
            .copied()
            .unwrap_or("<none>");
        let unexplained = release_ids
            .difference(&ledger_ids)
            .next()
            .copied()
            .unwrap_or("<none>");
        return Err(InventoryError::new(
            "release-join",
            format!(
                "release identities must equal compatibility identities; missing `{missing}`, unexplained `{unexplained}`"
            ),
        ));
    }
    Ok(())
}

fn validate_release_disposition(
    entry: &CompatibilityEntry,
    disposition: &super::ReleaseDisposition,
    terminal_corpus_paths: &BTreeMap<&str, BTreeSet<&str>>,
    repository_root: &Path,
) -> Result<(), InventoryError> {
    if disposition.rationale.len() < 24
        || disposition.rationale.trim() != disposition.rationale
        || disposition.references.is_empty()
        || disposition
            .references
            .iter()
            .any(|reference| reference.trim().is_empty())
    {
        return Err(InventoryError::new(
            "release-rationale",
            format!(
                "release disposition `{}` requires a substantive rationale and references",
                entry.id
            ),
        ));
    }

    reject_non_parity_authorities(entry)?;
    reject_mixed_commits(entry)?;
    validate_tolerance_references(entry, repository_root)?;

    let evidenced = |record: &EvidenceRecord| record.status == EvidenceStatus::Evidenced;
    let valid_outcome = match disposition.outcome {
        ReleaseOutcome::D1Canonical => {
            evidenced(&entry.evidence.implemented)
                && evidenced(&entry.evidence.unit_tested)
                && evidenced(&entry.evidence.differentially_validated)
        }
        ReleaseOutcome::CorpusTerminal => terminal_corpus_paths
            .get(entry.upstream_path.as_str())
            .is_some_and(|corpus_ids| {
                corpus_ids.iter().any(|corpus_id| {
                    disposition.references.iter().any(|reference| {
                        reference == &format!("reference/upstream-corpus.json#id={corpus_id}")
                    })
                })
            }),
        ReleaseOutcome::ReviewedDifference => disposition.references.iter().all(|reference| {
            !reference.starts_with("target/")
                && !reference.starts_with("reference/performance/")
                && !reference.starts_with("reference/coverage/")
                && reference != "COMPATIBILITY.md"
        }),
        ReleaseOutcome::IntentionalUnsupported => {
            entry.applicability.status == ApplicabilityStatus::ReviewedExclusion
                && evidenced(&entry.evidence.intentionally_unsupported)
        }
    };
    if !valid_outcome {
        return Err(InventoryError::new(
            "release-outcome",
            format!(
                "entry `{}` cannot claim release outcome `{}` from its authorities",
                entry.id,
                disposition.outcome.as_str()
            ),
        ));
    }
    Ok(())
}

fn reject_non_parity_authorities(entry: &CompatibilityEntry) -> Result<(), InventoryError> {
    for reference in &entry.evidence.differentially_validated.references {
        if [
            "reference/coverage/",
            "reference/performance/",
            "reference/platform/",
        ]
        .iter()
        .any(|prefix| reference.starts_with(prefix))
        {
            return Err(InventoryError::new(
                "release-authority",
                format!(
                    "entry `{}` promotes a coverage, performance, or D2 platform authority into D1 parity",
                    entry.id
                ),
            ));
        }
    }
    Ok(())
}

fn reject_mixed_commits(entry: &CompatibilityEntry) -> Result<(), InventoryError> {
    let mut commits = BTreeSet::new();
    for record in [
        &entry.evidence.implemented,
        &entry.evidence.unit_tested,
        &entry.evidence.differentially_validated,
    ] {
        for reference in &record.references {
            commits.extend(exact_commit_tokens(reference));
        }
    }
    if commits.len() > 1 {
        return Err(InventoryError::new(
            "release-commit",
            format!(
                "entry `{}` mixes parity evidence from multiple exact commits",
                entry.id
            ),
        ));
    }
    Ok(())
}

fn exact_commit_tokens(reference: &str) -> Vec<&str> {
    reference
        .split(|character: char| !character.is_ascii_hexdigit())
        .filter(|token| {
            token.len() == 40
                && token
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        })
        .collect()
}

fn validate_tolerance_references(
    entry: &CompatibilityEntry,
    repository_root: &Path,
) -> Result<(), InventoryError> {
    for reference in &entry.evidence.differentially_validated.references {
        if !reference.starts_with("protocol/tolerances/")
            || Path::new(reference)
                .extension()
                .is_none_or(|extension| extension != "toml")
        {
            continue;
        }
        let path = repository_root.join(reference);
        let contents = std::fs::read_to_string(&path).map_err(|error| {
            InventoryError::new(
                "release-tolerance",
                format!(
                    "entry `{}` references unreadable tolerance {}: {error}",
                    entry.id,
                    path.display()
                ),
            )
        })?;
        let profile: toml::Value = toml::from_str(&contents).map_err(|error| {
            InventoryError::new(
                "release-tolerance",
                format!("invalid tolerance {}: {error}", path.display()),
            )
        })?;
        let expected_id = path.file_stem().and_then(|name| name.to_str());
        if profile.get("version").and_then(toml::Value::as_integer) != Some(1)
            || profile.get("profile_id").and_then(toml::Value::as_str) != expected_id
        {
            return Err(InventoryError::new(
                "release-tolerance",
                format!("tolerance {} has stale identity", path.display()),
            ));
        }
    }
    Ok(())
}

fn validate_machine_authorities(repository_root: &Path) -> Result<(), InventoryError> {
    let artifacts = read_toml(repository_root, "reference/artifacts/manifest.toml")?;
    let performance = read_toml(repository_root, "reference/performance/manifest.toml")?;
    let regressions = read_toml(repository_root, "reference/regressions/manifest.toml")?;
    let coverage = read_json_value(repository_root, "reference/coverage/contract.json")?;
    let platform = read_json_value(repository_root, "reference/platform/support.json")?;

    let valid = artifacts
        .get("schema_version")
        .and_then(toml::Value::as_integer)
        == Some(2)
        && performance
            .get("schema_version")
            .and_then(toml::Value::as_integer)
            == Some(1)
        && regressions
            .get("schema_version")
            .and_then(toml::Value::as_integer)
            == Some(1)
        && coverage
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            == Some(1)
        && coverage
            .get("parity_authority")
            .and_then(serde_json::Value::as_bool)
            == Some(false)
        && platform
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            == Some(1)
        && platform
            .get("evidence_tier")
            .and_then(serde_json::Value::as_str)
            == Some("d2_supported");
    if !valid {
        return Err(InventoryError::new(
            "release-authority",
            "release machine-authority schemas or evidence tiers are stale",
        ));
    }
    Ok(())
}

fn read_toml(repository_root: &Path, relative: &str) -> Result<toml::Value, InventoryError> {
    let path = repository_root.join(relative);
    let contents = std::fs::read_to_string(&path).map_err(|error| {
        InventoryError::new(
            "release-authority",
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    toml::from_str(&contents).map_err(|error| {
        InventoryError::new(
            "release-authority",
            format!("invalid {}: {error}", path.display()),
        )
    })
}

fn read_json_value(
    repository_root: &Path,
    relative: &str,
) -> Result<serde_json::Value, InventoryError> {
    let path = repository_root.join(relative);
    let contents = std::fs::read_to_string(&path).map_err(|error| {
        InventoryError::new(
            "release-authority",
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        InventoryError::new(
            "release-authority",
            format!("invalid {}: {error}", path.display()),
        )
    })
}
