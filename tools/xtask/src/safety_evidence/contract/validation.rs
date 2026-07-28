#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use super::*;

pub(super) fn validate_regression_record(
    repository_root: &Path,
    record: &RegressionRecord,
) -> Result<(), ContractError> {
    if !is_identifier(&record.id)
        || record.target.is_empty()
        || record.generator.is_empty()
        || record.toolchain.is_empty()
        || !is_full_sha(&record.candidate_commit)
        || !is_full_sha(&record.fix_commit)
        || record.candidate_commit == record.fix_commit
        || record.first_divergence_signature.is_empty()
        || !record.named_test_path.contains("::")
        || record.review_status != ReviewStatus::Reviewed
    {
        return Err(ContractError::new(
            "regression identity, provenance, review, or named test is invalid",
        ));
    }
    if record.failure_class == FailureClass::PhysicsMismatch
        && (record.oracle_identity.as_deref().is_none_or(str::is_empty)
            || record
                .tolerance_identity
                .as_deref()
                .is_none_or(str::is_empty))
    {
        return Err(ContractError::new(
            "physics mismatch requires oracle and tolerance identities",
        ));
    }
    for maybe_identity in [&record.oracle_identity, &record.tolerance_identity] {
        if maybe_identity.as_deref().is_some_and(str::is_empty) {
            return Err(ContractError::new(
                "optional provenance identities cannot be empty",
            ));
        }
    }
    if !is_sha256(&record.minimized_sha256) {
        return Err(ContractError::new("minimized input SHA-256 is invalid"));
    }
    let relative = normalized_relative(&record.minimized_path)?;
    if !(relative.starts_with("scenarios/regressions")
        || relative.starts_with("fuzz/corpus/regressions"))
    {
        return Err(ContractError::new(
            "minimized input must remain under a reviewed regression root",
        ));
    }
    let bytes = read_confined_regular(repository_root, relative)?;
    if sha256(&bytes) != record.minimized_sha256 {
        return Err(ContractError::new("minimized input SHA-256 mismatch"));
    }
    Ok(())
}

pub(super) fn validate_coverage_sections(contract: &CoverageContract) -> Result<(), ContractError> {
    if contract.rust.evidence_kinds
        != [
            CoverageEvidenceKind::RustSanitizer,
            CoverageEvidenceKind::RustCoverage,
        ]
        || contract.cpp.evidence_kinds
            != [
                CoverageEvidenceKind::CppAsanUbsan,
                CoverageEvidenceKind::CppCoverage,
            ]
        || contract.differential.evidence_kinds != [CoverageEvidenceKind::DifferentialCoverage]
        || contract.rust.leaf_kind != CoverageLeafKind::Files
        || contract.cpp.leaf_kind != CoverageLeafKind::Files
        || contract.differential.leaf_kind != CoverageLeafKind::DifferentialLeaves
        || contract.rust.toolchain_identities.is_empty()
        || contract.cpp.toolchain_identities.is_empty()
        || contract.differential.toolchain_identities.is_empty()
        || contract
            .rust
            .toolchain_identities
            .iter()
            .chain(&contract.cpp.toolchain_identities)
            .chain(&contract.differential.toolchain_identities)
            .any(String::is_empty)
    {
        return Err(ContractError::new(
            "Rust, C++, and differential coverage identities must remain distinct and complete",
        ));
    }
    Ok(())
}

