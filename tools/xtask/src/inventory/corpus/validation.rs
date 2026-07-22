//! Fail-closed joins for the terminal semantic corpus authority.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use sha2::{Digest, Sha256};

#[path = "validation/io.rs"]
mod io;
#[path = "validation/schema.rs"]
mod schema;

use self::io::{read_bounded, read_json_bounded};
use self::schema::{ScenarioCatalog, ScenarioMapping, ScenarioMappings, ScenarioRecord};
use super::corpus::model::{
    CorpusItem, CorpusKind, CorpusManifest, EvidenceKind, TerminalDisposition,
};
use super::corpus::parse_manifest;
use super::{
    CompatibilityLedger, DiscoveryKind, DiscoveryLedger, InventoryError,
    require_schema_and_revision,
};

const EXPECTED_ITEMS: usize = 388;
const EXPECTED_UPSTREAM_TESTS: usize = 244;
const EXPECTED_PARAMETERIZED_TESTS: usize = 68;
const EXPECTED_EXAMPLES: usize = 73;
const EXPECTED_TESTBED_ENTRIES: usize = 71;
const EXPECTED_SCENARIOS: usize = 43;
const EXPECTED_MAPPING_SCHEMA: u32 = 1;

pub(super) fn load_and_validate(
    repository_root: &Path,
    oracle_revision: &str,
) -> Result<CorpusManifest, InventoryError> {
    let corpus_bytes = read_bounded(
        &repository_root.join("reference/upstream-corpus.json"),
        "corpus authority",
    )?;
    let manifest = parse_manifest(&corpus_bytes, oracle_revision)
        .map_err(|error| InventoryError::new(error.inventory_category(), error.to_string()))?;
    let discovery: DiscoveryLedger = read_json_bounded(
        &repository_root.join("reference/discovery.json"),
        "discovery authority",
    )?;
    let compatibility: CompatibilityLedger = read_json_bounded(
        &repository_root.join("reference/compatibility.json"),
        "compatibility authority",
    )?;
    let catalog_path = repository_root.join("reference/scenario-catalog.json");
    let catalog_bytes = read_bounded(&catalog_path, "scenario catalog")?;
    let catalog: ScenarioCatalog = serde_json::from_slice(&catalog_bytes).map_err(|error| {
        closure_error(
            "schema",
            format!(
                "invalid scenario catalog in {}: {error}",
                catalog_path.display()
            ),
        )
    })?;
    let catalog_sha256 = format!("{:x}", Sha256::digest(&catalog_bytes));
    let mappings: ScenarioMappings = read_json_bounded(
        &repository_root.join("reference/artifacts/phase11/scenario-mappings.json"),
        "scenario mappings",
    )?;

    validate_authority_headers(
        oracle_revision,
        &manifest,
        &discovery,
        &compatibility,
        &catalog,
        &catalog_sha256,
        &mappings,
    )?;
    validate_pinned_counts(&manifest)?;
    validate_joins(&manifest, &discovery, &compatibility, &catalog, &mappings)?;
    Ok(manifest)
}

fn validate_authority_headers(
    oracle_revision: &str,
    manifest: &CorpusManifest,
    discovery: &DiscoveryLedger,
    compatibility: &CompatibilityLedger,
    catalog: &ScenarioCatalog,
    catalog_sha256: &str,
    mappings: &ScenarioMappings,
) -> Result<(), InventoryError> {
    require_schema_and_revision(
        discovery.schema_version,
        &discovery.oracle_revision,
        oracle_revision,
    )?;
    require_schema_and_revision(
        compatibility.schema_version,
        &compatibility.oracle_revision,
        oracle_revision,
    )?;
    if manifest.oracle_revision() != oracle_revision {
        return Err(closure_error("revision", "corpus revision is stale"));
    }
    if catalog.schema_version != 1
        || catalog.scenarios.len() != EXPECTED_SCENARIOS
        || catalog.sort_contract
            != "scenarios are ordered lexicographically by stable slug then scenario_version"
        || catalog.description.trim().is_empty()
        || catalog.projection_sha256.len() != 64
    {
        return Err(closure_error(
            "catalog",
            "scenario catalog header or count is stale",
        ));
    }
    if mappings.schema_version != EXPECTED_MAPPING_SCHEMA
        || mappings.catalog_schema_version != catalog.schema_version
        || mappings.catalog_sha256 != catalog_sha256
        || mappings.record_count != mappings.records.len()
        || mappings.record_count != catalog.scenarios.len()
    {
        return Err(closure_error(
            "mapping",
            "scenario mapping header, catalog digest, or count is stale",
        ));
    }
    Ok(())
}

