//! Closed keyboard translation into typed controller or presentation-only actions.

#![allow(
    missing_docs,
    reason = "closed key and presentation variants are named by the UI contract"
)]

use liquidfun_differential::SessionState;
use liquidfun_test_protocol::{CheckpointId, ScenarioActionId};

use crate::controller_adapter::ControllerAction;

pub const MODULE_NAME: &str = "input";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardKey {
    Space,
    Right,
    R,
    C,
    Slash,
    F,
    LeftBracket,
    RightBracket,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Home,
    QuestionMark,
    Escape,
    Scenario(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationAction {
    FocusScenarioSearch,
    FocusDifference,
    PreviousDifference,
    NextDifference,
    ToggleOverlayGroup(u8),
    ResetCamera,
    OpenShortcutHelp,
    CloseTopmostOrClearFocus,
}

#[derive(Debug, Clone)]
pub enum InputEffect {
    Controller(ControllerAction),
    Presentation(PresentationAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioShortcut {
    key: char,
    action_id: ScenarioActionId,
    label: Box<str>,
}

impl ScenarioShortcut {
    #[must_use]
    pub fn new(key: char, action_id: ScenarioActionId, label: &str) -> Option<Self> {
        if !key.is_ascii_graphic()
            || is_reserved_scenario_key(key)
            || label.is_empty()
            || label.len() > 64
            || !label.is_ascii()
            || label.bytes().any(|byte| byte.is_ascii_control())
        {
            return None;
        }
        Some(Self {
            key: key.to_ascii_lowercase(),
            action_id,
            label: label.into(),
        })
    }

    #[must_use]
    pub const fn key(&self) -> char {
        self.key
    }

    #[must_use]
    pub const fn action_id(&self) -> &ScenarioActionId {
        &self.action_id
    }

    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Immutable input snapshot. Optional stable IDs are supplied by the controller model.
#[derive(Debug, Clone, Copy)]
pub struct InputContext<'a> {
    pub session_state: SessionState,
    pub editing_field: bool,
    pub maybe_checkpoint_id: Option<&'a CheckpointId>,
    pub scenario_shortcuts: &'a [ScenarioShortcut],
}

/// Resolves one key without advancing the run or creating a checkpoint itself.
#[must_use]
pub fn resolve_key(key: KeyboardKey, context: InputContext<'_>) -> Option<InputEffect> {
    if context.editing_field {
        return None;
    }
    match key {
        KeyboardKey::Space => pause_or_run(context.session_state),
        KeyboardKey::Right => step_once(context.session_state),
        KeyboardKey::R => restart(context.session_state),
        KeyboardKey::C => capture(context),
        KeyboardKey::Slash => Some(presentation(PresentationAction::FocusScenarioSearch)),
        KeyboardKey::F => Some(presentation(PresentationAction::FocusDifference)),
        KeyboardKey::LeftBracket => Some(presentation(PresentationAction::PreviousDifference)),
        KeyboardKey::RightBracket => Some(presentation(PresentationAction::NextDifference)),
        KeyboardKey::Digit1 => Some(presentation(PresentationAction::ToggleOverlayGroup(1))),
        KeyboardKey::Digit2 => Some(presentation(PresentationAction::ToggleOverlayGroup(2))),
        KeyboardKey::Digit3 => Some(presentation(PresentationAction::ToggleOverlayGroup(3))),
        KeyboardKey::Digit4 => Some(presentation(PresentationAction::ToggleOverlayGroup(4))),
        KeyboardKey::Home => Some(presentation(PresentationAction::ResetCamera)),
        KeyboardKey::QuestionMark => Some(presentation(PresentationAction::OpenShortcutHelp)),
        KeyboardKey::Escape => Some(presentation(PresentationAction::CloseTopmostOrClearFocus)),
        KeyboardKey::Scenario(key) => scenario_action(key, context.scenario_shortcuts),
    }
}

fn pause_or_run(state: SessionState) -> Option<InputEffect> {
    let action = match state {
        SessionState::Running => ControllerAction::Pause,
        SessionState::ReadyPaused => ControllerAction::Run,
        _ => return None,
    };
    Some(InputEffect::Controller(action))
}

fn step_once(state: SessionState) -> Option<InputEffect> {
    if !matches!(state, SessionState::ReadyPaused | SessionState::Running) {
        return None;
    }
    Some(InputEffect::Controller(ControllerAction::StepOnce))
}

fn restart(state: SessionState) -> Option<InputEffect> {
    let restartable = matches!(
        state,
        SessionState::ReadyPaused
            | SessionState::Running
            | SessionState::Completed
            | SessionState::RecoverableError
            | SessionState::HarnessFailure
    );
    restartable.then_some(InputEffect::Controller(ControllerAction::Restart))
}

fn capture(context: InputContext<'_>) -> Option<InputEffect> {
    let capturable = matches!(
        context.session_state,
        SessionState::ReadyPaused | SessionState::Running | SessionState::Completed
    );
    if !capturable {
        return None;
    }
    let checkpoint_id = context.maybe_checkpoint_id?.clone();
    Some(InputEffect::Controller(
        ControllerAction::CaptureCheckpoint(checkpoint_id),
    ))
}

fn scenario_action(key: char, shortcuts: &[ScenarioShortcut]) -> Option<InputEffect> {
    let normalized = key.to_ascii_lowercase();
    let shortcut = shortcuts.iter().find(|entry| entry.key == normalized)?;
    Some(InputEffect::Controller(
        ControllerAction::ApplyScenarioAction(shortcut.action_id.clone()),
    ))
}

const fn presentation(action: PresentationAction) -> InputEffect {
    InputEffect::Presentation(action)
}

fn is_reserved_scenario_key(key: char) -> bool {
    matches!(
        key.to_ascii_lowercase(),
        'r' | 'c' | 'f' | '/' | '[' | ']' | '1' | '2' | '3' | '4' | '?' | ' '
    )
}
