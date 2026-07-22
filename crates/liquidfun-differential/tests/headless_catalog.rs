//! Complete headless catalog/controller/capture/comparison capability gate.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use liquidfun_differential::{
    CatalogRunOutcome, NativeCatalogBackend, RunSettingsInput, SessionCommand, SessionController,
    SessionControllerErrorKind, SessionState, compare_catalog, execute_catalog_native,
    replay_catalog_exact_native,
};
use liquidfun_test_protocol::{
    ActionSchedule, CatalogDefinition, CatalogErrorKind, CatalogProgram, CatalogRunRequest,
    CatalogSlug, EvidenceTier, FloatBits, GeneratorId, GeneratorVersion, HarnessLimits, RequestId,
    ResolveRequest, RunProvenanceRequirements, RunSettings, ScenarioEligibility, ScenarioVersion,
    Sha256Hex, Vec2Bits, encode_catalog_run_request_jsonl, resolve_catalog,
    scenarios::scenario_definitions,
};

fn settings() -> RunSettings {
    RunSettings::new(FloatBits::from_f32(1.0 / 60.0), 8, 3, 8)
        .expect("reviewed settings should validate")
}

fn resolved(slug: &str) -> liquidfun_test_protocol::ResolvedScenario {
    resolve_catalog(
        &scenario_definitions().expect("catalog should validate"),
        &ResolveRequest::new(
            CatalogSlug::new(slug).expect("slug should validate"),
            None,
            settings(),
        ),
    )
    .expect("named scenario should resolve")
}

fn request(slug: &str) -> CatalogRunRequest {
    CatalogRunRequest::new(
        RequestId::new(format!("headless-{slug}")).expect("request ID should validate"),
        resolved(slug),
        RunProvenanceRequirements::new(
            Sha256Hex::new("1".repeat(64)).expect("identity should validate"),
            HarnessLimits::phase2_default_v1().profile_sha256(),
            EvidenceTier::D3Exploratory,
        ),
    )
    .expect("request should validate")
}

fn submit(controller: &mut SessionController<NativeCatalogBackend>, command: SessionCommand) {
    let command_id = controller
        .next_command_id()
        .expect("command identity should remain available");
    controller
        .submit(command_id, command)
        .expect("headless command should succeed");
}

#[test]
fn named_and_seeded_resolution_are_deterministic_and_strict() {
    // Arrange
    let named = resolved("rigid-runtime-mutation");
    let seeded = CatalogDefinition::new(
        CatalogSlug::new("seeded-headless-gravity").expect("slug should validate"),
        "Seeded headless gravity",
        ScenarioVersion::CURRENT,
        GeneratorId::new("phase11-headless-seed").expect("generator ID should validate"),
        GeneratorVersion::CURRENT,
        ScenarioEligibility::SeedRequired,
        Vec::new(),
        CatalogProgram::seeded_gravity_choices(
            vec![
                Vec2Bits {
                    x_bits: FloatBits::from_f32(0.0),
                    y_bits: FloatBits::from_f32(-10.0),
                },
                Vec2Bits {
                    x_bits: FloatBits::from_f32(1.0),
                    y_bits: FloatBits::from_f32(-9.0),
                },
            ],
            1,
        )
        .expect("seeded program should validate"),
    )
    .expect("seeded definition should validate");
    let request = ResolveRequest::new(seeded.slug().clone(), Some(42), settings());

    // Act
    let first = resolve_catalog(std::slice::from_ref(&seeded), &request)
        .expect("seeded scenario should resolve");
    let second = resolve_catalog(std::slice::from_ref(&seeded), &request)
        .expect("same seed should resolve again");
    let missing_seed = resolve_catalog(
        std::slice::from_ref(&seeded),
        &ResolveRequest::new(seeded.slug().clone(), None, settings()),
    );

    // Assert
    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
    assert_eq!(first.identity().maybe_seed(), Some(42));
    assert_eq!(named.identity().maybe_seed(), None);
    assert_eq!(
        missing_seed.expect_err("seed is mandatory").kind(),
        CatalogErrorKind::SeedRequired
    );
}

