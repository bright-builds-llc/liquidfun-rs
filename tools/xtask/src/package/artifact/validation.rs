#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use super::*;

pub(super) fn validate_identity(identity: &PackageArtifactIdentity) -> Result<(), PackageError> {
    if identity.schema_version != ARTIFACT_SCHEMA_VERSION
        || identity.package != PACKAGE_NAME
        || identity.rust_version != REQUIRED_RUST_VERSION
        || identity.features != REQUIRED_FEATURES
        || identity.normal_dependencies != REQUIRED_NORMAL_DEPENDENCIES
        || identity.license_files != ["LICENSE"]
        || identity.created_with_toolchain != CREATION_TOOLCHAIN
        || identity.scalar_mode != SCALAR_MODE
        || identity.compiler_class != COMPILER_CLASS
        || identity.tolerance_profile != TOLERANCE_PROFILE
        || identity.source_files.is_empty()
        || identity
            .source_files
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(PackageError::new(
            "artifact-identity",
            "artifact identity does not match the reviewed package contract",
        ));
    }
    validate_sha256(&identity.archive_sha256)?;
    validate_candidate_commit(&identity.candidate_commit)
}

pub(super) fn validate_candidate_matches_repository(
    repository_root: &Path,
    candidate_commit: &str,
) -> Result<(), PackageError> {
    if !repository_root.join(".git").exists() {
        return Ok(());
    }
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repository_root)
        .output()
        .map_err(|error| {
            PackageError::new(
                "artifact-identity",
                format!("failed to resolve candidate repository commit: {error}"),
            )
        })?;
    if !output.status.success() {
        return Err(PackageError::new(
            "artifact-identity",
            "failed to resolve candidate repository commit",
        ));
    }
    let repository_commit = String::from_utf8_lossy(&output.stdout);
    if repository_commit.trim() != candidate_commit {
        return Err(PackageError::new(
            "artifact-identity",
            "candidate_commit differs from the candidate repository HEAD",
        ));
    }
    Ok(())
}

pub(super) fn validate_sha256(value: &str) -> Result<(), PackageError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(PackageError::new(
            "artifact-identity",
            "archive_sha256 must be 64 lowercase hexadecimal characters",
        ));
    }
    Ok(())
}

pub(super) fn validate_candidate_commit(value: &str) -> Result<(), PackageError> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(PackageError::new(
            "artifact-identity",
            "candidate_commit must be a 40-character lowercase Git object ID",
        ));
    }
    Ok(())
}

pub(super) fn verify_hash_and_size(
    identity: &PackageArtifactIdentity,
    archive_bytes: &[u8],
) -> Result<(), PackageError> {
    let actual_bytes = u64::try_from(archive_bytes.len())
        .map_err(|_| PackageError::new("artifact-hash", "archive size is not representable"))?;
    if actual_bytes != identity.archive_bytes || sha256(archive_bytes) != identity.archive_sha256 {
        return Err(PackageError::new(
            "artifact-hash",
            "archive bytes do not match the content-addressed identity",
        ));
    }
    Ok(())
}

