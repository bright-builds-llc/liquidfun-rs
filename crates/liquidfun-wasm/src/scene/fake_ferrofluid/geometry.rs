//! Fake Ferrofluid geometry: a wide shallow static dish, the puddle box and
//! the magnet's resting place under the floor (toy meter scale, D-15).

use liquidfun::math::Vec2;

use crate::scene::RigidSegment;

/// Liquid particle radius: small enough for slender spikes (D-15).
pub(super) const PARTICLE_RADIUS: f32 = 0.04;
/// Liquid particle diameter, the width of one surface-profile bin.
#[cfg(test)]
pub(super) const PARTICLE_DIAMETER: f32 = 2.0 * PARTICLE_RADIUS;
/// Inner wall faces sit at `x = +-DISH_INNER_HALF_WIDTH`: a 6 m wide dish.
pub(super) const DISH_INNER_HALF_WIDTH: f32 = 3.0;
/// Walls are several particle diameters thick so nothing tunnels through.
pub(super) const WALL_THICKNESS: f32 = 0.3;
/// Outer wall faces and the floor slab ends.
pub(super) const DISH_OUTER_HALF_WIDTH: f32 = DISH_INNER_HALF_WIDTH + WALL_THICKNESS;
/// Top face of the floor slab: the puddle rests on `y = 0`.
pub(super) const FLOOR_TOP_Y: f32 = 0.0;
/// Bottom face of the floor slab; the resting magnet sits flush below it.
pub(super) const FLOOR_BOTTOM_Y: f32 = -0.25;
/// Wall tops: the tallest measured column plus at least a 0.3 m margin, so
/// no liquid climbs over (Research Pitfall 2). With a magnet held inside the
/// puddle at full strength the tallest column measured 1.21 m with these
/// walls but 1.42 m with 1.6 m walls, so 1.8 m is the lowest tested height
/// that keeps the margin (48-01 spike).
pub(super) const WALL_TOP_Y: f32 = 1.8;
/// Depth of the filled puddle box before it settles.
pub(super) const PUDDLE_DEPTH: f32 = 0.6;
/// The puddle box fills the dish floor, one radius clear of every face.
pub(super) const PUDDLE_HALF_WIDTH: f32 = DISH_INNER_HALF_WIDTH - PARTICLE_RADIUS;
/// Half of the puddle box height.
pub(super) const PUDDLE_HALF_HEIGHT: f32 = PUDDLE_DEPTH / 2.0;
/// Center of the puddle box, one radius above the floor.
pub(super) const PUDDLE_CENTER: Vec2 =
    Vec2::new(0.0, FLOOR_TOP_Y + PARTICLE_RADIUS + PUDDLE_HALF_HEIGHT);
/// Magnet rest position and field origin: under the floor center, with the
/// bar's top face flush with the slab bottom (D-06).
pub(super) const MAGNET_REST: Vec2 = Vec2::new(0.0, -0.35);
/// Half the bar magnet's length, along the dish.
#[cfg(test)]
pub(super) const MAGNET_HALF_LENGTH: f32 = 0.4;
/// Half the bar magnet's height, along the field axis.
#[cfg(test)]
pub(super) const MAGNET_HALF_HEIGHT: f32 = 0.1;

/// Scene frame: the dish and the resting marker plus a 0.3 m side margin and
/// at least 0.6 m of headroom above the wall tops (UI-D83). The web
/// `FAKE_FERROFLUID_VIEW_BOUNDS` mirrors these values.
#[cfg(test)]
pub(super) const FRAME_MIN_X: f32 = -3.6;
#[cfg(test)]
pub(super) const FRAME_MAX_X: f32 = 3.6;
#[cfg(test)]
pub(super) const FRAME_MIN_Y: f32 = -0.8;
#[cfg(test)]
pub(super) const FRAME_MAX_Y: f32 = WALL_TOP_Y + 0.6;

#[cfg(test)]
const _: () = assert!(FRAME_MAX_Y >= WALL_TOP_Y + 0.6);
#[cfg(test)]
const _: () = assert!(FRAME_MAX_X >= DISH_OUTER_HALF_WIDTH + 0.3);
#[cfg(test)]
const _: () = assert!(FRAME_MIN_X <= -DISH_OUTER_HALF_WIDTH - 0.3);

/// The dish as counter-clockwise convex quads, in a fixed order: floor slab,
/// left wall, right wall. The walls overlap the slab so the seams are sealed.
pub(super) fn dish_quads() -> [[Vec2; 4]; 3] {
    let outer = DISH_OUTER_HALF_WIDTH;
    let inner = DISH_INNER_HALF_WIDTH;
    [
        [
            Vec2::new(-outer, FLOOR_BOTTOM_Y),
            Vec2::new(outer, FLOOR_BOTTOM_Y),
            Vec2::new(outer, FLOOR_TOP_Y),
            Vec2::new(-outer, FLOOR_TOP_Y),
        ],
        [
            Vec2::new(-outer, FLOOR_BOTTOM_Y),
            Vec2::new(-inner, FLOOR_BOTTOM_Y),
            Vec2::new(-inner, WALL_TOP_Y),
            Vec2::new(-outer, WALL_TOP_Y),
        ],
        [
            Vec2::new(inner, FLOOR_BOTTOM_Y),
            Vec2::new(outer, FLOOR_BOTTOM_Y),
            Vec2::new(outer, WALL_TOP_Y),
            Vec2::new(inner, WALL_TOP_Y),
        ],
    ]
}

/// Closed outline loops of every dish quad, in `dish_quads` order.
pub(super) fn dish_segments() -> Vec<RigidSegment> {
    let mut segments = Vec::new();
    for quad in &dish_quads() {
        push_loop(&mut segments, quad);
    }
    segments
}

/// Whether a particle center has left the dish interior.
#[cfg(test)]
pub(super) fn out_of_dish(position: Vec2) -> bool {
    position.x.abs() > DISH_INNER_HALF_WIDTH + PARTICLE_RADIUS || position.y < -PARTICLE_RADIUS
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
