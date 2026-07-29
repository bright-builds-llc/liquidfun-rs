use super::super::DocsError;
#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use super::*;
use std::{collections::BTreeMap, fs};

#[allow(
    clippy::too_many_lines,
    reason = "the cohesive read-only contract validator is only two lines over the threshold"
)]
pub(in super::super) fn check_document_contracts(
    repository_root: &std::path::Path,
) -> Result<(), DocsError> {
    check_required_markers(repository_root, DOCUMENT_CONTRACTS, "phase4-contract")?;
    check_required_markers(
        repository_root,
        PHASE5_DOCUMENT_CONTRACTS,
        "phase5-contract",
    )?;
    check_required_markers(
        repository_root,
        PHASE6_DOCUMENT_CONTRACTS,
        "phase6-contract",
    )?;
    check_required_markers(
        repository_root,
        PHASE7_DOCUMENT_CONTRACTS,
        "phase7-contract",
    )?;
    check_required_markers(
        repository_root,
        PHASE8_DOCUMENT_CONTRACTS,
        "phase8-contract",
    )?;
    check_required_markers(
        repository_root,
        PHASE12_PUBLIC_DOCUMENT_CONTRACTS,
        "phase12-public-contract",
    )?;
    check_phase8_platform_evidence(repository_root)?;

    for relative_path in [
        "ARCHITECTURE.md",
        "TESTING.md",
        "COMPATIBILITY.md",
        "README.md",
    ] {
        let path = repository_root.join(relative_path);
        let contents = fs::read_to_string(&path).map_err(|error| {
            DocsError::new(
                "filesystem",
                format!("failed to read {}: {error}", path.display()),
            )
        })?;
        if contents.contains("/Users/") || contents.contains("C:\\Users\\") {
            return Err(DocsError::new(
                "local-path",
                format!("{relative_path} contains an absolute user path"),
            ));
        }
        let lowercase = contents.to_ascii_lowercase();
        if let Some(claim) = FORBIDDEN_PHASE5_CLAIMS
            .iter()
            .find(|claim| lowercase.contains(**claim))
        {
            return Err(DocsError::new(
                "phase5-overclaim",
                format!("{relative_path} contains forbidden claim `{claim}`"),
            ));
        }
        if let Some(claim) = FORBIDDEN_PHASE6_CLAIMS
            .iter()
            .find(|claim| lowercase.contains(**claim))
        {
            return Err(DocsError::new(
                "phase6-overclaim",
                format!("{relative_path} contains forbidden claim `{claim}`"),
            ));
        }
        if let Some(claim) = FORBIDDEN_PHASE7_CLAIMS
            .iter()
            .find(|claim| lowercase.contains(**claim))
        {
            return Err(DocsError::new(
                "phase7-overclaim",
                format!("{relative_path} contains forbidden claim `{claim}`"),
            ));
        }
        if let Some(claim) = FORBIDDEN_PHASE8_CLAIMS
            .iter()
            .find(|claim| lowercase.contains(**claim))
        {
            return Err(DocsError::new(
                "phase8-overclaim",
                format!("{relative_path} contains forbidden claim `{claim}`"),
            ));
        }
    }

    let readme_path = repository_root.join("README.md");
    let readme = fs::read_to_string(&readme_path).map_err(|error| {
        DocsError::new(
            "filesystem",
            format!("failed to read {}: {error}", readme_path.display()),
        )
    })?;
    let lowercase = readme.to_ascii_lowercase();
    if let Some(claim) = FORBIDDEN_CURRENT_CLAIMS
        .iter()
        .find(|claim| lowercase.contains(**claim))
    {
        return Err(DocsError::new(
            "current-overclaim",
            format!("README.md contains stale or unsupported claim `{claim}`"),
        ));
    }
    Ok(())
}

