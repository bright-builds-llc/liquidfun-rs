//! Focused boundary coverage for the semantic upstream corpus model.

use std::error::Error;

#[path = "../src/inventory/corpus.rs"]
mod corpus;

use corpus::model::{MAX_CORPUS_ITEMS, MAX_EVIDENCE_MAPPINGS};
use corpus::{MAX_CORPUS_BYTES, MAX_JSON_DEPTH, parse_manifest};
use serde_json::{Value, json};

const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn valid_manifest_round_trips_through_checked_types() -> TestResult {
    // Arrange
    let bytes = include_bytes!("fixtures/corpus/valid-minimal.json");

    // Act
    let manifest = parse_manifest(bytes, REVISION)?;
    let encoded = serde_json::to_vec_pretty(&manifest)?;
    let reparsed = parse_manifest(&encoded, REVISION)?;

    // Assert
    assert_eq!(manifest, reparsed);
    Ok(())
}

#[test]
fn unknown_disposition_is_a_schema_error() {
    // Arrange
    let bytes = include_bytes!("fixtures/corpus/invalid-disposition.json");

    // Act
    let error = parse_manifest(bytes, REVISION).expect_err("unknown disposition must fail");

    // Assert
    assert_eq!(error.category(), "schema");
}

#[test]
fn unknown_fields_are_rejected_before_domain_validation() -> TestResult {
    // Arrange
    let mut manifest = valid_manifest()?;
    manifest["items"][0]["unexpected"] = json!(true);

    // Act
    let error = parse_value(&manifest).expect_err("unknown field must fail");

    // Assert
    assert_eq!(error.category(), "schema");
    Ok(())
}

#[test]
fn absolute_traversing_and_platform_ambiguous_paths_are_rejected() -> TestResult {
    for invalid_path in [
        "/liquidfun/Box2D/Unittests/Common/CommonTests.cpp",
        "../CommonTests.cpp",
        "liquidfun\\CommonTests.cpp",
    ] {
        // Arrange
        let mut manifest = valid_manifest()?;
        manifest["items"][0]["source"]["path"] = json!(invalid_path);

        // Act
        let error = parse_value(&manifest).expect_err("untrusted path must fail");

        // Assert
        assert_eq!(error.category(), "path");
    }
    Ok(())
}

#[test]
fn duplicate_item_ids_are_rejected() -> TestResult {
    // Arrange
    let mut manifest = valid_manifest()?;
    let duplicate = manifest["items"][0].clone();
    manifest["items"]
        .as_array_mut()
        .ok_or("items must be an array")?
        .push(duplicate);

    // Act
    let error = parse_value(&manifest).expect_err("duplicate ID must fail");

    // Assert
    assert_eq!(error.category(), "duplicate-id");
    Ok(())
}

#[test]
fn distinct_ids_cannot_claim_the_same_source_identity() -> TestResult {
    // Arrange
    let mut manifest = valid_manifest()?;
    let mut duplicate = manifest["items"][0].clone();
    duplicate["id"] = json!("upstream-test.common.vec2-construction-copy");
    manifest["items"]
        .as_array_mut()
        .ok_or("items must be an array")?
        .push(duplicate);

    // Act
    let error = parse_value(&manifest).expect_err("duplicate source identity must fail");

    // Assert
    assert_eq!(error.category(), "duplicate-source-identity");
    Ok(())
}

#[test]
fn vague_and_self_referential_rationales_are_rejected() -> TestResult {
    for rationale in [
        "not applicable".to_owned(),
        "See upstream-test.common.vec2-construction for the complete rationale.".to_owned(),
    ] {
        // Arrange
        let mut manifest = valid_manifest()?;
        manifest["items"][0]["review"]["rationale"] = json!(rationale);

        // Act
        let error = parse_value(&manifest).expect_err("vague rationale must fail");

        // Assert
        assert_eq!(error.category(), "rationale");
    }
    Ok(())
}

#[test]
fn duplicate_and_self_referential_evidence_is_rejected() -> TestResult {
    // Arrange
    let mut duplicate_manifest = valid_manifest()?;
    let mapping = duplicate_manifest["items"][0]["evidence"][0].clone();
    duplicate_manifest["items"][0]["evidence"]
        .as_array_mut()
        .ok_or("evidence must be an array")?
        .push(mapping);
    let mut self_manifest = valid_manifest()?;
    self_manifest["items"][0]["evidence"][0]["reference"] =
        json!("upstream-test.common.vec2-construction");

    // Act
    let duplicate_error =
        parse_value(&duplicate_manifest).expect_err("duplicate evidence must fail");
    let self_error = parse_value(&self_manifest).expect_err("self evidence must fail");

    // Assert
    assert_eq!(duplicate_error.category(), "evidence");
    assert_eq!(self_error.category(), "evidence");
    Ok(())
}

