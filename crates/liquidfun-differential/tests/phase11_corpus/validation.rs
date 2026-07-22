use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use super::corpus_io::{exact_directory_files, parse_json, read_regular, sha256, verify_digest};
use super::model::{
    CaseBinding, CasePayload, CorpusManifest, ResolvedSource, RunBinding, RunContract,
    ScenarioMapping, ScenarioMappings,
};
use liquidfun_test_protocol::{
    CatalogRunRequest, EvidenceTier, HarnessLimits, RequestId, ResolveRequest,
    RunProvenanceRequirements, Sha256Hex, decode_resolved_scenario,
    encode_catalog_run_request_jsonl, resolve_catalog, reviewed_scenario_catalog,
};
use serde_json::Value;

pub(super) const MANIFEST: &str =
    "crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json";
pub(super) const MAPPINGS: &str = "reference/artifacts/phase11/scenario-mappings.json";
pub(super) const EXPECTED_MAPPING_COUNT: usize = 43;
pub(super) const PHASE6_SHA256: &str =
    "7f10df148852866fd20d11b8d27adcddc0ad463ac3d3d716a8946ca5c8f1c63a";
pub(super) const PHASE7_SHA256: &str =
    "fd772b2cf523a6d40bf978bc4d0da18a4564181a93e6b2bdeb8e4d40d5613311";

const UPSTREAM_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const REQUIRED_PROOFS: [&str; 5] = ["phase6", "phase7", "phase8", "phase9", "phase10"];
const CASES: [(&str, &[&str]); 3] = [
    ("rigid-joint-rope", &["joint", "rigid", "rope"]),
    ("particle-groups", &["group", "particle"]),
    (
        "queries-callbacks-mutations",
        &["callback", "mutation", "query"],
    ),
];
const CASE_RUNS: [(&str, &[&str]); 3] = [
    (
        "rigid-joint-rope",
        &[
            "rigid-stack-stability",
            "joint-rope-behavior",
            "standalone-rope-evolution",
        ],
    ),
    (
        "particle-groups",
        &[
            "particle-forces-and-statistics",
            "particle-group-construction-append",
        ],
    ),
    (
        "queries-callbacks-mutations",
        &[
            "particle-aabb-query-controls",
            "particle-lifecycle-callbacks",
            "particle-mutations",
        ],
    ),
];
const FORBIDDEN_LEAF_PARTS: [&str; 8] = [
    "pixel",
    "frame_rate",
    "framerate",
    "duration",
    "pass_id",
    "private",
    "render_order",
    "renderer_order",
];

pub(super) struct LoadedCorpus {
    pub(super) manifest: CorpusManifest,
    pub(super) mappings: ScenarioMappings,
    pub(super) payloads: Vec<CasePayload>,
}

