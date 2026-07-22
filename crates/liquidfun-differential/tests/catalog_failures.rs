//! Failure taxonomy and exact catalog evidence bundle coverage.

use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use liquidfun_differential::{
    CatalogFailureBundleRequest, CatalogFailureKind, CatalogOracleSupervisor, OracleExecutable,
    OraclePreset, SessionProfile, execute_catalog_native, persist_catalog_failure_bundle,
    replay_catalog_failure_bundle,
};
use liquidfun_test_protocol::{
    CatalogRunRequest, CatalogSlug, EvidenceTier, FloatBits, HarnessLimits, RequestId,
    ResolveRequest, RunProvenanceRequirements, RunSettings, Sha256Hex, resolve_catalog,
    scenarios::scenario_definitions,
};

static DIRECTORY_ID: AtomicU64 = AtomicU64::new(1);

fn repository() -> PathBuf {
    let id = DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/catalog-failure-tests")
        .join(format!("{}-{id}", std::process::id()));
    fs::create_dir_all(&root).expect("test repository should be creatable");
    root
}

fn request() -> CatalogRunRequest {
    request_with_provenance(
        Sha256Hex::new("1".repeat(64)).expect("identity should validate"),
        HarnessLimits::phase2_default_v1().profile_sha256(),
    )
}

fn request_with_provenance(identity: Sha256Hex, limits_profile: Sha256Hex) -> CatalogRunRequest {
    let definitions = scenario_definitions().expect("catalog definitions should validate");
    let settings = RunSettings::new(FloatBits::from_f32(1.0 / 60.0), 8, 3, 8)
        .expect("settings should validate");
    let resolved = resolve_catalog(
        &definitions,
        &ResolveRequest::new(
            CatalogSlug::new("rigid-runtime-mutation").expect("slug should validate"),
            None,
            settings,
        ),
    )
    .expect("catalog request should resolve");
    CatalogRunRequest::new(
        RequestId::new("catalog-failure").expect("request ID should validate"),
        resolved,
        RunProvenanceRequirements::new(identity, limits_profile, EvidenceTier::D3Exploratory),
    )
    .expect("run request should validate")
}

fn fake_supervisor(behavior: &str) -> CatalogOracleSupervisor {
    const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
    let root = repository();
    let output = root.join("target/reference/oracle-debug");
    fs::create_dir_all(&output).expect("fake oracle output should be creatable");
    let executable = output.join(if cfg!(windows) {
        "liquidfun-reference.exe"
    } else {
        "liquidfun-reference"
    });
    fs::copy(env!("CARGO_BIN_EXE_liquidfun-fake-oracle"), &executable)
        .expect("fake oracle should copy");
    fs::write(output.join("behavior.txt"), behavior).expect("behavior should write");
    let executable = OracleExecutable::resolve(&root, OraclePreset::Debug)
        .expect("confined fake oracle should resolve");
    CatalogOracleSupervisor::new(executable, SessionProfile::Reuse, REVISION)
}

#[test]
fn catalog_failure_bundle_persists_and_replays_exact_authority() {
    // Arrange
    let root = repository();
    let request = request();
    let capture = execute_catalog_native(&request).expect("native capture should succeed");
    let evidence = CatalogFailureBundleRequest::from_captures(
        CatalogFailureKind::HarnessFailure,
        &request,
        &capture,
        &capture,
        b"catalog harness failure",
        b"{\"controller_state\":\"harness_failure\"}\n",
    )
    .expect("complete exact evidence should validate");

    // Act
    let receipt =
        persist_catalog_failure_bundle(&root, &evidence).expect("bounded bundle should persist");
    let replay = replay_catalog_failure_bundle(&root, receipt.directory())
        .expect("persisted bundle should replay");

    // Assert
    assert_eq!(
        replay.resolved_bytes(),
        request.resolved().canonical_bytes()
    );
    assert_eq!(
        replay.resolved_sha256(),
        request.resolved().identity().content_sha256()
    );
}

#[test]
fn bundle_rejects_seed_only_authority_and_symlinked_boundaries() {
    // Arrange
    let root = repository();
    let request = request();

    // Act
    let incomplete = CatalogFailureBundleRequest::from_seed_only(
        CatalogFailureKind::HarnessFailure,
        request.resolved().identity().maybe_seed(),
    );

    // Assert
    assert!(incomplete.is_err());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let outside = repository();
        fs::create_dir_all(root.join("target")).expect("target should be creatable");
        symlink(&outside, root.join("target/differential"))
            .expect("test symlink should be creatable");
        let capture = execute_catalog_native(&request).expect("native capture should succeed");
        let evidence = CatalogFailureBundleRequest::from_captures(
            CatalogFailureKind::HarnessFailure,
            &request,
            &capture,
            &capture,
            b"catalog harness failure",
            b"{\"controller_state\":\"harness_failure\"}\n",
        )
        .expect("complete evidence should validate");
        assert!(persist_catalog_failure_bundle(&root, &evidence).is_err());
    }
}

#[test]
fn catalog_process_failures_preserve_distinct_taxonomy_and_reap() {
    // Arrange / Act / Assert
    for (behavior, expected) in [
        ("catalog_malformed", CatalogFailureKind::MalformedRecord),
        ("catalog_crash", CatalogFailureKind::ChildProcess),
        ("catalog_timeout", CatalogFailureKind::Timeout),
        ("catalog_reset", CatalogFailureKind::ResetFailure),
    ] {
        let mut supervisor = fake_supervisor(behavior);
        let identity = supervisor
            .discover_identity()
            .expect("fake handshake should validate");
        let request = request_with_provenance(
            identity.identity_sha256().clone(),
            supervisor.limits_profile_sha256(),
        );
        let failure = supervisor
            .execute(&request)
            .expect_err("injected behavior should fail");
        assert_eq!(failure.kind(), expected, "behavior {behavior}");
        assert!(failure.child_reaped(), "behavior {behavior}");
        assert!(failure.child_killed(), "behavior {behavior}");
    }
}

#[test]
fn catalog_provenance_fails_before_request_and_stderr_is_bounded() {
    // Arrange
    let mut provenance_supervisor = fake_supervisor("catalog_valid");
    let _identity = provenance_supervisor
        .discover_identity()
        .expect("fake handshake should validate");
    let wrong = request_with_provenance(
        Sha256Hex::new("f".repeat(64)).expect("wrong identity should be shaped"),
        provenance_supervisor.limits_profile_sha256(),
    );

    // Act
    let provenance = provenance_supervisor
        .execute(&wrong)
        .expect_err("wrong provenance must fail before request write");
    let mut stderr_supervisor = fake_supervisor("catalog_large_stderr_malformed");
    let identity = stderr_supervisor
        .discover_identity()
        .expect("fake handshake should validate");
    let request = request_with_provenance(
        identity.identity_sha256().clone(),
        stderr_supervisor.limits_profile_sha256(),
    );
    let stderr = stderr_supervisor
        .execute(&request)
        .expect_err("malformed response should poison and reap");

    // Assert
    assert_eq!(provenance.kind(), CatalogFailureKind::Provenance);
    assert_eq!(stderr.kind(), CatalogFailureKind::MalformedRecord);
    assert_eq!(stderr.stderr_bytes(), 1024 * 1024);
    assert_eq!(
        stderr.retained_stderr().len(),
        HarnessLimits::phase2_reuse_v1().retained_stderr_bytes()
    );
    assert!(stderr.child_reaped());
}
