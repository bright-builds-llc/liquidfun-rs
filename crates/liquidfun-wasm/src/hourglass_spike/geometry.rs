//! Fixed hourglass spike geometry (Phase 45 D-23): wall quads and the sand
//! fill, recorded in `45-HOURGLASS-SPIKE.md` before any measurement.
//!
//! The lower half is written out; the upper half is its point reflection
//! `q → −q` about `c = (0, 0)`, so the glass is exactly point-symmetric.

use liquidfun::math::Vec2;

/// Particle radius (D-15).
pub(super) const RADIUS: f32 = 0.05;
/// Fill spacing, `0.75 × diameter`.
pub(super) const SPACING: f32 = 0.075;
/// Inner floor of the lower bulb.
pub(super) const LOWER_FLOOR: f32 = -3.4;
/// Inner side walls of both bulbs are at `x = ±BULB_HALF_WIDTH`.
pub(super) const BULB_HALF_WIDTH: f32 = 1.5;
/// Top of the upper-bulb sand fill.
const FILL_TOP: f32 = 1.8;
/// Inner funnel corner at the neck (`±NECK_HALF_WIDTH`, 0).
const NECK_HALF_WIDTH: f32 = 0.25;
/// Height at which a funnel meets its side wall.
const FUNNEL_HEIGHT: f32 = 1.6;

/// Lower-half wall quads; each is counter-clockwise.
const LOWER_HALF: [[Vec2; 4]; 5] = [
    // Floor
    box_quad(-1.8, -3.7, 1.8, -3.4),
    // Left wall (overlaps the funnel)
    box_quad(-1.8, -3.7, -1.5, -1.3),
    // Right wall
    box_quad(1.5, -3.7, 1.8, -1.3),
    // Left funnel (lower-bulb ceiling)
    [
        Vec2::new(-1.5, -1.6),
        Vec2::new(-0.25, 0.0),
        Vec2::new(-0.4865, 0.1847),
        Vec2::new(-1.7365, -1.4153),
    ],
    // Right funnel, re-ordered counter-clockwise
    [
        Vec2::new(1.7365, -1.4153),
        Vec2::new(0.4865, 0.1847),
        Vec2::new(0.25, 0.0),
        Vec2::new(1.5, -1.6),
    ],
];

const fn box_quad(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> [Vec2; 4] {
    [
        Vec2::new(min_x, min_y),
        Vec2::new(max_x, min_y),
        Vec2::new(max_x, max_y),
        Vec2::new(min_x, max_y),
    ]
}

/// Every wall quad: the lower half followed by its point reflection. A point
/// reflection is a half turn, so the winding stays counter-clockwise.
pub(super) fn wall_quads() -> Vec<[Vec2; 4]> {
    let reflected = LOWER_HALF.map(|quad| quad.map(reflect));
    LOWER_HALF.into_iter().chain(reflected).collect()
}

/// Point reflection about `c = (0, 0)`.
pub(super) fn reflect(point: Vec2) -> Vec2 {
    Vec2::new(-point.x, -point.y)
}

/// Whether `point` lies inside the upper-bulb fill region: the funnel up to
/// the side walls plus the box up to `FILL_TOP`.
fn in_upper_fill(point: Vec2) -> bool {
    if point.y <= 0.0 || point.y > FILL_TOP {
        return false;
    }
    let half_width = if point.y < FUNNEL_HEIGHT {
        NECK_HALF_WIDTH + (BULB_HALF_WIDTH - NECK_HALF_WIDTH) * point.y / FUNNEL_HEIGHT
    } else {
        BULB_HALF_WIDTH
    };
    point.x.abs() < half_width
}

/// Distance from `point` to a counter-clockwise convex quad (0 inside).
pub(super) fn distance_to_quad(point: Vec2, quad: &[Vec2; 4]) -> f32 {
    let mut inside = true;
    let mut nearest = f32::INFINITY;
    for index in 0..4 {
        let start = quad[index];
        let end = quad[(index + 1) % 4];
        let edge = end - start;
        let offset = point - start;
        if edge.x * offset.y - edge.y * offset.x < 0.0 {
            inside = false;
        }
        let length_squared = edge.x * edge.x + edge.y * edge.y;
        let along = ((offset.x * edge.x + offset.y * edge.y) / length_squared).clamp(0.0, 1.0);
        let closest = start + edge * along;
        nearest = nearest.min((point - closest).length());
    }
    if inside { 0.0 } else { nearest }
}

/// One fill point and its 2 × 2 clump key (candidate 3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct FillPoint {
    pub(super) position: Vec2,
    pub(super) clump: (i32, i32),
}

/// Upper-bulb sand fill on one 0.075 m grid symmetric about `x = 0`
/// (`x = (i + 0.5)·s`, `y = (j + 0.5)·s`), kept when inside the fill region
/// and at least `r` from every wall quad. Ordered by row, then column.
pub(super) fn sand_fill() -> Vec<FillPoint> {
    let walls = wall_quads();
    let columns = (BULB_HALF_WIDTH / SPACING).ceil() as i32;
    let rows = (FILL_TOP / SPACING).ceil() as i32;
    let mut points = Vec::new();
    for row in 0..rows {
        for column in -columns..columns {
            let position = Vec2::new(
                (column as f32 + 0.5) * SPACING,
                (row as f32 + 0.5) * SPACING,
            );
            if !in_upper_fill(position) {
                continue;
            }
            if walls
                .iter()
                .any(|quad| distance_to_quad(position, quad) < RADIUS)
            {
                continue;
            }
            points.push(FillPoint {
                position,
                clump: (column.div_euclid(2), row.div_euclid(2)),
            });
        }
    }
    points
}
