//! Contract tests for the Phase 13 producer and immutable staged bundle.

#[allow(dead_code)]
#[path = "../src/phase13_evidence.rs"]
mod phase13_evidence;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use liquidfun_test_protocol::reviewed_scenario_catalog;
use phase13_evidence::bundle::{
    BundleDraft, BundleFile, ClosureEntry, ClosureIdentity, EvidenceMetadata, check_bundle,
    write_bundle,
};
use phase13_evidence::{
    CanonicalEnvironment, ProductionGate, ProductionGateErrorKind, select_rigid_stack_definition,
    validate_staging_root,
};

const SHA_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const UPSTREAM_SHA: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

fn producer_workflow() -> String {
    fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.github/workflows/phase13-evidence-producer.yml"),
    )
    .expect("Phase 13 producer workflow should exist")
}

#[test]
fn producer_selects_the_reviewed_rigid_stack_catalog_definition() {
    // Arrange
    let catalog = reviewed_scenario_catalog().expect("reviewed catalog should validate");

    // Act
    let (definition, slug) = select_rigid_stack_definition(&catalog)
        .expect("rigid-stack fixture should resolve to a reviewed definition");

    // Assert
    assert_eq!(slug.as_str(), "rigid-stack-stability");
    assert_eq!(definition.slug(), &slug);
}

fn temporary_directory(label: &str) -> PathBuf {
    let ordinal = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "liquidfun-phase13-{label}-{}-{ordinal}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("temporary directory should be created");
    path
}

fn metadata(record_class: &str) -> EvidenceMetadata {
    EvidenceMetadata {
        record_class: record_class.to_owned(),
        source_revision: UPSTREAM_SHA.to_owned(),
        source_path: ".".to_owned(),
        derivation_kind: "repository-authored-test-evidence".to_owned(),
        alteration_summary:
            "Repository-authored semantic evidence; no source or raw memory is copied.".to_owned(),
        notice_refs: vec!["THIRD_PARTY_NOTICES.md".to_owned()],
    }
}

fn closure(label: &str, digest: &str) -> ClosureIdentity {
    let entries = vec![ClosureEntry {
        path: format!("inputs/{label}.json"),
        sha256: digest.to_owned(),
    }];
    ClosureIdentity {
        schema_version: 1,
        label: label.to_owned(),
        digest: phase13_evidence::bundle::closure_digest(label, &entries),
        entries,
    }
}

fn gate() -> ProductionGate {
    ProductionGate {
        producer_sha: SHA_A.to_owned(),
        upstream_revision: UPSTREAM_SHA.to_owned(),
        environment: CanonicalEnvironment {
            operating_system: "linux".to_owned(),
            architecture: "x86_64".to_owned(),
            rust_target: "x86_64-unknown-linux-gnu".to_owned(),
            rust_version: "1.97.0".to_owned(),
            cmake_version: "4.3.3".to_owned(),
            ninja_version: "1.13.2".to_owned(),
            clang_version: "22.1.8".to_owned(),
            cmake_preset: "oracle-debug".to_owned(),
        },
        witness_repeat_sha256: [DIGEST_A.to_owned(), DIGEST_A.to_owned()],
        native_d0_repeat_sha256: [DIGEST_B.to_owned(), DIGEST_B.to_owned()],
        d1_oracle_passed: true,
        sealed_input_sha256: DIGEST_A.to_owned(),
        d1_input_sha256: DIGEST_A.to_owned(),
    }
}

