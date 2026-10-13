//! Construction-time puzzle tools placed on an integer grid (D-08, D-09).
//!
//! Integer poses (0.05 m cells, τ/24 steps) make canonical order and snapping
//! exact; tool geometry comes from [`TAU_24`] and never calls `sin` or `cos`.

use liquidfun::math::Vec2;

use super::angles::TAU_24;

/// Most placements one puzzle run accepts.
pub(crate) const MAX_PLACEMENTS: usize = 16;
/// Pose values `(x_grid, y_grid, angle_index)`; a 4th (Phase 40 valve time) is rejected.
pub(crate) const POSE_STRIDE: usize = 3;
/// Angle steps per full turn: τ/24 = 15°.
pub(crate) const ANGLE_STEPS: u8 = 24;
/// Grid cells per meter: meters = `f32::from(cell) / 20.0`.
pub(crate) const GRID_CELLS_PER_METER: f32 = 20.0;
/// Grid cell size in meters, reported by `levelInfo`.
pub(crate) const GRID_METERS: f32 = 0.05;
/// Half the plank length: a 3.0 m plank.
pub(crate) const PLANK_HALF_LENGTH: f32 = 1.5;
/// Half the plank thickness: a 0.25 m plank.
pub(crate) const PLANK_HALF_THICKNESS: f32 = 0.125;

/// Allowlisted tool kinds. Code 0 is never a tool, so zeroed lanes fail closed.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ToolKind {
    /// Fixed-size static plank.
    Plank = 1,
}

impl ToolKind {
    /// Wire code used in the `kinds` lane.
    pub(crate) const fn code(self) -> u8 {
        match self {
            Self::Plank => 1,
        }
    }

    /// Parses a wire code; unknown codes (including 0) are `None`.
    pub(crate) const fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Plank),
            _ => None,
        }
    }

    /// Fixture token (D-18), read by the test-only replay fixtures.
    #[cfg(test)]
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::Plank => "plank",
        }
    }

    /// Parses a fixture token, for the test-only replay fixtures.
    #[cfg(test)]
    pub(crate) fn from_token(token: &str) -> Option<Self> {
        match token {
            "plank" => Some(Self::Plank),
            _ => None,
        }
    }
}

/// Angle as a τ/24 step in `0..24`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AngleIndex(u8);

impl AngleIndex {
    /// `None` unless `index < 24`.
    pub(crate) const fn new(index: u8) -> Option<Self> {
        if index < ANGLE_STEPS {
            Some(Self(index))
        } else {
            None
        }
    }

    /// The step index in `0..24`.
    pub(crate) const fn get(self) -> u8 {
        self.0
    }
}

/// Inclusive rectangle in 0.05 m grid cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GridRect {
    min_x: i16,
    min_y: i16,
    max_x: i16,
    max_y: i16,
}

impl GridRect {
    /// Creates a rectangle from inclusive grid bounds.
    pub(crate) const fn new(min_x: i16, min_y: i16, max_x: i16, max_y: i16) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Inclusive containment in grid cells.
    pub(crate) const fn contains(self, x: i16, y: i16) -> bool {
        self.min_x <= x && x <= self.max_x && self.min_y <= y && y <= self.max_y
    }

    /// `[min_x, min_y, max_x, max_y]` in grid cells.
    pub(crate) const fn bounds(self) -> [i16; 4] {
        [self.min_x, self.min_y, self.max_x, self.max_y]
    }
}

/// How many tools of one kind a level allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ToolAllowance {
    /// Allowed kind.
    pub(crate) kind: ToolKind,
    /// Most placements of this kind.
    pub(crate) count: u8,
}

/// One validated tool pose. Field order is the canonical sort key
/// `(kind, x, y, angle)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Placement {
    kind: ToolKind,
    x: i16,
    y: i16,
    angle: AngleIndex,
}

impl Placement {
    /// Tool kind.
    #[cfg(test)]
    pub(crate) const fn kind(self) -> ToolKind {
        self.kind
    }

    /// Grid position `(x, y)` in 0.05 m cells.
    #[cfg(test)]
    pub(crate) const fn grid(self) -> (i16, i16) {
        (self.x, self.y)
    }

