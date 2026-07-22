use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CorpusManifest {
    pub(super) schema_version: u32,
    pub(super) profile: String,
    pub(super) upstream_revision: String,
    pub(super) catalog: FileDigest,
    pub(super) mapping: FileDigest,
    pub(super) payloads: Vec<PayloadDigest>,
    pub(super) inherited_proofs: Vec<InheritedProof>,
    pub(super) cases: Vec<CaseBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FileDigest {
    pub(super) path: String,
    pub(super) sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PayloadDigest {
    pub(super) case_id: String,
    pub(super) path: String,
    pub(super) sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct InheritedProof {
    pub(super) proof_id: String,
    pub(super) phase: u32,
    pub(super) path: String,
    pub(super) sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CaseBinding {
    pub(super) case_id: String,
    pub(super) families: Vec<String>,
    pub(super) payload_path: String,
    pub(super) payload_sha256: String,
    pub(super) inherited_proof_ids: Vec<String>,
    pub(super) eligibility: Eligibility,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct Eligibility {
    pub(super) regression: bool,
    pub(super) benchmark: bool,
    pub(super) visual: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CasePayload {
    pub(super) schema_version: u32,
    pub(super) case_id: String,
    pub(super) runs: Vec<RunBinding>,
    pub(super) observation_leaves: Vec<String>,
    pub(super) primitive_leaves: Vec<String>,
    pub(super) numeric_policies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RunBinding {
    pub(super) slug: String,
    pub(super) scenario_version: u32,
    pub(super) resolved_source: ResolvedSource,
    pub(super) resolved_sha256: String,
    pub(super) request_id: String,
    pub(super) request_sha256: String,
    pub(super) run_sha256: String,
    pub(super) action_ids: Vec<String>,
    pub(super) checkpoint_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum ResolvedSource {
    Path { path: String },
    Embedded { bytes: Vec<u8> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ScenarioMappings {
    pub(super) schema_version: u32,
    pub(super) catalog_schema_version: u32,
    pub(super) catalog_sha256: String,
    pub(super) record_count: usize,
    pub(super) records: Vec<ScenarioMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ScenarioMapping {
    pub(super) slug: String,
    pub(super) scenario_version: u32,
    pub(super) scenario_sha256: String,
    pub(super) test_ids: Vec<String>,
    pub(super) evidence: MappingEvidence,
    pub(super) regression: ConsumerMapping,
    pub(super) benchmark: ConsumerMapping,
    pub(super) visual: ConsumerMapping,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct MappingEvidence {
    pub(super) disposition: String,
    pub(super) references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ConsumerMapping {
    pub(super) eligible: bool,
    pub(super) mapping_id: String,
}

#[derive(Serialize)]
pub(super) struct RunContract<'a> {
    pub(super) resolved_sha256: &'a str,
    pub(super) request_sha256: &'a str,
    pub(super) action_ids: &'a [String],
    pub(super) checkpoint_ids: &'a [String],
}
