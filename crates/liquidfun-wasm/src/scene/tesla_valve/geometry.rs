//! Tesla valve walls, authored for downward flow.
//!
//! The main tube runs down the right. Each head is a loop on the left: it
//! leaves the tube, curves around, and turns back into the tube pointing
//! downstream. Reverse mirrors the heads about a horizontal axis, so those
//! returns point upstream and falling water is turned back into the loops.
//! The inlet, the right wall, and the drain stay put.

use liquidfun::math::Vec2;

use super::super::RigidSegment;

/// Horizontal axis of the valve heads. `y' = 2 * MIRROR_Y - y`.
pub(super) const MIRROR_Y: f32 = 1.48;
pub(super) const PARTICLE_RADIUS: f32 = 0.014;
/// Particles below this line, inside the drain width, are removed.
pub(super) const DRAIN_TOP_Y: f32 = 0.32;
pub(super) const DRAIN_HALF_WIDTH: f32 = 0.7;
pub(super) const DRAIN_HALF_HEIGHT: f32 = 0.22;
pub(super) const DRAIN_CENTER: Vec2 = Vec2::new(0.0, DRAIN_TOP_Y - DRAIN_HALF_HEIGHT);
pub(super) const SPAWN_Y: f32 = 2.86;
/// Three columns inside the main tube, to the right of the curved heads.
pub(super) const SPAWN_XS: [f32; 3] = [0.12, 0.20, 0.28];
/// Portrait and landscape frame. Keep in sync with the catalog view bounds.
///
/// The catalog owns the rectangle the canvas uses. These copies exist so the
/// wall test can reject a corner that leaves that frame.
#[cfg(test)]
pub(super) const FRAME_MIN_X: f32 = -0.70;
#[cfg(test)]
pub(super) const FRAME_MAX_X: f32 = 0.58;
#[cfg(test)]
pub(super) const FRAME_MIN_Y: f32 = -0.12;
#[cfg(test)]
pub(super) const FRAME_MAX_Y: f32 = 3.22;

const WALL_HALF_THICKNESS: f32 = 0.03;

/// Ends of the wall that turns a head back into the main tube.
///
/// `curve` sits out in the head. `tube` meets the straight conduit. Forward
/// points that return downstream. Reverse lifts `tube` above `curve`.
#[cfg(test)]
pub(super) struct TubeReturn {
    pub(super) curve: Vec2,
    pub(super) tube: Vec2,
}

struct Ribbon {
    quads: Vec<[Vec2; 4]>,
}

pub(super) fn collision_quads(forward: bool) -> Vec<[Vec2; 4]> {
    paths(forward)
        .into_iter()
        .map(|points| coarse(&points))
        .flat_map(|points| ribbon(&points).quads)
        .collect()
}

/// Drops centerline points that sit closer than the wall is thick.
///
/// A quad shorter than it is wide collapses in the polygon builder.
fn coarse(points: &[Vec2]) -> Vec<Vec2> {
    let mut kept = Vec::new();
    let Some(mut previous) = points.first().copied() else {
        return kept;
    };
    kept.push(previous);
    let count = points.len();
    for (index, point) in points.iter().copied().enumerate().skip(1) {
        let span = (point - previous).length();
        let is_end = index + 1 == count;
        if span >= 0.10 || (is_end && span >= 0.05) {
            kept.push(point);
            previous = point;
        } else if is_end && kept.len() > 1 {
            let end = kept.len() - 1;
            kept[end] = point;
        }
    }
    kept
}

/// Smooth centerlines. Collision stays a thick ribbon; the picture spends the
/// segment budget on the curve instead of on both edges of every wall.
pub(super) fn outline_segments(forward: bool) -> Vec<RigidSegment> {
    paths(forward)
        .into_iter()
        .flat_map(|points| polyline(&points))
        .collect()
}

pub(super) fn mirror_y(point: Vec2) -> Vec2 {
    Vec2::new(point.x, 2.0 * MIRROR_Y - point.y)
}