pub(super) fn load(root: &Path) -> Result<LoadedCorpus, String> {
    let manifest = parse_json::<CorpusManifest>(&read_regular(root, MANIFEST)?)?;
    let mappings = parse_json::<ScenarioMappings>(&read_regular(root, MAPPINGS)?)?;
    let payloads = manifest
        .payloads
        .iter()
        .map(|payload| read_regular(root, &payload.path).and_then(|bytes| parse_json(&bytes)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(LoadedCorpus {
        manifest,
        mappings,
        payloads,
    })
}

pub(super) fn validate(root: &Path, loaded: &LoadedCorpus) -> Result<(), String> {
    validate_manifest(root, loaded)?;
    validate_mappings(root, &loaded.mappings)?;
    validate_payloads(root, loaded)?;
    Ok(())
}

fn validate_manifest(root: &Path, loaded: &LoadedCorpus) -> Result<(), String> {
    let manifest = &loaded.manifest;
    if manifest.schema_version != 1
        || manifest.profile != "phase11-v1"
        || manifest.upstream_revision != UPSTREAM_REVISION
        || manifest.catalog.path != "reference/scenario-catalog.json"
        || manifest.mapping.path != MAPPINGS
    {
        return Err("manifest identity is stale".to_owned());
    }
    verify_digest(root, &manifest.catalog.path, &manifest.catalog.sha256)?;
    verify_digest(root, &manifest.mapping.path, &manifest.mapping.sha256)?;

    let expected_payloads = CASES
        .iter()
        .map(|(case_id, _families)| {
            format!("crates/liquidfun-differential/tests/fixtures/catalog/cases/{case_id}.jsonl")
        })
        .collect::<BTreeSet<_>>();
    let actual_payloads = manifest
        .payloads
        .iter()
        .map(|payload| payload.path.clone())
        .collect::<BTreeSet<_>>();
    if actual_payloads != expected_payloads || manifest.payloads.len() != expected_payloads.len() {
        return Err("payload allowlist is not exact".to_owned());
    }
    for payload in &manifest.payloads {
        verify_digest(root, &payload.path, &payload.sha256)?;
    }
    exact_directory_files(
        &root.join("crates/liquidfun-differential/tests/fixtures/catalog/cases"),
        &expected_payloads
            .iter()
            .filter_map(|path| Path::new(path).file_name()?.to_str().map(str::to_owned))
            .collect(),
    )?;
    exact_directory_files(
        &root.join("reference/artifacts/phase11"),
        &BTreeSet::from([
            "exact-ref.json".to_owned(),
            "scenario-mappings.json".to_owned(),
        ]),
    )?;

    let proof_ids = manifest
        .inherited_proofs
        .iter()
        .map(|proof| proof.proof_id.as_str())
        .collect::<BTreeSet<_>>();
    if proof_ids != BTreeSet::from(REQUIRED_PROOFS) || manifest.inherited_proofs.len() != 5 {
        return Err("inherited proof inventory is incomplete".to_owned());
    }
    let mut protected_paths = actual_payloads;
    protected_paths.insert(MANIFEST.to_owned());
    protected_paths.insert(MAPPINGS.to_owned());
    for proof in &manifest.inherited_proofs {
        if proof.phase < 6
            || proof.phase > 10
            || proof.proof_id != format!("phase{}", proof.phase)
            || protected_paths.contains(&proof.path)
            || proof.path.contains("phase11")
        {
            return Err("inherited proof is circular or outside Phase 6-10".to_owned());
        }
        verify_digest(root, &proof.path, &proof.sha256)?;
    }
    Ok(())
}

fn validate_mappings(root: &Path, mappings: &ScenarioMappings) -> Result<(), String> {
    if mappings.schema_version != 1
        || mappings.catalog_schema_version != 1
        || mappings.record_count != EXPECTED_MAPPING_COUNT
        || mappings.records.len() != EXPECTED_MAPPING_COUNT
    {
        return Err("mapping count or schema is stale".to_owned());
    }
    verify_digest(
        root,
        "reference/scenario-catalog.json",
        &mappings.catalog_sha256,
    )?;
    let projection: Value = parse_json(&read_regular(root, "reference/scenario-catalog.json")?)?;
    let scenarios = projection["scenarios"]
        .as_array()
        .ok_or_else(|| "catalog projection lacks scenarios".to_owned())?;
    if scenarios.len() != EXPECTED_MAPPING_COUNT {
        return Err("catalog projection count is stale".to_owned());
    }
    let catalog = reviewed_scenario_catalog().map_err(|error| error.to_string())?;
    if catalog.definitions().len() != EXPECTED_MAPPING_COUNT {
        return Err("live registry count is stale".to_owned());
    }

    let projected = scenarios
        .iter()
        .map(|scenario| {
            scenario["slug"]
                .as_str()
                .map(|slug| (slug, scenario))
                .ok_or_else(|| "catalog projection has an invalid slug".to_owned())
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut previous = None;
    let mut identities = BTreeSet::new();
    for mapping in &mappings.records {
        let identity = (mapping.slug.as_str(), mapping.scenario_version);
        if previous.is_some_and(|candidate| candidate >= identity) || !identities.insert(identity) {
            return Err("mapping identities are duplicate or unordered".to_owned());
        }
        previous = Some(identity);
        let expected = projected
            .get(mapping.slug.as_str())
            .ok_or_else(|| "mapping names an unknown scenario".to_owned())?;
        validate_mapping_projection(mapping, expected)?;
        let definition = catalog
            .definitions()
            .iter()
            .find(|definition| {
                definition.slug().as_str() == mapping.slug
                    && definition.scenario_version().get() == mapping.scenario_version
            })
            .ok_or_else(|| "mapping does not resolve in the live registry".to_owned())?;
        let metadata = definition
            .metadata()
            .ok_or_else(|| "live definition lacks reviewed metadata".to_owned())?;
        let resolved = resolve_catalog(
            catalog.definitions(),
            &ResolveRequest::new(definition.slug().clone(), None, metadata.default_settings()),
        )
        .map_err(|error| error.to_string())?;
        if resolved.identity().content_sha256().as_str() != mapping.scenario_sha256 {
            return Err("mapping scenario hash is stale".to_owned());
        }
    }
    Ok(())
}

fn validate_mapping_projection(mapping: &ScenarioMapping, expected: &Value) -> Result<(), String> {
    let expected_tests = strings(&expected["test_ids"])?;
    let expected_refs = strings(&expected["evidence"]["references"])?;
    let expected_disposition = expected["evidence"]["disposition"]
        .as_str()
        .ok_or_else(|| "projected evidence disposition is absent".to_owned())?;
    if mapping.test_ids.is_empty()
        || mapping.test_ids != expected_tests
        || mapping.evidence.references.is_empty()
        || mapping.evidence.references != expected_refs
        || mapping.evidence.disposition != expected_disposition
    {
        return Err("mapping test or evidence join is stale".to_owned());
    }
    for (name, consumer, field) in [
        ("regression", &mapping.regression, "regression_use"),
        ("benchmark", &mapping.benchmark, "benchmark_eligible"),
        ("visual", &mapping.visual, "visual_eligible"),
    ] {
        if consumer.eligible != expected[field].as_bool().unwrap_or(false)
            || consumer.mapping_id != format!("{name}/{}", mapping.slug)
        {
            return Err("mapping eligibility is contradictory".to_owned());
        }
    }
    Ok(())
}

fn validate_payloads(root: &Path, loaded: &LoadedCorpus) -> Result<(), String> {
    if loaded.payloads.len() != CASES.len() || loaded.manifest.cases.len() != CASES.len() {
        return Err("representative case count is not exact".to_owned());
    }
    let mappings = loaded
        .mappings
        .records
        .iter()
        .map(|mapping| (mapping.slug.as_str(), mapping))
        .collect::<BTreeMap<_, _>>();
    let mut all_slugs = BTreeSet::new();
    let mut all_request_ids = BTreeSet::new();
    let mut all_leaves = BTreeSet::new();
    for (expected_case, expected_families) in CASES {
        let case = loaded
            .manifest
            .cases
            .iter()
            .find(|case| case.case_id == expected_case)
            .ok_or_else(|| "representative case binding is missing".to_owned())?;
        let payload = loaded
            .payloads
            .iter()
            .find(|payload| payload.case_id == expected_case)
            .ok_or_else(|| "representative payload is missing".to_owned())?;
        validate_case(
            root,
            expected_case,
            expected_families,
            case,
            payload,
            &mappings,
            &mut all_slugs,
            &mut all_request_ids,
            &mut all_leaves,
        )?;
    }
    Ok(())
}

#[allow(
    clippy::too_many_arguments,
    reason = "the closed corpus namespaces are independently tracked across all three cases"
)]
fn validate_case(
    root: &Path,
    expected_case: &str,
    expected_families: &[&str],
    case: &CaseBinding,
    payload: &CasePayload,
    mappings: &BTreeMap<&str, &ScenarioMapping>,
    all_slugs: &mut BTreeSet<String>,
    all_request_ids: &mut BTreeSet<String>,
    all_leaves: &mut BTreeSet<String>,
) -> Result<(), String> {
    if case.families.iter().map(String::as_str).collect::<Vec<_>>() != expected_families
        || case.payload_path
            != format!(
                "crates/liquidfun-differential/tests/fixtures/catalog/cases/{expected_case}.jsonl"
            )
        || case.payload_sha256 != sha256(&read_regular(root, &case.payload_path)?)
        || payload.schema_version != 1
        || payload.runs.is_empty()
        || !case.eligibility.regression
        || !case.eligibility.benchmark
        || !case.eligibility.visual
    {
        return Err("representative case identity or eligibility is stale".to_owned());
    }
    let proof_ids = case
        .inherited_proof_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if proof_ids != BTreeSet::from(REQUIRED_PROOFS) || case.inherited_proof_ids.len() != 5 {
        return Err("case omits inherited Phase 6-10 proof".to_owned());
    }
    for leaf in payload
        .observation_leaves
        .iter()
        .chain(&payload.primitive_leaves)
    {
        if FORBIDDEN_LEAF_PARTS
            .iter()
            .any(|forbidden| leaf.contains(forbidden))
            || !all_leaves.insert(leaf.clone())
        {
            return Err("semantic leaf is forbidden or duplicated".to_owned());
        }
    }
    if payload.observation_leaves.is_empty()
        || payload.primitive_leaves.is_empty()
        || payload.numeric_policies.is_empty()
        || payload.runs.len() > 8
        || payload.observation_leaves.len() > 32
        || payload.primitive_leaves.len() > 32
        || payload.numeric_policies.len() > 16
    {
        return Err("case omits semantic leaves or numeric policies".to_owned());
    }
    let expected_runs = CASE_RUNS
        .iter()
        .find_map(|(case_id, runs)| (*case_id == expected_case).then_some(*runs))
        .ok_or_else(|| "representative run inventory is absent".to_owned())?;
    if payload
        .runs
        .iter()
        .map(|run| run.slug.as_str())
        .ne(expected_runs.iter().copied())
    {
        return Err("representative run inventory is stale".to_owned());
    }
    for policy in &payload.numeric_policies {
        serde_json::from_value::<liquidfun_test_protocol::MathProbePolicyPath>(Value::String(
            policy.clone(),
        ))
        .map_err(|_| "case names an open numeric policy".to_owned())?;
    }
    for run in &payload.runs {
        if !all_slugs.insert(run.slug.clone()) || !all_request_ids.insert(run.request_id.clone()) {
            return Err("representative run slug or request identity is duplicated".to_owned());
        }
        let mapping = mappings
            .get(run.slug.as_str())
            .ok_or_else(|| "representative run is not mapped".to_owned())?;
        if mapping.regression.eligible != case.eligibility.regression
            || mapping.benchmark.eligible != case.eligibility.benchmark
            || mapping.visual.eligible != case.eligibility.visual
        {
            return Err("case and scenario eligibility disagree".to_owned());
        }
        validate_run(root, run)?;
    }
    Ok(())
}

fn validate_run(root: &Path, run: &RunBinding) -> Result<(), String> {
    let bytes = match &run.resolved_source {
        ResolvedSource::Path { path } => read_regular(root, path)?,
        ResolvedSource::Embedded { bytes } if bytes.len() <= 1_048_576 => bytes.clone(),
        ResolvedSource::Embedded { .. } => {
            return Err("embedded resolved bytes exceed limit".into());
        }
    };
    if sha256(&bytes) != run.resolved_sha256 {
        return Err("resolved bytes hash is stale".to_owned());
    }
    let resolved_sha = Sha256Hex::new(run.resolved_sha256.clone()).map_err(|e| e.to_string())?;
    let resolved = decode_resolved_scenario(&bytes, &resolved_sha).map_err(|e| e.to_string())?;
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
        || run.action_ids.is_empty()
        || run.checkpoint_ids.is_empty()
    {
        return Err("resolved identity or schedule is stale".to_owned());
    }
    let request = CatalogRunRequest::new(
        RequestId::new(run.request_id.clone()).map_err(|e| e.to_string())?,
        resolved,
        RunProvenanceRequirements::new(
            Sha256Hex::new(PHASE6_SHA256.to_owned()).map_err(|e| e.to_string())?,
            Sha256Hex::new(PHASE7_SHA256.to_owned()).map_err(|e| e.to_string())?,
            EvidenceTier::D0Replay,
        ),
    )
    .map_err(|e| e.to_string())?;
    let request_bytes =
        encode_catalog_run_request_jsonl(&request, &HarnessLimits::phase2_default_v1())
            .map_err(|e| e.to_string())?;
    if sha256(&request_bytes) != run.request_sha256 {
        return Err("request hash is stale".to_owned());
    }
    let contract = RunContract {
        resolved_sha256: &run.resolved_sha256,
        request_sha256: &run.request_sha256,
        action_ids: &run.action_ids,
        checkpoint_ids: &run.checkpoint_ids,
    };
    if sha256(&serde_json::to_vec(&contract).map_err(|e| e.to_string())?) != run.run_sha256 {
        return Err("run hash is stale".to_owned());
    }
    Ok(())
}

fn strings(value: &Value) -> Result<Vec<String>, String> {
    value
        .as_array()
        .ok_or_else(|| "projected mapping list is invalid".to_owned())?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| "projected mapping identity is invalid".to_owned())
        })
        .collect()
}
