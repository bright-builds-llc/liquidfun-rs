//! Bottle geometry for the Cartesian Diver: walls, water grid, drawn outline,
//! squeeze hit region and the drawn bubble.
//!
//! Everything here is a pure function of constants and the ramped squeeze
//! value, so drawing never touches the physical walls (D-09).

use std::f32::consts::TAU;

use liquidfun::math::Vec2;

use crate::scene::RigidSegment;

/// Water particle radius in meters.
pub(super) const PARTICLE_RADIUS: f32 = 0.05;
/// Grid spacing: 0.75 particle diameters, the `LiquidFun` default stride.
pub(super) const PARTICLE_STRIDE_M: f32 = 0.075;
/// Water grid columns across the interior.
pub(super) const COLUMNS: u8 = 31;
/// Water grid rows up the interior (top row at y = 5.225).
pub(super) const ROWS: u8 = 70;
/// Center of the bottom-left grid particle.
pub(super) const GRID_ORIGIN: Vec2 = Vec2::new(-1.15, 0.05);

/// Half the interior width: the water-facing side walls sit at x = ±1.2.
pub(super) const INTERIOR_HALF_WIDTH: f32 = 1.2;
/// Interior height from the floor (y = 0) to the cap's underside.
pub(super) const INTERIOR_HEIGHT: f32 = 5.6;
/// Thickness of the static wall, floor and cap polygons.
pub(super) const WALL_THICKNESS: f32 = 0.4;

/// The drawn outline sits this far outside the inner faces.
pub(super) const OUTLINE_GAP: f32 = 0.25;
/// Inward bow at mid-height at full squeeze; equal to the gap, so the
/// drawn dent meets the inner face and never crosses into the water.
pub(super) const DENT_DEPTH: f32 = OUTLINE_GAP;
/// Half width of the drawn outline rectangle.
pub(super) const OUTLINE_HALF_WIDTH: f32 = INTERIOR_HALF_WIDTH + OUTLINE_GAP;
/// Bottom edge of the drawn outline rectangle.
pub(super) const OUTLINE_BOTTOM_Y: f32 = -OUTLINE_GAP;
/// Top edge of the drawn outline rectangle.
pub(super) const OUTLINE_TOP_Y: f32 = INTERIOR_HEIGHT + OUTLINE_GAP;
/// Chords per drawn side wall, as a loop counter.
const SIDE_STEPS: u8 = 6;
/// Chords per drawn side wall.
pub(super) const SIDE_SEGMENTS: usize = SIDE_STEPS as usize;
/// Floor, cap and both side walls.
pub(super) const OUTLINE_SEGMENT_COUNT: usize = 2 + 2 * SIDE_SEGMENTS;

/// Margin around the outer wall rectangle that still counts as the bottle
/// (D-02); the hit region corners below are written out so its edges are exact.
#[cfg(test)]
pub(super) const HIT_MARGIN: f32 = 0.3;
/// Lower-left corner of the squeeze hit region: the outer walls plus the margin.
pub(super) const HIT_REGION_MIN: Vec2 = Vec2::new(-1.9, -0.7);
/// Upper-right corner of the squeeze hit region: the outer walls plus the margin.
pub(super) const HIT_REGION_MAX: Vec2 = Vec2::new(1.9, 6.3);

/// Diver circle radius: six particle diameters, so single hits barely move it (D-04).
pub(super) const DIVER_RADIUS: f32 = 0.6;
/// Diver center at load, clear of the cap and of every water particle.
pub(super) const DIVER_START: Vec2 = Vec2::new(0.0, 4.6);
/// A diver center below this line is in the bottom third of the interior.
#[cfg(test)]
pub(super) const BOTTOM_THIRD_Y: f32 = INTERIOR_HEIGHT / 3.0;
/// A diver center at or above this line is in the top third of the interior.
#[cfg(test)]
pub(super) const TOP_THIRD_Y: f32 = INTERIOR_HEIGHT * 2.0 / 3.0;

/// Bubble radius as a fraction of the diver radius when released.
const BUBBLE_RELEASED_FRACTION: f32 = 0.6;
/// How much of the diver radius the bubble loses at full squeeze.
const BUBBLE_SHRINK_FRACTION: f32 = 0.3;
/// The bubble's far edge stays this fraction of the diver radius from its center.
const BUBBLE_REACH_FRACTION: f32 = 0.92;
/// Below this gravity length the bubble rides straight up.
const MIN_GRAVITY_LENGTH: f32 = 1e-6;

/// Floor, side walls and closed cap as overlapping static quads.
pub(super) fn wall_quads() -> [[Vec2; 4]; 4] {
    let outer_half_width = INTERIOR_HALF_WIDTH + WALL_THICKNESS;
    let floor_bottom = -WALL_THICKNESS;
    let cap_top = INTERIOR_HEIGHT + WALL_THICKNESS;
    [
        quad(
            Vec2::new(-outer_half_width, floor_bottom),
            Vec2::new(outer_half_width, 0.0),
        ),
        quad(
            Vec2::new(-outer_half_width, floor_bottom),
            Vec2::new(-INTERIOR_HALF_WIDTH, cap_top),
        ),
        quad(
            Vec2::new(INTERIOR_HALF_WIDTH, floor_bottom),
            Vec2::new(outer_half_width, cap_top),
        ),
        quad(
            Vec2::new(-outer_half_width, INTERIOR_HEIGHT),
            Vec2::new(outer_half_width, cap_top),
        ),
    ]
}

