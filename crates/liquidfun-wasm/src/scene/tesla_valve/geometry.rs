//! Tesla valve walls, authored for downward flow.
//!
//! Forward ramps slope toward a side gap, so falling water slides off and
//! drops to the next stage. Reverse mirrors those ramps about a horizontal
//! axis: each slope then runs away from its gap and water pools against the
//! closed end. The inlet duct and the drain stay put, so gravity, the source,
//! and the sink do not move when the valve flips.

use liquidfun::math::Vec2;

use super::super::RigidSegment;

/// Horizontal axis of the valve body. `y' = 2 * MIRROR_Y - y`.
pub(super) const MIRROR_Y: f32 = 1.55;
pub(super) const PARTICLE_RADIUS: f32 = 0.014;
/// Particles below this line, inside the drain width, are removed.
pub(super) const DRAIN_TOP_Y: f32 = 0.32;
pub(super) const DRAIN_HALF_WIDTH: f32 = 0.7;
pub(super) const DRAIN_HALF_HEIGHT: f32 = 0.22;
pub(super) const DRAIN_CENTER: Vec2 = Vec2::new(0.0, DRAIN_TOP_Y - DRAIN_HALF_HEIGHT);
pub(super) const SPAWN_Y: f32 = 2.84;
pub(super) const SPAWN_XS: [f32; 3] = [-0.04, 0.0, 0.04];
/// Portrait and landscape frame. Keep in sync with the catalog view bounds.
///
/// The catalog owns the rectangle the canvas uses. These copies exist so the
/// wall test can reject a corner that leaves that frame.
#[cfg(test)]
pub(super) const FRAME_MIN_X: f32 = -0.78;
#[cfg(test)]
pub(super) const FRAME_MAX_X: f32 = 0.78;
#[cfg(test)]
pub(super) const FRAME_MIN_Y: f32 = -0.16;
#[cfg(test)]
pub(super) const FRAME_MAX_Y: f32 = 3.28;

const RAMP_HALF_THICKNESS: f32 = 0.035;

pub(super) fn collision_quads(forward: bool) -> Vec<[Vec2; 4]> {
    let mut quads = shell_quads();
    for ramp in forward_ramps() {
        let placed = if forward { ramp } else { mirror_quad(ramp) };
        quads.push(placed);
    }
    quads
}

pub(super) fn outlines(quads: &[[Vec2; 4]]) -> Vec<RigidSegment> {
    let mut segments = Vec::with_capacity(quads.len() * 4);
    for corners in quads {
        for index in 0..corners.len() {
            segments.push(RigidSegment {
                start: corners[index],
                end: corners[(index + 1) % corners.len()],
            });
        }
    }
    segments
}

pub(super) fn mirror_y(point: Vec2) -> Vec2 {
    Vec2::new(point.x, 2.0 * MIRROR_Y - point.y)
}

fn shell_quads() -> Vec<[Vec2; 4]> {
    vec![
        aabb(-0.58, 0.48, -0.50, 2.62),
        aabb(0.50, 0.48, 0.58, 2.62),
        aabb(-0.16, 2.58, -0.08, 3.02),
        aabb(0.08, 2.58, 0.16, 3.02),
    ]
}

/// Three teeth. The closed end is listed first and the lip, where water falls
/// off, is listed second.
fn forward_ramps() -> [[Vec2; 4]; 3] {
    [
        ramp_quad(Vec2::new(-0.58, 2.38), Vec2::new(0.22, 2.02)),
        ramp_quad(Vec2::new(0.58, 1.78), Vec2::new(-0.22, 1.42)),
        ramp_quad(Vec2::new(-0.58, 1.18), Vec2::new(0.22, 0.82)),
    ]
}

fn ramp_quad(start: Vec2, end: Vec2) -> [Vec2; 4] {
    let delta = end - start;
    let length = delta.length();
    let tangent = delta * (1.0 / length);
    let normal = Vec2::new(-tangent.y, tangent.x);
    let offset = normal * RAMP_HALF_THICKNESS;
    [start + offset, end + offset, end - offset, start - offset]
}

fn mirror_quad(quad: [Vec2; 4]) -> [Vec2; 4] {
    quad.map(mirror_y)
}

fn aabb(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> [Vec2; 4] {
    [
        Vec2::new(min_x, min_y),
        Vec2::new(max_x, min_y),
        Vec2::new(max_x, max_y),
        Vec2::new(min_x, max_y),
    ]
}
