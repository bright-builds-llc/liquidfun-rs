use super::closure::{
    changed_path_set_sha256, derive_replay_closure, derive_witness_closure,
    promoted_path_set_sha256, receipt_semantic_sha256, reviewed_content_digests_from_root,
    valid_path_classification,
};
use super::execution::{git_text, is_ancestor, run_process};
use super::{
    ARTIFACT_MANIFEST_PATH, AcceptanceError, AcceptanceErrorKind, BTreeMap, BTreeSet, Command,
    Deserialize, EXACT_BYTES_DIGEST_MODE, IdentityContract, ORACLE_REVISION, PROMOTED_PATHS, Path,
    RECEIPT_PATH, RECEIPT_SEMANTIC_DIGEST_MODE, REPLAY_EVIDENCE_PATH, SOURCE_MAP_PATH, Serialize,
    WITNESS_PROVENANCE_PATH, file_sha256, fs, read_json, valid_digest, valid_revision,
};

pub(super) struct LoadedIdentity {
    pub(super) contract: IdentityContract,
    pub(super) upstream_revision: String,
    pub(super) oracle_build_identity_sha256: String,
    pub(super) reviewed_evidence_sha256: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Receipt {
    schema_version: u32,
    producer_sha: String,
    bundle_sha256: String,
    promotion_base_sha: String,
    acquisition: Acquisition,
    independent_reviewer_id: String,
    promoted_paths: Vec<String>,
    promoted_path_set_sha256: String,
    pub(super) promoted_content_sha256: String,
    changed_paths: Vec<String>,
    unchanged_paths: Vec<String>,
    changed_path_set_sha256: String,
    pub(super) changed_content_sha256: String,
    producer_closures: ProducerClosures,
    q_contract: PromotionContract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Acquisition {
    repository: String,
    run_id: u64,
    artifact_id: u64,
    artifact_name: String,
    provider_digest: String,
    artifact_created_at: String,
    artifact_expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProducerClosures {
    witness_sha256: String,
    replay_sha256: String,
    recomputed_at_r: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PromotionContract {
    required_first_parent: String,
    required_trailers: BTreeMap<String, String>,
    q_sha_recorded: bool,
    acceptance_sha_recorded: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MaterialsManifest {
    pub(super) schema_version: u32,
    pub(super) target: String,
    pub(super) preset: String,
    pub(super) materials: Vec<Material>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Material {
    pub(super) kind: String,
    pub(super) identity: String,
}

#[derive(Debug, Deserialize)]
struct WitnessProvenance {
    repository_revision: String,
    oracle_revision: String,
    compiler_id: String,
    compiler_version: String,
    target: String,
    cmake_preset: String,
}

#[derive(Debug, Deserialize)]
struct ReplayEvidence {
    upstream_revision: String,
    d1_oracle_identity_sha256: String,
}

pub(super) fn load_identity(
    repository_root: &Path,
    acceptance_sha: &str,
) -> Result<LoadedIdentity, AcceptanceError> {
    let receipt: Receipt = read_json(&repository_root.join(RECEIPT_PATH))?;
    validate_receipt(&receipt)?;
    let (actual_promoted_content_sha256, actual_changed_content_sha256) =
        reviewed_content_digests_from_root(repository_root, &receipt.changed_paths)?;
    let reviewed_evidence_sha256 = validate_ledgers(repository_root, &receipt)?;
    let witness: WitnessProvenance = read_json(&repository_root.join(WITNESS_PROVENANCE_PATH))?;
    let replay: ReplayEvidence = read_json(&repository_root.join(REPLAY_EVIDENCE_PATH))?;
    if witness.repository_revision != receipt.producer_sha
        || witness.oracle_revision != ORACLE_REVISION
        || replay.upstream_revision != ORACLE_REVISION
        || !valid_digest(&replay.d1_oracle_identity_sha256)
        || witness.compiler_id != "Clang"
        || witness.compiler_version != "22.1.8"
        || witness.target != "x86_64-unknown-linux-gnu"
        || witness.cmake_preset != "oracle-debug"
    {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "reviewed evidence disagrees with receipt P or the pinned oracle",
        ));
    }

    let producer_is_ancestor_of_base = is_ancestor(
        repository_root,
        &receipt.producer_sha,
        &receipt.promotion_base_sha,
    )?;
    let witness_closure_at_r =
        derive_witness_closure(repository_root, &receipt.promotion_base_sha)?;
    let replay_closure_at_r = derive_replay_closure(
        repository_root,
        &receipt.promotion_base_sha,
        &receipt.promotion_base_sha,
    )?;
    let witness_closure_at_a = derive_witness_closure(repository_root, acceptance_sha)?;
    let replay_closure_at_a =
        derive_replay_closure(repository_root, acceptance_sha, &receipt.promotion_base_sha)?;
    let promotion_sha = discover_promotion(repository_root, acceptance_sha, &receipt)?;
    let promotion_first_parent = first_parent(repository_root, &promotion_sha)?;
    let actual_trailers = commit_trailers(repository_root, &promotion_sha)?;
    let actual_paths = changed_paths(repository_root, &promotion_sha)?;
    validate_q_paths_and_tree(
        repository_root,
        &receipt.promotion_base_sha,
        &promotion_sha,
        acceptance_sha,
        &actual_paths,
        &receipt.changed_paths,
        &receipt.unchanged_paths,
    )?;
    let actual_changed_path_set_sha256 = changed_path_set_sha256(&actual_paths)?;

    Ok(LoadedIdentity {
        contract: IdentityContract {
            producer_sha: receipt.producer_sha,
            bundle_sha256: receipt.bundle_sha256,
            promotion_base_sha: receipt.promotion_base_sha,
            promotion_sha: promotion_sha.clone(),
            acceptance_sha: acceptance_sha.to_owned(),
            producer_is_ancestor_of_base,
            witness_closure_at_r,
            replay_closure_at_r,
            witness_closure_at_a,
            replay_closure_at_a,
            expected_witness_closure: receipt.producer_closures.witness_sha256,
            expected_replay_closure: receipt.producer_closures.replay_sha256,
            promotion_first_parent,
            required_trailers: receipt.q_contract.required_trailers,
            actual_trailers,
            expected_promoted_path_set_sha256: receipt.promoted_path_set_sha256,
            actual_promoted_path_set_sha256: promoted_path_set_sha256(&receipt.promoted_paths)?,
            expected_promoted_content_sha256: receipt.promoted_content_sha256,
            actual_promoted_content_sha256,
            expected_changed_path_set_sha256: receipt.changed_path_set_sha256,
            actual_changed_path_set_sha256,
            expected_changed_content_sha256: receipt.changed_content_sha256,
            actual_changed_content_sha256,
            changed_paths_match: actual_paths == receipt.changed_paths,
            unchanged_paths_equal_base: true,
            all_promoted_paths_equal_at_acceptance: true,
            promotion_is_ancestor_of_acceptance: is_ancestor(
                repository_root,
                &promotion_sha,
                acceptance_sha,
            )?,
        },
        upstream_revision: replay.upstream_revision,
        oracle_build_identity_sha256: replay.d1_oracle_identity_sha256,
        reviewed_evidence_sha256,
    })
}

fn validate_receipt(receipt: &Receipt) -> Result<(), AcceptanceError> {
    let expected_paths = PROMOTED_PATHS.map(str::to_owned).to_vec();
    if receipt.schema_version != 2
        || !valid_revision(&receipt.producer_sha)
        || !valid_digest(&receipt.bundle_sha256)
        || !valid_revision(&receipt.promotion_base_sha)
        || receipt.promoted_paths != expected_paths
        || receipt.promoted_path_set_sha256 != promoted_path_set_sha256(&receipt.promoted_paths)?
        || !valid_digest(&receipt.promoted_content_sha256)
        || receipt.changed_paths.is_empty()
        || receipt.changed_path_set_sha256 != changed_path_set_sha256(&receipt.changed_paths)?
        || !valid_digest(&receipt.changed_content_sha256)
        || !valid_path_classification(&receipt.changed_paths, &receipt.unchanged_paths)
        || !receipt.producer_closures.recomputed_at_r
        || receipt.q_contract.required_first_parent != receipt.promotion_base_sha
        || receipt.q_contract.q_sha_recorded
        || receipt.q_contract.acceptance_sha_recorded
        || receipt.acquisition.repository.trim().is_empty()
        || receipt.acquisition.run_id == 0
        || receipt.acquisition.artifact_id == 0
        || receipt.acquisition.artifact_name.trim().is_empty()
        || !receipt.acquisition.provider_digest.starts_with("sha256:")
        || receipt.acquisition.artifact_created_at.trim().is_empty()
        || receipt.acquisition.artifact_expires_at.trim().is_empty()
        || receipt.independent_reviewer_id.trim().is_empty()
    {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Schema,
            "tracked receipt is incomplete, circular, or has the wrong promoted set",
        ));
    }
    Ok(())
}

fn validate_ledgers(
    repository_root: &Path,
    receipt: &Receipt,
) -> Result<BTreeMap<String, String>, AcceptanceError> {
    let manifest: toml::Value = toml::from_str(
        &fs::read_to_string(repository_root.join(ARTIFACT_MANIFEST_PATH))
            .map_err(AcceptanceError::from)?,
    )
    .map_err(|error| {
        AcceptanceError::new(
            AcceptanceErrorKind::Ledger,
            format!("invalid artifact manifest: {error}"),
        )
    })?;
    let records = manifest
        .get("artifact_schemas")
        .and_then(|value| value.get("phase13_evidence"))
        .and_then(|value| value.get("records"))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| {
            AcceptanceError::new(
                AcceptanceErrorKind::Ledger,
                "Phase 13 artifact records are absent",
            )
        })?;
    let mut digests = BTreeMap::new();
    for record in records {
        let path = toml_string(record, "path")?;
        let digest = toml_string(record, "sha256")?;
        let digest_mode = toml_string(record, "digest_mode")?;
        let producer = toml_string(record, "producer_sha")?;
        let bundle = toml_string(record, "bundle_sha256")?;
        let required_mode = if path == RECEIPT_PATH {
            RECEIPT_SEMANTIC_DIGEST_MODE
        } else {
            EXACT_BYTES_DIGEST_MODE
        };
        let actual_digest = if path == RECEIPT_PATH {
            receipt_semantic_sha256(
                &fs::read(repository_root.join(path)).map_err(AcceptanceError::from)?,
            )?
        } else {
            file_sha256(&repository_root.join(path))?
        };
        if producer != receipt.producer_sha
            || bundle != receipt.bundle_sha256
            || digest_mode != required_mode
            || actual_digest != digest
            || digests.insert(path.to_owned(), digest.to_owned()).is_some()
        {
            return Err(AcceptanceError::new(
                AcceptanceErrorKind::Ledger,
                "artifact ledger disagrees with P/B or reviewed bytes",
            ));
        }
    }
    let source_map: toml::Value = toml::from_str(
        &fs::read_to_string(repository_root.join(SOURCE_MAP_PATH))
            .map_err(AcceptanceError::from)?,
    )
    .map_err(|error| {
        AcceptanceError::new(
            AcceptanceErrorKind::Ledger,
            format!("invalid source map: {error}"),
        )
    })?;
    let mapped = source_map
        .get("mapping")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| AcceptanceError::new(AcceptanceErrorKind::Ledger, "source map is empty"))?
        .iter()
        .filter_map(|entry| entry.get("local_path").and_then(toml::Value::as_str))
        .collect::<BTreeSet<_>>();
    let expected_records = [
        "reference/artifacts/phase9/lifecycle-contact-witnesses.json",
        WITNESS_PROVENANCE_PATH,
        REPLAY_EVIDENCE_PATH,
        RECEIPT_PATH,
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    if digests.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected_records
        || !digests.keys().all(|path| mapped.contains(path.as_str()))
    {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Ledger,
            "reviewed evidence is incomplete in the artifact or FND-04 source ledger",
        ));
    }
    Ok(digests)
}

