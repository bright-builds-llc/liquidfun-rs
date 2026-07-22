use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use liquidfun_test_protocol::{
    CatalogRunRequest, EvidenceTier, HarnessLimits, RequestId, ResolveRequest,
    RunProvenanceRequirements, Sha256Hex, decode_resolved_scenario,
    encode_catalog_run_request_jsonl, resolve_catalog, reviewed_scenario_catalog,
};
use serde::Deserialize;

use super::{
    CASE_IDS, FORBIDDEN_PARTS, MAPPINGS, PHASE6_SHA256, PHASE7_SHA256, UPSTREAM_REVISION,
    model::{
        CaseBinding, CasePayload, CaseSemantic, CorpusManifest, InheritedProof, ResolvedSource,
        RunBinding, RunContract, ScenarioMapping, ScenarioMappings,
    },
};
use crate::phase11_evidence::{
    Phase11EvidenceError,
    paths::{MAX_JSON_BYTES, canonical_sha256, read_regular, require_sha256, sha256},
};

pub(super) fn validate_manifest(
    repository_root: &Path,
    artifact_root: &Path,
    manifest: &CorpusManifest,
) -> Result<(), Phase11EvidenceError> {
    if manifest.schema_version != 1
        || manifest.profile != "phase11-v1"
        || manifest.upstream_revision != UPSTREAM_REVISION
        || manifest.catalog.path != "reference/scenario-catalog.json"
        || manifest.mapping.path != MAPPINGS
        || manifest.payloads.len() != CASE_IDS.len()
        || manifest.cases.len() != CASE_IDS.len()
    {
        return Err(Phase11EvidenceError::new(
            "manifest",
            "corpus schema, provenance, or cardinality is stale",
        ));
    }
    require_sha256(
        "catalog",
        &manifest.catalog.sha256,
        &sha256(&read_regular(
            &repository_root.join(&manifest.catalog.path),
            "catalog",
            MAX_JSON_BYTES,
        )?),
    )?;
    require_sha256(
        "mapping",
        &manifest.mapping.sha256,
        &sha256(&read_regular(
            &repository_root.join(&manifest.mapping.path),
            "mapping",
            MAX_JSON_BYTES,
        )?),
    )?;
    let expected_cases = CASE_IDS.into_iter().collect::<BTreeSet<_>>();
    let actual_cases = manifest
        .payloads
        .iter()
        .map(|payload| payload.case_id.as_str())
        .collect::<BTreeSet<_>>();
    if actual_cases != expected_cases {
        return Err(Phase11EvidenceError::new(
            "manifest",
            "payload inventory is omitted, duplicated, or unknown",
        ));
    }
    for payload in &manifest.payloads {
        let filename = payload_filename(&payload.path)?;
        let bytes = read_regular(
            &artifact_root.join("cases").join(filename),
            "payload",
            MAX_JSON_BYTES,
        )?;
        require_sha256("payload", &payload.sha256, &sha256(&bytes))?;
        let binding = unique(
            manifest
                .cases
                .iter()
                .filter(|binding| binding.case_id == payload.case_id),
            "case binding",
        )?;
        if binding.payload_path != payload.path || binding.payload_sha256 != payload.sha256 {
            return Err(Phase11EvidenceError::new(
                "manifest",
                "case binding and payload digest disagree",
            ));
        }
    }
    validate_inherited(repository_root, &manifest.inherited_proofs)
}