    /// τ/24 angle step.
    #[cfg(test)]
    pub(crate) const fn angle(self) -> AngleIndex {
        self.angle
    }

    /// Centre in meters (correctly rounded division of an exact integer).
    pub(crate) fn center(self) -> Vec2 {
        Vec2::new(
            f32::from(self.x) / GRID_CELLS_PER_METER,
            f32::from(self.y) / GRID_CELLS_PER_METER,
        )
    }

    /// World-space plank corners in counter-clockwise order.
    ///
    /// Each corner is `(cos·lx − sin·ly + cx, sin·lx + cos·ly + cy)` in exactly
    /// this expression order. Rust never fuses these into FMA, so the order is
    /// part of the native/wasm determinism contract.
    pub(crate) fn vertices(self) -> [Vec2; 4] {
        let (half_length, half_thickness) = (PLANK_HALF_LENGTH, PLANK_HALF_THICKNESS);
        // AngleIndex::new only accepts values below 24, so the index is always in range.
        let (cos, sin) = TAU_24[usize::from(self.angle.get())];
        let center = self.center();
        [
            (-half_length, -half_thickness),
            (half_length, -half_thickness),
            (half_length, half_thickness),
            (-half_length, half_thickness),
        ]
        .map(|(local_x, local_y)| {
            Vec2::new(
                cos * local_x - sin * local_y + center.x,
                sin * local_x + cos * local_y + center.y,
            )
        })
    }
}

/// Canonically ordered, budget-checked placements for one run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Placements(Vec<Placement>);

impl Placements {
    /// No placements; production parses even empty lanes through `parse`.
    #[cfg(test)]
    pub(crate) const fn empty() -> Self {
        Self(Vec::new())
    }

    /// Parses copied `kinds` and stride-3 `poses` lanes, failing closed at the
    /// first problem, then sorts into canonical `(kind, x, y, angle)` order.
    pub(crate) fn parse(
        kinds: &[u8],
        poses: &[i32],
        budget: &[ToolAllowance],
        build_area: GridRect,
    ) -> Result<Self, PlacementError> {
        check_lanes(kinds, poses)?;
        let mut placements = Vec::with_capacity(kinds.len());
        for (code, pose) in kinds.iter().zip(poses.chunks_exact(POSE_STRIDE)) {
            placements.push(parse_one(*code, pose, budget, build_area)?);
        }
        check_budget(&placements, budget)?;
        placements.sort_unstable();
        Ok(Self(placements))
    }

    /// Placements in canonical order.
    pub(crate) fn as_slice(&self) -> &[Placement] {
        &self.0
    }
}

/// Rejects oversized or misaligned lanes before anything is allocated.
pub(super) fn check_lanes(kinds: &[u8], poses: &[i32]) -> Result<(), PlacementError> {
    if kinds.len() > MAX_PLACEMENTS {
        return Err(PlacementError::TooManyPlacements);
    }
    if kinds.len().checked_mul(POSE_STRIDE) != Some(poses.len()) {
        return Err(PlacementError::LaneLengthMismatch);
    }
    Ok(())
}

/// The placement a raw pose describes, ignoring budget and build area, for
/// tests that need a tool outline. `None` when the kind, angle or grid
/// position cannot describe a tool.
#[cfg(test)]
pub(super) fn pose_placement(code: u8, pose: &[i32]) -> Option<Placement> {
    decode_pose(code, pose).ok()
}

/// Decodes the kind, angle and grid position of one stride-3 pose, in that
/// order. This is the only pose decoder, so [`parse_one`] and the inspector's
/// outline lane cannot disagree about which poses describe a tool.
pub(super) fn decode_pose(code: u8, pose: &[i32]) -> Result<Placement, PlacementError> {
    let [x, y, angle] = *pose else {
        return Err(PlacementError::LaneLengthMismatch);
    };
    let kind = ToolKind::from_code(code).ok_or(PlacementError::UnknownToolKind)?;
    let angle = u8::try_from(angle)
        .ok()
        .and_then(AngleIndex::new)
        .ok_or(PlacementError::AngleOutOfRange)?;
    let x = i16::try_from(x).map_err(|_error| PlacementError::PositionOutOfRange)?;
    let y = i16::try_from(y).map_err(|_error| PlacementError::PositionOutOfRange)?;
    Ok(Placement { kind, x, y, angle })
}