fn toml_string<'a>(value: &'a toml::Value, key: &str) -> Result<&'a str, AcceptanceError> {
    value.get(key).and_then(toml::Value::as_str).ok_or_else(|| {
        AcceptanceError::new(
            AcceptanceErrorKind::Ledger,
            format!("artifact record omitted `{key}`"),
        )
    })
}

fn discover_promotion(
    repository_root: &Path,
    acceptance_sha: &str,
    receipt: &Receipt,
) -> Result<String, AcceptanceError> {
    let revisions = git_text(repository_root, &["rev-list", acceptance_sha])?;
    let mut matches = Vec::new();
    for revision in revisions.lines() {
        if commit_trailers(repository_root, revision)? == receipt.q_contract.required_trailers {
            matches.push(revision.to_owned());
        }
    }
    if matches.len() != 1 {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "A history must contain exactly one Q with the required P/B/R trailers",
        ));
    }
    Ok(matches.remove(0))
}

fn commit_trailers(
    repository_root: &Path,
    revision: &str,
) -> Result<BTreeMap<String, String>, AcceptanceError> {
    let body = git_text(repository_root, &["show", "-s", "--format=%B", revision])?;
    let required_names = [
        "Phase13-Bundle-SHA256",
        "Phase13-Producer-SHA",
        "Phase13-Promotion-Base-SHA",
    ];
    let mut trailers = BTreeMap::new();
    for line in body.lines() {
        let Some((name, value)) = line.split_once(": ") else {
            continue;
        };
        if required_names.contains(&name)
            && trailers.insert(name.to_owned(), value.to_owned()).is_some()
        {
            return Err(AcceptanceError::new(
                AcceptanceErrorKind::Identity,
                "Q contains a duplicate Phase 13 trailer",
            ));
        }
    }
    Ok(trailers)
}