#[test]
fn evidence_collection_accepts_the_limit_and_rejects_one_more() -> TestResult {
    // Arrange
    let mut manifest = valid_manifest()?;
    let evidence: Vec<Value> = (0..MAX_EVIDENCE_MAPPINGS)
        .map(|index| {
            json!({
                "kind": "native_test",
                "reference": format!("crates/liquidfun/tests/corpus_evidence_{index}.rs")
            })
        })
        .collect();
    manifest["items"][0]["evidence"] = Value::Array(evidence);

    // Act
    let at_limit = parse_value(&manifest);
    manifest["items"][0]["evidence"]
        .as_array_mut()
        .ok_or("evidence must be an array")?
        .push(json!({
            "kind": "native_test",
            "reference": "crates/liquidfun/tests/corpus_evidence_over_limit.rs"
        }));
    let over_limit = parse_value(&manifest).expect_err("one-over-limit evidence must fail");

    // Assert
    assert!(at_limit.is_ok());
    assert_eq!(over_limit.category(), "collection-limit");
    Ok(())
}

#[test]
fn item_collection_accepts_the_limit_and_rejects_one_more() -> TestResult {
    // Arrange
    let manifest = valid_manifest()?;
    let template = manifest["items"][0].clone();
    let items: Vec<Value> = (0..MAX_CORPUS_ITEMS)
        .map(|index| unique_item(&template, index))
        .collect();
    let mut bounded = manifest.clone();
    bounded["items"] = Value::Array(items.clone());
    let mut oversized = manifest;
    let mut over_limit_items = items;
    over_limit_items.push(unique_item(&template, MAX_CORPUS_ITEMS));
    oversized["items"] = Value::Array(over_limit_items);

    // Act
    let at_limit = parse_value(&bounded);
    let over_limit = parse_value(&oversized).expect_err("one-over-limit items must fail");

    // Assert
    assert!(at_limit.is_ok());
    assert_eq!(over_limit.category(), "collection-limit");
    Ok(())
}

#[test]
fn oversized_input_is_rejected_before_json_allocation() {
    // Arrange
    let bytes = vec![b' '; MAX_CORPUS_BYTES + 1];

    // Act
    let error = parse_manifest(&bytes, REVISION).expect_err("oversized input must fail");

    // Assert
    assert_eq!(error.category(), "input-limit");
}

#[test]
fn excessive_json_depth_is_rejected_before_deserialization() {
    // Arrange
    let bytes = format!(
        "{}0{}",
        "[".repeat(MAX_JSON_DEPTH + 1),
        "]".repeat(MAX_JSON_DEPTH + 1)
    );

    // Act
    let error = parse_manifest(bytes.as_bytes(), REVISION).expect_err("deep JSON must fail");

    // Assert
    assert_eq!(error.category(), "depth-limit");
}

#[test]
fn manifest_revision_must_match_the_pinned_oracle() -> TestResult {
    // Arrange
    let mut manifest = valid_manifest()?;
    manifest["oracle_revision"] = json!("0000000000000000000000000000000000000000");

    // Act
    let error = parse_value(&manifest).expect_err("wrong revision must fail");

    // Assert
    assert_eq!(error.category(), "revision");
    Ok(())
}

#[test]
fn disposition_and_applicability_must_form_a_terminal_outcome() -> TestResult {
    // Arrange
    let mut manifest = valid_manifest()?;
    manifest["items"][0]["applicability"] = json!("reviewed_exclusion");

    // Act
    let error = parse_value(&manifest).expect_err("incoherent terminal outcome must fail");

    // Assert
    assert_eq!(error.category(), "terminal-outcome");
    Ok(())
}

#[test]
fn unreviewed_discovery_item_is_accepted_without_a_terminal_outcome() -> TestResult {
    // Arrange
    let mut manifest = valid_manifest()?;
    let item = manifest["items"][0]
        .as_object_mut()
        .ok_or("item must be an object")?;
    for field in [
        "applicability",
        "disposition",
        "compatibility_impact",
        "evidence",
        "review",
    ] {
        item.remove(field);
    }

    // Act
    let parsed = parse_value(&manifest);

    // Assert
    assert!(parsed.is_ok());
    Ok(())
}

#[test]
fn partially_reviewed_discovery_item_is_rejected() -> TestResult {
    // Arrange
    let mut manifest = valid_manifest()?;
    manifest["items"][0]
        .as_object_mut()
        .ok_or("item must be an object")?
        .remove("review");

    // Act
    let error = parse_value(&manifest).expect_err("partial review must fail closed");

    // Assert
    assert_eq!(error.category(), "terminal-outcome");
    Ok(())
}

fn valid_manifest() -> Result<Value, serde_json::Error> {
    serde_json::from_slice(include_bytes!("fixtures/corpus/valid-minimal.json"))
}

fn parse_value(manifest: &Value) -> Result<corpus::CorpusManifest, corpus::CorpusError> {
    let bytes = serde_json::to_vec(manifest)
        .map_err(|_| corpus::CorpusError::new(corpus::CorpusErrorKind::Schema))?;
    parse_manifest(&bytes, REVISION)
}

fn unique_item(template: &Value, index: usize) -> Value {
    let mut item = template.clone();
    item["id"] = json!(format!("upstream-test.generated.case-{index}"));
    item["source"]["symbol"] = json!(format!("Generated.Case{index}"));
    item["evidence"][0]["reference"] =
        json!(format!("crates/liquidfun/tests/generated_case_{index}.rs"));
    item
}
