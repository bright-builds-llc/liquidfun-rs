//! Closed tank, explicit particle lattice and the seeded interface. The seed
//! is a periodic polynomial, not `sin`, so native and wasm builds produce
//! bit-identical starting positions.

use liquidfun::math::Vec2;
use liquidfun::math::settings::PARTICLE_STRIDE;

use crate::scene::RigidSegment;

/// Inner wall faces sit at `x = +-TANK_HALF_WIDTH`: a 2.8 m wide tank (D-11).
pub(super) const TANK_HALF_WIDTH: f32 = 1.4;
/// Top face of the floor slab; the light layer rests on it.
pub(super) const FLOOR_TOP_Y: f32 = 0.0;
/// Underside of the lid, just above the settled fluid top (D-06a).
pub(super) const LID_BOTTOM_Y: f32 = 1.6;
/// Walls, floor and lid are several particle diameters thick so nothing tunnels through.
pub(super) const WALL_THICKNESS: f32 = 0.3;
/// Highest lattice row. The column stays well under the engine pressure cap,
/// so it settles without slumping (D-06a).
pub(super) const FILL_TOP_Y: f32 = 1.5;
/// Mean interface height: the two layers start about equally deep (D-11).
pub(super) const INTERFACE_MEAN_Y: f32 = 0.75;
/// Inner tank height, the reference for the 20% finger depth line (D-06a).
#[cfg(test)]
pub(super) const TANK_INNER_HEIGHT: f32 = LID_BOTTOM_Y - FLOOR_TOP_Y;

/// Outer wall faces and the slab ends.
const TANK_OUTER_HALF_WIDTH: f32 = TANK_HALF_WIDTH + WALL_THICKNESS;
/// Bottom face of the floor slab.
const FLOOR_BOTTOM_Y: f32 = FLOOR_TOP_Y - WALL_THICKNESS;
/// Top face of the lid.
const LID_TOP_Y: f32 = LID_BOTTOM_Y + WALL_THICKNESS;

/// Scene frame: the outer tank box plus 0.3 m on every side (D-13). The web
/// `RAYLEIGH_TAYLOR_VIEW_BOUNDS` mirrors these values.
#[cfg(test)]
pub(super) const FRAME_MIN_X: f32 = -2.0;
/// Lower edge of the landscape frame.
#[cfg(test)]
pub(super) const FRAME_MIN_Y: f32 = -0.6;
/// Right edge of the landscape frame.
#[cfg(test)]
pub(super) const FRAME_MAX_X: f32 = 2.0;
/// Upper edge of the landscape frame.
#[cfg(test)]
pub(super) const FRAME_MAX_Y: f32 = 2.2;

/// One periodic interface mode: `troughs` dips across the tank, `amplitude`
/// meters deep. An amplitude of 0 disables the mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SeedMode {
    pub(super) troughs: u16,
    pub(super) amplitude: f32,
}

/// The tank as counter-clockwise convex quads, in a fixed order: floor, left
/// wall, right wall, lid. The walls overlap the floor and lid at the corners
/// so every seam is sealed.
pub(super) fn tank_quads() -> [[Vec2; 4]; 4] {
    let outer = TANK_OUTER_HALF_WIDTH;
    let inner = TANK_HALF_WIDTH;
    [
        box_quad(-outer, FLOOR_BOTTOM_Y, outer, FLOOR_TOP_Y),
        box_quad(-outer, FLOOR_BOTTOM_Y, -inner, LID_TOP_Y),
        box_quad(inner, FLOOR_BOTTOM_Y, outer, LID_TOP_Y),
        box_quad(-outer, LID_BOTTOM_Y, outer, LID_TOP_Y),
    ]
}

/// Closed outline loops of every tank quad, in `tank_quads` order.
pub(super) fn tank_segments() -> Vec<RigidSegment> {
    let mut segments = Vec::with_capacity(16);
    for quad in &tank_quads() {
        push_loop(&mut segments, quad);
    }
    segments
}

/// Parabolic sine stand-in: odd, period 1, `+1` at `u = 0.25` and `-1` at
/// `u = -0.25`. Uses no libm, so every build gives the same bits (D-04).
pub(super) fn wave(u: f32) -> f32 {
    let t = u - u.round();
    8.0 * t * (1.0 - 2.0 * t.abs())
}

/// Seeded interface height at `x`: troughs in the interior, crests at the
/// walls. Lattice points at or above it are heavy.
pub(super) fn interface_y(x: f32, modes: &[SeedMode; 2]) -> f32 {
    let s = (x + TANK_HALF_WIDTH) / (2.0 * TANK_HALF_WIDTH);
    modes.iter().fold(INTERFACE_MEAN_Y, |height, mode| {
        height - mode.amplitude * wave(f32::from(mode.troughs) * s - 0.25)
    })
}

/// Lattice spacing for `radius`: the engine's own particle stride.
pub(super) fn lattice_stride(radius: f32) -> f32 {
    PARTICLE_STRIDE * 2.0 * radius
}

/// Particle centers filling the inner tank up to `FILL_TOP_Y`, one radius
/// clear of the walls and floor, row-major from the bottom row. Integer
/// counters keep every coordinate exact (no accumulated sums).
pub(super) fn lattice(radius: f32) -> Vec<Vec2> {
    let stride = lattice_stride(radius);
    let x0 = -TANK_HALF_WIDTH + radius;
    let y0 = FLOOR_TOP_Y + radius;
    let mut points = Vec::new();
    for row in 0_u16.. {
        let y = y0 + f32::from(row) * stride;
        if y > FILL_TOP_Y {
            break;
        }
        for column in 0_u16.. {
            let x = x0 + f32::from(column) * stride;
            if x > TANK_HALF_WIDTH - radius {
                break;
            }
            points.push(Vec2::new(x, y));
        }
    }
    points
}

/// Splits lattice points into `(light, heavy)` at the seeded interface, each
/// in lattice order.
pub(super) fn split_layers(points: Vec<Vec2>, modes: &[SeedMode; 2]) -> (Vec<Vec2>, Vec<Vec2>) {
    let (heavy, light) = points
        .into_iter()
        .partition(|point| point.y >= interface_y(point.x, modes));
    (light, heavy)
}

/// The axis-aligned box from `(min_x, min_y)` to `(max_x, max_y)`, counter-clockwise.
fn box_quad(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> [Vec2; 4] {
    [
        Vec2::new(min_x, min_y),
        Vec2::new(max_x, min_y),
        Vec2::new(max_x, max_y),
        Vec2::new(min_x, max_y),
    ]
}

/// Appends `corners` as a closed loop of segments.
fn push_loop(segments: &mut Vec<RigidSegment>, corners: &[Vec2]) {
    for index in 0..corners.len() {
        segments.push(RigidSegment {
            start: corners[index],
            end: corners[(index + 1) % corners.len()],
        });
    }
}
