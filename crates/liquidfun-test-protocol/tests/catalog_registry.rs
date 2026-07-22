//! Closed scenario registry and deterministic projection integration tests.

use liquidfun_test_protocol::{
    CatalogErrorKind, CatalogEvidence, CatalogEvidenceDisposition, CatalogEvidenceId,
    CatalogMapping, CatalogSlug, ScenarioCatalog, ScenarioConsumer, ScenarioVersion,
    check_scenario_catalog_projection, reviewed_scenario_catalog, scenario_definitions,
    scenario_mappings,
};

const TRACKED_CATALOG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../reference/scenario-catalog.json"
));

#[test]
fn registry_is_complete_unique_and_stably_sorted() {
    // Arrange and Act
    let catalog = reviewed_scenario_catalog().expect("reviewed catalog should validate");
    let identities = catalog
        .definitions()
        .iter()
        .map(|definition| {
            (
                definition.slug().as_str(),
                definition.scenario_version().get(),
            )
        })
        .collect::<Vec<_>>();
    let mut sorted = identities.clone();
    sorted.sort_unstable();

    // Assert
    assert_eq!(catalog.definitions().len(), 43);
    assert_eq!(catalog.mappings().len(), 43);
    assert_eq!(identities, sorted);
    assert!(identities.windows(2).all(|pair| pair[0] != pair[1]));
}

#[test]
fn every_scenario_has_one_complete_closed_consumer_mapping() {
    // Arrange
    let catalog = reviewed_scenario_catalog().expect("reviewed catalog should validate");

    // Act and Assert
    for definition in catalog.definitions() {
        let mapping = catalog
            .mapping(definition.slug(), definition.scenario_version())
            .expect("every definition should have exactly one mapping");
        assert!(!mapping.test_ids().is_empty());
        assert!(matches!(
            mapping.evidence_disposition(),
            CatalogEvidenceDisposition::Oracle { .. }
                | CatalogEvidenceDisposition::ReviewedEquivalent { .. }
        ));
        assert!(mapping.regression_use());
        assert!(mapping.is_eligible(ScenarioConsumer::Benchmark));
        assert!(mapping.is_eligible(ScenarioConsumer::Visual));
        assert!(!mapping.upstream_corpus_ids().is_empty());
        assert!(!mapping.compatibility_refs().is_empty());
    }
}

#[test]
fn duplicate_definition_identity_is_rejected() {
    // Arrange
    let mut definitions = scenario_definitions().expect("definitions should build");
    let mappings = scenario_mappings(&definitions).expect("mappings should build");
    definitions.push(definitions[0].clone());

    // Act
    let error = ScenarioCatalog::new(definitions, mappings)
        .expect_err("duplicate identity must fail closed");

    // Assert
    assert_eq!(error.kind(), CatalogErrorKind::DuplicateScenarioIdentity);
}

#[test]
fn missing_mapping_is_rejected() {
    // Arrange
    let definitions = scenario_definitions().expect("definitions should build");
    let mut mappings = scenario_mappings(&definitions).expect("mappings should build");
    let _removed = mappings.pop().expect("registry has mappings");

    // Act
    let error = ScenarioCatalog::new(definitions, mappings)
        .expect_err("unmapped definition must fail closed");

    // Assert
    assert_eq!(error.kind(), CatalogErrorKind::MissingMapping);
}

#[test]
fn unknown_mapping_identity_is_rejected() {
    // Arrange
    let definitions = scenario_definitions().expect("definitions should build");
    let mut mappings = scenario_mappings(&definitions).expect("mappings should build");
    let template = mappings[0].clone();
    mappings.push(mapping_with(
        &template,
        CatalogSlug::new("unknown-native-scenario").expect("valid unknown slug"),
        template.evidence_disposition().clone(),
        template.regression_use(),
        template.is_eligible(ScenarioConsumer::Benchmark),
        template.is_eligible(ScenarioConsumer::Visual),
    ));

    // Act
    let error =
        ScenarioCatalog::new(definitions, mappings).expect_err("unknown mapping must fail closed");

    // Assert
    assert_eq!(error.kind(), CatalogErrorKind::UnknownMapping);
}

#[test]
fn stale_evidence_identity_is_rejected() {
    // Arrange
    let definitions = scenario_definitions().expect("definitions should build");
    let mut mappings = scenario_mappings(&definitions).expect("mappings should build");
    let template = mappings[0].clone();
    mappings[0] = mapping_with(
        &template,
        template.slug().clone(),
        CatalogEvidenceDisposition::ReviewedEquivalent {
            evidence: vec![CatalogEvidence::Phase10(
                CatalogEvidenceId::new("phase10/stale-leaf").expect("well-formed stale ID"),
            )],
        },
        template.regression_use(),
        template.is_eligible(ScenarioConsumer::Benchmark),
        template.is_eligible(ScenarioConsumer::Visual),
    );

    // Act
    let error =
        ScenarioCatalog::new(definitions, mappings).expect_err("stale evidence must fail closed");

    // Assert
    assert_eq!(error.kind(), CatalogErrorKind::StaleEvidence);
}

#[test]
fn contradictory_consumer_eligibility_is_rejected() {
    // Arrange
    let definitions = scenario_definitions().expect("definitions should build");
    let mut mappings = scenario_mappings(&definitions).expect("mappings should build");
    let template = mappings[0].clone();
    mappings[0] = mapping_with(
        &template,
        template.slug().clone(),
        template.evidence_disposition().clone(),
        template.regression_use(),
        !template.is_eligible(ScenarioConsumer::Benchmark),
        template.is_eligible(ScenarioConsumer::Visual),
    );

    // Act
    let error = ScenarioCatalog::new(definitions, mappings)
        .expect_err("eligibility drift must fail closed");

    // Assert
    assert_eq!(error.kind(), CatalogErrorKind::ContradictoryEligibility);
}

#[test]
fn display_title_cannot_be_used_as_catalog_identity() {
    // Arrange
    let catalog = reviewed_scenario_catalog().expect("reviewed catalog should validate");
    let title = catalog.definitions()[0].display_title();

    // Act
    let error = catalog
        .mapping_by_text(title, ScenarioVersion::CURRENT)
        .expect_err("presentation title must not resolve as stable identity");

    // Assert
    assert_eq!(error.kind(), CatalogErrorKind::TitleAsIdentity);
}

#[test]
fn tracked_projection_is_the_exact_in_memory_projection() {
    // Arrange and Act
    let result = check_scenario_catalog_projection(TRACKED_CATALOG);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn projection_drift_is_rejected_without_rewriting_the_snapshot() {
    // Arrange
    let before = TRACKED_CATALOG.to_vec();
    let mut drifted = before.clone();
    drifted.push(b' ');

    // Act
    let error = check_scenario_catalog_projection(&drifted)
        .expect_err("noncanonical tracked bytes must fail closed");

    // Assert
    assert_eq!(error.kind(), CatalogErrorKind::ProjectionMismatch);
    assert_eq!(TRACKED_CATALOG, before);
}

fn mapping_with(
    template: &CatalogMapping,
    slug: CatalogSlug,
    evidence_disposition: CatalogEvidenceDisposition,
    regression_use: bool,
    benchmark_eligible: bool,
    visual_eligible: bool,
) -> CatalogMapping {
    CatalogMapping::new(
        slug,
        template.scenario_version(),
        template.test_ids().to_vec(),
        evidence_disposition,
        regression_use,
        benchmark_eligible,
        visual_eligible,
        template.upstream_corpus_ids().to_vec(),
        template.compatibility_refs().to_vec(),
    )
    .expect("test mapping should satisfy local shape bounds")
}
