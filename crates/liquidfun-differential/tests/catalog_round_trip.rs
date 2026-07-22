//! End-to-end resolved catalog replay and comparison coverage.

use liquidfun_differential::{
    CatalogOracleSupervisor, CatalogRunCapture, CatalogRunOutcome, NativeCatalogBackend,
    OracleExecutable, OraclePreset, SessionCommand, SessionController, SessionProfile,
    SessionState, compare_catalog, execute_catalog_native, replay_catalog_exact_native,
};
use liquidfun_test_protocol::{
    CatalogRunRequest, CatalogSlug, EvidenceTier, FloatBits, HarnessLimits, RequestId,
    ResolveRequest, RunProvenanceRequirements, RunSettings, Sha256Hex,
    encode_catalog_run_request_jsonl, resolve_catalog, scenarios::scenario_definitions,
};

fn request() -> CatalogRunRequest {
    request_with_provenance(
        Sha256Hex::new("1".repeat(64)).expect("identity should validate"),
        HarnessLimits::phase2_default_v1().profile_sha256(),
    )
}

fn request_with_provenance(identity: Sha256Hex, limits_profile: Sha256Hex) -> CatalogRunRequest {
    request_for_provenance("rigid-runtime-mutation", 8, identity, limits_profile)
}

fn request_for_provenance(
    slug: &str,
    particle_iterations: u32,
    identity: Sha256Hex,
    limits_profile: Sha256Hex,
) -> CatalogRunRequest {
    let definitions = scenario_definitions().expect("catalog definitions should validate");
    let settings = RunSettings::new(FloatBits::from_f32(1.0 / 60.0), 8, 3, particle_iterations)
        .expect("reviewed settings should validate");
    let resolved = resolve_catalog(
        &definitions,
        &ResolveRequest::new(
            CatalogSlug::new(slug).expect("slug should validate"),
            None,
            settings,
        ),
    )
    .expect("catalog request should resolve");
    CatalogRunRequest::new(
        RequestId::new("catalog-round-trip").expect("request ID should validate"),
        resolved,
        RunProvenanceRequirements::new(identity, limits_profile, EvidenceTier::D3Exploratory),
    )
    .expect("run request should validate")
}

#[test]
fn cpp_catalog_accepts_a_reviewed_joint_scenario() {
    // Arrange
    const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
    let root = repository_root();
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: configure and build oracle-debug for catalog integration");
        return;
    };
    let mut supervisor =
        CatalogOracleSupervisor::new(executable, SessionProfile::OneShot, REVISION);
    let identity = supervisor
        .discover_identity()
        .expect("oracle handshake should validate");
    let request = request_for_provenance(
        "joint-distance-behavior",
        1,
        identity.identity_sha256().clone(),
        supervisor.limits_profile_sha256(),
    );

    // Act
    let result = supervisor.execute(&request);

    // Assert
    let capture = result.unwrap_or_else(|error| {
        panic!(
            "joint scenario failed as {:?}: {}",
            error.kind(),
            String::from_utf8_lossy(error.retained_stderr())
        )
    });
    assert_eq!(
        capture.capture().checkpoints().len(),
        request.resolved().checkpoints().len()
    );
}

#[test]
fn cpp_catalog_accepts_phase11_rigid_stack() {
    // Arrange
    const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
    let root = repository_root();
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: configure and build oracle-debug for catalog integration");
        return;
    };
    let mut supervisor =
        CatalogOracleSupervisor::new(executable, SessionProfile::OneShot, REVISION);
    let identity = supervisor
        .discover_identity()
        .expect("oracle handshake should validate");
    let request = request_for_provenance(
        "rigid-stack-stability",
        1,
        identity.identity_sha256().clone(),
        supervisor.limits_profile_sha256(),
    );

    // Act
    let result = supervisor.execute(&request);

    // Assert
    let capture = result.unwrap_or_else(|error| {
        panic!(
            "rigid stack failed as {:?}: {}",
            error.kind(),
            String::from_utf8_lossy(error.retained_stderr())
        )
    });
    assert_eq!(
        capture.capture().checkpoints().len(),
        request.resolved().checkpoints().len()
    );
}

fn repository_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn submit(controller: &mut SessionController<NativeCatalogBackend>, command: SessionCommand) {
    let command_id = controller
        .next_command_id()
        .expect("test command identity should remain available");
    controller
        .submit(command_id, command)
        .expect("catalog controller command should succeed");
}

#[test]
fn exact_resolved_bytes_feed_native_replay_and_comparison() {
    // Arrange
    let request = request();
    let request_bytes =
        encode_catalog_run_request_jsonl(&request, &HarnessLimits::phase2_default_v1())
            .expect("request should encode");

    // Act
    let first = execute_catalog_native(&request).expect("native run should execute");
    let replay = replay_catalog_exact_native(&request_bytes).expect("exact replay should execute");
    let outcome = compare_catalog(&first, &replay).expect("same run should compare");

    // Assert
    assert!(matches!(outcome, CatalogRunOutcome::Match(_)));
    assert_eq!(first.resolved_bytes(), request.resolved().canonical_bytes());
    assert_eq!(first.resolved_bytes(), replay.resolved_bytes());
    assert_eq!(first.action_log(), replay.action_log());
    assert_eq!(first.checkpoint_schedule(), replay.checkpoint_schedule());
    assert_eq!(first.checkpoints(), replay.checkpoints());
}

