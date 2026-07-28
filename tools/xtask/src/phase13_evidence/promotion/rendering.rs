use super::{
    ARTIFACT_MANIFEST_PATH, Acquisition, BTreeMap, BUNDLE_SHA256, BundleFileEntry, BundleManifest,
    CATALOG_PATH, EXACT_BYTES_DIGEST_MODE, PRODUCER_SHA, Path, ProducerClosures,
    PromotionCommitContract, PromotionError, PromotionErrorKind, PromotionReceipt, RECEIPT_PATH,
    RECEIPT_SEMANTIC_DIGEST_MODE, REPLAY_EVIDENCE_PATH, SOURCE_MAP_PATH, UPSTREAM_REVISION,
    WITNESS_PATH, WITNESS_PROVENANCE_PATH, changed_path_set_sha256, filesystem_error, fs,
    json_bytes, promoted_path_set_sha256, promoted_paths, read_json, receipt_semantic_sha256,
    sha256,
};
use std::fmt::Write as _;

type ArtifactHash<'a> = (
    &'a str,
    String,
    &'a str,
    Option<&'a BundleFileEntry>,
    &'a str,
    &'a str,
);

pub(super) struct ReceiptFields<'a> {
    pub(super) producer_sha: &'a str,
    pub(super) bundle_sha256: &'a str,
    pub(super) promotion_base_sha: &'a str,
    pub(super) reviewer_id: &'a str,
    pub(super) acquisition: &'a Acquisition,
    pub(super) manifest: &'a BundleManifest,
    pub(super) changed_paths: &'a [String],
    pub(super) unchanged_paths: &'a [String],
    pub(super) promoted_content_sha256: &'a str,
    pub(super) changed_content_sha256: &'a str,
}

pub(super) fn render_replacements(
    repository_root: &Path,
    bundle_root: &Path,
    manifest: &BundleManifest,
    receipt: &[u8],
    reviewer_id: &str,
    promotion_base_sha: &str,
    generation_timestamp: &str,
) -> Result<BTreeMap<String, Vec<u8>>, PromotionError> {
    let mut replacements = BTreeMap::new();
    replacements.insert(
        WITNESS_PATH.to_owned(),
        fs::read(bundle_root.join("evidence/witness.json")).map_err(filesystem_error)?,
    );
    replacements.insert(
        WITNESS_PROVENANCE_PATH.to_owned(),
        render_witness_provenance(bundle_root, generation_timestamp)?,
    );
    replacements.insert(
        REPLAY_EVIDENCE_PATH.to_owned(),
        fs::read(bundle_root.join("evidence/replay.json")).map_err(filesystem_error)?,
    );
    replacements.insert(RECEIPT_PATH.to_owned(), receipt.to_vec());
    replacements.insert(
        CATALOG_PATH.to_owned(),
        render_catalog(
            &fs::read_to_string(repository_root.join(CATALOG_PATH)).map_err(|error| {
                PromotionError::new(
                    PromotionErrorKind::Filesystem,
                    format!("failed to read catalog replay source: {error}"),
                )
            })?,
        )?
        .into_bytes(),
    );
    replacements.insert(
        SOURCE_MAP_PATH.to_owned(),
        render_source_map(
            &fs::read_to_string(repository_root.join(SOURCE_MAP_PATH)).map_err(filesystem_error)?,
        )?
        .into_bytes(),
    );
    let replacement_digest = |path: &'static str| {
        replacements
            .get(path)
            .map(|bytes| sha256(bytes))
            .ok_or_else(|| {
                PromotionError::new(
                    PromotionErrorKind::Schema,
                    format!("replacement `{path}` is absent"),
                )
            })
    };
    let artifact_hashes = [
        (
            WITNESS_PATH,
            replacement_digest(WITNESS_PATH)?,
            "witness",
            manifest
                .files
                .iter()
                .find(|entry| entry.path == "evidence/witness.json"),
            PRODUCER_SHA,
            EXACT_BYTES_DIGEST_MODE,
        ),
        (
            WITNESS_PROVENANCE_PATH,
            replacement_digest(WITNESS_PROVENANCE_PATH)?,
            "witness",
            manifest
                .files
                .iter()
                .find(|entry| entry.path == "evidence/witness.provenance.json"),
            PRODUCER_SHA,
            EXACT_BYTES_DIGEST_MODE,
        ),
        (
            REPLAY_EVIDENCE_PATH,
            replacement_digest(REPLAY_EVIDENCE_PATH)?,
            "replay_evidence",
            manifest
                .files
                .iter()
                .find(|entry| entry.path == "evidence/replay.json"),
            PRODUCER_SHA,
            EXACT_BYTES_DIGEST_MODE,
        ),
        (
            RECEIPT_PATH,
            receipt_semantic_sha256(receipt)?,
            "promotion_receipt",
            None,
            promotion_base_sha,
            RECEIPT_SEMANTIC_DIGEST_MODE,
        ),
    ];
    replacements.insert(
        ARTIFACT_MANIFEST_PATH.to_owned(),
        render_artifact_manifest(
            &fs::read_to_string(repository_root.join(ARTIFACT_MANIFEST_PATH))
                .map_err(filesystem_error)?,
            &artifact_hashes,
            reviewer_id,
        )?
        .into_bytes(),
    );
    Ok(replacements)
}