/// Parses one stride-3 pose, failing at the first Phase 37 rule it breaks:
/// kind, allowance, angle, position, build area.
pub(super) fn parse_one(
    code: u8,
    pose: &[i32],
    budget: &[ToolAllowance],
    build_area: GridRect,
) -> Result<Placement, PlacementError> {
    if pose.len() != POSE_STRIDE {
        return Err(PlacementError::LaneLengthMismatch);
    }
    let kind = ToolKind::from_code(code).ok_or(PlacementError::UnknownToolKind)?;
    if !budget.iter().any(|allowance| allowance.kind == kind) {
        return Err(PlacementError::ToolNotAllowed);
    }
    let placement = decode_pose(code, pose)?;
    if !build_area.contains(placement.x, placement.y) {
        return Err(PlacementError::OutsideBuildArea);
    }
    Ok(placement)
}

fn check_budget(placements: &[Placement], budget: &[ToolAllowance]) -> Result<(), PlacementError> {
    for allowance in budget {
        let used = placements
            .iter()
            .filter(|placement| placement.kind == allowance.kind)
            .count();
        if used > usize::from(allowance.count) {
            return Err(PlacementError::ToolBudgetExceeded);
        }
    }
    Ok(())
}

/// Why a placement list was rejected. Messages never echo input values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlacementError {
    /// More than [`MAX_PLACEMENTS`] kinds.
    TooManyPlacements,
    /// `poses.len()` is not `kinds.len() * POSE_STRIDE`.
    LaneLengthMismatch,
    /// A kind code is not a tool.
    UnknownToolKind,
    /// The level does not allow this kind.
    ToolNotAllowed,
    /// More placements of a kind than the level allows.
    ToolBudgetExceeded,
    /// Angle index outside `0..24`.
    AngleOutOfRange,
    /// Grid coordinate outside `i16`.
    PositionOutOfRange,
    /// Grid position outside the level build area.
    OutsideBuildArea,
    /// Tool outline overlaps a level wall.
    OverlapsWall,
    /// Tool outline overlaps a cup wall.
    OverlapsCupWall,
    /// Tool outline overlaps the cup interior.
    InsideCup,
    /// Tool outline overlaps an initial liquid block.
    OverlapsWater,
    /// Tool outline overlaps another placed tool.
    OverlapsTool,
    /// Tool outline overlaps an authored no-build zone.
    OverlapsNoPlace,
}

impl PlacementError {
    /// Bounded, static error message.
    pub(crate) const fn message(self) -> &'static str {
        match self {
            Self::TooManyPlacements => "Rust/WASM puzzle placement count exceeds 16",
            Self::LaneLengthMismatch => "Rust/WASM puzzle placement lanes are inconsistent",
            Self::UnknownToolKind => "Rust/WASM puzzle placement tool kind is unknown",
            Self::ToolNotAllowed => "Rust/WASM puzzle placement tool is not allowed in this level",
            Self::ToolBudgetExceeded => "Rust/WASM puzzle placement exceeds the tool budget",
            Self::AngleOutOfRange => "Rust/WASM puzzle placement angle index must be 0 through 23",
            Self::PositionOutOfRange => "Rust/WASM puzzle placement grid position is out of range",
            Self::OutsideBuildArea => "Rust/WASM puzzle placement is outside the build area",
            Self::OverlapsWall => "Rust/WASM puzzle placement overlaps a wall",
            Self::OverlapsCupWall => "Rust/WASM puzzle placement overlaps the cup",
            Self::InsideCup => "Rust/WASM puzzle placement is inside the cup",
            Self::OverlapsWater => "Rust/WASM puzzle placement overlaps the water",
            Self::OverlapsTool => "Rust/WASM puzzle placement overlaps another tool",
            Self::OverlapsNoPlace => "Rust/WASM puzzle placement overlaps a no-build zone",
        }
    }
}

#[cfg(test)]
mod tests;
