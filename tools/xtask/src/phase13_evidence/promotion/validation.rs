use super::{
    ARTIFACT_MANIFEST_PATH, BTreeMap, BTreeSet, BUNDLE_SHA256, BundleClosure, BundleManifest,
    ClosureEntry, Command, Digest, EXACT_BYTES_DIGEST_MODE, MATERIALS_MANIFEST, MaterialsManifest,
    PRODUCER_SHA, PROMOTED_PATHS, Path, PromotionError, PromotionErrorKind, PromotionReceipt,
    RECEIPT_PATH, RECEIPT_SEMANTIC_DIGEST_MODE, REPLAY_EVIDENCE_PATH, Sha256, UPSTREAM_REVISION,
    WITNESS_PATH, WITNESS_PROVENANCE_PATH, WITNESS_REPOSITORY_PREFIXES, closure_digest,
    collect_regular_files, file_sha256, filesystem_error, fs, git_file, promoted_paths, read_json,
    run_process, sha256, update_field, valid_digest, validate_relative_path,
};
#[cfg(test)]
use super::{
    Acquisition, PROVIDER_ARTIFACT_ID, PROVIDER_ARTIFACT_NAME, PROVIDER_DIGEST,
    PROVIDER_REPOSITORY, PROVIDER_RUN_ID, ReceiptFields, render_receipt,
};

pub(super) fn validate_bundle_contract(manifest: &BundleManifest) -> Result<(), PromotionError> {
    if manifest.producer_sha != PRODUCER_SHA
        || manifest.bundle_sha256 != BUNDLE_SHA256
        || manifest.upstream_revision != UPSTREAM_REVISION
        || manifest.sealed_input_sha256 != manifest.d1_input_sha256
        || manifest.native_d0_repeat_sha256[0] != manifest.native_d0_repeat_sha256[1]
        || !valid_digest(&manifest.native_d0_repeat_sha256[0])
        || !valid_digest(&manifest.d1_oracle_identity_sha256)
        || manifest.d1_result != "match"
        || manifest
            .diagnosis
            .get("drift_class")
            .and_then(serde_json::Value::as_str)
            != Some("capture_schema_drift")
        || manifest
            .diagnosis
            .pointer("/reviewed_schema/projection_version")
            .and_then(serde_json::Value::as_str)
            != Some("legacy_physics_v1")
        || manifest
            .diagnosis
            .pointer("/current_schema/projection_version")
            .and_then(serde_json::Value::as_str)
            != Some("expanded_checkpoint_v1")
    {
        return Err(PromotionError::new(
            PromotionErrorKind::Bundle,
            "bundle does not satisfy the canonical diagnosis-selected D0/D1 contract",
        ));
    }
    Ok(())
}

pub(super) fn validate_bundle_files(
    bundle_root: &Path,
    manifest: &BundleManifest,
) -> Result<(), PromotionError> {
    let expected = [
        "evidence/replay.json",
        "evidence/witness.json",
        "evidence/witness.provenance.json",
        "sealed/rigid-stack-v1.json",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let actual = manifest
        .files
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(PromotionError::new(
            PromotionErrorKind::Bundle,
            "bundle evidence file set is not canonical",
        ));
    }
    for entry in &manifest.files {
        let actual = file_sha256(&bundle_root.join(&entry.path))?;
        if actual != entry.sha256
            || entry.source_revision != UPSTREAM_REVISION
            || entry.derivation_kind.trim().is_empty()
            || entry.alteration_summary.trim().is_empty()
            || entry.notice_refs != ["THIRD_PARTY_NOTICES.md"]
            || !matches!(entry.record_class.as_str(), "witness" | "replay_evidence")
        {
            return Err(PromotionError::new(
                PromotionErrorKind::Bundle,
                format!("bundle file `{}` has stale bytes or metadata", entry.path),
            ));
        }
    }
    let sealed =
        fs::read(bundle_root.join("sealed/rigid-stack-v1.json")).map_err(filesystem_error)?;
    let tracked = fs::read(
        bundle_root
            .ancestors()
            .find(|candidate| candidate.join("Cargo.toml").is_file())
            .unwrap_or(bundle_root)
            .join("scenarios/catalog/rigid-stack-v1.json"),
    )
    .unwrap_or_else(|_| sealed.clone());
    if sha256(&sealed) != manifest.sealed_input_sha256 || sealed != tracked {
        return Err(PromotionError::new(
            PromotionErrorKind::Bundle,
            "sealed rigid-stack input differs from the reviewed repository bytes",
        ));
    }
    Ok(())
}