/// Return nozzles, curve end then tube end, in forward orientation.
#[cfg(test)]
pub(super) fn forward_tube_returns() -> [TubeReturn; 2] {
    [upper_outer(), lower_outer()].map(|outer| TubeReturn {
        curve: outer[outer.len() - 2],
        tube: outer[outer.len() - 1],
    })
}

const HEAD_STRIDE: f32 = 1.14;

fn paths(forward: bool) -> Vec<Vec<Vec2>> {
    let mut paths = vec![
        vec![Vec2::new(0.02, 3.02), Vec2::new(0.02, 2.42)],
        vec![Vec2::new(0.42, 3.02), Vec2::new(0.42, 0.40)],
    ];
    for curve in [
        upper_outer().to_vec(),
        lower_outer().to_vec(),
        upper_inner().to_vec(),
        lower_inner().to_vec(),
    ] {
        paths.push(placed(curve, forward));
    }
    paths
}

fn placed(points: Vec<Vec2>, forward: bool) -> Vec<Vec2> {
    if forward {
        return points;
    }
    points.into_iter().map(mirror_y).collect()
}

fn upper_outer() -> [Vec2; 16] {
    [
        Vec2::new(0.01, 2.40),
        Vec2::new(-0.08, 2.39),
        Vec2::new(-0.16, 2.36),
        Vec2::new(-0.23, 2.32),
        Vec2::new(-0.29, 2.26),
        Vec2::new(-0.33, 2.19),
        Vec2::new(-0.36, 2.11),
        Vec2::new(-0.38, 2.03),
        Vec2::new(-0.38, 1.94),
        Vec2::new(-0.36, 1.86),
        Vec2::new(-0.32, 1.79),
        Vec2::new(-0.27, 1.72),
        Vec2::new(-0.20, 1.67),
        Vec2::new(-0.08, 1.60),
        Vec2::new(0.04, 1.52),
        Vec2::new(0.12, 1.42),
    ]
}

fn lower_outer() -> [Vec2; 16] {
    upper_outer().map(|point| Vec2::new(point.x, point.y - HEAD_STRIDE))
}

fn upper_inner() -> [Vec2; 11] {
    [
        Vec2::new(-0.13, 2.13),
        Vec2::new(-0.16, 2.09),
        Vec2::new(-0.17, 2.05),
        Vec2::new(-0.18, 2.00),
        Vec2::new(-0.17, 1.95),
        Vec2::new(-0.16, 1.91),
        Vec2::new(-0.13, 1.87),
        Vec2::new(-0.06, 1.80),
        Vec2::new(0.06, 1.70),
        Vec2::new(0.16, 1.60),
        Vec2::new(0.28, 1.50),
    ]
}

fn lower_inner() -> [Vec2; 11] {
    upper_inner().map(|point| Vec2::new(point.x, point.y - HEAD_STRIDE))
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

fn ribbon(points: &[Vec2]) -> Ribbon {
    let normals = vertex_normals(points);
    let left: Vec<Vec2> = points
        .iter()
        .zip(&normals)
        .map(|(point, normal)| *point + *normal * WALL_HALF_THICKNESS)
        .collect();
    let right: Vec<Vec2> = points
        .iter()
        .zip(&normals)
        .map(|(point, normal)| *point - *normal * WALL_HALF_THICKNESS)
        .collect();
    let mut quads = Vec::with_capacity(points.len().saturating_sub(1));
    for index in 0..points.len() - 1 {
        quads.push([left[index], left[index + 1], right[index + 1], right[index]]);
    }
    Ribbon { quads }
}

fn vertex_normals(points: &[Vec2]) -> Vec<Vec2> {
    let mut segment_normals = Vec::with_capacity(points.len().saturating_sub(1));
    for pair in points.windows(2) {
        let delta = pair[1] - pair[0];
        let length = delta.length().max(1.0e-4);
        let tangent = delta * (1.0 / length);
        segment_normals.push(Vec2::new(-tangent.y, tangent.x));
    }
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
    let length = sum.length().max(1.0e-4);
    let direction = sum * (1.0 / length);
    let alignment = direction.x * before.x + direction.y * before.y;
    if alignment > 0.5 {
        return direction * (1.0 / alignment).min(2.0);
    }
    direction
}