fn draft() -> BundleDraft {
    BundleDraft {
        producer: gate(),
        witness_closure: closure("witness", DIGEST_A),
        replay_closure: closure("replay", DIGEST_B),
        materials_manifest_sha256: DIGEST_A.to_owned(),
        materials_sha256: DIGEST_B.to_owned(),
        probe_source_sha256: DIGEST_A.to_owned(),
        schema_identity: "phase13-evidence-v1".to_owned(),
        tolerance_identity: DIGEST_B.to_owned(),
        witness_invocation: vec!["phase9-lifecycle-contact-witness".to_owned()],
        replay_invocations: vec![
            "native-d0-repeat-1".to_owned(),
            "native-d0-repeat-2".to_owned(),
            "pinned-oracle-d1".to_owned(),
        ],
        d1_oracle_identity_sha256: DIGEST_A.to_owned(),
        d1_result: "match".to_owned(),
        diagnosis: serde_json::json!({
            "drift_class": "capture_schema_drift",
            "first_divergence": "$.checkpoints[0].debug_primitives.length"
        }),
        bundle_metadata: metadata("staged_bundle"),
    }
}

fn files() -> Vec<BundleFile> {
    vec![
        BundleFile {
            path: "evidence/replay.json".to_owned(),
            bytes: br#"{"record":"replay"}"#.to_vec(),
            metadata: metadata("replay_evidence"),
        },
        BundleFile {
            path: "evidence/witness.json".to_owned(),
            bytes: br#"{"record":"witness"}"#.to_vec(),
            metadata: metadata("witness"),
        },
    ]
}

#[test]
fn producer_rejects_nonidentical_witness_repeats() {
    // Arrange
    let mut candidate = gate();
    candidate.witness_repeat_sha256[1] = DIGEST_B.to_owned();

    // Act
    let error = candidate
        .validate()
        .expect_err("different witness runs must fail");

    // Assert
    assert_eq!(error.kind(), ProductionGateErrorKind::WitnessRepeatMismatch);
}

#[test]
fn producer_rejects_nonidentical_native_d0_repeats() {
    // Arrange
    let mut candidate = gate();
    candidate.native_d0_repeat_sha256[1] = DIGEST_A.to_owned();

    // Act
    let error = candidate
        .validate()
        .expect_err("different native D0 runs must fail");

    // Assert
    assert_eq!(error.kind(), ProductionGateErrorKind::NativeRepeatMismatch);
}

#[test]
fn producer_rejects_failed_d1_or_different_sealed_input() {
    // Arrange
    let mut failed = gate();
    failed.d1_oracle_passed = false;
    let mut substituted = gate();
    substituted.d1_input_sha256 = DIGEST_B.to_owned();

    // Act
    let failed_error = failed.validate().expect_err("failed D1 must fail");
    let substituted_error = substituted
        .validate()
        .expect_err("D1 over different bytes must fail");

    // Assert
    assert_eq!(failed_error.kind(), ProductionGateErrorKind::D1Failure);
    assert_eq!(
        substituted_error.kind(),
        ProductionGateErrorKind::D1InputMismatch
    );
}

#[test]
fn producer_rejects_wrong_producer_or_environment() {
    // Arrange
    let mut wrong_sha = gate();
    wrong_sha.producer_sha = "short".to_owned();
    let mut wrong_environment = gate();
    wrong_environment.environment.operating_system = "macos".to_owned();

    // Act
    let sha_error = wrong_sha.validate().expect_err("short P must fail");
    let environment_error = wrong_environment
        .validate()
        .expect_err("D2 environment must fail");

    // Assert
    assert_eq!(sha_error.kind(), ProductionGateErrorKind::Identity);
    assert_eq!(
        environment_error.kind(),
        ProductionGateErrorKind::Environment
    );
}

#[test]
fn producer_rejects_malformed_fnd04_metadata() {
    // Arrange
    let root = temporary_directory("metadata").join("bundle");
    let mut malformed_files = files();
    malformed_files[0].metadata.notice_refs.clear();

    // Act
    let error = write_bundle(&root, draft(), malformed_files)
        .expect_err("missing notice reference must fail");

    // Assert
    assert_eq!(
        error.kind(),
        phase13_evidence::bundle::BundleErrorKind::Metadata
    );
}