pub(super) fn validate_coverage_record(
    repository_root: &Path,
    contract: &CoverageContract,
    candidate_commit: &str,
    record: &CoverageRecord,
) -> Result<(), ContractError> {
    let expected_toolchains = match record.evidence_kind {
        CoverageEvidenceKind::RustSanitizer | CoverageEvidenceKind::RustCoverage => {
            &contract.rust.toolchain_identities
        }
        CoverageEvidenceKind::CppAsanUbsan | CoverageEvidenceKind::CppCoverage => {
            &contract.cpp.toolchain_identities
        }
        CoverageEvidenceKind::DifferentialCoverage => &contract.differential.toolchain_identities,
    };
    let relative_artifact = normalized_relative(&record.artifact_path)?;
    if record.candidate_commit != candidate_commit
        || !expected_toolchains.contains(&record.toolchain_identity)
        || !is_sha256(&record.artifact_sha256)
        || !relative_artifact.starts_with("target")
        || record.subsystems.is_empty()
    {
        return Err(ContractError::new(
            "coverage record identity or subsystem inventory is invalid",
        ));
    }
    let artifact_bytes = read_confined_regular(repository_root, relative_artifact)?;
    if sha256(&artifact_bytes) != record.artifact_sha256 {
        return Err(ContractError::new("coverage artifact SHA-256 mismatch"));
    }
    let mut subsystems = BTreeSet::new();
    for subsystem in &record.subsystems {
        let exercised = subsystem
            .exercised_files_or_leaves
            .iter()
            .collect::<BTreeSet<_>>();
        let missed = subsystem
            .missed_files_or_leaves
            .iter()
            .collect::<BTreeSet<_>>();
        if subsystem.name.is_empty()
            || !subsystems.insert(subsystem.name.as_str())
            || (subsystem.exercised_files_or_leaves.is_empty()
                && subsystem.missed_files_or_leaves.is_empty())
            || exercised.len() != subsystem.exercised_files_or_leaves.len()
            || missed.len() != subsystem.missed_files_or_leaves.len()
            || exercised.iter().any(|leaf| leaf.is_empty())
            || missed.iter().any(|leaf| leaf.is_empty())
            || !exercised.is_disjoint(&missed)
        {
            return Err(ContractError::new(
                "coverage subsystem leaves are incomplete or duplicated",
            ));
        }
    }
    Ok(())
}

pub(super) fn coverage_kinds(contract: &CoverageContract) -> BTreeSet<CoverageEvidenceKind> {
    contract
        .rust
        .evidence_kinds
        .iter()
        .chain(&contract.cpp.evidence_kinds)
        .chain(&contract.differential.evidence_kinds)
        .copied()
        .collect()
}

pub(super) fn normalized_relative(value: &str) -> Result<&Path, ContractError> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ContractError::new("path is not normalized and relative"));
    }
    Ok(path)
}

pub(super) fn read_confined_regular(
    repository_root: &Path,
    relative: &Path,
) -> Result<Vec<u8>, ContractError> {
    let canonical_root = fs::canonicalize(repository_root).map_err(|error| {
        ContractError::new(format!(
            "failed to resolve {}: {error}",
            repository_root.display()
        ))
    })?;
    let mut path = repository_root.to_path_buf();
    for component in relative {
        path.push(component);
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            ContractError::new(format!("failed to inspect {}: {error}", path.display()))
        })?;
        if metadata.file_type().is_symlink() {
            return Err(ContractError::new(format!(
                "{} contains a symbolic link",
                path.display()
            )));
        }
    }
    let metadata = fs::symlink_metadata(&path).map_err(|error| {
        ContractError::new(format!("failed to inspect {}: {error}", path.display()))
    })?;
    if !metadata.is_file() {
        return Err(ContractError::new(format!(
            "{} is not an ordinary file",
            path.display()
        )));
    }
    let canonical = fs::canonicalize(&path).map_err(|error| {
        ContractError::new(format!("failed to resolve {}: {error}", path.display()))
    })?;
    if !canonical.starts_with(canonical_root) {
        return Err(ContractError::new("contract artifact escaped repository"));
    }
    let length = usize::try_from(metadata.len())
        .map_err(|_error| ContractError::new("contract file length exceeds usize"))?;
    if length > MAXIMUM_CONTRACT_BYTES {
        return Err(ContractError::new("contract file exceeds reviewed bound"));
    }
    fs::read(&path)
        .map_err(|error| ContractError::new(format!("failed to read {}: {error}", path.display())))
}

pub(super) fn enforce_size(field: &str, bytes: &[u8]) -> Result<(), ContractError> {
    if bytes.len() > MAXIMUM_CONTRACT_BYTES {
        return Err(ContractError::new(format!(
            "{field} exceeds reviewed byte bound"
        )));
    }
    Ok(())
}

pub(super) fn is_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

pub(super) fn is_leaf_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.')
        })
}

pub(super) fn is_full_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
