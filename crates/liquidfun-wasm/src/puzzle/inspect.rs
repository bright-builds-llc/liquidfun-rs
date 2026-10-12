//! Stateless wasm placement inspector for live editing (D-02).

use wasm_bindgen::prelude::*;

use super::validate::{self, Inspection};

/// Per-placement verdicts and outlines, in input order (D-02).
#[wasm_bindgen]
pub struct PlacementInspection {
    statuses: Vec<u8>,
    outlines: Vec<f32>,
}

impl From<Inspection> for PlacementInspection {
    fn from(inspection: Inspection) -> Self {
        Self {
            statuses: inspection.statuses,
            outlines: inspection.outlines,
        }
    }
}

#[wasm_bindgen]
impl PlacementInspection {
    /// Returns one status code per placement: 0 is valid, 1 through 12 name
    /// the first rule the placement breaks.
    #[must_use]
    pub fn statuses(&self) -> Box<[u8]> {
        self.statuses.clone().into_boxed_slice()
    }

    /// Returns eight floats per placement (`x0, y0, …, x3, y3`, counter-clockwise
    /// world-space corners), or eight zeros when the pose has no outline.
    #[must_use]
    pub fn outlines(&self) -> Box<[f32]> {
        self.outlines.clone().into_boxed_slice()
    }
}

/// Stateless placement check for live editing (D-02); builds no session and no world.
///
/// # Errors
///
/// Returns a bounded JavaScript error for an unknown level id, more than 16
/// placements, or a `poses` lane that is not three values per kind.
#[wasm_bindgen(js_name = inspectPuzzlePlacements)]
pub fn inspect_puzzle_placements(
    level_id: &str,
    kinds: &[u8],
    poses: &[i32],
) -> Result<PlacementInspection, JsError> {
    validate::inspect_level(level_id, kinds, poses)
        .map(PlacementInspection::from)
        .map_err(|error| JsError::new(error.message()))
}