fn validate_pinned_counts(manifest: &CorpusManifest) -> Result<(), InventoryError> {
    let count = |kind| {
        manifest
            .items()
            .iter()
            .filter(|item| item.kind() == kind)
            .count()
    };
    let parameterized = manifest
        .items()
        .iter()
        .filter(|item| {
            item.kind() == CorpusKind::UpstreamTest && item.source_symbol().contains('/')
        })
        .count();
    if manifest.items().len() != EXPECTED_ITEMS
        || count(CorpusKind::UpstreamTest) != EXPECTED_UPSTREAM_TESTS
        || count(CorpusKind::Example) != EXPECTED_EXAMPLES
        || count(CorpusKind::TestbedEntry) != EXPECTED_TESTBED_ENTRIES
        || parameterized != EXPECTED_PARAMETERIZED_TESTS
    {
        return Err(closure_error(
            "counts",
            "semantic item, kind, or parameterized-case totals drifted",
        ));
    }
    let has_rope = manifest.items().iter().any(|item| {
        item.id() == "example.rope-create"
            && item.source_path() == "liquidfun/Box2D/Testbed/Tests/Rope.h"
            && item.source_symbol() == "Rope::Create"
    });
    let has_hello_world = manifest.items().iter().any(|item| {
        item.id() == "example.main"
            && item.source_path() == "liquidfun/Box2D/HelloWorld/HelloWorld.cpp"
            && item.source_symbol() == "main"
    });
    if !has_rope || !has_hello_world {
        return Err(closure_error(
            "counts",
            "unregistered Rope.h or HelloWorld is missing",
        ));
    }
    Ok(())
}

fn validate_joins(
    manifest: &CorpusManifest,
    discovery: &DiscoveryLedger,
    compatibility: &CompatibilityLedger,
    catalog: &ScenarioCatalog,
    mappings: &ScenarioMappings,
) -> Result<(), InventoryError> {
    let discovered_paths: BTreeSet<_> = discovery
        .entries
        .iter()
        .filter(|entry| matches!(entry.kind, DiscoveryKind::Test | DiscoveryKind::Example))
        .map(|entry| entry.upstream_path.as_str())
        .collect();
    let compatibility_ids: BTreeSet<_> = compatibility
        .entries
        .iter()
        .map(|entry| entry.id.as_str())
        .collect();
    let scenario_by_slug = unique_scenarios(catalog)?;
    let mapping_by_slug = unique_mappings(mappings)?;
    let corpus_ids: BTreeSet<_> = manifest.items().iter().map(CorpusItem::id).collect();

    for scenario in &catalog.scenarios {
        for corpus_id in &scenario.upstream_corpus_ids {
            if !corpus_ids.contains(corpus_id.as_str()) {
                return Err(closure_error(
                    "catalog",
                    format!(
                        "scenario `{}` names unknown corpus item `{corpus_id}`",
                        scenario.slug
                    ),
                ));
            }
        }
        validate_mapping(scenario, &mapping_by_slug)?;
    }

    for item in manifest.items() {
        if !source_resolves(item, &discovered_paths) {
            return Err(item_error(
                "source",
                item,
                "source path is absent from discovery",
            ));
        }
        validate_terminal_item(
            item,
            &scenario_by_slug,
            &mapping_by_slug,
            &compatibility_ids,
        )?;
    }
    Ok(())
}

fn source_resolves(item: &CorpusItem, discovered_paths: &BTreeSet<&str>) -> bool {
    if discovered_paths.contains(item.source_path()) {
        return true;
    }
    if item.kind() != CorpusKind::TestbedEntry
        || item.source_path() != "liquidfun/Box2D/Testbed/Tests/TestEntries.cpp"
    {
        return false;
    }
    let Some((_, factory)) = item.source_symbol().split_once('|') else {
        return false;
    };
    let Some((class_name, method)) = factory.split_once("::") else {
        return false;
    };
    if method != "Create" {
        return false;
    }
    let implementation_path = format!("liquidfun/Box2D/Testbed/Tests/{class_name}.h");
    discovered_paths.contains(implementation_path.as_str())
}

