//! Private wire schemas joined by the corpus closure validator.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ScenarioCatalog {
    pub(super) schema_version: u32,
    pub(super) sort_contract: String,
    pub(super) description: String,
    pub(super) projection_sha256: String,
    pub(super) scenarios: Vec<ScenarioRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ScenarioRecord {
    pub(super) slug: String,
    pub(super) scenario_version: u32,
    pub(super) display_title: String,
    pub(super) generator_id: String,
    pub(super) generator_version: u32,
    pub(super) seed_policy: String,
    pub(super) test_ids: Vec<String>,
    pub(super) evidence: MappingEvidence,
    pub(super) regression_use: bool,
    pub(super) benchmark_eligible: bool,
    pub(super) visual_eligible: bool,
    pub(super) upstream_corpus_ids: Vec<String>,
    pub(super) compatibility_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ScenarioMappings {
    pub(super) schema_version: u32,
    pub(super) catalog_schema_version: u32,
    pub(super) catalog_sha256: String,
    pub(super) record_count: usize,
    pub(super) records: Vec<ScenarioMapping>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct MappingEvidence {
    pub(super) disposition: String,
    pub(super) references: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ConsumerMapping {
    pub(super) eligible: bool,
    pub(super) mapping_id: String,
}