#[test]
fn native_replay_is_d0_byte_identical() {
    // Arrange
    let request = request();

    // Act
    let first = execute_catalog_native(&request).expect("first run should execute");
    let second = execute_catalog_native(&request).expect("second run should execute");

    // Assert
    assert_eq!(
        first.canonical_checkpoint_bytes(),
        second.canonical_checkpoint_bytes()
    );
}

#[test]
fn cpp_catalog_reuses_one_supervised_child_with_reset_proof() {
    // Arrange
    const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
    let root = repository_root();
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: configure and build the oracle-debug preset for catalog integration");
        return;
    };
    let mut supervisor = CatalogOracleSupervisor::new(executable, SessionProfile::Reuse, REVISION);
    let identity = supervisor
        .discover_identity()
        .expect("oracle handshake should validate");
    let request = request_with_provenance(
        identity.identity_sha256().clone(),
        supervisor.limits_profile_sha256(),
    );

    // Act
    let first = supervisor
        .execute(&request)
        .expect("first catalog request should complete");
    let second = supervisor
        .execute(&request)
        .expect("second catalog request should reuse cleanly");

    // Assert
    assert_eq!(
        first.capture().resolved_bytes(),
        request.resolved().canonical_bytes()
    );
    assert_eq!(
        first.capture().checkpoints(),
        second.capture().checkpoints()
    );
    assert_eq!((first.reset_epoch(), second.reset_epoch()), (1, 2));
    assert!(first.reset_verified() && second.reset_verified());
    assert_eq!(supervisor.process_generation(), 1);
    assert!(!first.response_bytes().is_empty());
}

#[test]
fn pause_step_and_restart_preserve_exact_resolved_authority() {
    // Arrange
    let request = request();
    let resolved = request.resolved().clone();
    let first_checkpoint = resolved.checkpoints()[0].checkpoint_id().clone();
    let mut backend = NativeCatalogBackend::new();
    backend.set_request_id(request.request_id().clone());
    let mut controller = SessionController::new(backend);
    submit(
        &mut controller,
        SessionCommand::Select {
            resolved: resolved.clone(),
        },
    );

    // Act
    submit(&mut controller, SessionCommand::Run);
    submit(&mut controller, SessionCommand::Pause);

    // Assert
    assert_eq!(controller.state(), SessionState::ReadyPaused);
    assert_eq!(controller.completed_logical_steps(), 0);
    assert!(controller.captures().is_empty());

    // Act
    submit(&mut controller, SessionCommand::StepOnce);
    submit(
        &mut controller,
        SessionCommand::CaptureCheckpoint {
            checkpoint_id: first_checkpoint.clone(),
        },
    );
    let before_restart =
        serde_json::to_vec(controller.captures()[0].value()).expect("checkpoint should serialize");
    submit(&mut controller, SessionCommand::Restart);

    // Assert
    assert_eq!(controller.completed_logical_steps(), 0);
    assert!(controller.captures().is_empty());
    assert_eq!(
        controller
            .selected()
            .expect("selection should survive restart")
            .canonical_bytes(),
        resolved.canonical_bytes()
    );

    // Act
    submit(&mut controller, SessionCommand::StepOnce);
    submit(
        &mut controller,
        SessionCommand::CaptureCheckpoint {
            checkpoint_id: first_checkpoint,
        },
    );
    let after_restart =
        serde_json::to_vec(controller.captures()[0].value()).expect("checkpoint should serialize");

    // Assert
    assert_eq!(before_restart, after_restart);
}

#[test]
fn completed_semantic_difference_is_a_physics_mismatch() {
    // Arrange
    let request = request();
    let native = execute_catalog_native(&request).expect("native run should execute");
    let mut records = native
        .canonical_checkpoint_bytes()
        .iter()
        .map(|bytes| bytes.to_vec())
        .collect::<Vec<_>>();
    let mut changed: serde_json::Value =
        serde_json::from_slice(&records[0]).expect("checkpoint should be JSON");
    changed["observations"][0]["value"]["value"] = serde_json::json!(999);
    records[0] = serde_json::to_vec(&changed).expect("changed checkpoint should encode");
    records[0].push(b'\n');
    let oracle = CatalogRunCapture::from_checkpoint_jsonl(&request, &records)
        .expect("changed semantic checkpoint should remain structurally valid");

    // Act
    let outcome = compare_catalog(&native, &oracle).expect("completed runs should compare");

    // Assert
    assert!(matches!(outcome, CatalogRunOutcome::PhysicsMismatch(_)));
}