pub(super) fn derive_witness_closure(
    repository_root: &Path,
    revision: &str,
) -> Result<BundleClosure, PromotionError> {
    let manifest: MaterialsManifest = read_json(&repository_root.join(MATERIALS_MANIFEST))?;
    if manifest.schema_version != 1
        || manifest.target != "phase9-lifecycle-contact-witness"
        || manifest.preset != "oracle-debug"
    {
        return Err(PromotionError::new(
            PromotionErrorKind::Closure,
            "witness materials manifest identity is invalid",
        ));
    }
    let mut entries = derive_git_entries(repository_root, revision, &WITNESS_REPOSITORY_PREFIXES)?;
    for material in manifest.materials {
        if !matches!(material.kind.as_str(), "source" | "header" | "build_rule") {
            continue;
        }
        let candidate = repository_root.join(&material.identity);
        if candidate.is_file() {
            entries.insert(material.identity, file_sha256(&candidate)?);
        }
    }
    closure_from_entries("witness", entries)
}

pub(super) fn derive_git_closure(
    repository_root: &Path,
    revision: &str,
    label: &str,
    prefixes: &[&str],
) -> Result<BundleClosure, PromotionError> {
    closure_from_entries(
        label,
        derive_git_entries(repository_root, revision, prefixes)?,
    )
}

pub(super) fn derive_git_entries(
    repository_root: &Path,
    revision: &str,
    prefixes: &[&str],
) -> Result<BTreeMap<String, String>, PromotionError> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repository_root)
        .args(["ls-tree", "-r", "--name-only", revision, "--"])
        .args(prefixes);
    let output = run_process(&mut command, "enumerate producer-affecting Git inputs")?;
    let names = String::from_utf8(output.stdout).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Git,
            format!("Git returned non-UTF-8 paths: {error}"),
        )
    })?;
    let mut entries = BTreeMap::new();
    for path in names.lines() {
        validate_relative_path(path)?;
        entries.insert(
            path.to_owned(),
            sha256(&git_file(repository_root, revision, path)?),
        );
    }
    Ok(entries)
}

pub(super) fn closure_from_entries(
    label: &str,
    entries: BTreeMap<String, String>,
) -> Result<BundleClosure, PromotionError> {
    if entries.is_empty() {
        return Err(PromotionError::new(
            PromotionErrorKind::Closure,
            format!("{label} closure is empty"),
        ));
    }
    let entries = entries
        .into_iter()
        .map(|(path, sha256)| ClosureEntry { path, sha256 })
        .collect::<Vec<_>>();
    Ok(BundleClosure {
        schema_version: 1,
        label: label.to_owned(),
        digest: closure_digest(label, &entries),
        entries,
    })
}

pub(super) fn validate_staged_tree(
    staging_root: &Path,
) -> Result<BTreeMap<String, String>, PromotionError> {
    let actual = collect_regular_files(staging_root)?;
    let expected = PROMOTED_PATHS
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    validate_exact_paths(&actual, &expected)?;
    actual
        .into_iter()
        .map(|path| {
            let digest = file_sha256(&staging_root.join(&path))?;
            Ok((path, digest))
        })
        .collect()
}

pub(super) fn baseline_sha256(
    repository_root: &Path,
    revision: &str,
) -> Result<BTreeMap<String, String>, PromotionError> {
    PROMOTED_PATHS
        .into_iter()
        .map(|path| {
            git_file(repository_root, revision, path).map(|bytes| (path.to_owned(), sha256(&bytes)))
        })
        .collect()
}

pub(super) fn replacement_sha256(
    replacements: &BTreeMap<String, Vec<u8>>,
) -> BTreeMap<String, String> {
    replacements
        .iter()
        .map(|(path, bytes)| (path.clone(), sha256(bytes)))
        .collect()
}

pub(super) fn receipt_semantic_sha256(receipt: &[u8]) -> Result<String, PromotionError> {
    let mut normalized: PromotionReceipt = serde_json::from_slice(receipt).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Schema,
            format!("invalid promotion receipt for semantic hashing: {error}"),
        )
    })?;
    normalized.promoted_content_sha256.clear();
    normalized.changed_content_sha256.clear();
    let canonical = serde_json::to_vec(&normalized).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Schema,
            format!("failed to normalize promotion receipt: {error}"),
        )
    })?;
    let mut hasher = Sha256::new();
    update_field(&mut hasher, b"phase13-promotion-receipt-semantic-leaf-v2");
    update_field(&mut hasher, &canonical);
    Ok(format!("{:x}", hasher.finalize()))
}

