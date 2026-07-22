//! Closed Phase 11 scenario, mapping, and inherited-proof corpus tests.

use std::path::{Path, PathBuf};

#[path = "phase11_corpus/io.rs"]
mod corpus_io;
#[path = "phase11_corpus/model.rs"]
mod model;
#[path = "phase11_corpus/validation.rs"]
mod validation;

use corpus_io::sha256;
use validation::{EXPECTED_MAPPING_COUNT, MANIFEST, MAPPINGS, load, validate};

const REVIEWED_MANIFEST_SHA256: &str =
    "ea5c1364ab3e2c50aafc2edb9aa09fe436e19f4b3fe8d48ff69ece5da1bd0860";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("differential crate should live below the repository root")
        .to_path_buf()
}

#[test]
fn sealed_phase11_corpus_matches_live_registry_and_tracked_proofs() {
    // Arrange
    let root = repository_root();
    let manifest_bytes =
        std::fs::read(root.join(MANIFEST)).expect("the sealed Phase 11 manifest must exist");
    let mappings_bytes =
        std::fs::read(root.join(MAPPINGS)).expect("the reviewed scenario mappings must exist");
    let loaded = load(&root).expect("the closed corpus should decode strictly");

    // Act
    let result = validate(&root, &loaded);

    // Assert
    assert!(
        result.is_ok(),
        "closed corpus validation failed: {result:?}"
    );
    assert_eq!(loaded.mappings.records.len(), EXPECTED_MAPPING_COUNT);
    assert_eq!(loaded.manifest.inherited_proofs.len(), 5);
    assert_eq!(sha256(&manifest_bytes), REVIEWED_MANIFEST_SHA256);
    assert_eq!(
        sha256(&mappings_bytes),
        loaded.manifest.mapping.sha256,
        "mapping bytes must stay content-addressed"
    );
}

#[test]
fn corpus_validation_is_read_only_and_repeatable() {
    // Arrange
    let root = repository_root();
    let before_manifest = std::fs::read(root.join(MANIFEST)).expect("manifest should exist");
    let before_mappings = std::fs::read(root.join(MAPPINGS)).expect("mappings should exist");

    // Act
    for _ in 0..2 {
        let loaded = load(&root).expect("corpus should decode");
        validate(&root, &loaded).expect("corpus should validate repeatedly");
    }

    // Assert
    assert_eq!(
        std::fs::read(root.join(MANIFEST)).expect("manifest should remain readable"),
        before_manifest
    );
    assert_eq!(
        std::fs::read(root.join(MAPPINGS)).expect("mappings should remain readable"),
        before_mappings
    );
}

#[test]
fn corpus_rejects_missing_unknown_duplicate_and_stale_claims() {
    // Arrange
    let root = repository_root();
    let loaded = load(&root).expect("baseline corpus should decode");

    // Act / Assert: missing mapping
    let mut missing = load(&root).expect("baseline corpus should reload");
    missing.mappings.records.pop();
    assert!(validate(&root, &missing).is_err());

    // Act / Assert: unknown mapping
    let mut unknown = load(&root).expect("baseline corpus should reload");
    unknown.mappings.records[0].slug = "unknown-scenario".to_owned();
    assert!(validate(&root, &unknown).is_err());

    // Act / Assert: duplicate semantic leaf
    let mut duplicate = load(&root).expect("baseline corpus should reload");
    let repeated = duplicate.payloads[0].observation_leaves[0].clone();
    duplicate.payloads[1].observation_leaves[0] = repeated;
    assert!(validate(&root, &duplicate).is_err());

    // Act / Assert: stale payload digest
    let mut stale = loaded;
    stale.manifest.payloads[0].sha256 = "0".repeat(64);
    assert!(validate(&root, &stale).is_err());
}

#[test]
fn corpus_rejects_incomplete_proofs_contradictory_eligibility_and_forbidden_leaves() {
    // Arrange
    let root = repository_root();

    // Act / Assert: incomplete inherited proof
    let mut incomplete = load(&root).expect("baseline corpus should decode");
    incomplete.manifest.inherited_proofs.pop();
    assert!(validate(&root, &incomplete).is_err());

    // Act / Assert: circular inherited proof
    let mut circular = load(&root).expect("baseline corpus should reload");
    circular.manifest.inherited_proofs[0].path = MANIFEST.to_owned();
    assert!(validate(&root, &circular).is_err());

    // Act / Assert: contradictory eligibility
    let mut contradictory = load(&root).expect("baseline corpus should reload");
    contradictory.manifest.cases[0].eligibility.visual = false;
    assert!(validate(&root, &contradictory).is_err());

    // Act / Assert: renderer and private diagnostic authority
    let mut forbidden = load(&root).expect("baseline corpus should reload");
    forbidden.payloads[0].primitive_leaves[0] = "ui.pixel.render_order".to_owned();
    assert!(validate(&root, &forbidden).is_err());
}
