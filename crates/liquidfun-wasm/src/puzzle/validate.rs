//! The one placement validator shared by the live inspector and `newPuzzle`
//! (D-01, D-03, D-04, D-29).
//!
//! Every tool outline, inflated by one particle diameter, must clear the
//! walls, cup walls, cup interior, initial liquid, authored no-place zones
//! and every other tool. Both entry points run [`placement_statuses`], so the
//! browser tint and the build decision cannot disagree.

use liquidfun::math::Vec2;

use super::sat::{rect_quad, separated_by_more_than};
use super::spec::LevelSpec;
use super::tools::{
    GRID_CELLS_PER_METER, GridRect, POSE_STRIDE, Placement, PlacementError, Placements, ToolKind,
    check_lanes, parse_one, pose_placement,
};
use super::{level_spec, parse_level_id};
use crate::session::SessionError;

/// Floats per outline in [`Inspection::outlines`]: `x0, y0, …, x3, y3`.
pub(crate) const OUTLINE_STRIDE: usize = 8;

/// Per-placement verdict. The codes are a wire contract mirrored by
/// `web/src/puzzle/placement-status.ts`; never renumber them.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlacementStatus {
    /// The placement may be built.
    Valid = 0,
    /// The kind code is not a tool.
    UnknownKind = 1,
    /// The level does not allow this kind.
    KindNotAllowed = 2,
    /// More placements of this kind than the level allows, counted in input order.
    OverBudget = 3,
    /// Angle index outside `0..24`.
    AngleOutOfRange = 4,
    /// Grid coordinate outside `i16`.
    PositionOutOfRange = 5,
    /// Pose centre outside the build area.
    OutsideBuildArea = 6,
    /// Outline overlaps a level wall.
    OverlapsWall = 7,
    /// Outline overlaps a cup wall.
    OverlapsCupWall = 8,
    /// Outline overlaps the cup interior.
    InsideCup = 9,
    /// Outline overlaps an initial liquid block.
    OverlapsWater = 10,
    /// Outline overlaps another placed tool.
    OverlapsTool = 11,
    /// Outline overlaps an authored no-build zone.
    OverlapsNoPlace = 12,
}

impl PlacementStatus {
    /// Wire code in the inspector status lane.
    pub(crate) const fn code(self) -> u8 {
        self as u8
    }

    /// The per-placement status for a Phase 37 parse error; `None` for
    /// lane-level errors, which reject the whole request instead.
    const fn from_error(error: PlacementError) -> Option<Self> {
        match error {
            PlacementError::TooManyPlacements | PlacementError::LaneLengthMismatch => None,
            PlacementError::UnknownToolKind => Some(Self::UnknownKind),
            PlacementError::ToolNotAllowed => Some(Self::KindNotAllowed),
            PlacementError::ToolBudgetExceeded => Some(Self::OverBudget),
            PlacementError::AngleOutOfRange => Some(Self::AngleOutOfRange),
            PlacementError::PositionOutOfRange => Some(Self::PositionOutOfRange),
            PlacementError::OutsideBuildArea => Some(Self::OutsideBuildArea),
            PlacementError::OverlapsWall => Some(Self::OverlapsWall),
            PlacementError::OverlapsCupWall => Some(Self::OverlapsCupWall),
            PlacementError::InsideCup => Some(Self::InsideCup),
            PlacementError::OverlapsWater => Some(Self::OverlapsWater),
            PlacementError::OverlapsTool => Some(Self::OverlapsTool),
            PlacementError::OverlapsNoPlace => Some(Self::OverlapsNoPlace),
        }
    }