fn validate_terminal_item(
    item: &CorpusItem,
    scenarios: &BTreeMap<&str, &ScenarioRecord>,
    mappings: &BTreeMap<&str, &ScenarioMapping>,
    compatibility_ids: &BTreeSet<&str>,
) -> Result<(), InventoryError> {
    let Some(disposition) = item.disposition() else {
        return Err(item_error(
            "unresolved",
            item,
            "terminal disposition is missing",
        ));
    };
    if item.applicability().is_none()
        || item.compatibility_impact().is_none()
        || item.review().is_none()
    {
        return Err(item_error(
            "unresolved",
            item,
            "terminal classification or review is incomplete",
        ));
    }
    let evidence = item
        .evidence()
        .ok_or_else(|| item_error("unresolved", item, "terminal evidence mappings are missing"))?;
    match disposition {
        TerminalDisposition::NativePort | TerminalDisposition::EquivalentEvidence => {
            validate_supported_evidence(item, evidence, scenarios, mappings, compatibility_ids)
        }
        TerminalDisposition::ReviewedIrrelevance | TerminalDisposition::IntentionalNonSupport => {
            validate_review_evidence(item, evidence)
        }
        TerminalDisposition::DocumentedDifference => {
            validate_difference_evidence(item, evidence, compatibility_ids)
        }
    }
}

fn validate_supported_evidence(
    item: &CorpusItem,
    evidence: &[super::corpus::model::EvidenceMapping],
    scenarios: &BTreeMap<&str, &ScenarioRecord>,
    mappings: &BTreeMap<&str, &ScenarioMapping>,
    compatibility_ids: &BTreeSet<&str>,
) -> Result<(), InventoryError> {
    let scenario_slug = exactly_one_reference(
        item,
        evidence,
        EvidenceKind::NativeScenario,
        "reference/scenario-catalog.json#scenario=",
    )?;
    let test_id = exactly_one_reference(
        item,
        evidence,
        EvidenceKind::NativeTest,
        "reference/scenario-catalog.json#test=",
    )?;
    let regression_id = exactly_one_reference(
        item,
        evidence,
        EvidenceKind::RegressionFixture,
        "reference/artifacts/phase11/scenario-mappings.json#regression=",
    )?;
    let compatibility_id = exactly_one_reference(
        item,
        evidence,
        EvidenceKind::CompatibilityLedger,
        "reference/compatibility.json#id=",
    )?;
    if evidence.len() != 4 {
        return Err(item_error(
            "evidence",
            item,
            "supported outcome must have exactly four joined evidence mappings",
        ));
    }
    let Some(scenario) = scenarios.get(scenario_slug) else {
        return Err(item_error("evidence", item, "native scenario is unknown"));
    };
    let Some(mapping) = mappings.get(scenario_slug) else {
        return Err(item_error("mapping", item, "scenario mapping is missing"));
    };
    if !scenario.test_ids.iter().any(|value| value == test_id)
        || !mapping.test_ids.iter().any(|value| value == test_id)
        || !mapping.regression.eligible
        || mapping.regression.mapping_id != regression_id
        || !scenario
            .compatibility_refs
            .iter()
            .any(|value| value == compatibility_id)
        || !compatibility_ids.contains(compatibility_id)
    {
        return Err(item_error(
            "evidence",
            item,
            "scenario, public test, regression, or compatibility join is stale",
        ));
    }
    Ok(())
}

fn validate_review_evidence(
    item: &CorpusItem,
    evidence: &[super::corpus::model::EvidenceMapping],
) -> Result<(), InventoryError> {
    let review_id = exactly_one_reference(
        item,
        evidence,
        EvidenceKind::Review,
        "reference/upstream-corpus.json#review=",
    )?;
    if evidence.len() != 1 || review_id != item.id() {
        return Err(item_error(
            "review",
            item,
            "review evidence does not resolve to the embedded reviewed record",
        ));
    }
    Ok(())
}

fn validate_difference_evidence(
    item: &CorpusItem,
    evidence: &[super::corpus::model::EvidenceMapping],
    compatibility_ids: &BTreeSet<&str>,
) -> Result<(), InventoryError> {
    let compatibility_id = exactly_one_reference(
        item,
        evidence,
        EvidenceKind::CompatibilityLedger,
        "reference/compatibility.json#id=",
    )?;
    if evidence.len() != 1 || !compatibility_ids.contains(compatibility_id) {
        return Err(item_error(
            "evidence",
            item,
            "documented difference does not resolve to one compatibility row",
        ));
    }
    Ok(())
}