pub(super) fn render_receipt(fields: &ReceiptFields<'_>) -> Result<Vec<u8>, PromotionError> {
    let required_trailers = BTreeMap::from([
        (
            "Phase13-Producer-SHA".to_owned(),
            fields.producer_sha.to_owned(),
        ),
        (
            "Phase13-Bundle-SHA256".to_owned(),
            fields.bundle_sha256.to_owned(),
        ),
        (
            "Phase13-Promotion-Base-SHA".to_owned(),
            fields.promotion_base_sha.to_owned(),
        ),
    ]);
    json_bytes(&PromotionReceipt {
        schema_version: 2,
        producer_sha: fields.producer_sha.to_owned(),
        bundle_sha256: fields.bundle_sha256.to_owned(),
        promotion_base_sha: fields.promotion_base_sha.to_owned(),
        acquisition: fields.acquisition.clone(),
        independent_reviewer_id: fields.reviewer_id.to_owned(),
        promoted_paths: promoted_paths(),
        promoted_path_set_sha256: promoted_path_set_sha256(),
        promoted_content_sha256: fields.promoted_content_sha256.to_owned(),
        changed_paths: fields.changed_paths.to_vec(),
        unchanged_paths: fields.unchanged_paths.to_vec(),
        changed_path_set_sha256: changed_path_set_sha256(fields.changed_paths),
        changed_content_sha256: fields.changed_content_sha256.to_owned(),
        producer_closures: ProducerClosures {
            witness_sha256: fields.manifest.witness_closure.digest.clone(),
            replay_sha256: fields.manifest.replay_closure.digest.clone(),
            recomputed_at_r: true,
        },
        q_contract: PromotionCommitContract {
            required_first_parent: fields.promotion_base_sha.to_owned(),
            required_trailers,
            q_sha_recorded: false,
            acceptance_sha_recorded: false,
        },
    })
}

pub(super) fn render_witness_provenance(
    bundle_root: &Path,
    generation_timestamp: &str,
) -> Result<Vec<u8>, PromotionError> {
    let mut value: serde_json::Value =
        read_json(&bundle_root.join("evidence/witness.provenance.json"))?;
    let object = value.as_object_mut().ok_or_else(|| {
        PromotionError::new(
            PromotionErrorKind::Schema,
            "witness provenance is not an object",
        )
    })?;
    object.insert(
        "exact_argv".to_owned(),
        serde_json::json!([
            "target/reference/oracle-debug/phase9-lifecycle-contact-witness",
            "--output",
            WITNESS_PATH,
            "--provenance",
            WITNESS_PROVENANCE_PATH
        ]),
    );
    object.insert(
        "generation_timestamp".to_owned(),
        serde_json::Value::String(generation_timestamp.to_owned()),
    );
    json_bytes(&value)
}

pub(super) fn render_catalog(current: &str) -> Result<String, PromotionError> {
    if current.contains("RIGID_STACK_REPLAY_EVIDENCE_PATH") {
        if current.contains("fn validate_rigid_stack_replay_evidence(")
            && current
                .contains("validate_rigid_stack_replay_evidence(&canonical_root, &manifest)?;")
        {
            return Ok(current.to_owned());
        }
        return Err(PromotionError::new(
            PromotionErrorKind::Schema,
            "existing Phase 13 catalog binding is incomplete",
        ));
    }
    let with_constant = current.replacen(
        "const MANIFEST_PATH: &str = \"scenarios/regressions/catalog-manifest.json\";",
        "const MANIFEST_PATH: &str = \"scenarios/regressions/catalog-manifest.json\";\n\
const RIGID_STACK_REPLAY_EVIDENCE_PATH: &str =\n\
    \"reference/artifacts/catalog/rigid-stack-v1.replay-evidence.json\";",
        1,
    );
    if with_constant == current {
        return Err(PromotionError::new(
            PromotionErrorKind::Schema,
            "catalog manifest constant insertion point is absent",
        ));
    }
    let with_call = with_constant.replacen(
        "    validate_manifest_header(&manifest)?;\n",
        "    validate_manifest_header(&manifest)?;\n\
    validate_rigid_stack_replay_evidence(&canonical_root, &manifest)?;\n",
        1,
    );
    if with_call == with_constant {
        return Err(PromotionError::new(
            PromotionErrorKind::Schema,
            "catalog replay-evidence validation insertion point is absent",
        ));
    }
    Ok(format!("{with_call}\n{CATALOG_VALIDATOR}"))
}

