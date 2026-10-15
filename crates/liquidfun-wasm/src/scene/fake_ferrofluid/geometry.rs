//! Fake Ferrofluid geometry: a wide shallow static dish, the puddle box and
//! the magnet's resting place under the floor (toy meter scale, D-15).

use liquidfun::math::Vec2;

use super::field::FIELD;
use crate::scene::RigidSegment;
use crate::scene::particle_force::GravityFrame;

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
/// no liquid climbs over (Research Pitfall 2). The 48-01 field needed 1.8 m
/// (tallest column 1.21 m). Since the sideways pull uses `|E(h)|` (48-02), a
/// magnet held 1.3 to 1.5 m up gathers the liquid below it into columns that
/// rise to about 2.67 m, above 2.5 m walls, so the walls are 3.0 m.
pub(super) const WALL_TOP_Y: f32 = 3.0;
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
pub(super) const MAGNET_HALF_LENGTH: f32 = 0.4;
/// Half the bar magnet's height, along the field axis.
pub(super) const MAGNET_HALF_HEIGHT: f32 = 0.1;

/// Gap between the bar's top face and the field ticks, in meters.
const TICK_GAP: f32 = 0.04;
/// Length of each field tick, in meters.
const TICK_LENGTH: f32 = 0.16;
/// Field tick offsets along the bar from its center, in meters.
const TICK_OFFSETS: [f32; 3] = [-0.25, 0.0, 0.25];

/// Scene frame: the dish and the resting marker plus a 0.3 m side margin and
/// at least 0.6 m of headroom above the wall tops (UI-D83). The web
/// `FAKE_FERROFLUID_VIEW_BOUNDS` mirrors these values.
pub(super) const FRAME_MIN_X: f32 = -3.6;
pub(super) const FRAME_MAX_X: f32 = 3.6;
pub(super) const FRAME_MIN_Y: f32 = -0.8;
pub(super) const FRAME_MAX_Y: f32 = WALL_TOP_Y + 0.6;

/// Highest magnet center a drag may reach: one reach below the wall tops.
/// The lift is even about the magnet, so liquid near a higher magnet floats
/// with it; a slow drag up and across then carried a blob over the wall and
/// out of the dish (WR-02).
pub(super) const MAGNET_MAX_Y: f32 = WALL_TOP_Y - FIELD.reach;

const _: () = assert!(FRAME_MAX_Y >= WALL_TOP_Y + 0.6);
const _: () = assert!(MAGNET_MAX_Y <= FRAME_MAX_Y - MAGNET_HALF_HEIGHT);
const _: () = assert!(FRAME_MAX_X >= DISH_OUTER_HALF_WIDTH + 0.3);
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

/// Clamps a magnet position to the frame inset by the bar half extents, so
/// the whole marker stays on screen (UI-D82), and below `MAGNET_MAX_Y`, so a
/// drag cannot lift liquid over the walls (D-09).
pub(super) fn clamp_magnet(position: Vec2) -> Vec2 {
    Vec2::new(
        position.x.clamp(
            FRAME_MIN_X + MAGNET_HALF_LENGTH,
            FRAME_MAX_X - MAGNET_HALF_LENGTH,
        ),
        position
            .y
            .clamp(FRAME_MIN_Y + MAGNET_HALF_HEIGHT, MAGNET_MAX_Y),
    )
}

/// The bar-magnet marker centered on `center`, drawn in the gravity frame so
/// it tilts with gravity (UI-D81, D-14): the outline always, plus the pole
/// divider and three field ticks while the magnet is on. Without a gravity
/// frame it falls back to an axis-aligned outline.
pub(super) fn magnet_marker(
    center: Vec2,
    maybe_frame: Option<GravityFrame>,
    on: bool,
) -> Vec<RigidSegment> {
    let Some(frame) = maybe_frame else {
        return bar_outline(center, Vec2::new(0.0, 1.0), Vec2::new(1.0, 0.0));
    };
    let mut segments = bar_outline(center, frame.up, frame.across);
    if !on {
        return segments;
    }
    let half_height = frame.up * MAGNET_HALF_HEIGHT;
    segments.push(RigidSegment {
        start: center - half_height,
        end: center + half_height,
    });
    for offset in TICK_OFFSETS {
        let start = center + frame.across * offset + frame.up * (MAGNET_HALF_HEIGHT + TICK_GAP);
        segments.push(RigidSegment {
            start,
            end: start + frame.up * TICK_LENGTH,
        });
    }
    segments
}

/// The bar's closed outline: long sides along `across`, short along `up`.
fn bar_outline(center: Vec2, up: Vec2, across: Vec2) -> Vec<RigidSegment> {
    let half_length = across * MAGNET_HALF_LENGTH;
    let half_height = up * MAGNET_HALF_HEIGHT;
    let corners = [
        center - half_length - half_height,
        center + half_length - half_height,
        center + half_length + half_height,
        center - half_length + half_height,
    ];
    let mut segments = Vec::with_capacity(8);
    push_loop(&mut segments, &corners);
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