pub(super) fn reviewed_content_digest(
    domain: &str,
    replacements: &BTreeMap<String, Vec<u8>>,
    paths: &[String],
) -> Result<String, PromotionError> {
    let unique_paths = paths.iter().collect::<BTreeSet<_>>();
    if unique_paths.len() != paths.len() {
        return Err(PromotionError::new(
            PromotionErrorKind::Path,
            "content digest path set contains duplicates",
        ));
    }
    let mut hasher = Sha256::new();
    update_field(&mut hasher, domain.as_bytes());
    for path in unique_paths {
        let bytes = replacements.get(path).ok_or_else(|| {
            PromotionError::new(
                PromotionErrorKind::Path,
                format!("content digest path `{path}` is absent"),
            )
        })?;
        let leaf = if path == RECEIPT_PATH {
            receipt_semantic_sha256(bytes)?
        } else {
            sha256(bytes)
        };
        update_field(&mut hasher, path.as_bytes());
        update_field(&mut hasher, leaf.as_bytes());
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub(super) fn reviewed_content_digests(
    replacements: &BTreeMap<String, Vec<u8>>,
    changed_paths: &[String],
) -> Result<(String, String), PromotionError> {
    let promoted = promoted_paths();
    if replacements
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != promoted.iter().map(String::as_str).collect::<BTreeSet<_>>()
    {
        return Err(PromotionError::new(
            PromotionErrorKind::Path,
            "content digest replacements do not equal the promoted set",
        ));
    }
    Ok((
        reviewed_content_digest("phase13-promoted-content-set-v2", replacements, &promoted)?,
        reviewed_content_digest(
            "phase13-changed-content-set-v2",
            replacements,
            changed_paths,
        )?,
    ))
}

pub(super) fn reviewed_content_digests_from_root(
    root: &Path,
    changed_paths: &[String],
) -> Result<(String, String), PromotionError> {
    reviewed_content_digests(&reviewed_replacements_from_root(root)?, changed_paths)
}

pub(super) fn reviewed_replacements_from_root(
    root: &Path,
) -> Result<BTreeMap<String, Vec<u8>>, PromotionError> {
    promoted_paths()
        .into_iter()
        .map(|path| {
            fs::read(root.join(&path))
                .map(|bytes| (path, bytes))
                .map_err(filesystem_error)
        })
        .collect()
}

pub(super) fn validate_content_digest_claims(
    replacements: &BTreeMap<String, Vec<u8>>,
) -> Result<(String, String), PromotionError> {
    let receipt_bytes = replacements.get(RECEIPT_PATH).ok_or_else(|| {
        PromotionError::new(
            PromotionErrorKind::Path,
            "promotion receipt is absent from reviewed content",
        )
    })?;
    let receipt: PromotionReceipt = serde_json::from_slice(receipt_bytes).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Schema,
            format!("invalid promotion receipt content claims: {error}"),
        )
    })?;
    let actual = reviewed_content_digests(replacements, &receipt.changed_paths)?;
    if !valid_digest(&receipt.promoted_content_sha256)
        || !valid_digest(&receipt.changed_content_sha256)
        || actual
            != (
                receipt.promoted_content_sha256,
                receipt.changed_content_sha256,
            )
    {
        return Err(PromotionError::new(
            PromotionErrorKind::Schema,
            "promotion receipt content digest claims do not match reviewed bytes",
        ));
    }
    Ok(actual)
}

#[cfg(test)]
#[allow(
    dead_code,
    reason = "the path-included integration contract consumes this digest seam"
)]
pub(crate) fn reviewed_content_digests_for_test(
    replacements: &BTreeMap<String, Vec<u8>>,
    changed_paths: &[String],
) -> Result<(String, String), PromotionError> {
    reviewed_content_digests(replacements, changed_paths)
}

#[cfg(test)]
#[allow(
    dead_code,
    reason = "the path-included integration contract consumes this validation seam"
)]
pub(crate) fn validate_content_digest_claims_for_test(
    replacements: &BTreeMap<String, Vec<u8>>,
) -> Result<(String, String), PromotionError> {
    validate_content_digest_claims(replacements)
}

