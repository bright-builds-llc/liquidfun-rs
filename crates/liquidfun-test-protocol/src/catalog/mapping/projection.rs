use serde::Serialize;
use sha2::{Digest, Sha256};

use super::{
    CatalogError, CatalogErrorKind, CatalogEvidence, CatalogEvidenceDisposition, CatalogMapping,
    CatalogReferenceId, ScenarioCatalog, ScenarioConsumer, ScenarioEligibility,
};
use crate::CatalogDefinition;

pub(super) fn render(catalog: &ScenarioCatalog) -> Result<Vec<u8>, CatalogError> {
    let scenarios = catalog
        .definitions()
        .iter()
        .zip(catalog.mappings())
        .map(|(definition, mapping)| projected_scenario(definition, mapping))
        .collect::<Result<Vec<_>, _>>()?;
    let scenario_bytes = serde_json::to_vec(&scenarios)
        .map_err(|_| CatalogError::new(CatalogErrorKind::CanonicalEncoding))?;
    let projection = CatalogProjection {
        schema_version: 1,
        description: "Deterministic review projection only; typed Rust definitions and mappings remain runtime authority.",
        sort_contract: "scenarios are ordered lexicographically by stable slug then scenario_version",
        projection_sha256: format!("{:x}", Sha256::digest(&scenario_bytes)),
        scenarios,
    };
    let mut rendered = serde_json::to_vec_pretty(&projection)
        .map_err(|_| CatalogError::new(CatalogErrorKind::CanonicalEncoding))?;
    rendered.push(b'\n');
    Ok(rendered)
}

#[derive(Serialize)]
struct CatalogProjection {
    schema_version: u32,
    description: &'static str,
    sort_contract: &'static str,
    projection_sha256: String,
    scenarios: Vec<ProjectedScenario>,
}

#[derive(Serialize)]
struct ProjectedScenario {
    slug: String,
    scenario_version: u32,
    display_title: String,
    generator_id: String,
    generator_version: u32,
    seed_policy: &'static str,
    test_ids: Vec<String>,
    evidence: ProjectedEvidence,
    regression_use: bool,
    benchmark_eligible: bool,
    visual_eligible: bool,
    upstream_corpus_ids: Vec<String>,
    compatibility_refs: Vec<String>,
}

#[derive(Serialize)]
#[serde(tag = "disposition", rename_all = "snake_case")]
enum ProjectedEvidence {
    Oracle { references: Vec<String> },
    ReviewedEquivalent { references: Vec<String> },
}

fn projected_scenario(
    definition: &CatalogDefinition,
    mapping: &CatalogMapping,
) -> Result<ProjectedScenario, CatalogError> {
    let evidence = match mapping.evidence_disposition() {
        CatalogEvidenceDisposition::Oracle { artifacts } => ProjectedEvidence::Oracle {
            references: strings(artifacts),
        },
        CatalogEvidenceDisposition::ReviewedEquivalent { evidence } => {
            ProjectedEvidence::ReviewedEquivalent {
                references: evidence
                    .iter()
                    .map(projected_evidence_id)
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
    };
    Ok(ProjectedScenario {
        slug: definition.slug().as_str().to_owned(),
        scenario_version: definition.scenario_version().get(),
        display_title: definition.display_title().to_owned(),
        generator_id: definition.generator_id().as_str().to_owned(),
        generator_version: definition.generator_version().get(),
        seed_policy: match definition.eligibility() {
            ScenarioEligibility::NamedOnly => "named_only",
            ScenarioEligibility::SeedRequired => "seed_required",
        },
        test_ids: mapping
            .test_ids()
            .iter()
            .map(|id| id.as_str().to_owned())
            .collect(),
        evidence,
        regression_use: mapping.regression_use(),
        benchmark_eligible: mapping.is_eligible(ScenarioConsumer::Benchmark),
        visual_eligible: mapping.is_eligible(ScenarioConsumer::Visual),
        upstream_corpus_ids: strings(mapping.upstream_corpus_ids()),
        compatibility_refs: strings(mapping.compatibility_refs()),
    })
}

fn projected_evidence_id(evidence: &CatalogEvidence) -> Result<String, CatalogError> {
    match evidence {
        CatalogEvidence::Rigid(witness) => serde_json::to_value(witness)
            .ok()
            .and_then(|value| value.as_str().map(|id| format!("rigid/{id}")))
            .ok_or_else(|| CatalogError::new(CatalogErrorKind::CanonicalEncoding)),
        CatalogEvidence::Phase9(id) => Ok(format!("phase9/{}", id.as_str())),
        CatalogEvidence::Phase10(id) => Ok(id.as_str().to_owned()),
    }
}

fn strings(values: &[CatalogReferenceId]) -> Vec<String> {
    values.iter().map(|id| id.as_str().to_owned()).collect()
}