#[test]
fn producer_bundle_detects_tampering_and_closure_drift() {
    // Arrange
    let root = temporary_directory("tamper").join("bundle");
    let identity = write_bundle(&root, draft(), files()).expect("bundle should be written");
    fs::write(root.join("evidence/replay.json"), b"tampered")
        .expect("test bundle should be mutable");

    // Act
    let tamper_error = check_bundle(
        &root,
        &identity.producer_sha,
        &identity.bundle_sha256,
        None,
        None,
    )
    .expect_err("tampering must fail");
    let root_two = temporary_directory("closure").join("bundle");
    let identity_two = write_bundle(&root_two, draft(), files()).expect("bundle should be written");
    let closure_error = check_bundle(
        &root_two,
        &identity_two.producer_sha,
        &identity_two.bundle_sha256,
        Some(DIGEST_B),
        Some(DIGEST_B),
    )
    .expect_err("substituted witness closure must fail");

    // Assert
    assert_eq!(
        tamper_error.kind(),
        phase13_evidence::bundle::BundleErrorKind::Digest
    );
    assert_eq!(
        closure_error.kind(),
        phase13_evidence::bundle::BundleErrorKind::Closure
    );
}

#[test]
fn producer_bundle_rejects_path_escape_and_extra_files() {
    // Arrange
    let escape_root = temporary_directory("escape").join("bundle");
    let mut escaping = files();
    escaping[0].path = "../tracked.json".to_owned();

    // Act
    let escape_error =
        write_bundle(&escape_root, draft(), escaping).expect_err("path escape must fail");
    let extra_root = temporary_directory("extra").join("bundle");
    let identity = write_bundle(&extra_root, draft(), files()).expect("bundle should be written");
    fs::write(extra_root.join("extra.json"), b"extra").expect("extra file should be written");
    let extra_error = check_bundle(
        &extra_root,
        &identity.producer_sha,
        &identity.bundle_sha256,
        None,
        None,
    )
    .expect_err("unmanifested file must fail");

    // Assert
    assert_eq!(
        escape_error.kind(),
        phase13_evidence::bundle::BundleErrorKind::Path
    );
    assert_eq!(
        extra_error.kind(),
        phase13_evidence::bundle::BundleErrorKind::FileSet
    );
}

#[test]
fn producer_rejects_any_staging_root_that_can_write_tracked_artifacts() {
    // Arrange
    let repository_root = temporary_directory("repository");
    fs::create_dir_all(repository_root.join("target")).expect("target should be created");
    let tracked_candidates = [
        repository_root.join("reference/artifacts"),
        repository_root.join("reference/source-map.toml"),
        repository_root.join("scenarios"),
        repository_root.join("protocol"),
    ];

    // Act and Assert
    for candidate in tracked_candidates {
        assert!(
            validate_staging_root(&repository_root, &candidate).is_err(),
            "{} must be rejected",
            candidate.display()
        );
    }
    assert!(
        validate_staging_root(
            &repository_root,
            &repository_root.join("target/phase13/staged")
        )
        .is_ok()
    );
}

#[cfg(unix)]
#[test]
fn producer_bundle_rejects_symlinks() {
    use std::os::unix::fs::symlink;

    // Arrange
    let root = temporary_directory("symlink").join("bundle");
    let identity = write_bundle(&root, draft(), files()).expect("bundle should be written");
    fs::remove_file(root.join("evidence/replay.json")).expect("file should be removed");
    symlink(
        root.join("evidence/witness.json"),
        root.join("evidence/replay.json"),
    )
    .expect("symlink should be created");

    // Act
    let error = check_bundle(
        &root,
        &identity.producer_sha,
        &identity.bundle_sha256,
        None,
        None,
    )
    .expect_err("symlink must fail");

    // Assert
    assert_eq!(
        error.kind(),
        phase13_evidence::bundle::BundleErrorKind::Symlink
    );
}