    /// The build error for an invalid status; `None` when valid.
    const fn error(self) -> Option<PlacementError> {
        match self {
            Self::Valid => None,
            Self::UnknownKind => Some(PlacementError::UnknownToolKind),
            Self::KindNotAllowed => Some(PlacementError::ToolNotAllowed),
            Self::OverBudget => Some(PlacementError::ToolBudgetExceeded),
            Self::AngleOutOfRange => Some(PlacementError::AngleOutOfRange),
            Self::PositionOutOfRange => Some(PlacementError::PositionOutOfRange),
            Self::OutsideBuildArea => Some(PlacementError::OutsideBuildArea),
            Self::OverlapsWall => Some(PlacementError::OverlapsWall),
            Self::OverlapsCupWall => Some(PlacementError::OverlapsCupWall),
            Self::InsideCup => Some(PlacementError::InsideCup),
            Self::OverlapsWater => Some(PlacementError::OverlapsWater),
            Self::OverlapsTool => Some(PlacementError::OverlapsTool),
            Self::OverlapsNoPlace => Some(PlacementError::OverlapsNoPlace),
        }
    }
}

/// Inspector answer in input order: one status code and one
/// [`OUTLINE_STRIDE`]-float outline per placement.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Inspection {
    /// [`PlacementStatus`] codes.
    pub(crate) statuses: Vec<u8>,
    /// Baked world-space corners, or eight zeros when the pose has no outline.
    pub(crate) outlines: Vec<f32>,
}

/// Checks every placement of `spec` in input order (D-01, D-02).
///
/// # Errors
///
/// Returns `TooManyPlacements` or `LaneLengthMismatch` before allocating when
/// the lanes cannot describe placements.
pub(crate) fn inspect(
    spec: &LevelSpec,
    kinds: &[u8],
    poses: &[i32],
) -> Result<Inspection, PlacementError> {
    let (statuses, quads) = placement_statuses(spec, kinds, poses)?;
    let outlines = quads
        .iter()
        .flat_map(|maybe_quad| {
            maybe_quad.map_or([0.0; OUTLINE_STRIDE], |[v0, v1, v2, v3]| {
                [v0.x, v0.y, v1.x, v1.y, v2.x, v2.y, v3.x, v3.y]
            })
        })
        .collect();
    Ok(Inspection {
        statuses: statuses.into_iter().map(PlacementStatus::code).collect(),
        outlines,
    })
}

/// Parses placements with the unchanged Phase 37 rules and error order, then
/// rejects the first placement the inspector would mark invalid (D-03).
///
/// # Errors
///
/// Returns the Phase 37 parse error, or the overlap error of the first
/// invalid placement in input order.
pub(crate) fn validate_for_build(
    spec: &LevelSpec,
    kinds: &[u8],
    poses: &[i32],
) -> Result<Placements, PlacementError> {
    let placements = Placements::parse(kinds, poses, spec.tool_budget, spec.build_area)?;
    let (statuses, _quads) = placement_statuses(spec, kinds, poses)?;
    match statuses.into_iter().find_map(PlacementStatus::error) {
        Some(error) => Err(error),
        None => Ok(placements),
    }
}

/// Inspects an allowlisted level by id.
///
/// # Errors
///
/// Returns `UnknownLevel` for ids outside the level registry and
/// `InvalidPlacement` for malformed lanes.
pub(crate) fn inspect_level(
    level_id: &str,
    kinds: &[u8],
    poses: &[i32],
) -> Result<Inspection, SessionError> {
    let level = parse_level_id(level_id).ok_or(SessionError::UnknownLevel)?;
    inspect(level_spec(level), kinds, poses).map_err(SessionError::InvalidPlacement)
}

type Quad = [Vec2; 4];