fn validate_inherited(root: &Path, proofs: &[InheritedProof]) -> Result<(), Phase11EvidenceError> {
    let expected = (6_u32..=10)
        .map(|phase| format!("phase{phase}"))
        .collect::<BTreeSet<_>>();
    let actual = proofs
        .iter()
        .map(|proof| proof.proof_id.clone())
        .collect::<BTreeSet<_>>();
    if proofs.len() != 5 || actual != expected {
        return Err(Phase11EvidenceError::new(
            "inherited",
            "Phase 6-10 inherited evidence inventory is incomplete",
        ));
    }
    for proof in proofs {
        if proof.phase < 6
            || proof.phase > 10
            || proof.proof_id != format!("phase{}", proof.phase)
            || proof.path.contains("phase11")
        {
            return Err(Phase11EvidenceError::new(
                "inherited",
                "inherited evidence is circular or outside Phase 6-10",
            ));
        }
        require_sha256(
            "inherited evidence",
            &proof.sha256,
            &sha256(&read_regular(
                &root.join(&proof.path),
                "inherited evidence",
                MAX_JSON_BYTES,
            )?),
        )?;
    }
    Ok(())
}

pub(super) fn validate_mapping_authority(
    root: &Path,
    manifest: &CorpusManifest,
    mappings: &ScenarioMappings,
) -> Result<(), Phase11EvidenceError> {
    if mappings.schema_version != 1
        || mappings.catalog_schema_version != 1
        || mappings.record_count != 43
        || mappings.records.len() != 43
    {
        return Err(Phase11EvidenceError::new(
            "mapping",
            "mapping schema or exact 43-row count is stale",
        ));
    }
    require_sha256(
        "mapping catalog",
        &mappings.catalog_sha256,
        &sha256(&read_regular(
            &root.join(&manifest.catalog.path),
            "catalog",
            MAX_JSON_BYTES,
        )?),
    )?;
    let catalog = reviewed_scenario_catalog()
        .map_err(|error| Phase11EvidenceError::new("mapping", error.to_string()))?;
    let mut previous = None;
    for mapping in &mappings.records {
        let identity = (mapping.slug.as_str(), mapping.scenario_version);
        if previous.is_some_and(|candidate| candidate >= identity)
            || mapping.test_ids.is_empty()
            || mapping.evidence.references.is_empty()
            || ![&mapping.regression, &mapping.benchmark, &mapping.visual]
                .iter()
                .all(|consumer| consumer.mapping_id.ends_with(&mapping.slug))
        {
            return Err(Phase11EvidenceError::new(
                "mapping",
                "mapping identities, references, or consumers are open",
            ));
        }
        previous = Some(identity);
        let definition = catalog
            .definitions()
            .iter()
            .find(|definition| {
                definition.slug().as_str() == mapping.slug
                    && definition.scenario_version().get() == mapping.scenario_version
            })
            .ok_or_else(|| Phase11EvidenceError::new("mapping", "unknown scenario mapping"))?;
        let metadata = definition
            .metadata()
            .ok_or_else(|| Phase11EvidenceError::new("mapping", "mapping lacks metadata"))?;
        let resolved = resolve_catalog(
            catalog.definitions(),
            &ResolveRequest::new(definition.slug().clone(), None, metadata.default_settings()),
        )
        .map_err(|error| Phase11EvidenceError::new("mapping", error.to_string()))?;
        if resolved.identity().content_sha256().as_str() != mapping.scenario_sha256 {
            return Err(Phase11EvidenceError::new(
                "mapping",
                "mapping scenario hash is stale",
            ));
        }
    }
    Ok(())
}

pub(super) fn evaluate_cases(
    repository_root: &Path,
    artifact_root: &Path,
    manifest: &CorpusManifest,
    mappings: &ScenarioMappings,
) -> Result<Vec<CaseSemantic>, Phase11EvidenceError> {
    let mapping_index = mappings
        .records
        .iter()
        .map(|mapping| (mapping.slug.as_str(), mapping))
        .collect::<BTreeMap<_, _>>();
    CASE_IDS
        .iter()
        .map(|case_id| {
            let binding = unique(
                manifest
                    .cases
                    .iter()
                    .filter(|binding| binding.case_id == *case_id),
                "case binding",
            )?;
            let digest = unique(
                manifest
                    .payloads
                    .iter()
                    .filter(|payload| payload.case_id == *case_id),
                "payload digest",
            )?;
            let bytes = read_regular(
                &artifact_root
                    .join("cases")
                    .join(payload_filename(&digest.path)?),
                "payload",
                MAX_JSON_BYTES,
            )?;
            let mut deserializer = serde_json::Deserializer::from_slice(&bytes);
            let payload = CasePayload::deserialize(&mut deserializer)
                .map_err(|error| Phase11EvidenceError::new("payload", error.to_string()))?;
            deserializer
                .end()
                .map_err(|error| Phase11EvidenceError::new("payload", error.to_string()))?;
            evaluate_case(repository_root, binding, &payload, &mapping_index, manifest)
        })
        .collect()
}