const CATALOG_VALIDATOR: &str = r#"#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RigidStackReplayEvidence {
    schema_version: u32,
    upstream_revision: String,
    resolved_scenario_path: String,
    sealed_input_sha256: String,
    native_d0_repeat_sha256: [String; 2],
    d1_oracle_identity_sha256: String,
    d1_result: String,
    diagnosis: serde_json::Value,
}

fn validate_rigid_stack_replay_evidence(
    canonical_root: &Path,
    manifest: &RegressionManifest,
) -> Result<(), CatalogRegressionError> {
    let bytes = read_regular_confined(
        canonical_root,
        Path::new(RIGID_STACK_REPLAY_EVIDENCE_PATH),
        MAXIMUM_MANIFEST_BYTES,
        CatalogRegressionErrorKind::InvalidManifest,
    )?;
    let evidence: RigidStackReplayEvidence =
        serde_json::from_slice(&bytes).map_err(|_error| {
            CatalogRegressionError::new(CatalogRegressionErrorKind::InvalidManifest)
        })?;
    let rigid_stack = manifest
        .entries
        .iter()
        .find(|entry| entry.fixture_id == "rigid-stack-v1")
        .ok_or_else(|| {
            CatalogRegressionError::new(CatalogRegressionErrorKind::InvalidManifest)
        })?;
    let diagnosis = &evidence.diagnosis;
    let reviewed_projection = diagnosis
        .pointer("/reviewed_schema/projection_version")
        .and_then(serde_json::Value::as_str);
    let current_projection = diagnosis
        .pointer("/current_schema/projection_version")
        .and_then(serde_json::Value::as_str);
    let reviewed_resolved = diagnosis
        .get("reviewed_resolved_sha256")
        .and_then(serde_json::Value::as_str);
    let current_resolved = diagnosis
        .get("current_resolved_sha256")
        .and_then(serde_json::Value::as_str);
    let valid_digest = |value: &str| {
        value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    };
    if evidence.schema_version != 1
        || evidence.upstream_revision != PINNED_UPSTREAM_REVISION
        || evidence.resolved_scenario_path != rigid_stack.path
        || evidence.sealed_input_sha256 != rigid_stack.resolved_sha256.as_str()
        || evidence.native_d0_repeat_sha256[0] != evidence.native_d0_repeat_sha256[1]
        || !valid_digest(&evidence.native_d0_repeat_sha256[0])
        || !valid_digest(&evidence.d1_oracle_identity_sha256)
        || evidence.d1_result != "match"
        || diagnosis.get("drift_class").and_then(serde_json::Value::as_str)
            != Some("capture_schema_drift")
        || reviewed_projection != Some("legacy_physics_v1")
        || current_projection != Some("expanded_checkpoint_v1")
        || reviewed_resolved != Some(rigid_stack.resolved_sha256.as_str())
        || current_resolved != Some(rigid_stack.resolved_sha256.as_str())
    {
        return Err(CatalogRegressionError::new(
            CatalogRegressionErrorKind::InvalidManifest,
        ));
    }
    Ok(())
}
"#;