#[test]
fn controller_pause_step_restart_settings_and_scenario_actions_are_headless() {
    // Arrange
    let selected = resolved("particle-system-pause-action");
    let original_bytes = selected.canonical_bytes().to_vec();
    let scenario_action = selected
        .actions()
        .iter()
        .find(|action| {
            matches!(
                action.schedule(),
                ActionSchedule::LogicalStep { ordinal: 1 }
            )
        })
        .expect("scenario should declare its first logical action")
        .action_id()
        .clone();
    let mut controller = SessionController::new(NativeCatalogBackend::new());
    submit(
        &mut controller,
        SessionCommand::Select { resolved: selected },
    );

    // Act / Assert: pause performs no tick.
    submit(&mut controller, SessionCommand::Run);
    submit(&mut controller, SessionCommand::Pause);
    assert_eq!(controller.state(), SessionState::ReadyPaused);
    assert_eq!(controller.completed_logical_steps(), 0);

    // Act / Assert: step performs exactly one tick and remains paused.
    submit(&mut controller, SessionCommand::StepOnce);
    assert_eq!(controller.state(), SessionState::ReadyPaused);
    assert_eq!(controller.completed_logical_steps(), 1);

    // Act / Assert: restart reconstructs identical authority.
    submit(&mut controller, SessionCommand::Restart);
    assert_eq!(controller.completed_logical_steps(), 0);
    assert_eq!(
        controller
            .selected()
            .expect("selection should survive restart")
            .canonical_bytes(),
        original_bytes
    );

    // Act / Assert: invalid settings fail before replacement or native effects.
    let replacement = controller
        .selected()
        .expect("selection should remain available")
        .clone();
    let command_id = controller
        .next_command_id()
        .expect("command identity should remain available");
    let error = controller
        .submit(
            command_id,
            SessionCommand::ApplySettingsAndRestart {
                settings: RunSettingsInput::new(FloatBits::from_f32(0.0), 8, 3, 8),
                resolved: replacement,
            },
        )
        .expect_err("zero timestep should fail before restart");
    assert_eq!(error.kind(), SessionControllerErrorKind::InvalidRunSettings);
    assert_eq!(controller.state(), SessionState::ReadyPaused);
    assert_eq!(
        RunSettings::new(FloatBits::from_f32(1.0 / 60.0), 0, 3, 8)
            .expect_err("zero iterations should fail")
            .kind(),
        CatalogErrorKind::InvalidRunSettings
    );

    // Act / Assert: declared actions are the only action boundary.
    submit(
        &mut controller,
        SessionCommand::ApplyScenarioAction {
            action_id: scenario_action,
        },
    );
    assert_eq!(controller.state(), SessionState::ReadyPaused);
}

#[test]
fn every_representative_family_executes_and_captures_without_a_display() {
    // Arrange
    let slugs = [
        "rigid-runtime-mutation",
        "joint-revolute-behavior",
        "standalone-rope-evolution",
        "particle-system-pause-action",
        "particle-group-construction-append",
        "particle-aabb-query-controls",
        "rigid-callback-timing",
        "particle-mutations",
    ];

    // Act / Assert
    for slug in slugs {
        let capture = execute_catalog_native(&request(slug))
            .unwrap_or_else(|error| panic!("{slug} failed: {:?}", error.kind()));
        assert!(!capture.checkpoints().is_empty(), "{slug}");
        assert!(!capture.canonical_checkpoint_bytes().is_empty(), "{slug}");
    }
}

#[test]
fn capture_replay_and_comparison_share_exact_resolved_bytes() {
    // Arrange
    let request = request("particle-group-construction-append");
    let encoded = encode_catalog_run_request_jsonl(&request, &HarnessLimits::phase2_default_v1())
        .expect("request should encode");

    // Act
    let captured = execute_catalog_native(&request).expect("native capture should succeed");
    let replayed = replay_catalog_exact_native(&encoded).expect("exact replay should succeed");
    let outcome = compare_catalog(&captured, &replayed).expect("captures should compare");

    // Assert
    assert_eq!(captured.resolved_bytes(), replayed.resolved_bytes());
    assert_eq!(
        captured.canonical_checkpoint_bytes(),
        replayed.canonical_checkpoint_bytes()
    );
    assert!(matches!(outcome, CatalogRunOutcome::Match(_)));
}

#[test]
fn missing_oracle_is_an_explicit_prerequisite_and_never_a_match() {
    // Arrange
    let repository = EmptyOracleRepository::new();
    let output = Command::new(env!("CARGO_BIN_EXE_liquidfun-differential"))
        .current_dir(&repository.root)
        .args([
            "catalog",
            "compare",
            "--scenario",
            "rigid-runtime-mutation",
            "--seed",
            "none",
            "--timestep",
            "0.016666668",
            "--velocity-iterations",
            "8",
            "--position-iterations",
            "3",
            "--particle-iterations",
            "8",
            "--oracle-preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
            "--output",
            "human",
            "--commands",
            "auto",
        ])
        .output()
        .expect("headless differential command should start");

    // Act
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assert
    assert_eq!(output.status.code(), Some(69));
    assert!(stderr.contains("catalog/oracle-unavailable: pinned oracle executable is unavailable"));
    assert!(!stderr.to_ascii_lowercase().contains("match:"));
}

struct EmptyOracleRepository {
    root: PathBuf,
}

impl EmptyOracleRepository {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "liquidfun-headless-no-oracle-{}-{id}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("stale fixture should be removable");
        }
        fs::create_dir_all(root.join("reference"))
            .expect("empty reference directory should be creatable");
        fs::write(root.join("Cargo.toml"), "[workspace]\n")
            .expect("workspace marker should be writable");
        Self { root }
    }
}

impl Drop for EmptyOracleRepository {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.root) {
            assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
        }
    }
}