fn evaluate_case(
    root: &Path,
    binding: &CaseBinding,
    payload: &CasePayload,
    mappings: &BTreeMap<&str, &ScenarioMapping>,
    manifest: &CorpusManifest,
) -> Result<CaseSemantic, Phase11EvidenceError> {
    if payload.schema_version != 1
        || payload.case_id != binding.case_id
        || binding.families.is_empty()
        || !binding.eligibility.regression
        || !binding.eligibility.benchmark
        || !binding.eligibility.visual
        || binding.inherited_proof_ids.len() != 5
        || payload.runs.is_empty()
        || payload.observation_leaves.is_empty()
        || payload.primitive_leaves.is_empty()
        || payload.numeric_policies.is_empty()
    {
        return Err(Phase11EvidenceError::new(
            "case",
            "case identity, eligibility, inherited proof, or leaf coverage is partial",
        ));
    }
    validate_leaves(&payload.observation_leaves, "observation")?;
    validate_leaves(&payload.primitive_leaves, "primitive")?;
    for policy in &payload.numeric_policies {
        serde_json::from_value::<liquidfun_test_protocol::MathProbePolicyPath>(
            serde_json::Value::String(policy.clone()),
        )
        .map_err(|_| Phase11EvidenceError::new("policy", "unknown numeric policy"))?;
    }

    let mut resolved = Vec::new();
    let mut requests = Vec::new();
    let mut checkpoints = Vec::new();
    let mut comparisons = Vec::new();
    let mut case_mappings = Vec::new();
    for run in &payload.runs {
        validate_run(root, run)?;
        let mapping = mappings
            .get(run.slug.as_str())
            .ok_or_else(|| Phase11EvidenceError::new("mapping", "case run is unmapped"))?;
        resolved.push(run.resolved_sha256.as_str());
        requests.push(run.request_sha256.as_str());
        checkpoints.push(&run.checkpoint_ids);
        comparisons.push(run.run_sha256.as_str());
        case_mappings.push(*mapping);
    }
    Ok(CaseSemantic {
        case_id: payload.case_id.clone(),
        resolved_sha256: canonical_sha256(&resolved)?,
        request_sha256: canonical_sha256(&requests)?,
        checkpoint_sha256: canonical_sha256(&checkpoints)?,
        comparison_sha256: canonical_sha256(&comparisons)?,
        mapping_sha256: canonical_sha256(&case_mappings)?,
        policy_sha256: canonical_sha256(&payload.numeric_policies)?,
        inherited_evidence_sha256: canonical_sha256(&manifest.inherited_proofs)?,
        observation_leaves: payload.observation_leaves.clone(),
        primitive_leaves: payload.primitive_leaves.clone(),
        numeric_policies: payload.numeric_policies.clone(),
    })
}

