//! Read-only verification for content-addressed catalog regression fixtures.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use liquidfun_differential::{
    CatalogRegressionErrorKind, NativeCatalogBackend, SessionCommand, SessionController,
    replay_catalog_regressions,
};
use liquidfun_test_protocol::{Sha256Hex, decode_resolved_scenario};
use sha2::{Digest, Sha256};

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
fn independently_replayed_checkpoints_match_reviewed_d0_identities() {
    // Arrange
    let root = repository_root();
    let reviewed = [
        (
            FIXTURES[0],
            "6d22b12e5abc8e7f06dac08dda38c9001a5b31c8f98f81938129a66334c6c052",
            "3105d34f1d7437bba4936c96150770895db4292668a5793765fd5df88e6a938b",
        ),
        (
            FIXTURES[1],
            "f7407741563b445f40effc6ef67f3a31906e6e6fcf65db7c835b5c692bddf85a",
            "55cab91288224010636d82640cf7ed17026abca2d6743dbc28839785a2a3a52b",
        ),
        (
            FIXTURES[2],
            "93a0f8c793f213b3dda6911e62830d0f02a3b14324193244d0cd6e1693512cda",
            "420b74c891786deb301e06cf8df959d5fd188fc6d5e25c1bd9ab53ad0c8a7ea2",
        ),
    ];

    // Act / Assert
    for (path, resolved_sha256, expected_d0) in reviewed {
        let bytes = fs::read(root.join(path)).expect("reviewed fixture should be readable");
        let resolved_sha256 = Sha256Hex::new(resolved_sha256).expect("reviewed hash should parse");
        let resolved = decode_resolved_scenario(&bytes, &resolved_sha256)
            .expect("reviewed canonical bytes should decode");
        assert_eq!(independent_native_d0(&resolved), expected_d0, "{path}");
    }
}

fn independent_native_d0(resolved: &liquidfun_test_protocol::ResolvedScenario) -> String {
    let mut controller = SessionController::new(NativeCatalogBackend::new());
    submit(
        &mut controller,
        SessionCommand::Select {
            resolved: resolved.clone(),
        },
    );
    for checkpoint in resolved.checkpoints() {
        submit(&mut controller, SessionCommand::StepOnce);
        submit(
            &mut controller,
            SessionCommand::CaptureCheckpoint {
                checkpoint_id: checkpoint.checkpoint_id().clone(),
            },
        );
    }
    let mut hasher = Sha256::new();
    for capture in controller.captures() {
        let mut bytes = serde_json::to_vec(capture.value())
            .expect("semantic checkpoint should serialize independently");
        bytes.push(b'\n');
        hasher.update(bytes);
    }
    Sha256Hex::from_digest(hasher.finalize().into())
        .as_str()
        .to_owned()
}

fn submit(controller: &mut SessionController<NativeCatalogBackend>, command: SessionCommand) {
    let command_id = controller
        .next_command_id()
        .expect("reviewed command count should fit");
    controller
        .submit(command_id, command)
        .expect("reviewed native action should execute");
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
