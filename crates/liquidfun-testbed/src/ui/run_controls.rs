//! Pure run-control presentation derived from controller state.

#![allow(
    missing_docs,
    reason = "closed run-control fields mirror the UI contract"
)]

use liquidfun_differential::SessionState;

use crate::controller_adapter::{ControlCapability, ControllerProjection};

pub const MODULE_NAME: &str = "run_controls";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunControl {
    RunScenario,
    Pause,
    Resume,
    StepOnce,
    Restart,
    CaptureCheckpoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunControlButton {
    pub control: RunControl,
    pub label: &'static str,
    pub enabled: bool,
    pub primary: bool,
    pub tooltip: &'static str,
}

/// Exact run-strip controls in visual and keyboard traversal order.
#[must_use]
pub const fn run_controls(
    state: SessionState,
    completed_logical_steps: u32,
) -> [RunControlButton; 4] {
    let projection = ControllerProjection::from_state(state);
    let running = matches!(state, SessionState::Running);
    let run_control = if running {
        RunControl::Pause
    } else if matches!(state, SessionState::ReadyPaused) && completed_logical_steps > 0 {
        RunControl::Resume
    } else {
        RunControl::RunScenario
    };
    let run_label = match run_control {
        RunControl::Pause => "Pause",
        RunControl::Resume => "Resume",
        _ => "Run Scenario",
    };
    [
        RunControlButton {
            control: run_control,
            label: run_label,
            enabled: projection.enabled(ControlCapability::Run)
                || projection.enabled(ControlCapability::Pause),
            primary: projection.enabled(ControlCapability::Run)
                || projection.enabled(ControlCapability::Pause),
            tooltip: "Run or pause the controller-owned session",
        },
        RunControlButton {
            control: RunControl::StepOnce,
            label: "Step Once",
            enabled: projection.enabled(ControlCapability::StepOnce),
            primary: false,
            tooltip: "Execute exactly one logical tick and remain paused",
        },
        RunControlButton {
            control: RunControl::Restart,
            label: "Restart",
            enabled: projection.enabled(ControlCapability::Restart),
            primary: false,
            tooltip: "Restart from step 0",
        },
        RunControlButton {
            control: RunControl::CaptureCheckpoint,
            label: "Capture Checkpoint",
            enabled: projection.enabled(ControlCapability::Capture),
            primary: false,
            tooltip: "Capture an authoritative deterministic semantic checkpoint",
        },
    ]
}