/// Statuses and outlines in input order. Precedence per placement: the Phase
/// 37 parse rules, then [`first_overlap`], then the per-kind budget.
fn placement_statuses(
    spec: &LevelSpec,
    kinds: &[u8],
    poses: &[i32],
) -> Result<(Vec<PlacementStatus>, Vec<Option<Quad>>), PlacementError> {
    check_lanes(kinds, poses)?;
    let mut statuses = Vec::with_capacity(kinds.len());
    let mut quads = Vec::with_capacity(kinds.len());
    for (&code, pose) in kinds.iter().zip(poses.chunks_exact(POSE_STRIDE)) {
        let status = match parse_one(code, pose, spec.tool_budget, spec.build_area) {
            Ok(_placement) => PlacementStatus::Valid,
            Err(error) => PlacementStatus::from_error(error).ok_or(error)?,
        };
        statuses.push(status);
        quads.push(pose_placement(code, pose).map(Placement::vertices));
    }
    mark_overlaps(spec, &mut statuses, &quads);
    mark_over_budget(spec, kinds, &mut statuses);
    Ok((statuses, quads))
}

/// Marks still-valid placements whose outline overlaps an obstacle. A tool
/// that is invalid for another reason still blocks the tools it overlaps.
fn mark_overlaps(spec: &LevelSpec, statuses: &mut [PlacementStatus], quads: &[Option<Quad>]) {
    for (index, (status, maybe_quad)) in statuses.iter_mut().zip(quads).enumerate() {
        let Some(quad) = maybe_quad else {
            continue;
        };
        if *status != PlacementStatus::Valid {
            continue;
        }
        let other_tools = quads
            .iter()
            .enumerate()
            .filter(|(other, _)| *other != index)
            .filter_map(|(_, maybe_other)| maybe_other.as_ref());
        if let Some(overlap) = first_overlap(spec, quad, other_tools) {
            *status = overlap;
        }
    }
}

/// Marks still-valid placements beyond their kind's allowance, counting every
/// placement of that kind in input order.
fn mark_over_budget(spec: &LevelSpec, kinds: &[u8], statuses: &mut [PlacementStatus]) {
    for allowance in spec.tool_budget {
        let same_kind = kinds
            .iter()
            .zip(statuses.iter_mut())
            .filter(|(code, _)| ToolKind::from_code(**code) == Some(allowance.kind));
        for (_, status) in same_kind.skip(usize::from(allowance.count)) {
            if *status == PlacementStatus::Valid {
                *status = PlacementStatus::OverBudget;
            }
        }
    }
}

/// The first obstacle `quad` fails to clear by one particle diameter (D-04),
/// in precedence order: walls, cup walls, cup interior, water, no-place zones,
/// other tools.
fn first_overlap<'a>(
    spec: &LevelSpec,
    quad: &Quad,
    other_tools: impl IntoIterator<Item = &'a Quad>,
) -> Option<PlacementStatus> {
    let gap = 2.0 * spec.particle_radius;
    let overlaps = |obstacle: &Quad| !separated_by_more_than(quad, obstacle, gap);
    if spec.walls.iter().any(overlaps) {
        return Some(PlacementStatus::OverlapsWall);
    }
    if spec.cup_walls.iter().any(overlaps) {
        return Some(PlacementStatus::OverlapsCupWall);
    }
    let interior = spec.cup_interior;
    if overlaps(&rect_quad(interior.min(), interior.max())) {
        return Some(PlacementStatus::InsideCup);
    }
    let mut water = spec
        .liquid_blocks
        .iter()
        .map(|block| rect_quad(block.min, block.max));
    if water.any(|block| overlaps(&block)) {
        return Some(PlacementStatus::OverlapsWater);
    }
    if spec
        .no_place
        .iter()
        .any(|zone| overlaps(&grid_rect_quad(*zone)))
    {
        return Some(PlacementStatus::OverlapsNoPlace);
    }
    if other_tools.into_iter().any(overlaps) {
        return Some(PlacementStatus::OverlapsTool);
    }
    None
}

/// A grid rectangle's corners in meters.
fn grid_rect_quad(zone: GridRect) -> Quad {
    let [min_x, min_y, max_x, max_y] = zone
        .bounds()
        .map(|cell| f32::from(cell) / GRID_CELLS_PER_METER);
    rect_quad(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
}

#[cfg(test)]
mod tests;