fn validate_mapping(
    scenario: &ScenarioRecord,
    mappings: &BTreeMap<&str, &ScenarioMapping>,
) -> Result<(), InventoryError> {
    let Some(mapping) = mappings.get(scenario.slug.as_str()) else {
        return Err(closure_error(
            "mapping",
            format!("scenario `{}` has no mapping record", scenario.slug),
        ));
    };
    if mapping.scenario_version != scenario.scenario_version
        || mapping.test_ids != scenario.test_ids
        || mapping.evidence != scenario.evidence
        || mapping.evidence.disposition != "reviewed_equivalent"
        || mapping.evidence.references.is_empty()
        || mapping.regression.eligible != scenario.regression_use
        || mapping.benchmark.eligible != scenario.benchmark_eligible
        || mapping.visual.eligible != scenario.visual_eligible
        || mapping.regression.mapping_id != format!("regression/{}", scenario.slug)
        || mapping.benchmark.mapping_id != format!("benchmark/{}", scenario.slug)
        || mapping.visual.mapping_id != format!("visual/{}", scenario.slug)
        || mapping.scenario_sha256.len() != 64
    {
        return Err(closure_error(
            "mapping",
            format!("scenario `{}` mapping drifted", scenario.slug),
        ));
    }
    Ok(())
}

fn exactly_one_reference<'a>(
    item: &CorpusItem,
    evidence: &'a [super::corpus::model::EvidenceMapping],
    kind: EvidenceKind,
    prefix: &str,
) -> Result<&'a str, InventoryError> {
    let references: Vec<_> = evidence
        .iter()
        .filter(|mapping| mapping.kind() == kind)
        .collect();
    if references.len() != 1 {
        return Err(item_error(
            "evidence",
            item,
            format!("expected one {} mapping", kind.as_str()),
        ));
    }
    references[0]
        .reference()
        .strip_prefix(prefix)
        .ok_or_else(|| {
            item_error(
                "evidence",
                item,
                format!("{} mapping has an invalid authority prefix", kind.as_str()),
            )
        })
}

fn unique_scenarios(
    catalog: &ScenarioCatalog,
) -> Result<BTreeMap<&str, &ScenarioRecord>, InventoryError> {
    let mut scenarios = BTreeMap::new();
    for scenario in &catalog.scenarios {
        if scenario.slug.is_empty()
            || scenario.display_title.trim().is_empty()
            || scenario.generator_id.trim().is_empty()
            || scenario.generator_version == 0
            || scenario.seed_policy.trim().is_empty()
            || scenario.test_ids.is_empty()
            || scenario.compatibility_refs.is_empty()
        {
            return Err(closure_error(
                "catalog",
                "scenario catalog contains an incomplete semantic record",
            ));
        }
        if scenarios.insert(scenario.slug.as_str(), scenario).is_some() {
            return Err(closure_error("catalog", "duplicate scenario slug"));
        }
    }
    Ok(scenarios)
}

fn unique_mappings(
    mappings: &ScenarioMappings,
) -> Result<BTreeMap<&str, &ScenarioMapping>, InventoryError> {
    let mut records = BTreeMap::new();
    for mapping in &mappings.records {
        if records.insert(mapping.slug.as_str(), mapping).is_some() {
            return Err(closure_error("mapping", "duplicate scenario mapping"));
        }
    }
    Ok(records)
}

pub(super) fn closure_error(category: &'static str, message: impl Into<String>) -> InventoryError {
    let category = match category {
        "catalog" => "corpus-catalog",
        "counts" => "corpus-counts",
        "evidence" => "corpus-evidence",
        "filesystem" => "corpus-filesystem",
        "input-limit" => "corpus-input-limit",
        "mapping" => "corpus-mapping",
        "revision" => "corpus-revision",
        "schema" => "corpus-schema",
        "source" => "corpus-source",
        "unresolved" => "corpus-unresolved",
        _ => "corpus-closure",
    };
    InventoryError::new(category, message)
}

fn item_error(
    category: &'static str,
    item: &CorpusItem,
    message: impl Into<String>,
) -> InventoryError {
    closure_error(
        category,
        format!("item `{}`: {}", item.id(), message.into()),
    )
}
