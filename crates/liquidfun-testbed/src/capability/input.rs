//! Replacement-renderer keyboard events translated into presentation-only typed intents.

use crate::renderer::SemanticKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputIntent {
    ToggleRunPause,
    StepOnce,
    Restart,
    FocusNext,
    CaptureSemanticCheckpoint,
    SaveDiagnosticScreenshot,
}

const EXPECTED_BINDINGS: [(SemanticKey, InputIntent); 6] = [
    (SemanticKey::Space, InputIntent::ToggleRunPause),
    (SemanticKey::Character('n'), InputIntent::StepOnce),
    (SemanticKey::Character('r'), InputIntent::Restart),
    (SemanticKey::Enter, InputIntent::FocusNext),
    (
        SemanticKey::Character('c'),
        InputIntent::CaptureSemanticCheckpoint,
    ),
    (
        SemanticKey::Character('s'),
        InputIntent::SaveDiagnosticScreenshot,
    ),
];

pub(super) fn verified_keyboard_binding_count() -> usize {
    EXPECTED_BINDINGS
        .iter()
        .filter(|(key, expected)| map_key(*key) == Some(*expected))
        .count()
}

const fn map_key(key: SemanticKey) -> Option<InputIntent> {
    match key {
        SemanticKey::Space => Some(InputIntent::ToggleRunPause),
        SemanticKey::Character('n') => Some(InputIntent::StepOnce),
        SemanticKey::Character('r') => Some(InputIntent::Restart),
        SemanticKey::Enter => Some(InputIntent::FocusNext),
        SemanticKey::Character('c') => Some(InputIntent::CaptureSemanticCheckpoint),
        SemanticKey::Character('s') => Some(InputIntent::SaveDiagnosticScreenshot),
        _ => None,
    }
}