pub(super) fn render_source_map(current: &str) -> Result<String, PromotionError> {
    let replay_count = current.match_indices(REPLAY_EVIDENCE_PATH).count();
    let receipt_count = current.match_indices(RECEIPT_PATH).count();
    if replay_count == 1 && receipt_count == 1 {
        return Ok(current.to_owned());
    }
    if replay_count != 0 || receipt_count != 0 {
        return Err(PromotionError::new(
            PromotionErrorKind::Ledger,
            "source map contains an incomplete or duplicate Phase 13 mapping",
        ));
    }
    Ok(format!(
        "{current}\n[[mapping]]\n\
local_path = \"{REPLAY_EVIDENCE_PATH}\"\n\
upstream_revision = \"{UPSTREAM_REVISION}\"\n\
upstream_path = \".\"\n\
derivation_kind = \"repository-authored-replay-verification\"\n\
alteration_summary = \"Repository-authored canonical D0/D1 replay evidence preserving the reviewed legacy physics projection while validating expanded checkpoint diagnostics separately; no upstream source, raw object memory, or Rust-produced expectation is copied.\"\n\
notice_class = \"provenance-only\"\n\n\
[[mapping]]\n\
local_path = \"{RECEIPT_PATH}\"\n\
upstream_revision = \"{UPSTREAM_REVISION}\"\n\
upstream_path = \".\"\n\
derivation_kind = \"repository-authored-promotion-receipt\"\n\
alteration_summary = \"Repository-authored non-circular P/B/R acquisition and review contract for the exact promoted file set; no upstream source, raw object memory, or Rust-produced expectation is copied.\"\n\
notice_class = \"provenance-only\"\n"
    ))
}

pub(super) fn render_artifact_manifest(
    current: &str,
    artifact_hashes: &[ArtifactHash<'_>; 4],
    reviewer_id: &str,
) -> Result<String, PromotionError> {
    const MARKER: &str = "[[artifact_schemas.phase13_evidence.records]]";
    let existing: toml::Value = toml::from_str(current).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Ledger,
            format!("invalid artifact manifest before replacement: {error}"),
        )
    })?;
    let maybe_records = existing
        .get("artifact_schemas")
        .and_then(|value| value.get("phase13_evidence"))
        .and_then(|value| value.get("records"))
        .and_then(toml::Value::as_array);
    let mut rendered = match (current.find(MARKER), maybe_records) {
        (None, None) => current.to_owned(),
        (Some(index), Some(records)) if records.len() == 4 => {
            let suffix = &current[index..];
            if suffix.match_indices(MARKER).count() != 4 {
                return Err(PromotionError::new(
                    PromotionErrorKind::Ledger,
                    "existing Phase 13 artifact rows are not a closed four-record tail",
                ));
            }
            current[..index].trim_end().to_owned()
        }
        _ => {
            return Err(PromotionError::new(
                PromotionErrorKind::Ledger,
                "existing Phase 13 artifact rows are incomplete or duplicated",
            ));
        }
    };
    for (path, digest, record_class, maybe_entry, generator_revision, digest_mode) in
        artifact_hashes
    {
        let (
            source_revision,
            source_path,
            derivation_kind,
            alteration_summary,
            notice_refs,
        ) = maybe_entry.map_or_else(
            || {
                (
                    UPSTREAM_REVISION,
                    ".",
                    "repository-authored-promotion-receipt",
                    "Repository-authored review and promotion identity for a byte-exact staged evidence bundle; no upstream source, raw object memory, or Rust-produced expectations are copied.",
                    vec!["THIRD_PARTY_NOTICES.md".to_owned()],
                )
            },
            |entry| {
                (
                    entry.source_revision.as_str(),
                    entry.source_path.as_str(),
                    entry.derivation_kind.as_str(),
                    entry.alteration_summary.as_str(),
                    entry.notice_refs.clone(),
                )
            },
        );
        if notice_refs != ["THIRD_PARTY_NOTICES.md"] {
            return Err(PromotionError::new(
                PromotionErrorKind::Ledger,
                "bundle evidence record has incomplete notice metadata",
            ));
        }
        write!(
            rendered,
            "\n[[artifact_schemas.phase13_evidence.records]]\n\
record_class = \"{record_class}\"\n\
path = \"{path}\"\n\
sha256 = \"{digest}\"\n\
digest_mode = \"{digest_mode}\"\n\
generator_revision = \"{generator_revision}\"\n\
producer_sha = \"{PRODUCER_SHA}\"\n\
bundle_sha256 = \"{BUNDLE_SHA256}\"\n\
source_revision = \"{source_revision}\"\n\
source_path = \"{source_path}\"\n\
derivation_kind = \"{derivation_kind}\"\n\
alteration_summary = \"{alteration_summary}\"\n\
notice_refs = [\"THIRD_PARTY_NOTICES.md\"]\n\
reviewer = \"{reviewer_id}\"\n"
        )
        .map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Ledger,
                format!("failed to render artifact manifest row: {error}"),
            )
        })?;
    }
    Ok(rendered)
}
