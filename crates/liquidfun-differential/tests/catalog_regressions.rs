//! Read-only verification for content-addressed catalog regression fixtures.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use liquidfun_differential::{
    CatalogRegressionErrorKind, ReplayDiagnosisErrorKind, ReplayDriftClass,
    ReplayProjectionVersion, ReplaySchemaIdentity, ReplaySemanticDocument, ReplaySemanticValue,
    diagnose_replay_drift, replay_catalog_regressions,
};

const MANIFEST: &str = "scenarios/regressions/catalog-manifest.json";
const FIXTURES: &[&str] = &[
    "scenarios/catalog/rigid-stack-v1.json",
    "scenarios/catalog/joint-rope-v1.json",
    "scenarios/catalog/particle-group-v1.json",
];

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root should exist")
}

#[test]
fn tracked_catalog_regressions_replay_byte_identically_without_writes() {
    // Arrange
    let root = repository_root();
    let tracked_paths = std::iter::once(MANIFEST).chain(FIXTURES.iter().copied());
    let before = tracked_paths
        .clone()
        .map(|path| {
            (
                path,
                fs::read(root.join(path)).expect("tracked fixture should exist"),
            )
        })
        .collect::<Vec<_>>();

    // Act
    let first = replay_catalog_regressions(&root).expect("tracked regressions should replay");
    let second = replay_catalog_regressions(&root).expect("tracked regressions should repeat");

    // Assert
    assert_eq!(first, second);
    assert_eq!(first.entries().len(), 3);
    for (path, bytes) in before {
        assert_eq!(
            fs::read(root.join(path)).expect("tracked fixture should remain readable"),
            bytes,
            "replay must not rewrite {path}"
        );
    }
}

#[test]
fn diagnosis_resolved_scenario_drift_precedes_checkpoint_comparison() {
    // Arrange
    let reviewed = semantic_document(
        serde_json::json!({"observations": [{"value": 1}]}),
        serde_json::json!({"observations": [{"value": 1}]}),
    );
    let current = semantic_document(
        serde_json::json!({"observations": [{"value": 999}]}),
        serde_json::json!({"observations": [{"value": 999}]}),
    );

    // Act
    let diagnosis = diagnose_replay_drift(
        br#"{"identity":"reviewed"}"#,
        br#"{"identity":"current"}"#,
        &reviewed,
        &current,
    )
    .expect("supported schemas should diagnose")
    .expect("resolved byte drift should produce a diagnosis");

    // Assert
    assert_eq!(
        diagnosis.drift_class(),
        ReplayDriftClass::ResolvedScenarioDrift
    );
    assert_eq!(diagnosis.first_divergence().semantic_path(), "$.identity");
    assert_eq!(
        diagnosis.first_divergence().reviewed_value(),
        &ReplaySemanticValue::Json(serde_json::json!("reviewed"))
    );
    assert_eq!(
        diagnosis.first_divergence().current_value(),
        &ReplaySemanticValue::Json(serde_json::json!("current"))
    );
}

#[test]
fn diagnosis_physics_drift_uses_first_parity_bearing_path() {
    // Arrange
    let reviewed = semantic_document(
        serde_json::json!({"observations": [{"value": 1}]}),
        serde_json::json!({"observations": [{"value": 1}]}),
    );
    let current = semantic_document(
        serde_json::json!({"observations": [{"value": 2}]}),
        serde_json::json!({"observations": [{"value": 2}]}),
    );

    // Act
    let diagnosis = diagnose_replay_drift(b"sealed", b"sealed", &reviewed, &current)
        .expect("supported schemas should diagnose")
        .expect("physics drift should produce a diagnosis");

    // Assert
    assert_eq!(diagnosis.drift_class(), ReplayDriftClass::PhysicsDrift);
    assert_eq!(
        diagnosis.first_divergence().semantic_path(),
        "$.observations[0].value"
    );
    assert_eq!(diagnosis.reviewed_schema(), reviewed.schema());
    assert_eq!(diagnosis.current_schema(), current.schema());
}