/// Counter-clockwise corners of an axis-aligned box.
fn quad(min: Vec2, max: Vec2) -> [Vec2; 4] {
    [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)]
}

/// Water grid positions in row-major order, skipping the diver's start disk.
pub(super) fn water_positions() -> Vec<Vec2> {
    let clearance = DIVER_RADIUS + PARTICLE_RADIUS;
    let mut positions = Vec::with_capacity(usize::from(COLUMNS) * usize::from(ROWS));
    for row in 0..ROWS {
        for column in 0..COLUMNS {
            let position = Vec2::new(
                GRID_ORIGIN.x + f32::from(column) * PARTICLE_STRIDE_M,
                GRID_ORIGIN.y + f32::from(row) * PARTICLE_STRIDE_M,
            );
            if (position - DIVER_START).length() > clearance {
                positions.push(position);
            }
        }
    }
    positions
}

/// Drawn bottle outline as a closed loop: floor, right wall bottom to top,
/// cap, left wall top to bottom. The sides bow inward with `squeeze` (UI-D61).
pub(super) fn outline_segments(squeeze: f32) -> [RigidSegment; OUTLINE_SEGMENT_COUNT] {
    let mut right = [Vec2::ZERO; SIDE_SEGMENTS + 1];
    let mut left = [Vec2::ZERO; SIDE_SEGMENTS + 1];
    for step in 0..=SIDE_STEPS {
        let (y, inset) = side_point(step, squeeze);
        right[usize::from(step)] = Vec2::new(OUTLINE_HALF_WIDTH - inset, y);
        left[usize::from(step)] = Vec2::new(-OUTLINE_HALF_WIDTH + inset, y);
    }

    let mut segments = [RigidSegment {
        start: Vec2::ZERO,
        end: Vec2::ZERO,
    }; OUTLINE_SEGMENT_COUNT];
    segments[0] = RigidSegment {
        start: left[0],
        end: right[0],
    };
    for index in 0..SIDE_SEGMENTS {
        segments[1 + index] = RigidSegment {
            start: right[index],
            end: right[index + 1],
        };
    }
    segments[1 + SIDE_SEGMENTS] = RigidSegment {
        start: right[SIDE_SEGMENTS],
        end: left[SIDE_SEGMENTS],
    };
    for index in 0..SIDE_SEGMENTS {
        segments[2 + SIDE_SEGMENTS + index] = RigidSegment {
            start: left[SIDE_SEGMENTS - index],
            end: left[SIDE_SEGMENTS - index - 1],
        };
    }
    segments
}

/// Height and inward offset of side point `step` (0 at the floor corner).
///
/// The corners are exact so the loop closes on the floor and cap; the
/// interior points follow half a sine period, deepest at mid-height.
fn side_point(step: u8, squeeze: f32) -> (f32, f32) {
    if step == 0 {
        return (OUTLINE_BOTTOM_Y, 0.0);
    }
    if step >= SIDE_STEPS {
        return (OUTLINE_TOP_Y, 0.0);
    }
    let t = f32::from(step) / f32::from(SIDE_STEPS);
    let y = OUTLINE_BOTTOM_Y + t * (OUTLINE_TOP_Y - OUTLINE_BOTTOM_Y);
    let inset = DENT_DEPTH * squeeze * (TAU / 2.0 * t).sin();
    (y, inset)
}

/// Whether a pointer press at `point` lands on the bottle (D-02, inclusive edges).
pub(super) fn in_squeeze_region(point: Vec2) -> bool {
    (HIT_REGION_MIN.x..=HIT_REGION_MAX.x).contains(&point.x)
        && (HIT_REGION_MIN.y..=HIT_REGION_MAX.y).contains(&point.y)
}

/// Drawn air bubble inside the diver: it shrinks with `squeeze` and rides on
/// the side opposite gravity, always fully inside the diver (D-10, UI-D62).
pub(super) fn bubble_circle(
    diver_center: Vec2,
    diver_radius: f32,
    squeeze: f32,
    gravity: Vec2,
) -> (Vec2, f32) {
    let radius = diver_radius * (BUBBLE_RELEASED_FRACTION - BUBBLE_SHRINK_FRACTION * squeeze);
    let gravity_length = gravity.length();
    let up = if gravity_length < MIN_GRAVITY_LENGTH {
        Vec2::new(0.0, 1.0)
    } else {
        -gravity / gravity_length
    };
    let offset = BUBBLE_REACH_FRACTION * diver_radius - radius;
    (diver_center + up * offset, radius)
}