fn validate_run(root: &Path, run: &RunBinding) -> Result<(), Phase11EvidenceError> {
    let bytes = match &run.resolved_source {
        ResolvedSource::Path { path } => {
            read_regular(&root.join(path), "resolved", MAX_JSON_BYTES)?
        }
        ResolvedSource::Embedded { bytes } if bytes.len() <= 1_048_576 => bytes.clone(),
        ResolvedSource::Embedded { .. } => {
            return Err(Phase11EvidenceError::new(
                "resolved",
                "embedded resolved bytes exceed bound",
            ));
        }
    };
    require_sha256("resolved", &run.resolved_sha256, &sha256(&bytes))?;
    let resolved_sha = Sha256Hex::new(run.resolved_sha256.clone())
        .map_err(|error| Phase11EvidenceError::new("resolved", error.to_string()))?;
    let resolved = decode_resolved_scenario(&bytes, &resolved_sha)
        .map_err(|error| Phase11EvidenceError::new("resolved", error.to_string()))?;
    if resolved.identity().slug().as_str() != run.slug
        || resolved.identity().scenario_version().get() != run.scenario_version
        || resolved
            .actions()
            .iter()
            .map(|action| action.action_id().as_str())
            .ne(run.action_ids.iter().map(String::as_str))
        || resolved
            .checkpoints()
            .iter()
            .map(|checkpoint| checkpoint.checkpoint_id().as_str())
            .ne(run.checkpoint_ids.iter().map(String::as_str))
    {
        return Err(Phase11EvidenceError::new(
            "resolved",
            "resolved identity or action/checkpoint schedule is stale",
        ));
    }
    let request = CatalogRunRequest::new(
        RequestId::new(run.request_id.clone())
            .map_err(|error| Phase11EvidenceError::new("request", error.to_string()))?,
        resolved,
        RunProvenanceRequirements::new(
            Sha256Hex::new(PHASE6_SHA256.to_owned())
                .map_err(|error| Phase11EvidenceError::new("request", error.to_string()))?,
            Sha256Hex::new(PHASE7_SHA256.to_owned())
                .map_err(|error| Phase11EvidenceError::new("request", error.to_string()))?,
            EvidenceTier::D0Replay,
        ),
    )
    .map_err(|error| Phase11EvidenceError::new("request", error.to_string()))?;
    let request_bytes =
        encode_catalog_run_request_jsonl(&request, &HarnessLimits::phase2_default_v1())
            .map_err(|error| Phase11EvidenceError::new("request", error.to_string()))?;
    require_sha256("request", &run.request_sha256, &sha256(&request_bytes))?;
    let contract = RunContract {
        resolved_sha256: &run.resolved_sha256,
        request_sha256: &run.request_sha256,
        action_ids: &run.action_ids,
        checkpoint_ids: &run.checkpoint_ids,
    };
    require_sha256(
        "comparison contract",
        &run.run_sha256,
        &canonical_sha256(&contract)?,
    )
}

fn validate_leaves(leaves: &[String], label: &'static str) -> Result<(), Phase11EvidenceError> {
    let unique = leaves.iter().collect::<BTreeSet<_>>();
    if unique.len() != leaves.len()
        || leaves.len() > 32
        || leaves.iter().any(|leaf| {
            leaf.is_empty()
                || FORBIDDEN_PARTS
                    .iter()
                    .any(|forbidden| leaf.contains(forbidden))
        })
    {
        return Err(Phase11EvidenceError::new(
            "leaves",
            format!("{label} leaves are omitted, duplicated, private, or diagnostic-only"),
        ));
    }
    Ok(())
}

#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "the closed artifact topology requires canonical lowercase .jsonl names"
)]
pub(super) fn payload_filename(path: &str) -> Result<&str, Phase11EvidenceError> {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| value.ends_with(".jsonl"))
        .ok_or_else(|| Phase11EvidenceError::new("path", "invalid payload filename"))
}

fn unique<'a, T>(
    mut values: impl Iterator<Item = &'a T>,
    label: &'static str,
) -> Result<&'a T, Phase11EvidenceError> {
    let first = values
        .next()
        .ok_or_else(|| Phase11EvidenceError::new("manifest", format!("missing {label}")))?;
    if values.next().is_some() {
        return Err(Phase11EvidenceError::new(
            "manifest",
            format!("duplicate {label}"),
        ));
    }
    Ok(first)
}
