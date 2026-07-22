//! Human and machine reports with explicit semantic and diagnostic labels.

use liquidfun_differential::{CatalogRunCapture, OraclePreset, SessionProfile, SessionState};
use liquidfun_test_protocol::{CatalogRunRequest, CheckpointDeclaration};
use serde::Serialize;

use super::parse::{CatalogCliError, ExecutionConfig, OutputMode};

#[derive(Serialize)]
pub(super) struct InspectReport<'a> {
    pub(super) record_kind: &'static str,
    pub(super) scenario: &'a str,
    pub(super) title: &'a str,
    pub(super) scenario_version: u32,
    pub(super) seed: &'static str,
    pub(super) timestep_bits: u32,
    pub(super) velocity_iterations: u32,
    pub(super) position_iterations: u32,
    pub(super) particle_iterations: u32,
    pub(super) tags: Vec<&'a str>,
    pub(super) visual_eligible: bool,
}

pub(super) struct RunSnapshot {
    pub(super) state: SessionState,
    pub(super) logical_steps: u32,
    pub(super) checkpoints: Vec<String>,
}

#[derive(Serialize)]
pub(super) struct RunReport<'a> {
    record_kind: &'static str,
    run_identity: String,
    resolved_sha256: &'a str,
    controller_state: &'static str,
    logical_step: u32,
    action_ordinal: u32,
    semantic_checkpoint_ids: Vec<String>,
    comparison_summary: String,
    diagnostic_profile: String,
    screenshot_diagnostic: &'static str,
    wall_time_diagnostic: &'static str,
}

impl<'a> RunReport<'a> {
    pub(super) fn from_snapshot(
        config: &ExecutionConfig,
        request: &'a CatalogRunRequest,
        snapshot: RunSnapshot,
        comparison: &str,
    ) -> Self {
        Self::new(
            config,
            request,
            snapshot.state,
            snapshot.logical_steps,
            snapshot.checkpoints,
            comparison,
        )
    }

    pub(super) fn from_capture(
        config: &ExecutionConfig,
        request: &'a CatalogRunRequest,
        capture: &CatalogRunCapture,
        comparison: &str,
    ) -> Self {
        let checkpoints = capture
            .checkpoint_schedule()
            .iter()
            .map(|checkpoint| checkpoint.checkpoint_id().as_str().to_owned())
            .collect();
        let logical_steps = capture
            .checkpoint_schedule()
            .last()
            .map_or(0, CheckpointDeclaration::logical_step);
        Self::new(
            config,
            request,
            SessionState::Completed,
            logical_steps,
            checkpoints,
            comparison,
        )
    }

    fn new(
        config: &ExecutionConfig,
        request: &'a CatalogRunRequest,
        state: SessionState,
        logical_steps: u32,
        checkpoints: Vec<String>,
        comparison: &str,
    ) -> Self {
        Self {
            record_kind: "catalog_run_report",
            run_identity: format!(
                "{}@{}",
                request.resolved().identity().slug().as_str(),
                request.resolved().identity().scenario_version().get()
            ),
            resolved_sha256: request.resolved().identity().content_sha256().as_str(),
            controller_state: state_name(state),
            logical_step: logical_steps,
            action_ordinal: logical_steps,
            semantic_checkpoint_ids: checkpoints,
            comparison_summary: comparison.to_owned(),
            diagnostic_profile: format!(
                "oracle_preset={};session_profile={}",
                preset_name(config.preset),
                profile_name(config.profile)
            ),
            screenshot_diagnostic: "none; screenshots are diagnostic and do not prove compatibility",
            wall_time_diagnostic: "not_recorded; timing is diagnostic and not compatibility evidence",
        }
    }
}

pub(super) fn render_inspection(
    mode: OutputMode,
    report: &InspectReport<'_>,
) -> Result<(), CatalogCliError> {
    match mode {
        OutputMode::Human => {
            println!("scenario: {}", report.scenario);
            println!("title: {}", report.title);
            println!("scenario_version: {}", report.scenario_version);
            println!("seed: {}", report.seed);
            println!("timestep_bits: {}", report.timestep_bits);
            println!(
                "iterations: velocity={} position={} particle={}",
                report.velocity_iterations, report.position_iterations, report.particle_iterations
            );
        }
        OutputMode::Json => println!(
            "{}",
            serde_json::to_string(report)
                .map_err(|_| CatalogCliError::harness("inspection serialization failed"))?
        ),
    }
    Ok(())
}

pub(super) fn render_run(mode: OutputMode, report: &RunReport<'_>) -> Result<(), CatalogCliError> {
    match mode {
        OutputMode::Human => {
            println!("run_identity: {}", report.run_identity);
            println!("resolved_sha256: {}", report.resolved_sha256);
            println!("controller_state: {}", report.controller_state);
            println!("logical_step: {}", report.logical_step);
            println!("action_ordinal: {}", report.action_ordinal);
            println!(
                "semantic_checkpoint_ids: {}",
                report.semantic_checkpoint_ids.join(",")
            );
            println!("comparison_summary: {}", report.comparison_summary);
            println!("diagnostic_profile: {}", report.diagnostic_profile);
            println!("screenshot_diagnostic: {}", report.screenshot_diagnostic);
            println!("wall_time_diagnostic: {}", report.wall_time_diagnostic);
        }
        OutputMode::Json => println!(
            "{}",
            serde_json::to_string(report)
                .map_err(|_| CatalogCliError::harness("run serialization failed"))?
        ),
    }
    Ok(())
}

const fn state_name(state: SessionState) -> &'static str {
    match state {
        SessionState::NoSelection => "no_selection",
        SessionState::Resolving => "resolving",
        SessionState::ReadyPaused => "ready_paused",
        SessionState::Running => "running",
        SessionState::Stepping => "stepping",
        SessionState::Comparing => "comparing",
        SessionState::Completed => "completed",
        SessionState::RecoverableError => "recoverable_error",
        SessionState::HarnessFailure => "harness_failure",
    }
}

const fn preset_name(preset: OraclePreset) -> &'static str {
    match preset {
        OraclePreset::Debug => "oracle-debug",
        OraclePreset::Release => "oracle-release",
        OraclePreset::AsanUbsan => "oracle-asan-ubsan",
    }
}

const fn profile_name(profile: SessionProfile) -> &'static str {
    match profile {
        SessionProfile::OneShot => "one-shot",
        SessionProfile::Reuse => "reuse",
        SessionProfile::Sanitizer => "sanitizer",
    }
}