#[cfg(test)]
#[allow(
    dead_code,
    reason = "the path-included integration contract consumes this receipt fixture"
)]
pub(crate) fn promotion_receipt_for_test(
    promoted_content_sha256: &str,
    changed_content_sha256: &str,
) -> Vec<u8> {
    let promotion_base_sha = "b".repeat(40);
    let changed_paths = promoted_paths();
    let acquisition = Acquisition {
        repository: PROVIDER_REPOSITORY.to_owned(),
        run_id: PROVIDER_RUN_ID,
        artifact_id: PROVIDER_ARTIFACT_ID,
        artifact_name: PROVIDER_ARTIFACT_NAME.to_owned(),
        provider_digest: PROVIDER_DIGEST.to_owned(),
        artifact_created_at: "2026-07-26T00:52:47Z".to_owned(),
        artifact_expires_at: "2026-10-24T00:50:42Z".to_owned(),
    };
    let manifest = BundleManifest {
        producer_sha: PRODUCER_SHA.to_owned(),
        bundle_sha256: BUNDLE_SHA256.to_owned(),
        upstream_revision: UPSTREAM_REVISION.to_owned(),
        witness_closure: BundleClosure {
            schema_version: 1,
            label: "witness".to_owned(),
            digest: "a".repeat(64),
            entries: Vec::new(),
        },
        replay_closure: BundleClosure {
            schema_version: 1,
            label: "replay".to_owned(),
            digest: "b".repeat(64),
            entries: Vec::new(),
        },
        sealed_input_sha256: "c".repeat(64),
        d1_input_sha256: "c".repeat(64),
        native_d0_repeat_sha256: ["d".repeat(64), "d".repeat(64)],
        d1_oracle_identity_sha256: "e".repeat(64),
        d1_result: "match".to_owned(),
        diagnosis: serde_json::Value::Null,
        files: Vec::new(),
    };
    render_receipt(&ReceiptFields {
        producer_sha: PRODUCER_SHA,
        bundle_sha256: BUNDLE_SHA256,
        promotion_base_sha: &promotion_base_sha,
        reviewer_id: "pRizz",
        acquisition: &acquisition,
        manifest: &manifest,
        changed_paths: &changed_paths,
        unchanged_paths: &[],
        promoted_content_sha256,
        changed_content_sha256,
    })
    .expect("test promotion receipt should serialize")
}

pub(crate) fn validate_exact_paths(
    actual: &BTreeSet<String>,
    expected: &BTreeSet<String>,
) -> Result<(), PromotionError> {
    if actual == expected {
        return Ok(());
    }
    Err(PromotionError::new(
        PromotionErrorKind::Path,
        "staging tree must contain exactly the seven promoted paths",
    ))
}

pub(crate) fn validate_staged_ledgers(
    staging_root: &Path,
    replacement_sha256: &BTreeMap<String, String>,
) -> Result<(), PromotionError> {
    let contents =
        fs::read_to_string(staging_root.join(ARTIFACT_MANIFEST_PATH)).map_err(filesystem_error)?;
    let manifest: toml::Value = toml::from_str(&contents).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Ledger,
            format!("invalid staged artifact manifest: {error}"),
        )
    })?;
    let records = manifest
        .get("artifact_schemas")
        .and_then(|value| value.get("phase13_evidence"))
        .and_then(|value| value.get("records"))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| {
            PromotionError::new(
                PromotionErrorKind::Ledger,
                "staged artifact records are absent",
            )
        })?;
    if records.len() != 4 {
        return Err(PromotionError::new(
            PromotionErrorKind::Ledger,
            "staged artifact records are incomplete",
        ));
    }
    let expected_record_paths = [
        WITNESS_PATH,
        WITNESS_PROVENANCE_PATH,
        REPLAY_EVIDENCE_PATH,
        RECEIPT_PATH,
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let mut actual_record_paths = BTreeSet::new();
    for record in records {
        let path = record
            .get("path")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                PromotionError::new(PromotionErrorKind::Ledger, "staged record path is absent")
            })?;
        let digest = record
            .get("sha256")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                PromotionError::new(PromotionErrorKind::Ledger, "staged record digest is absent")
            })?;
        let digest_mode = record
            .get("digest_mode")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                PromotionError::new(
                    PromotionErrorKind::Ledger,
                    "staged record digest mode is absent",
                )
            })?;
        let expected_mode = if path == RECEIPT_PATH {
            RECEIPT_SEMANTIC_DIGEST_MODE
        } else {
            EXACT_BYTES_DIGEST_MODE
        };
        let expected_digest = if path == RECEIPT_PATH {
            receipt_semantic_sha256(&fs::read(staging_root.join(path)).map_err(filesystem_error)?)?
        } else {
            replacement_sha256.get(path).cloned().ok_or_else(|| {
                PromotionError::new(
                    PromotionErrorKind::Ledger,
                    format!("staged artifact path `{path}` is not reviewed"),
                )
            })?
        };
        if digest_mode != expected_mode
            || expected_digest != digest
            || !actual_record_paths.insert(path)
        {
            return Err(PromotionError::new(
                PromotionErrorKind::Ledger,
                format!("staged artifact ledger digest contract for `{path}` is stale"),
            ));
        }
    }
    if actual_record_paths != expected_record_paths {
        return Err(PromotionError::new(
            PromotionErrorKind::Ledger,
            "staged artifact record paths are incomplete or unexpected",
        ));
    }
    Ok(())
}