pub(super) fn verify_contents(
    identity: &PackageArtifactIdentity,
    contents: &ArchiveContents,
) -> Result<(), PackageError> {
    if contents.manifest.name != identity.package
        || contents.manifest.version != identity.version
        || contents.manifest.rust_version != identity.rust_version
        || contents.manifest.features != identity.features
        || contents.manifest.normal_dependencies != identity.normal_dependencies
        || contents.manifest.license != "MIT"
        || contents.source_files != identity.source_files
        || contents.license_files != identity.license_files
        || contents.notice_files != identity.notice_files
    {
        return Err(PackageError::new(
            "artifact-identity",
            "extracted package metadata or source inventory differs from its identity",
        ));
    }
    Ok(())
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn validate_platform(
    repository_root: &Path,
    toolchain: &str,
    target: &str,
) -> Result<(), PackageError> {
    let support_path = repository_root.join("reference/platform/support.json");
    let schema_path = repository_root.join("reference/platform/schema.json");
    let support_value = read_json(&support_path, "platform-policy")?;
    let schema = read_json(&schema_path, "platform-policy")?;
    validate_schema_contract(&schema)?;
    let support: PlatformSupport = serde_json::from_value(support_value).map_err(|error| {
        PackageError::new(
            "platform-policy",
            format!("invalid {}: {error}", support_path.display()),
        )
    })?;
    validate_support_contract(&support)?;
    validate_toolchain(toolchain, target)?;
    if DURABLE_TARGETS.contains(&target) {
        return Ok(());
    }
    let conditional = &support.conditional_targets[0];
    if target != conditional.target {
        return Err(PackageError::new(
            "platform-policy",
            format!("target `{target}` is outside the reviewed support policy"),
        ));
    }
    validate_native_evidence(conditional)
}

pub(super) fn validate_schema_contract(schema: &serde_json::Value) -> Result<(), PackageError> {
    let expected = [
        ("/additionalProperties", serde_json::json!(false)),
        (
            "/properties/evidence_tier/const",
            serde_json::json!("d2_supported"),
        ),
        (
            "/properties/conditional_evidence_policy/properties/max_age_days/const",
            serde_json::json!(MAX_EVIDENCE_AGE_DAYS),
        ),
        (
            "/properties/conditional_evidence_policy/properties/missing_or_expired_outcome/const",
            serde_json::json!("unsupported"),
        ),
    ];
    if expected
        .iter()
        .any(|(pointer, value)| schema.pointer(pointer) != Some(value))
    {
        return Err(PackageError::new(
            "platform-policy",
            "support schema does not enforce the reviewed closed D2 contract",
        ));
    }
    Ok(())
}

pub(super) fn read_json(
    path: &Path,
    category: &'static str,
) -> Result<serde_json::Value, PackageError> {
    let bytes = fs::read(path).map_err(|error| {
        PackageError::new(
            category,
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        PackageError::new(category, format!("invalid {}: {error}", path.display()))
    })
}

pub(super) fn validate_support_contract(support: &PlatformSupport) -> Result<(), PackageError> {
    let durable_targets = DURABLE_TARGETS.map(str::to_owned);
    if support.schema_version != 1
        || support.evidence_tier != "d2_supported"
        || support.durable_targets != durable_targets
        || support.conditional_targets.len() != 1
        || support.conditional_targets[0].target != CONDITIONAL_TARGET
        || support.conditional_targets[0].tier != "conditional_supported"
        || support.conditional_evidence_policy.max_age_days != MAX_EVIDENCE_AGE_DAYS
        || support
            .conditional_evidence_policy
            .missing_or_expired_outcome
            != "unsupported"
        || support.scalar_mode != SCALAR_MODE
        || support.compiler_class != COMPILER_CLASS
        || support.tolerance_profile != TOLERANCE_PROFILE
    {
        return Err(PackageError::new(
            "platform-policy",
            "support manifest does not match the reviewed D2 platform contract",
        ));
    }
    Ok(())
}

pub(super) fn validate_toolchain(toolchain: &str, target: &str) -> Result<(), PackageError> {
    let valid_msrv = toolchain == MSRV_TOOLCHAIN && target == CANONICAL_TARGET;
    let valid_native = toolchain == NATIVE_TOOLCHAIN
        && (DURABLE_TARGETS.contains(&target) || target == CONDITIONAL_TARGET);
    if !valid_msrv && !valid_native {
        return Err(PackageError::new(
            "platform-toolchain",
            format!("toolchain `{toolchain}` is not valid for target `{target}`"),
        ));
    }
    Ok(())
}

pub(super) fn validate_native_evidence(
    conditional: &ConditionalTarget,
) -> Result<(), PackageError> {
    let Some(evidence) = &conditional.native_evidence else {
        return Err(PackageError::new(
            "platform-evidence",
            "conditional macOS x86_64 support is downgraded because native evidence is missing",
        ));
    };
    if evidence.runner.trim().is_empty() {
        return Err(PackageError::new(
            "platform-evidence",
            "conditional native evidence runner must be named",
        ));
    }
    let maximum_age = MAX_EVIDENCE_AGE_DAYS * SECONDS_PER_DAY;
    let expected_expiry = evidence
        .recorded_at_unix
        .checked_add(maximum_age)
        .ok_or_else(|| PackageError::new("platform-evidence", "evidence expiry overflow"))?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PackageError::new("platform-evidence", error.to_string()))?
        .as_secs();
    if evidence.expires_at_unix != expected_expiry
        || evidence.recorded_at_unix > now
        || evidence.expires_at_unix < now
    {
        return Err(PackageError::new(
            "platform-evidence",
            "conditional macOS x86_64 support is downgraded because native evidence is expired",
        ));
    }
    Ok(())
}
