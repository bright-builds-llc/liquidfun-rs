#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use super::*;

pub(super) fn exact_commit_tokens(reference: &str) -> Vec<&str> {
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

pub(super) fn validate_tolerance_references(
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

pub(super) fn validate_machine_authorities(repository_root: &Path) -> Result<(), InventoryError> {
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

pub(super) fn read_toml(
    repository_root: &Path,
    relative: &str,
) -> Result<toml::Value, InventoryError> {
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

pub(super) fn read_json_value(
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
