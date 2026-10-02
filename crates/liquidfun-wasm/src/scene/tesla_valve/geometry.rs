//! Alternating rounded valve lobes enclosing solid teardrop splitters.
//!
//! Both drawing and collision use these authored points. The two continuous
//! outer paths wind around four alternating islands, leaving the center open.

use liquidfun::math::Vec2;

use super::super::RigidSegment;

/// Horizontal reflection axis used to reverse the valve's internal stages.
pub(super) const MIRROR_Y: f32 = 1.48;
pub(super) const PARTICLE_RADIUS: f32 = 0.014;
pub(super) const DRAIN_TOP_Y: f32 = 0.32;
pub(super) const DRAIN_HALF_WIDTH: f32 = 0.7;
pub(super) const DRAIN_HALF_HEIGHT: f32 = 0.22;
pub(super) const DRAIN_CENTER: Vec2 = Vec2::new(0.0, DRAIN_TOP_Y - DRAIN_HALF_HEIGHT);
pub(super) const SPAWN_Y: f32 = 2.86;
pub(super) const SPAWN_XS: [f32; 3] = [-0.08, 0.0, 0.08];
/// Keep these regression bounds in sync with the catalog's world rectangle.
#[cfg(test)]
pub(super) const FRAME_MIN_X: f32 = -0.70;
#[cfg(test)]
pub(super) const FRAME_MAX_X: f32 = 0.58;
#[cfg(test)]
pub(super) const FRAME_MIN_Y: f32 = -0.12;
#[cfg(test)]
pub(super) const FRAME_MAX_Y: f32 = 3.22;

// Polygon welding uses a squared-distance threshold: ribbons need a full
// thickness above 0.05 m even though the particle radius is much smaller.
const WALL_HALF_THICKNESS: f32 = 0.03;
const STAGE_OFFSET: f32 = 0.568;
const STAGE_OFFSETS: [f32; 4] = [0.0, STAGE_OFFSET, 2.0 * STAGE_OFFSET, 3.0 * STAGE_OFFSET];
const DOWNSTREAM_SCALE: f32 = 0.8;
const FIRST_STAGE_Y: f32 = 2.62;
// Pull each outer cusp toward its lobe to open the opposite splitter's neck.
const NECK_TRANSVERSE: f32 = 0.030;

/// Template coordinates are downstream distance and distance toward the lobe.
const OUTER_LOBE: [Vec2; 9] = [
    Vec2::new(0.0, NECK_TRANSVERSE),
    Vec2::new(-0.09, 0.06),
    Vec2::new(-0.16, 0.175),
    Vec2::new(-0.16, 0.30),
    Vec2::new(-0.10, 0.395),
    Vec2::new(0.0, 0.434),
    Vec2::new(0.12, 0.405),
    Vec2::new(0.62, 0.19),
    Vec2::new(1.136, NECK_TRANSVERSE),
];

const SPLITTER: [Vec2; 6] = [
    Vec2::new(0.0, 0.12),
    Vec2::new(-0.04, 0.16),
    Vec2::new(-0.03, 0.22),
    Vec2::new(0.04, 0.25),
    Vec2::new(0.12, 0.235),
    Vec2::new(0.53, 0.08),
];

pub(super) struct ValveGeometry {
    pub(super) outer_paths: [Vec<Vec2>; 2],
    pub(super) splitters: [[Vec2; 6]; 4],
}

pub(super) fn valve_geometry(forward: bool) -> ValveGeometry {
    let splitters =
        std::array::from_fn(|stage| SPLITTER.map(|point| place_stage(point, stage, forward)));
    let outer_paths = std::array::from_fn(|side| {
        let mut stages: Vec<Vec2> = [side, side + 2]
            .into_iter()
            .enumerate()
            .flat_map(|(index, stage)| {
                OUTER_LOBE
                    .iter()
                    .copied()
                    .skip(usize::from(index != 0))
                    .map(move |point| place_stage(point, stage, forward))
            })
            .collect();
        if !forward {
            stages.reverse();
        }
        let sign = if side == 0 { 1.0 } else { -1.0 };
        stages.insert(0, Vec2::new(sign * 0.38, 3.02));
        stages.push(Vec2::new(sign * 0.38, 0.10));
        stages
    });
    ValveGeometry {
        outer_paths,
        splitters,
    }
}

fn place_stage(point: Vec2, stage: usize, forward: bool) -> Vec2 {
    let sign = if stage.is_multiple_of(2) { 1.0 } else { -1.0 };
    let placed = Vec2::new(
        sign * point.y,
        FIRST_STAGE_Y - DOWNSTREAM_SCALE * (point.x + STAGE_OFFSETS[stage]),
    );
    if forward { placed } else { mirror_y(placed) }
}

pub(super) fn mirror_y(point: Vec2) -> Vec2 {
    Vec2::new(point.x, 2.0 * MIRROR_Y - point.y)
}

pub(super) fn collision_polygons(forward: bool) -> Vec<Vec<Vec2>> {
    let geometry = valve_geometry(forward);
    let mut polygons: Vec<Vec<Vec2>> = geometry
        .outer_paths
        .iter()
        .flat_map(|points| ribbon(points))
        .collect();
    polygons.extend(geometry.splitters.into_iter().map(Vec::from));
    polygons
}

pub(super) fn outline_segments(forward: bool) -> Vec<RigidSegment> {
    let geometry = valve_geometry(forward);
    let mut segments: Vec<RigidSegment> = geometry
        .outer_paths
        .iter()
        .flat_map(|points| polyline(points))
        .collect();
    for island in geometry.splitters {
        segments.extend(polyline(&island));
        segments.push(RigidSegment {
            start: island[5],
            end: island[0],
        });
    }
    segments
}

fn polyline(points: &[Vec2]) -> Vec<RigidSegment> {
    points
        .windows(2)
        .map(|pair| RigidSegment {
            start: pair[0],
            end: pair[1],
        })
        .collect()
}

fn ribbon(points: &[Vec2]) -> Vec<Vec<Vec2>> {
    let normals = vertex_normals(points);
    points
        .windows(2)
        .enumerate()
        .map(|(index, pair)| {
            let before = normals[index] * WALL_HALF_THICKNESS;
            let after = normals[index + 1] * WALL_HALF_THICKNESS;
            vec![
                pair[0] + before,
                pair[1] + after,
                pair[1] - after,
                pair[0] - before,
            ]
        })
        .collect()
}

fn vertex_normals(points: &[Vec2]) -> Vec<Vec2> {
    let segment_normals: Vec<Vec2> = points
        .windows(2)
        .map(|pair| {
            let delta = pair[1] - pair[0];
            let tangent = delta * (1.0 / delta.length());
            Vec2::new(-tangent.y, tangent.x)
        })
        .collect();
    (0..points.len())
        .map(|index| {
            if index == 0 {
                return segment_normals[0];
            }
            if index + 1 == points.len() {
                return segment_normals[index - 1];
            }
            miter(segment_normals[index - 1], segment_normals[index])
        })
        .collect()
}

fn miter(before: Vec2, after: Vec2) -> Vec2 {
    let sum = before + after;
    let direction = sum * (1.0 / sum.length());
    let alignment = direction.x * before.x + direction.y * before.y;
    if alignment > 0.5 {
        return direction * (1.0 / alignment).min(2.0);
    }
    direction
}