fn check_phase8_platform_evidence(repository_root: &std::path::Path) -> Result<(), DocsError> {
    let path = repository_root.join("reference/compatibility.json");
    let contents = fs::read_to_string(&path).map_err(|error| {
        DocsError::new(
            "filesystem",
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    let ledger: CompatibilityEvidenceLedger = serde_json::from_str(&contents).map_err(|error| {
        DocsError::new(
            "phase8-evidence",
            format!("failed to parse {}: {error}", path.display()),
        )
    })?;
    let expected_references = [
        PHASE8_RUN_URL,
        PHASE8_CANONICAL_IDENTITY,
        PHASE8_SANITIZER_IDENTITY,
        "TESTING.md#phase-8-canonical-rigid-world-sign-off",
    ];
    let platform_entries = ledger
        .entries
        .iter()
        .filter(|entry| {
            entry.evidence.platform_validated.status == "evidenced"
                && entry.evidence.platform_validated.references == expected_references
        })
        .collect::<Vec<_>>();
    if platform_entries.len() != PHASE8_PLATFORM_VALIDATED_ROWS {
        return Err(DocsError::new(
            "phase8-evidence",
            format!(
                "expected {PHASE8_PLATFORM_VALIDATED_ROWS} platform-validated rows bound to Phase 8 evidence, actual {}",
                platform_entries.len()
            ),
        ));
    }
    Ok(())
}

fn check_required_markers<const N: usize>(
    repository_root: &std::path::Path,
    contracts: [(&str, &[&str]); N],
    category: &'static str,
) -> Result<(), DocsError> {
    for (relative_path, required_markers) in contracts {
        let path = repository_root.join(relative_path);
        let contents = fs::read_to_string(&path).map_err(|error| {
            DocsError::new(
                "filesystem",
                format!("failed to read {}: {error}", path.display()),
            )
        })?;
        for marker in required_markers {
            if !contents.contains(marker) {
                return Err(DocsError::new(
                    category,
                    format!("{relative_path} must contain `{marker}`"),
                ));
            }
        }
    }
    Ok(())
}

pub(in super::super) fn check_testing_contract(contents: &str) -> Result<(), DocsError> {
    let table = parse_contract_table(contents)?;
    let mut rows = BTreeMap::new();
    for cells in table {
        validate_cells(&cells)?;
        let layer = cells[0].clone();
        if rows.insert(layer.clone(), cells).is_some() {
            return Err(DocsError::new(
                "layer",
                format!("duplicate testing layer `{layer}`"),
            ));
        }
    }

    if rows.len() != LAYER_RULES.len() {
        return Err(DocsError::new(
            "layer",
            format!(
                "testing contract must contain exactly {} layers, actual {}",
                LAYER_RULES.len(),
                rows.len()
            ),
        ));
    }

    for rule in LAYER_RULES {
        let Some(cells) = rows.get(rule.name) else {
            return Err(DocsError::new(
                "layer",
                format!("missing testing layer `{}`", rule.name),
            ));
        };
        validate_layer(rule, cells)?;
    }
    Ok(())
}

fn parse_contract_table(contents: &str) -> Result<Vec<Vec<String>>, DocsError> {
    let mut lines = contents.lines();
    let Some(_) = lines.find(|line| line.trim() == TABLE_HEADING) else {
        return Err(DocsError::new(
            "schema",
            format!("missing `{TABLE_HEADING}` heading"),
        ));
    };
    let Some(header) = lines.find(|line| line.trim_start().starts_with('|')) else {
        return Err(DocsError::new("schema", "missing testing table header"));
    };
    let header_cells = parse_row(header)?;
    if header_cells != COLUMNS {
        return Err(DocsError::new(
            "schema",
            format!(
                "testing table columns must be exactly: {}",
                COLUMNS.join(", ")
            ),
        ));
    }
    let Some(separator) = lines.next() else {
        return Err(DocsError::new("schema", "missing testing table separator"));
    };
    let separator_cells = parse_row(separator)?;
    if separator_cells.len() != COLUMNS.len()
        || separator_cells
            .iter()
            .any(|cell| cell.len() < 3 || !cell.chars().all(|character| character == '-'))
    {
        return Err(DocsError::new(
            "schema",
            "testing table separator must contain one dash group per column",
        ));
    }

    let mut rows = Vec::new();
    for line in lines {
        if !line.trim_start().starts_with('|') {
            break;
        }
        rows.push(parse_row(line)?);
    }
    if rows.is_empty() {
        return Err(DocsError::new("schema", "testing table has no rows"));
    }
    Ok(rows)
}

fn parse_row(line: &str) -> Result<Vec<String>, DocsError> {
    let trimmed = line.trim();
    let Some(contents) = trimmed
        .strip_prefix('|')
        .and_then(|contents| contents.strip_suffix('|'))
    else {
        return Err(DocsError::new(
            "schema",
            format!("invalid Markdown table row `{trimmed}`"),
        ));
    };
    let cells = contents
        .split('|')
        .map(|cell| cell.trim().to_owned())
        .collect::<Vec<_>>();
    if cells.len() != COLUMNS.len() {
        return Err(DocsError::new(
            "schema",
            format!(
                "testing row must contain {} cells, actual {}",
                COLUMNS.len(),
                cells.len()
            ),
        ));
    }
    Ok(cells)
}

fn validate_cells(cells: &[String]) -> Result<(), DocsError> {
    for (index, cell) in cells.iter().enumerate() {
        if cell.trim().is_empty() {
            return Err(DocsError::new(
                "schema",
                format!("testing row column `{}` cannot be blank", COLUMNS[index]),
            ));
        }
        let uppercase = cell.to_ascii_uppercase();
        if let Some(placeholder) = PLACEHOLDERS
            .iter()
            .find(|placeholder| uppercase.contains(*placeholder))
        {
            return Err(DocsError::new(
                "placeholder",
                format!(
                    "testing row column `{}` contains forbidden placeholder `{placeholder}`",
                    COLUMNS[index]
                ),
            ));
        }
    }
    Ok(())
}

fn validate_layer(rule: LayerRule, cells: &[String]) -> Result<(), DocsError> {
    require_allowed(&cells[1], &ALLOWED_STATUSES, "status", rule.name)?;
    let placements = cells[7].split(',').map(str::trim).collect::<Vec<_>>();
    for placement in placements {
        require_allowed(placement, &ALLOWED_PLACEMENTS, "placement", rule.name)?;
    }

    for (column, value, marker) in [
        ("Command", cells[3].as_str(), rule.command),
        ("Prerequisites", cells[4].as_str(), rule.prerequisite),
        (
            "Reports and failure artifacts",
            cells[5].as_str(),
            rule.artifact,
        ),
        ("Retry policy", cells[6].as_str(), rule.retry),
        ("Semantic interpretation", cells[8].as_str(), rule.semantics),
    ] {
        if !value
            .to_ascii_lowercase()
            .contains(&marker.to_ascii_lowercase())
        {
            return Err(DocsError::new(
                "content",
                format!(
                    "testing layer `{}` column `{column}` must contain `{marker}`",
                    rule.name
                ),
            ));
        }
    }
    Ok(())
}

fn require_allowed(
    value: &str,
    allowed: &[&str],
    field: &'static str,
    layer: &str,
) -> Result<(), DocsError> {
    if allowed.contains(&value) {
        return Ok(());
    }
    Err(DocsError::new(
        field,
        format!(
            "testing layer `{layer}` has invalid {field} `{value}`; allowed values: {}",
            allowed.join(", ")
        ),
    ))
}
