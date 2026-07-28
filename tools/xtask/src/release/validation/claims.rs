use super::{
    CONDITIONAL_SECONDS, CONDITIONAL_TARGET, Deserialize, MAXIMUM_ARTIFACT_BYTES,
    MAXIMUM_MANIFEST_BYTES, PACKAGE_NAME, PACKAGE_RUST_VERSION, Path, ReleaseError, SystemTime,
    UNIX_EPOCH, ids, normalized_relative, platform_support, read_confined_regular, require_sha256,
    sha256, validate_coverage_contract_bytes, validate_regression_manifest_bytes,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PackageClaims {
    package_name: String,
    pub(super) package_sha256: String,
    archive_path: String,
    archive_sha256: String,
    rust_version: String,
    scalar_mode: String,
    package_drift: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PackageJoinClaims {
    pub(super) package_sha256: String,
    pub(super) package_drift: bool,
    pub(super) rust_version: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PlatformClaims {
    pub(super) package_sha256: String,
    package_drift: bool,
    evidence_tier: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ConditionalDisposition {
    Supported,
    Unsupported,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ConditionalPlatformClaims {
    pub(super) package_sha256: String,
    package_drift: bool,
    disposition: ConditionalDisposition,
    recorded_at_unix: Option<u64>,
    expires_at_unix: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DifferentialClaims {
    pub(super) parity_tier: String,
    pub(super) coverage_authority: bool,
    pub(super) performance_authority: bool,
    pub(super) gap_count: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RustSafetyClaims {
    pub(super) unsafe_waivers: u64,
    pub(super) advisory_waivers: u64,
    pub(super) unsafe_code: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FindingsClaims {
    pub(super) findings: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FuzzClaims {
    pub(super) findings: u64,
    pub(super) target_count: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RegressionClaims {
    manifest_sha256: String,
    missing_results: u64,
    unreviewed_results: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CoverageClaims {
    contract_sha256: String,
    parity_authority: bool,
    missing_subsystems: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PerformanceClaims {
    policy_sha256: String,
    timing_authority: String,
    claim_scope: String,
    claim_status: String,
    profile_authority: bool,
    reviewed_report_count: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DocsClaims {
    pub(super) docs_complete: bool,
    pub(super) rustdoc_warnings: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NoticeClaims {
    pub(super) notices_complete: bool,
    pub(super) license: String,
    pub(super) advisory_waivers: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CorpusClosureClaims {
    authority_sha256: String,
    item_count: usize,
    unresolved_count: usize,
    nonterminal_count: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CompatibilityClosureClaims {
    authority_sha256: String,
    gap_count: usize,
    unexplained_count: usize,
    mixed_commit_count: usize,
    coverage_promoted_to_parity: bool,
    platform_promoted_to_parity: bool,
}

pub(super) fn parse_claims<T: for<'de> Deserialize<'de>>(
    value: &serde_json::Value,
) -> Result<T, ReleaseError> {
    serde_json::from_value(value.clone())
        .map_err(|error| ReleaseError::new("claims-schema", error.to_string()))
}

pub(super) fn validate_package_claims(
    repository_root: &Path,
    claims: &PackageClaims,
) -> Result<(), ReleaseError> {
    require_sha256(&claims.package_sha256, "package-drift")?;
    require_sha256(&claims.archive_sha256, "package-drift")?;
    if claims.package_name != PACKAGE_NAME
        || claims.rust_version != PACKAGE_RUST_VERSION
        || claims.scalar_mode != "strict_f32"
        || claims.package_drift
        || claims.package_sha256 != claims.archive_sha256
    {
        return Err(ReleaseError::new(
            "package-drift",
            "package archive identity differs from the reviewed consumer contract",
        ));
    }
    let archive = read_confined_regular(
        repository_root,
        normalized_relative(&claims.archive_path, "package-drift")?,
        64 * 1024 * 1024,
        "package-drift",
    )?;
    if sha256(&archive) != claims.archive_sha256 {
        return Err(ReleaseError::new(
            "package-drift",
            "package archive bytes differ from their identity",
        ));
    }
    Ok(())
}

pub(super) fn validate_durable_platform(
    repository_root: &Path,
    target: &str,
    claims: &PlatformClaims,
) -> Result<(), ReleaseError> {
    require_sha256(&claims.package_sha256, "platform")?;
    let support = platform_support(repository_root)?;
    let durable = support
        .get("durable_targets")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| ReleaseError::new("platform", "durable target policy is invalid"))?;
    if claims.package_drift
        || claims.evidence_tier != "d2_supported"
        || !durable.iter().any(|value| value.as_str() == Some(target))
    {
        return Err(ReleaseError::new(
            "platform",
            "durable platform evidence differs from the support policy",
        ));
    }
    Ok(())
}

pub(super) fn validate_conditional_platform(
    repository_root: &Path,
    target: &str,
    claims: &ConditionalPlatformClaims,
) -> Result<(), ReleaseError> {
    require_sha256(&claims.package_sha256, "conditional-platform")?;
    let support = platform_support(repository_root)?;
    let conditional = support
        .get("conditional_targets")
        .and_then(serde_json::Value::as_array)
        .and_then(|values| values.first())
        .ok_or_else(|| ReleaseError::new("conditional-platform", "conditional policy is absent"))?;
    if target != CONDITIONAL_TARGET
        || conditional
            .get("target")
            .and_then(serde_json::Value::as_str)
            != Some(target)
        || claims.package_drift
    {
        return Err(ReleaseError::new(
            "conditional-platform",
            "conditional platform identity differs",
        ));
    }
    let maybe_native = conditional.get("native_evidence");
    match (&claims.disposition, maybe_native) {
        (ConditionalDisposition::Unsupported, Some(value)) if value.is_null() => {
            if claims.recorded_at_unix.is_some() || claims.expires_at_unix.is_some() {
                return Err(ReleaseError::new(
                    "conditional-platform",
                    "unsupported disposition cannot carry native freshness",
                ));
            }
        }
        (ConditionalDisposition::Supported, Some(value)) if !value.is_null() => {
            let recorded = claims.recorded_at_unix.ok_or_else(|| {
                ReleaseError::new(
                    "conditional-platform",
                    "native evidence timestamp is missing",
                )
            })?;
            let expires = claims.expires_at_unix.ok_or_else(|| {
                ReleaseError::new("conditional-platform", "native evidence expiry is missing")
            })?;
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| ReleaseError::new("conditional-platform", error.to_string()))?
                .as_secs();
            if recorded.checked_add(CONDITIONAL_SECONDS) != Some(expires)
                || recorded > now
                || expires < now
            {
                return Err(ReleaseError::new(
                    "conditional-platform",
                    "conditional platform evidence is stale",
                ));
            }
        }
        _ => {
            return Err(ReleaseError::new(
                "conditional-platform",
                "conditional disposition disagrees with tracked native evidence",
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_safety_authorities(repository_root: &Path) -> Result<(), ReleaseError> {
    let deny = read_confined_regular(
        repository_root,
        Path::new("deny.toml"),
        MAXIMUM_MANIFEST_BYTES,
        "safety",
    )?;
    let deny: toml::Value =
        toml::from_slice(&deny).map_err(|error| ReleaseError::new("safety", error.to_string()))?;
    let ignores = deny
        .get("advisories")
        .and_then(|value| value.get("ignore"))
        .and_then(toml::Value::as_array);
    let cargo = read_confined_regular(
        repository_root,
        Path::new("Cargo.toml"),
        MAXIMUM_MANIFEST_BYTES,
        "safety",
    )?;
    let cargo = std::str::from_utf8(&cargo)
        .map_err(|error| ReleaseError::new("safety", error.to_string()))?;
    if ignores.is_none_or(|values| !values.is_empty())
        || !cargo.contains("unsafe_code = \"forbid\"")
    {
        return Err(ReleaseError::new(
            "safety",
            "tracked unsafe or advisory policy was weakened",
        ));
    }
    Ok(())
}

pub(super) fn validate_regressions(
    repository_root: &Path,
    claims: &RegressionClaims,
) -> Result<(), ReleaseError> {
    let bytes = read_confined_regular(
        repository_root,
        Path::new("reference/regressions/manifest.toml"),
        MAXIMUM_MANIFEST_BYTES,
        "regressions",
    )?;
    validate_regression_manifest_bytes(repository_root, &bytes)
        .map_err(|error| ReleaseError::new("regressions", error.to_string()))?;
    if claims.manifest_sha256 != sha256(&bytes)
        || claims.missing_results != 0
        || claims.unreviewed_results != 0
    {
        return Err(ReleaseError::new(
            "regressions",
            "regression evidence is incomplete, stale, or unreviewed",
        ));
    }
    Ok(())
}

pub(super) fn validate_coverage(
    repository_root: &Path,
    claims: &CoverageClaims,
) -> Result<(), ReleaseError> {
    let bytes = read_confined_regular(
        repository_root,
        Path::new("reference/coverage/contract.json"),
        MAXIMUM_MANIFEST_BYTES,
        "coverage",
    )?;
    validate_coverage_contract_bytes(&bytes)
        .map_err(|error| ReleaseError::new("coverage", error.to_string()))?;
    if claims.contract_sha256 != sha256(&bytes)
        || claims.parity_authority
        || claims.missing_subsystems != 0
    {
        return Err(ReleaseError::new(
            "coverage",
            "coverage evidence is incomplete or promoted into parity",
        ));
    }
    Ok(())
}

pub(super) fn validate_performance(
    repository_root: &Path,
    claims: &PerformanceClaims,
) -> Result<(), ReleaseError> {
    let bytes = read_confined_regular(
        repository_root,
        Path::new("reference/performance/manifest.toml"),
        MAXIMUM_MANIFEST_BYTES,
        "performance",
    )?;
    let manifest: toml::Value = toml::from_slice(&bytes)
        .map_err(|error| ReleaseError::new("performance", error.to_string()))?;
    let policy_sha256 = manifest.get("policy_sha256").and_then(toml::Value::as_str);
    let reviewed_count = manifest
        .get("reviewed_reports")
        .and_then(toml::Value::as_array)
        .map(Vec::len);
    if policy_sha256 != Some(claims.policy_sha256.as_str())
        || reviewed_count != Some(claims.reviewed_report_count)
        || claims.timing_authority != "unprofiled_wall_clock"
        || claims.claim_scope != "workload_only"
        || claims.claim_status != "no_generalized_performance_claim"
        || claims.profile_authority
    {
        return Err(ReleaseError::new(
            "performance",
            "performance evidence overclaims or differs from the reviewed policy",
        ));
    }
    Ok(())
}

pub(super) fn validate_corpus_closure(
    repository_root: &Path,
    claims: &CorpusClosureClaims,
) -> Result<(), ReleaseError> {
    let bytes = read_confined_regular(
        repository_root,
        Path::new("reference/upstream-corpus.json"),
        MAXIMUM_ARTIFACT_BYTES,
        "corpus-closure",
    )?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| ReleaseError::new("corpus-closure", error.to_string()))?;
    let items = value
        .get("items")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| ReleaseError::new("corpus-closure", "corpus items are absent"))?;
    let unresolved = items
        .iter()
        .filter(|item| item.get("review").is_none() || item.get("evidence").is_none())
        .count();
    let nonterminal = items
        .iter()
        .filter(|item| {
            item.get("disposition")
                .is_none_or(serde_json::Value::is_null)
        })
        .count();
    if claims.authority_sha256 != sha256(&bytes)
        || claims.item_count != items.len()
        || claims.unresolved_count != 0
        || claims.nonterminal_count != 0
        || unresolved != 0
        || nonterminal != 0
    {
        return Err(ReleaseError::new(
            "corpus-closure",
            "semantic corpus contains unresolved or nonterminal items",
        ));
    }
    Ok(())
}

pub(super) fn validate_compatibility_closure(
    repository_root: &Path,
    claims: &CompatibilityClosureClaims,
) -> Result<(), ReleaseError> {
    let bytes = read_confined_regular(
        repository_root,
        Path::new("reference/compatibility.json"),
        MAXIMUM_ARTIFACT_BYTES,
        "compatibility-closure",
    )?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| ReleaseError::new("compatibility-closure", error.to_string()))?;
    let entries = value
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| ReleaseError::new("compatibility-closure", "entries are absent"))?;
    let dispositions = value
        .get("release_dispositions")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| ReleaseError::new("compatibility-closure", "dispositions are absent"))?;
    let entry_ids = ids(entries)?;
    let disposition_ids = ids(dispositions)?;
    if claims.authority_sha256 != sha256(&bytes)
        || entry_ids != disposition_ids
        || claims.gap_count != 0
        || claims.unexplained_count != 0
        || claims.mixed_commit_count != 0
        || claims.coverage_promoted_to_parity
        || claims.platform_promoted_to_parity
    {
        return Err(ReleaseError::new(
            "compatibility-closure",
            "compatibility release closure contains gaps or invalid authority promotion",
        ));
    }
    Ok(())
}
