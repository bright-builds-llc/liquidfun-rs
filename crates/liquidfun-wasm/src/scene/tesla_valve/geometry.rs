//! Analytic wall centerlines, thick overlapping boxes, and solid teardrops.

use std::f32::consts::TAU;

use liquidfun::math::Vec2;

use super::super::RigidSegment;

pub(super) const MIRROR_Y: f32 = 1.48;
pub(super) const PARTICLE_RADIUS: f32 = 0.005;
pub(super) const WALL_HALF_THICKNESS: f32 = 0.03;
const WALL_END_OVERLAP: f32 = 0.01;
pub(super) const DRAIN_TOP_Y: f32 = 0.32;
pub(super) const DRAIN_HALF_WIDTH: f32 = 0.7;
pub(super) const DRAIN_HALF_HEIGHT: f32 = 0.22;
pub(super) const DRAIN_CENTER: Vec2 = Vec2::new(0.0, DRAIN_TOP_Y - DRAIN_HALF_HEIGHT);
pub(super) const SPAWN_Y: f32 = 2.86;
const SOURCE_COLUMNS: u32 = 24;
const SOURCE_SLOTS: u32 = SOURCE_COLUMNS * 2;
const SOURCE_SPACING: f32 = 0.014;
#[cfg(test)]
pub(super) const FRAME_MIN_X: f32 = -0.70;
#[cfg(test)]
pub(super) const FRAME_MAX_X: f32 = 0.58;
#[cfg(test)]
pub(super) const FRAME_MIN_Y: f32 = -0.12;
#[cfg(test)]
pub(super) const FRAME_MAX_Y: f32 = 3.22;

const STAGE_OFFSET: f32 = 0.568;
const STAGE_OFFSETS: [f32; 4] = [0.0, STAGE_OFFSET, 2.0 * STAGE_OFFSET, 3.0 * STAGE_OFFSET];
const DOWNSTREAM_SCALE: f32 = 0.8;
const FIRST_STAGE_Y: f32 = 2.62;
const NECK_TRANSVERSE: f32 = 0.030;
pub(super) const CAP_SAMPLES: u16 = 28;
pub(super) const RETURN_SAMPLES: u16 = 20;
pub(super) const ISLAND_ARC_SAMPLES: u16 = 24;

/// Template coordinates are downstream distance and distance toward the lobe.
pub(super) const OUTER_CURVES: [[Vec2; 4]; 2] = [
    [
        Vec2::new(0.0, NECK_TRANSVERSE),
        Vec2::new(-0.24, 0.08),
        Vec2::new(-0.24, 0.434),
        Vec2::new(0.0, 0.434),
    ],
    [
        Vec2::new(0.0, 0.434),
        Vec2::new(0.15, 0.434),
        Vec2::new(0.80, 0.15),
        Vec2::new(1.136, NECK_TRANSVERSE),
    ],
];

pub(super) struct SplitterIsland {
    pub(super) center: Vec2,
    pub(super) radius: f32,
    pub(super) triangle: [Vec2; 3],
    pub(super) outline: Vec<Vec2>,
}

pub(super) struct ValveGeometry {
    pub(super) outer_paths: [Vec<Vec2>; 2],
    pub(super) splitters: [SplitterIsland; 4],
}

/// A frame walks distinct slots before reusing any location. The upper row
/// follows the lower row, so falling water clears it before the next frame.
pub(super) fn source_position(cursor: u32) -> Vec2 {
    let slot = cursor % SOURCE_SLOTS;
    let column = u16::try_from(slot % SOURCE_COLUMNS).expect("source columns fit u16");
    let row = u16::try_from(slot / SOURCE_COLUMNS).expect("source rows fit u16");
    Vec2::new(
        (f32::from(column) - 11.5) * SOURCE_SPACING,
        SPAWN_Y + f32::from(row) * SOURCE_SPACING,
    )
}

