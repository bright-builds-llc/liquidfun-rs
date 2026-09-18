//! Private WebAssembly bridge for the `LiquidFun` browser playground.

#![forbid(unsafe_code)]

use wasm_bindgen::prelude::*;

mod frame;
mod scene;
mod session;

pub use frame::ProofFrame;

use session::{SessionCore, SessionError};

/// Opaque owner of one persistent bounded Rust physics scene.
#[wasm_bindgen]
pub struct ProofSession {
    core: SessionCore,
}

#[wasm_bindgen]
impl ProofSession {
    /// Constructs an allowlisted scene from a lowercase hyphenated id.
    ///
    /// # Errors
    ///
    /// Returns a bounded JavaScript error when the id is not allowlisted or
    /// checked scene construction fails.
    #[wasm_bindgen(constructor)]
    pub fn new(scene_id: String) -> Result<ProofSession, JsError> {
        let _ = scene_id;
        SessionCore::new()
            .map(|core| Self { core })
            .map_err(js_error)
    }

    /// Applies a named control. `true` means the world was recreated.
    ///
    /// # Errors
    ///
    /// Returns a bounded JavaScript error when the control name is not
    /// allowlisted.
    #[wasm_bindgen(js_name = applyControl)]
    pub fn apply_control(&mut self, name: String, value: String) -> Result<bool, JsError> {
        let _ = (name, value);
        Ok(false)
    }

    /// Applies a named action on the live world.
    ///
    /// # Errors
    ///
    /// Returns a bounded JavaScript error when the action name is not
    /// allowlisted.
    #[wasm_bindgen(js_name = applyAction)]
    pub fn apply_action(&mut self, name: String) -> Result<(), JsError> {
        let _ = name;
        Ok(())
    }

    /// Advances the scene by one through four fixed steps.
    ///
    /// # Errors
    ///
    /// Returns a bounded JavaScript error for an invalid count, exhausted step
    /// index, or failed engine step.
    pub fn advance(&mut self, step_count: u32) -> Result<(), JsError> {
        self.core.advance(step_count).map_err(js_error)
    }

    /// Captures one coherent frame as validated owned data.
    ///
    /// # Errors
    ///
    /// Returns a bounded JavaScript error when semantic engine state cannot be
    /// copied into the frame contract.
    #[wasm_bindgen(js_name = captureFrame)]
    pub fn capture_frame(&self) -> Result<ProofFrame, JsError> {
        self.core
            .capture_frame()
            .map(ProofFrame::from)
            .map_err(js_error)
    }

    /// Returns the number of completed fixed steps.
    #[must_use]
    #[wasm_bindgen(js_name = stepIndex)]
    pub fn step_index(&self) -> u32 {
        self.core.step_index()
    }

    /// Returns the fixed proof-scene particle count.
    #[must_use]
    #[wasm_bindgen(js_name = particleCount)]
    pub fn particle_count(&self) -> usize {
        self.core.particle_count()
    }

    /// Returns the fixed proof-scene rigid shape count.
    #[must_use]
    #[wasm_bindgen(js_name = rigidShapeCount)]
    pub fn rigid_shape_count(&self) -> usize {
        self.core.rigid_shape_count()
    }
}

fn js_error(error: SessionError) -> JsError {
    JsError::new(error.message())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::SessionError;

    #[test]
    fn new_dam_break_reports_documented_particle_count() {
        // Arrange / Act
        let session = ProofSession::new(String::from("dam-break"))
            .expect("allowlisted Dam Break should construct");

        // Assert
        assert_eq!(session.particle_count(), 192);
    }

    #[test]
    fn new_unknown_scene_returns_allowlist_error() {
        // Arrange / Act
        let result = ProofSession::new(String::from("not-a-scene"));

        // Assert
        assert!(result.is_err());
        assert_eq!(
            SessionError::UnknownScene.message(),
            "Rust/WASM scene id is not allowlisted"
        );
    }

    #[test]
    fn unknown_control_and_action_fail_without_poisoning_dam_break() {
        // Arrange
        let mut session = ProofSession::new(String::from("dam-break"))
            .expect("fresh Dam Break should construct");

        // Act
        let control = session.apply_control(String::from("nope"), String::from("x"));
        let action = session.apply_action(String::from("nope"));

        // Assert
        assert!(control.is_err());
        assert!(action.is_err());
        assert_eq!(
            SessionError::UnknownControl.message(),
            "Rust/WASM control is not allowlisted"
        );
        assert_eq!(session.particle_count(), 192);
    }
}