#[test]
fn workflow_uses_exact_checkout_and_immutable_action_pins() {
    // Arrange
    let workflow = producer_workflow();

    // Act
    let uses = workflow
        .lines()
        .filter_map(|line| line.trim().strip_prefix("uses: "))
        .collect::<Vec<_>>();

    // Assert
    assert!(workflow.contains("workflow_dispatch:"));
    assert!(workflow.contains("runs-on: ubuntu-24.04"));
    assert!(workflow.contains("ref: ${{ github.sha }}"));
    assert!(workflow.contains("persist-credentials: false"));
    assert!(workflow.contains("submodules: recursive"));
    assert_eq!(
        uses,
        [
            "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0",
            "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
        ]
    );
    for action in uses {
        let (_, revision) = action
            .rsplit_once('@')
            .expect("every workflow action should have a revision");
        assert_eq!(revision.len(), 40);
        assert!(revision.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
}

#[test]
fn workflow_checks_exact_producer_upstream_and_tool_identities() {
    // Arrange
    let workflow = producer_workflow();

    // Act
    let required_contract = [
        r#"test "$(git rev-parse HEAD)" = "$GITHUB_SHA""#,
        r#"test -z "$(git status --porcelain --untracked-files=no)""#,
        r#"test "$(git -C third_party/liquidfun rev-parse HEAD)" = "7f20402173fd143a3988c921bc384459c6a858f2""#,
        "rustc 1.97.0",
        "cmake version 4.3.3",
        r#"test "$(ninja --version)" = "1.13.2""#,
        r#"clang version 22\.1\.8"#,
        "oracle-debug",
    ];

    // Assert
    for required in required_contract {
        assert!(
            workflow.contains(required),
            "workflow is missing exact identity check: {required}"
        );
    }
}

#[test]
fn workflow_invokes_one_aggregate_producer_and_rechecks_the_bundle() {
    // Arrange
    let workflow = producer_workflow();

    // Act
    let producer_invocations = workflow
        .match_indices("cargo xtask phase13 evidence produce")
        .count();

    // Assert
    assert_eq!(producer_invocations, 1);
    assert!(workflow.contains("--staging-root target/phase13/staged"));
    assert!(workflow.contains(r#"--producer-sha "$GITHUB_SHA""#));
    assert!(workflow.contains("cargo xtask phase13 evidence check"));
    assert!(workflow.contains(r#"--expected-producer-sha "$GITHUB_SHA""#));
    assert!(workflow.contains(r#"--expected-bundle-sha256 "$bundle_sha256""#));
}

#[test]
fn workflow_uploads_one_unique_bundle_and_publishes_the_acquisition_tuple() {
    // Arrange
    let workflow = producer_workflow();

    // Act
    let acquisition_fields = [
        "producer-run-id:",
        "artifact-id:",
        "artifact-name:",
        "artifact-url:",
        "provider-digest:",
        "producer-sha:",
        "bundle-sha256:",
    ];

    // Assert
    assert!(workflow.contains("phase13-staged-${{ github.run_id }}-${{ github.sha }}"));
    assert!(workflow.contains("if-no-files-found: error"));
    assert!(workflow.contains("retention-days: 90"));
    assert!(workflow.contains("GITHUB_STEP_SUMMARY"));
    for field in acquisition_fields {
        assert!(
            workflow.contains(field),
            "workflow is missing acquisition field: {field}"
        );
    }
}

#[test]
fn workflow_cannot_promote_or_mutate_tracked_evidence() {
    // Arrange
    let workflow = producer_workflow();

    // Act
    let forbidden_contract = [
        "reference/artifacts",
        "git add",
        "git commit",
        "git push",
        "download-artifact",
        "workflow_run",
        "promotion",
    ];

    // Assert
    for forbidden in forbidden_contract {
        assert!(
            !workflow.contains(forbidden),
            "producer workflow must not contain mutable operation: {forbidden}"
        );
    }
}