#[test]
fn diagnosis_capture_schema_drift_follows_equal_physics_projection() {
    // Arrange
    let physics = serde_json::json!({"observations": [{"value": 1}]});
    let reviewed = semantic_document(physics.clone(), physics.clone());
    let current = semantic_document(
        physics,
        serde_json::json!({
            "observations": [{"value": 1}],
            "debug_primitives": [{"kind": "segment"}]
        }),
    );

    // Act
    let diagnosis = diagnose_replay_drift(b"sealed", b"sealed", &reviewed, &current)
        .expect("supported schemas should diagnose")
        .expect("capture expansion should produce a diagnosis");

    // Assert
    assert_eq!(
        diagnosis.drift_class(),
        ReplayDriftClass::CaptureSchemaDrift
    );
    assert_eq!(
        diagnosis.first_divergence().semantic_path(),
        "$.debug_primitives"
    );
    assert_eq!(
        diagnosis.first_divergence().reviewed_value(),
        &ReplaySemanticValue::Missing
    );
}

#[test]
fn diagnosis_rejects_unknown_and_incomparable_schema_versions() {
    // Arrange
    let unknown = ReplaySemanticDocument::new(
        ReplaySchemaIdentity::new(99, 1, ReplayProjectionVersion::LegacyPhysicsV1),
        serde_json::json!({}),
        serde_json::json!({}),
    );
    let current = semantic_document(serde_json::json!({}), serde_json::json!({}));

    // Act
    let unknown_error = diagnose_replay_drift(b"sealed", b"sealed", &unknown, &current)
        .expect_err("unknown schema versions must fail closed");

    // Assert
    assert_eq!(
        unknown_error.kind(),
        ReplayDiagnosisErrorKind::UnsupportedSchema
    );

    // Arrange
    let reviewed_expanded = ReplaySemanticDocument::new(
        ReplaySchemaIdentity::new(1, 1, ReplayProjectionVersion::ExpandedCheckpointV1),
        serde_json::json!({}),
        serde_json::json!({}),
    );

    // Act
    let incomparable_error =
        diagnose_replay_drift(b"sealed", b"sealed", &reviewed_expanded, &current)
            .expect_err("schema regression must fail closed");

    // Assert
    assert_eq!(
        incomparable_error.kind(),
        ReplayDiagnosisErrorKind::IncomparableSchema
    );
}

fn semantic_document(
    physics_projection: serde_json::Value,
    expanded_checkpoint: serde_json::Value,
) -> ReplaySemanticDocument {
    ReplaySemanticDocument::new(
        ReplaySchemaIdentity::new(1, 1, ReplayProjectionVersion::LegacyPhysicsV1),
        physics_projection,
        expanded_checkpoint,
    )
}

#[test]
fn replay_rejects_hash_drift_seed_only_and_path_escape_before_execution() {
    // Arrange
    let source = repository_root();
    let repository = TestRepository::copy_from(&source);

    // Act / Assert: content drift
    fs::write(
        repository.root.join(FIXTURES[0]),
        br#"{"not":"the reviewed resolved scenario"}"#,
    )
    .expect("fixture mutation should succeed");
    assert_eq!(
        replay_catalog_regressions(&repository.root)
            .expect_err("hash drift must fail")
            .kind(),
        CatalogRegressionErrorKind::FixtureMismatch
    );

    // Act / Assert: seed-only substitution
    repository.restore(&source);
    repository.mutate_manifest(|manifest| {
        manifest["entries"][0]
            .as_object_mut()
            .expect("entry should be an object")
            .remove("path");
    });
    assert_eq!(
        replay_catalog_regressions(&repository.root)
            .expect_err("seed-only metadata must not substitute for bytes")
            .kind(),
        CatalogRegressionErrorKind::InvalidManifest
    );

    // Act / Assert: traversal
    repository.restore(&source);
    repository.mutate_manifest(|manifest| {
        manifest["entries"][0]["path"] = serde_json::json!("../outside.json");
    });
    assert_eq!(
        replay_catalog_regressions(&repository.root)
            .expect_err("path traversal must fail")
            .kind(),
        CatalogRegressionErrorKind::UnsafePath
    );
}