pub(super) fn valve_geometry(forward: bool) -> ValveGeometry {
    let splitters = std::array::from_fn(|stage| splitter(stage, forward));
    let template = outer_template();
    let outer_paths = std::array::from_fn(|side| {
        let mut points: Vec<Vec2> = [side, side + 2]
            .into_iter()
            .enumerate()
            .flat_map(|(index, stage)| {
                template
                    .iter()
                    .copied()
                    .skip(usize::from(index != 0))
                    .map(move |point| place_stage(point, stage, forward))
            })
            .collect();
        if !forward {
            points.reverse();
        }
        let sign = if side == 0 { 1.0 } else { -1.0 };
        points.insert(0, Vec2::new(sign * 0.38, 3.02));
        points.push(Vec2::new(sign * 0.38, 0.10));
        points
    });
    ValveGeometry {
        outer_paths,
        splitters,
    }
}

pub(super) struct WallBox {
    pub(super) center: Vec2,
    pub(super) half_length: f32,
    pub(super) angle: f32,
}

pub(super) fn wall_boxes(paths: &[Vec<Vec2>; 2]) -> Vec<WallBox> {
    paths
        .iter()
        .flat_map(|path| path.windows(2))
        .map(|edge| {
            let delta = edge[1] - edge[0];
            WallBox {
                center: (edge[0] + edge[1]) * 0.5,
                half_length: delta.length() * 0.5 + WALL_END_OVERLAP,
                angle: delta.y.atan2(delta.x),
            }
        })
        .collect()
}

fn outer_template() -> Vec<Vec2> {
    OUTER_CURVES
        .into_iter()
        .zip([CAP_SAMPLES, RETURN_SAMPLES])
        .enumerate()
        .flat_map(|(index, (curve, samples))| {
            (0..=samples)
                .skip(usize::from(index != 0))
                .map(move |step| cubic_point(curve, f32::from(step) / f32::from(samples)))
        })
        .collect()
}

pub(super) fn cubic_point(curve: [Vec2; 4], fraction: f32) -> Vec2 {
    let before = 1.0 - fraction;
    curve[0] * (before * before * before)
        + curve[1] * (3.0 * before * before * fraction)
        + curve[2] * (3.0 * before * fraction * fraction)
        + curve[3] * (fraction * fraction * fraction)
}

pub(super) fn place_stage(point: Vec2, stage: usize, forward: bool) -> Vec2 {
    let sign = if stage.is_multiple_of(2) { 1.0 } else { -1.0 };
    let placed = Vec2::new(
        sign * point.y,
        FIRST_STAGE_Y - DOWNSTREAM_SCALE * (point.x + STAGE_OFFSETS[stage]),
    );
    if forward { placed } else { mirror_y(placed) }
}

fn splitter(stage: usize, forward: bool) -> SplitterIsland {
    let center = place_stage(Vec2::new(0.04, 0.19), stage, true);
    let tip = place_stage(Vec2::new(0.53, 0.08), stage, true);
    let radius = 0.068;
    let delta = tip - center;
    let angle = delta.y.atan2(delta.x);
    let tangent_angle = (radius / delta.length()).acos();
    let start = angle + tangent_angle;
    let span = TAU - 2.0 * tangent_angle;
    let mut outline: Vec<Vec2> = (0..=ISLAND_ARC_SAMPLES)
        .map(|step| {
            let theta = start + span * f32::from(step) / f32::from(ISLAND_ARC_SAMPLES);
            center + Vec2::new(theta.cos(), theta.sin()) * radius
        })
        .collect();
    let triangle = [outline[0], outline[outline.len() - 1], tip];
    outline.push(tip);
    if forward {
        return SplitterIsland {
            center,
            radius,
            triangle,
            outline,
        };
    }
    SplitterIsland {
        center: mirror_y(center),
        radius,
        triangle: triangle.map(mirror_y),
        outline: outline.into_iter().map(mirror_y).collect(),
    }
}

pub(super) fn mirror_y(point: Vec2) -> Vec2 {
    Vec2::new(point.x, 2.0 * MIRROR_Y - point.y)
}

pub(super) fn outline_segments(forward: bool) -> Vec<RigidSegment> {
    let geometry = valve_geometry(forward);
    let mut segments: Vec<RigidSegment> = geometry
        .outer_paths
        .iter()
        .flat_map(|points| polyline(points))
        .collect();
    for island in geometry.splitters {
        segments.extend(polyline(&island.outline));
        segments.push(RigidSegment {
            start: island.triangle[2],
            end: island.outline[0],
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
