//! Macroquad keyboard events translated into presentation-only typed intents.

use macroquad::input::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputIntent {
    ToggleRunPause,
    StepOnce,
    Restart,
    FocusNext,
    CaptureSemanticCheckpoint,
    SaveDiagnosticScreenshot,
}

const EXPECTED_BINDINGS: [(KeyCode, InputIntent); 6] = [
    (KeyCode::Space, InputIntent::ToggleRunPause),
    (KeyCode::N, InputIntent::StepOnce),
    (KeyCode::R, InputIntent::Restart),
    (KeyCode::Tab, InputIntent::FocusNext),
    (KeyCode::C, InputIntent::CaptureSemanticCheckpoint),
    (KeyCode::F12, InputIntent::SaveDiagnosticScreenshot),
];

pub(super) fn verified_keyboard_binding_count() -> usize {
    EXPECTED_BINDINGS
        .iter()
        .filter(|(key, expected)| map_key(*key) == Some(*expected))
        .count()
}

const fn map_key(key: KeyCode) -> Option<InputIntent> {
    match key {
        KeyCode::Space => Some(InputIntent::ToggleRunPause),
        KeyCode::N => Some(InputIntent::StepOnce),
        KeyCode::R => Some(InputIntent::Restart),
        KeyCode::Tab => Some(InputIntent::FocusNext),
        KeyCode::C => Some(InputIntent::CaptureSemanticCheckpoint),
        KeyCode::F12 => Some(InputIntent::SaveDiagnosticScreenshot),
        _ => None,
    }
}
