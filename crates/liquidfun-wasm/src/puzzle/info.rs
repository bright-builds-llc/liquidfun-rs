//! Static level description returned by `ProofSession.levelInfo()` (D-16, D-24).

use wasm_bindgen::prelude::*;

use super::spec::LevelSpec;
use super::tools::GRID_METERS;

/// Static description of one puzzle level for the player (D-16, D-24).
#[wasm_bindgen]
pub struct PuzzleLevelInfo {
    spec: &'static LevelSpec,
}

impl PuzzleLevelInfo {
    /// Describes an allowlisted level spec.
    pub(crate) const fn from_spec(spec: &'static LevelSpec) -> Self {
        Self { spec }
    }
}

#[wasm_bindgen]
impl PuzzleLevelInfo {
    /// Returns the stable kebab-case level id.
    #[must_use]
    #[wasm_bindgen(js_name = levelId)]
    pub fn level_id(&self) -> String {
        self.spec.id.as_str().to_owned()
    }

    /// Returns the player-facing level title.
    #[must_use]
    pub fn title(&self) -> String {
        self.spec.title.to_owned()
    }

    /// Returns the count that must hold for the settle window to win.
    #[must_use]
    #[wasm_bindgen(js_name = targetCount)]
    pub fn target_count(&self) -> u32 {
        self.spec.target_count
    }

    /// Returns the authored count for 100% fill.
    #[must_use]
    #[wasm_bindgen(js_name = fullCount)]
    pub fn full_count(&self) -> u32 {
        self.spec.full_count
    }

    /// Returns the consecutive steps at or above target needed to win.
    #[must_use]
    #[wasm_bindgen(js_name = settleSteps)]
    pub fn settle_steps(&self) -> u32 {
        self.spec.settle_steps
    }

    /// Returns the last step at which a win may complete.
    #[must_use]
    #[wasm_bindgen(js_name = stepLimit)]
    pub fn step_limit(&self) -> u32 {
        self.spec.step_limit
    }

    /// Returns the cup interior as `[min_x, min_y, max_x, max_y]` in meters.
    #[must_use]
    #[wasm_bindgen(js_name = cupInterior)]
    pub fn cup_interior(&self) -> Box<[f32]> {
        let (min, max) = (self.spec.cup_interior.min(), self.spec.cup_interior.max());
        Box::new([min.x, min.y, max.x, max.y])
    }

    /// Returns the inclusive build area as `[min_x, min_y, max_x, max_y]` grid cells.
    #[must_use]
    #[wasm_bindgen(js_name = buildArea)]
    pub fn build_area(&self) -> Box<[i32]> {
        Box::new(self.spec.build_area.bounds().map(i32::from))
    }

    /// Returns the grid cell size in meters.
    #[must_use]
    #[wasm_bindgen(js_name = gridMeters)]
    pub fn grid_meters(&self) -> f32 {
        GRID_METERS
    }

    /// Returns the allowed tool kind codes, aligned with `toolCounts`.
    #[must_use]
    #[wasm_bindgen(js_name = toolKinds)]
    pub fn tool_kinds(&self) -> Box<[u8]> {
        self.spec
            .tool_budget
            .iter()
            .map(|allowance| allowance.kind.code())
            .collect()
    }

    /// Returns the most placements of each kind, aligned with `toolKinds`.
    #[must_use]
    #[wasm_bindgen(js_name = toolCounts)]
    pub fn tool_counts(&self) -> Box<[u32]> {
        self.spec
            .tool_budget
            .iter()
            .map(|allowance| u32::from(allowance.count))
            .collect()
    }
}
