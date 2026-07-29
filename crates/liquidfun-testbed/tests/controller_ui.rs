//! Controller, input, settings, semantic viewport, and diagnostic screenshot contracts.

use std::path::Path;

use liquidfun::DebugLayer;
use liquidfun_differential::{SessionCommand, SessionState};
use liquidfun_test_protocol::{CheckpointId, FloatBits, RunSettings, ScenarioActionId};
use liquidfun_testbed::{
    controller_adapter::{
        ControlCapability, ControllerAction, ControllerAdapter, ControllerAdapterError,
        ControllerProjection, PARTICLE_PAUSE_ACTION_LABEL, SESSION_PAUSED_LABEL,
    },
    input::{
        InputContext, InputEffect, KeyboardKey, PresentationAction, ScenarioShortcut, resolve_key,
    },
    interactive::InteractiveTestbed,
    ui::{
        SCREENSHOT_CLARIFICATION,
        overlays::{DiagnosticProfile, OverlayKind, OverlayState},
        run_controls::{RunControl, run_controls},
        settings::{
            APPLY_LABEL, ITERATION_GUIDANCE, SettingsEditor, SettingsField, TIMESTEP_GUIDANCE,
        },
        viewport::{
            DiagnosticScreenshotPath, ScreenPoint, ScreenPrimitive, ScreenSize, SemanticViewport,
            SynchronizedCamera,
        },
    },
};

#[path = "controller_ui/support.rs"]
mod support;
use support::{arrow, point, resolved, selected_controller, settings, submit};

include!("controller_ui/controls.rs");

include!("controller_ui/viewport.rs");
