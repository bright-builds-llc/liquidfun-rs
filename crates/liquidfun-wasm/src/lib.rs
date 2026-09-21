//! Private WebAssembly bridge for the `LiquidFun` browser playground.

#![forbid(unsafe_code)]

use wasm_bindgen::prelude::*;

mod frame;
mod scene;
mod session;

pub use frame::ProofFrame;

#[cfg(not(target_arch = "wasm32"))]
pub mod dam_break_bench;
#[cfg(not(target_arch = "wasm32"))]
pub use dam_break_bench::{DamBreakBenchError, DamBreakBenchReport, run_dam_break_bench};
#[cfg(not(target_arch = "wasm32"))]
pub mod dam_break_timers;
#[cfg(not(target_arch = "wasm32"))]
pub use dam_break_timers::{DamBreakTimersError, DamBreakTimersReport, run_dam_break_timers};
#[cfg(not(target_arch = "wasm32"))]
pub mod scene_spot;
#[cfg(not(target_arch = "wasm32"))]
pub use scene_spot::{SceneSpotError, SceneSpotSample, run_scene_spot};

use scene::parse_scene_id;
use session::{SessionCore, SessionError};

/// Opaque owner of one persistent bounded Rust physics scene.
#[wasm_bindgen]
pub struct ProofSession {
    core: SessionCore,
}

#[allow(clippy::needless_pass_by_value)] // wasm-bindgen JS strings are owned.
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
        build_core(&scene_id)
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
        self.core.apply_control(&name, &value).map_err(js_error)
    }

    /// Applies a named action on the live world.
    ///
    /// # Errors
    ///
    /// Returns a bounded JavaScript error when the action name is not
    /// allowlisted.
    #[wasm_bindgen(js_name = applyAction)]
    pub fn apply_action(&mut self, name: String) -> Result<(), JsError> {
        self.core.apply_action(&name).map_err(js_error)
    }

    /// Applies a captured canvas pointer sample in world coordinates.
    ///
    /// # Errors
    ///
    /// Returns a bounded JavaScript error when the kind is not allowlisted or
    /// the coordinates are not finite.
    #[wasm_bindgen(js_name = pointerAction)]
    pub fn pointer_action(
        &mut self,
        kind: String,
        world_x: f32,
        world_y: f32,
    ) -> Result<(), JsError> {
        self.core
            .apply_pointer(&kind, world_x, world_y)
            .map_err(js_error)
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

fn build_core(scene_id: &str) -> Result<SessionCore, SessionError> {
    SessionCore::create(parse_scene_id(scene_id)?)
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
        assert_eq!(session.particle_count(), 1920);
    }

    #[test]
    fn new_unknown_scene_returns_allowlist_error() {
        // Arrange / Act
        let result = build_core("not-a-scene");

        // Assert
        assert_eq!(result.err(), Some(SessionError::UnknownScene));
        assert_eq!(
            SessionError::UnknownScene.message(),
            "Rust/WASM scene id is not allowlisted"
        );
    }

    #[test]
    fn unknown_control_and_action_fail_without_poisoning_dam_break() {
        // Arrange
        let mut core = build_core("dam-break").expect("fresh Dam Break should construct");

        // Act
        let control = core.apply_control("nope", "x");
        let action = core.apply_action("nope");

        // Assert
        assert_eq!(control, Err(SessionError::UnknownControl));
        assert_eq!(action, Err(SessionError::UnknownControl));
        assert_eq!(
            SessionError::UnknownControl.message(),
            "Rust/WASM control is not allowlisted"
        );
        assert_eq!(core.particle_count(), 1920);
    }

    #[test]
    fn apply_pointer_rejects_non_finite_coordinates_without_mutating() {
        // Arrange
        let mut core = build_core("dam-break").expect("fresh Dam Break should construct");

        // Act
        let result = core.apply_pointer("move", f32::NAN, 1.0);

        // Assert
        assert_eq!(result, Err(SessionError::InvalidPointer));
        assert_eq!(
            SessionError::InvalidPointer.message(),
            "Rust/WASM pointer coordinates must be finite"
        );
        assert_eq!(core.particle_count(), 1920);
    }

    #[test]
    fn apply_pointer_rejects_unknown_kind_without_disposing() {
        // Arrange
        let mut core = build_core("dam-break").expect("fresh Dam Break should construct");

        // Act
        let result = core.apply_pointer("nope", 0.0, 0.0);

        // Assert
        assert_eq!(result, Err(SessionError::UnknownControl));
        assert_eq!(
            SessionError::UnknownControl.message(),
            "Rust/WASM control is not allowlisted"
        );
        assert_eq!(core.particle_count(), 1920);
    }

    #[test]
    fn pointer_action_exists_on_proof_session() {
        // Arrange / Act
        // Native tests cannot construct JsError; assert the wasm-bindgen method exists.
        let method: fn(&mut ProofSession, String, f32, f32) -> Result<(), JsError> =
            ProofSession::pointer_action;
        assert_eq!(
            core::mem::size_of_val(&method),
            core::mem::size_of::<fn(&mut ProofSession, String, f32, f32) -> Result<(), JsError>>()
        );
        assert_eq!("pointerAction", "pointerAction");
    }
}