fn first_parent(repository_root: &Path, revision: &str) -> Result<String, AcceptanceError> {
    let parents = git_text(repository_root, &["show", "-s", "--format=%P", revision])?;
    let values = parents.split_whitespace().collect::<Vec<_>>();
    if values.len() != 1 {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "Q must have exactly one parent",
        ));
    }
    Ok(values[0].to_owned())
}

fn changed_paths(repository_root: &Path, revision: &str) -> Result<Vec<String>, AcceptanceError> {
    let output = git_text(
        repository_root,
        &["diff-tree", "--no-commit-id", "--name-only", "-r", revision],
    )?;
    Ok(output.lines().map(str::to_owned).collect())
}

fn validate_q_paths_and_tree(
    repository_root: &Path,
    promotion_base_sha: &str,
    promotion_sha: &str,
    acceptance_sha: &str,
    actual_paths: &[String],
    expected_changed_paths: &[String],
    unchanged_paths: &[String],
) -> Result<(), AcceptanceError> {
    let actual = actual_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected = expected_changed_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "Q Git diff does not equal the reviewed changed subset",
        ));
    }
    for path in unchanged_paths {
        let output = run_process(
            Command::new("git").arg("-C").arg(repository_root).args([
                "diff",
                "--quiet",
                promotion_base_sha,
                promotion_sha,
                "--",
                path,
            ]),
            "verify unchanged reviewed path against R",
        )?;
        if !output.status.success() {
            return Err(AcceptanceError::new(
                AcceptanceErrorKind::Identity,
                format!("unchanged reviewed path `{path}` differs between R and Q"),
            ));
        }
    }
    let output = run_process(
        Command::new("git")
            .arg("-C")
            .arg(repository_root)
            .args(["diff", "--quiet", promotion_sha, acceptance_sha, "--"])
            .args(PROMOTED_PATHS),
        "compare Q promoted bytes with A",
    )?;
    if !output.status.success() {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "A changed promoted evidence after Q",
        ));
    }
    Ok(())
}
