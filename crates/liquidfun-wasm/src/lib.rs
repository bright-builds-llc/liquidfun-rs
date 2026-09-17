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
    /// Constructs the named Dam Break scene from the documented reset constants.
    ///
    /// # Errors
    ///
    /// Returns a bounded JavaScript error when checked scene construction fails.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<ProofSession, JsError> {
        SessionCore::new()
            .map(|core| Self { core })
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

fn js_error(error: SessionError) -> JsError {
    JsError::new(error.message())
}