#[test]
fn replay_rejects_stale_versions_unknown_actions_and_duplicate_authority() {
    // Arrange
    let source = repository_root();
    let repository = TestRepository::copy_from(&source);

    // Act / Assert: stale manifest schema
    repository.mutate_manifest(|manifest| manifest["schema_version"] = serde_json::json!(2));
    assert_eq!(
        replay_catalog_regressions(&repository.root)
            .expect_err("stale manifest schema must fail")
            .kind(),
        CatalogRegressionErrorKind::UnsupportedVersion
    );

    // Act / Assert: unknown closed action
    repository.restore(&source);
    let fixture_path = repository.root.join(FIXTURES[1]);
    let bytes = fs::read(&fixture_path).expect("fixture should be readable");
    let mut fixture: serde_json::Value =
        serde_json::from_slice(&bytes).expect("fixture should be JSON");
    fixture["actions"][0]["action"] = serde_json::json!({"kind": "unknown_action"});
    fs::write(
        fixture_path,
        serde_json::to_vec(&fixture).expect("fixture should serialize"),
    )
    .expect("fixture mutation should succeed");
    assert_eq!(
        replay_catalog_regressions(&repository.root)
            .expect_err("unknown actions must fail before native execution")
            .kind(),
        CatalogRegressionErrorKind::FixtureMismatch
    );

    // Act / Assert: duplicate path/hash authority
    repository.restore(&source);
    repository.mutate_manifest(|manifest| {
        let duplicate = manifest["entries"][0].clone();
        manifest["entries"]
            .as_array_mut()
            .expect("entries should be an array")
            .push(duplicate);
    });
    assert_eq!(
        replay_catalog_regressions(&repository.root)
            .expect_err("duplicate authority must fail")
            .kind(),
        CatalogRegressionErrorKind::DuplicateIdentity
    );
}

#[cfg(unix)]
#[test]
fn replay_rejects_symlinked_fixture_before_execution() {
    use std::os::unix::fs::symlink;

    // Arrange
    let source = repository_root();
    let repository = TestRepository::copy_from(&source);
    let fixture = repository.root.join(FIXTURES[0]);
    fs::remove_file(&fixture).expect("copied fixture should be removable");
    symlink(source.join(FIXTURES[0]), fixture).expect("test symlink should be creatable");

    // Act
    let error = replay_catalog_regressions(&repository.root)
        .expect_err("linked fixtures must not cross the tracked authority boundary");

    // Assert
    assert_eq!(error.kind(), CatalogRegressionErrorKind::UnsafePath);
}

struct TestRepository {
    root: PathBuf,
}

impl TestRepository {
    fn copy_from(source: &Path) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "liquidfun-catalog-regression-{}-{id}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("stale test repository should be removable");
        }
        let repository = Self { root };
        repository.restore(source);
        repository
    }

    fn restore(&self, source: &Path) {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).expect("test repository should be replaceable");
        }
        fs::create_dir_all(self.root.join("scenarios/catalog"))
            .expect("catalog fixture directory should be creatable");
        fs::create_dir_all(self.root.join("scenarios/regressions"))
            .expect("regression manifest directory should be creatable");
        for path in std::iter::once(MANIFEST).chain(FIXTURES.iter().copied()) {
            fs::copy(source.join(path), self.root.join(path))
                .expect("tracked regression file should copy");
        }
    }

    fn mutate_manifest(&self, mutate: impl FnOnce(&mut serde_json::Value)) {
        let path = self.root.join(MANIFEST);
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).expect("manifest should be readable"))
                .expect("manifest should be JSON");
        mutate(&mut manifest);
        fs::write(
            path,
            serde_json::to_vec(&manifest).expect("manifest should serialize"),
        )
        .expect("manifest mutation should succeed");
    }
}

impl Drop for TestRepository {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.root) {
            assert_eq!(
                error.kind(),
                std::io::ErrorKind::NotFound,
                "test repository cleanup failed: {error}"
            );
        }
    }
}
