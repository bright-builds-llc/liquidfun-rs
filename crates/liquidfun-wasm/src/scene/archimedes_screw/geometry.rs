//! Tube frame and static geometry for the Archimedes Screw: housings, roof,
//! floor slab, reservoir, basin, weir, chute, the vane lattice, the spawn
//! grid and the clipped vane outlines.
//!
//! A world point is `TUBE_ORIGIN + s * AXIS + t * NORMAL`: `s` runs along the
//! tube and `t` is the height above the floor line. Vanes are cut along lines
//! of constant vane coordinate `q = s - (t - TUBE_GAP / 2) * VANE_TAN`. The
//! outlet end of the floor slab is cut parallel to the vanes. Every face a
//! vane closes on in water leans away from the vanes instead: the bottom
//! housing face leans further uphill, the top housing face stands square to
//! the axis and the inlet end of the floor slab is undercut. A closing vane
//! meets each one first at one end and squeezes the water out of an open
//! wedge instead of trapping a film between two parallel faces. All
//! directions are literals, so building the scene needs no trigonometry on
//! native or wasm.

use liquidfun::math::Vec2;

use crate::scene::RigidSegment;

/// Water particle radius in meters (D-12).
pub(super) const PARTICLE_RADIUS: f32 = 0.05;
/// Water particle diameter; the vane speed guard is measured against it (D-04).
pub(super) const PARTICLE_DIAMETER: f32 = 2.0 * PARTICLE_RADIUS;
/// Spawn grid spacing: 0.75 particle diameters, the `LiquidFun` default stride.
pub(super) const PARTICLE_STRIDE: f32 = 0.075;

/// Tube axis: cos and sin of the 30 degree tilt `TAU / 12`, written as
/// literals so neither build nor step calls trig (D-12).
pub(super) const AXIS: Vec2 = Vec2::new(0.866_025_4, 0.5);
/// Tube normal: the axis turned a quarter turn counter-clockwise.
pub(super) const NORMAL: Vec2 = Vec2::new(-0.5, 0.866_025_4);
/// World point of `s = 0`, `t = 0`: the bottom housing face on the floor line.
pub(super) const TUBE_ORIGIN: Vec2 = Vec2::new(-4.0, -0.3);
/// Interior gap between floor and roof: seven particle diameters (D-12).
pub(super) const TUBE_GAP: f32 = 0.7;
/// Floor slab, roof and housing thickness beyond the gap; thicker than the
/// vane embed so no vane tip ever pokes out of a wall.
pub(super) const SLAB_THICKNESS: f32 = 0.35;
/// Housing face to housing face along the tube; `VANE_COUNT * PITCH - HOUSING_LENGTH`.
pub(super) const TUBE_LENGTH: f32 = 6.0;
/// Solid end housings that hide every vane added or removed by a snap (D-03).
pub(super) const HOUSING_LENGTH: f32 = 1.0;
/// Floor opening over the reservoir, running up to the bottom housing face.
/// At least one pitch plus the vane overhang, so water in front of a vane
/// entering the housing always has a way out (research Pitfall 1).
pub(super) const INLET_LENGTH: f32 = 1.4;
/// Floor opening over the basin, running up to the top housing face.
pub(super) const OUTLET_LENGTH: f32 = 1.4;
/// Vane spacing; one screw revolution advances the vanes one pitch (D-01).
pub(super) const PITCH: f32 = 1.0;
/// Vane thickness measured normal to the vane: three particle diameters (D-04).
pub(super) const VANE_THICKNESS: f32 = 0.3;
/// How far each vane tip reaches into the floor and roof slabs (D-03, D-04).
pub(super) const VANE_EMBED: f32 = 0.1;
/// Vanes in the train: enough that one sits in each housing at a snap.
pub(super) const VANE_COUNT: u8 = 7;
/// Vane 0 center at phase 0: the middle of the bottom housing.
pub(super) const TRAIN_START_S: f32 = -HOUSING_LENGTH / 2.0;
/// Tangent of the vane slant from the tube normal, `TAU / 18` (20 degrees),
/// top leaning uphill so the vanes read as helix flight lines (D-05).
pub(super) const VANE_TAN: f32 = 0.363_970_23;
/// Cosine of the vane slant `TAU / 18`.
pub(super) const VANE_COS: f32 = 0.939_692_6;
/// Lean of the bottom housing face from the tube normal: `tan(TAU / 8)`
/// (45 degrees), steeper than the vanes. A reversing vane meets it at the
/// roof first, so the wedge in front of the vane opens down into the inlet.
pub(super) const BOTTOM_FACE_TAN: f32 = 1.0;
/// Lean of the top housing face: square to the tube axis, less than the
/// vanes. A rising vane meets it at the roof first, so the wedge in front of
/// the vane opens down through the outlet into the basin.
pub(super) const TOP_FACE_TAN: f32 = 0.0;
/// Lean of the inlet end of the floor slab: the vanes' mirror image, an
/// undercut whose wedge against a rising vane tip opens downward.
const INLET_END_TAN: f32 = -VANE_TAN;
/// Axial lean of a vane from its bottom tip to its top tip.
#[cfg(test)]
pub(super) const VANE_OVERHANG: f32 = (TUBE_GAP + VANE_EMBED) * VANE_TAN;
/// Half the vane width along `q`.
pub(super) const VANE_HALF_SPAN: f32 = VANE_THICKNESS / (2.0 * VANE_COS);

/// Inner face of the reservoir's left wall.
pub(super) const RESERVOIR_LEFT_X: f32 = -5.6;
/// Flat reservoir floor height; low enough to leave water a path under the bottom housing.
pub(super) const RESERVOIR_FLOOR_Y: f32 = -1.5;
/// Where the flat floor turns into the return ramp under the tube.
pub(super) const RAMP_START_X: f32 = -4.0;
/// Top-right end of the return ramp (about 20 degrees), under the chute, so
/// chute water runs back down to the inlet (D-07).
pub(super) const RAMP_END: Vec2 = Vec2::new(2.45, 0.85);
/// Thickness of the reservoir wall, floor and ramp.
const RESERVOIR_WALL_THICKNESS: f32 = 0.3;
/// How far the flat floor runs on under the ramp. Buries the ramp's start
/// edge, so no seam between the two quads reaches the wet corner: under
/// high pressure a particle slid down such a seam and out of the floor.
const FLOOR_UNDER_RAMP: f32 = 0.5;
/// Top of the reservoir's left wall.
const RESERVOIR_WALL_TOP_Y: f32 = 1.6;
/// Initial reservoir surface: high enough that each pocket seals about half
/// full and the inlet stays covered while the basin fills (D-08).
pub(super) const RESERVOIR_FILL_TOP_Y: f32 = 1.15;

/// Basin floor top; the basin starts empty.
pub(super) const BASIN_FLOOR_Y: f32 = 1.6;
/// Basin floor underside, clear of the reservoir surface below it.
pub(super) const BASIN_FLOOR_BOTTOM_Y: f32 = 1.35;
/// Left end of the basin floor, buried inside the tube floor slab.
pub(super) const BASIN_FLOOR_LEFT_X: f32 = -0.5;
/// Basin side of the weir; right of the outlet so outlet water lands in the basin.
pub(super) const WEIR_INNER_X: f32 = 1.45;
/// Chute side of the weir.
pub(super) const WEIR_OUTER_X: f32 = 1.7;
/// Overflow lip: above the outlet's lowest floor point and below the top
/// housing, so the outlet is submerged when the basin is at its lip (D-06).
pub(super) const WEIR_LIP_Y: f32 = 2.05;
/// Inner face of the chute's outer wall.
pub(super) const CHUTE_INNER_X: f32 = 2.2;
/// Outer face of the chute's outer wall.
pub(super) const CHUTE_OUTER_X: f32 = 2.45;
/// Top of the chute's outer wall.
pub(super) const CHUTE_TOP_Y: f32 = 2.8;
/// Bottom of the chute's outer wall, buried in the return ramp.
const CHUTE_WALL_BOTTOM_Y: f32 = 0.5;
/// Left end of the hood over the chute, buried in the top housing's underside.
pub(super) const HOOD_LEFT_X: f32 = 1.9;
/// Hood underside: above the chute wall's inner top corner, so the hood
/// buries it, and below the top housing's lower back corner.
pub(super) const HOOD_BOTTOM_Y: f32 = 2.72;
/// Hood top.
const HOOD_TOP_Y: f32 = 3.0;

/// Static wall polygons: housings, roof, floor slab, reservoir wall, floor
/// and ramp, basin floor, weir, chute wall and the hood that closes the gap
/// between the top housing and the chute wall. Splashes off the weir cannot
/// leave the scene over the chute, even at low gravity (D-20).
pub(super) const WALL_QUAD_COUNT: usize = 11;

/// Landscape frame: the bounding box of every static wall vertex plus about
/// 0.2 m, mirrored by the web `ARCHIMEDES_SCREW_VIEW_BOUNDS` (UI-D71).
#[cfg(test)]
pub(super) const FRAME_MIN_X: f32 = -6.1;
/// Lower edge of the landscape frame.
#[cfg(test)]
pub(super) const FRAME_MIN_Y: f32 = -2.0;
/// Right edge of the landscape frame.
#[cfg(test)]
pub(super) const FRAME_MAX_X: f32 = 2.65;
/// Upper edge of the landscape frame.
#[cfg(test)]
pub(super) const FRAME_MAX_Y: f32 = 4.45;

/// World y of the floor line at `q = INLET_LENGTH`, where a pocket seals (D-08).
#[cfg(test)]
pub(super) const INLET_SEAL_Y: f32 =
    TUBE_ORIGIN.y + (INLET_LENGTH - TUBE_GAP / 2.0 * VANE_TAN) * AXIS.y;

/// World y of the floor line at `q = INLET_LENGTH / 2`: a reservoir surface
/// at or above it still floods the inlet's lower half (D-08).
#[cfg(test)]
pub(super) const INLET_COVER_Y: f32 =
    TUBE_ORIGIN.y + (INLET_LENGTH / 2.0 - TUBE_GAP / 2.0 * VANE_TAN) * AXIS.y;

/// World point at tube coordinates `s` (along the axis) and `t` (above the floor line).
pub(super) fn tube_point(s: f32, t: f32) -> Vec2 {
    TUBE_ORIGIN + AXIS * s + NORMAL * t
}

/// World point at height `t` on the cut through `s = offset` at mid-height that leans by `lean_tan`.
fn cut_point(offset: f32, lean_tan: f32, t: f32) -> Vec2 {
    tube_point(offset + (t - TUBE_GAP / 2.0) * lean_tan, t)
}

/// World point at vane coordinate `q` and height `t`.
fn vane_point(q: f32, t: f32) -> Vec2 {
    cut_point(q, VANE_TAN, t)
}

/// Tube coordinates `(s, t)` of a world point.
fn tube_coordinates(point: Vec2) -> (f32, f32) {
    let offset = point - TUBE_ORIGIN;
    (offset.dot(AXIS), offset.dot(NORMAL))
}

/// Vane coordinate `q` for tube coordinates `(s, t)`.
fn vane_coordinate(s: f32, t: f32) -> f32 {
    s - (t - TUBE_GAP / 2.0) * VANE_TAN
}

/// A straight cut across the tube: its `s` at mid-height and its lean from the normal.
#[derive(Clone, Copy)]
struct Cut {
    offset: f32,
    lean_tan: f32,
}

/// A cut parallel to the vanes at vane coordinate `q`.
const fn vane_cut(q: f32) -> Cut {
    Cut {
        offset: q,
        lean_tan: VANE_TAN,
    }
}

/// Counter-clockwise quad between a low cut, a high cut and two heights.
fn cut_band(low: Cut, high: Cut, t_min: f32, t_max: f32) -> [Vec2; 4] {
    [
        cut_point(low.offset, low.lean_tan, t_min),
        cut_point(high.offset, high.lean_tan, t_min),
        cut_point(high.offset, high.lean_tan, t_max),
        cut_point(low.offset, low.lean_tan, t_max),
    ]
}

/// Counter-clockwise parallelogram between two vane-parallel cuts and two heights.
fn vane_band(q_min: f32, q_max: f32, t_min: f32, t_max: f32) -> [Vec2; 4] {
    cut_band(vane_cut(q_min), vane_cut(q_max), t_min, t_max)
}

/// Counter-clockwise corners of an axis-aligned box.
fn axis_box(min: Vec2, max: Vec2) -> [Vec2; 4] {
    [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)]
}

/// Solid bottom end of the vane track. Its face crosses mid-height at `s = 0`
/// and leans `BOTTOM_FACE_TAN`.
pub(super) fn bottom_housing() -> [Vec2; 4] {
    cut_band(
        vane_cut(-HOUSING_LENGTH),
        BOTTOM_FACE,
        -SLAB_THICKNESS,
        TUBE_GAP + SLAB_THICKNESS,
    )
}

/// The bottom housing face: crosses mid-height at `s = 0`, leaning `BOTTOM_FACE_TAN`.
const BOTTOM_FACE: Cut = Cut {
    offset: 0.0,
    lean_tan: BOTTOM_FACE_TAN,
};
/// The top housing face: crosses mid-height at `s = TUBE_LENGTH`, leaning `TOP_FACE_TAN`.
const TOP_FACE: Cut = Cut {
    offset: TUBE_LENGTH,
    lean_tan: TOP_FACE_TAN,
};

/// Solid top end of the vane track. Its face crosses mid-height at
/// `s = TUBE_LENGTH` and leans `TOP_FACE_TAN`.
pub(super) fn top_housing() -> [Vec2; 4] {
    cut_band(
        TOP_FACE,
        vane_cut(TUBE_LENGTH + HOUSING_LENGTH),
        -SLAB_THICKNESS,
        TUBE_GAP + SLAB_THICKNESS,
    )
}

/// Floor slab over the sealed run only: the inlet and outlet openings run
/// from its ends to the housing faces (D-06). Its top starts at vane
/// coordinate `INLET_LENGTH`; the inlet end is undercut by `INLET_END_TAN`.
pub(super) fn floor_slab() -> [Vec2; 4] {
    let inlet_end = Cut {
        offset: INLET_LENGTH - TUBE_GAP / 2.0 * (VANE_TAN - INLET_END_TAN),
        lean_tan: INLET_END_TAN,
    };
    cut_band(
        inlet_end,
        vane_cut(TUBE_LENGTH - OUTLET_LENGTH),
        -SLAB_THICKNESS,
        0.0,
    )
}

/// Reservoir floor height at `x`: flat, then the return ramp.
pub(super) fn reservoir_floor_y(x: f32) -> f32 {
    if x <= RAMP_START_X {
        return RESERVOIR_FLOOR_Y;
    }
    let slope = (RAMP_END.y - RESERVOIR_FLOOR_Y) / (RAMP_END.x - RAMP_START_X);
    RESERVOIR_FLOOR_Y + (x - RAMP_START_X) * slope
}

/// Every static wall polygon in a fixed order. Overlaps are allowed.
pub(super) fn wall_quads() -> [[Vec2; 4]; WALL_QUAD_COUNT] {
    let wall_bottom_y = RESERVOIR_FLOOR_Y - RESERVOIR_WALL_THICKNESS;
    let wall_outer_x = RESERVOIR_LEFT_X - RESERVOIR_WALL_THICKNESS;
    let ramp_start = Vec2::new(RAMP_START_X, RESERVOIR_FLOOR_Y);
    let below = Vec2::new(0.0, RESERVOIR_WALL_THICKNESS);
    [
        bottom_housing(),
        top_housing(),
        vane_band(
            -HOUSING_LENGTH / 2.0,
            TUBE_LENGTH + HOUSING_LENGTH / 2.0,
            TUBE_GAP,
            TUBE_GAP + SLAB_THICKNESS,
        ),
        floor_slab(),
        axis_box(
            Vec2::new(wall_outer_x, wall_bottom_y),
            Vec2::new(RESERVOIR_LEFT_X, RESERVOIR_WALL_TOP_Y),
        ),
        axis_box(
            Vec2::new(wall_outer_x, wall_bottom_y),
            Vec2::new(RAMP_START_X + FLOOR_UNDER_RAMP, RESERVOIR_FLOOR_Y),
        ),
        [ramp_start - below, RAMP_END - below, RAMP_END, ramp_start],
        axis_box(
            Vec2::new(BASIN_FLOOR_LEFT_X, BASIN_FLOOR_BOTTOM_Y),
            Vec2::new(WEIR_OUTER_X, BASIN_FLOOR_Y),
        ),
        axis_box(
            Vec2::new(WEIR_INNER_X, BASIN_FLOOR_BOTTOM_Y),
            Vec2::new(WEIR_OUTER_X, WEIR_LIP_Y),
        ),
        axis_box(
            Vec2::new(CHUTE_INNER_X, CHUTE_WALL_BOTTOM_Y),
            Vec2::new(CHUTE_OUTER_X, CHUTE_TOP_Y),
        ),
        axis_box(
            Vec2::new(HOOD_LEFT_X, HOOD_BOTTOM_Y),
            Vec2::new(CHUTE_OUTER_X, HOOD_TOP_Y),
        ),
    ]
}

/// Vane `index` center `q` at train phase `phase`.
pub(super) fn vane_center(index: u8, phase: f32) -> f32 {
    TRAIN_START_S + f32::from(index) * PITCH + phase
}

/// Vane polygons local to the train body at phase 0 (body origin `TUBE_ORIGIN`, angle 0).
pub(super) fn vane_quads() -> [[Vec2; 4]; VANE_COUNT as usize] {
    let mut quads = [[Vec2::ZERO; 4]; VANE_COUNT as usize];
    for (index, quad) in (0..VANE_COUNT).zip(quads.iter_mut()) {
        let center = vane_center(index, 0.0);
        let world = vane_band(
            center - VANE_HALF_SPAN,
            center + VANE_HALF_SPAN,
            -VANE_EMBED,
            TUBE_GAP + VANE_EMBED,
        );
        *quad = world.map(|corner| corner - TUBE_ORIGIN);
    }
    quads
}

/// Drawn outline of the vane centered at `q_center`, clipped only against
/// the two housing faces, or `None` while it is wholly inside a housing.
/// The embedded tips are drawn as they collide (UI-D69).
pub(super) fn visible_vane_outline(q_center: f32) -> Option<Vec<Vec2>> {
    let q_min = q_center - VANE_HALF_SPAN;
    let q_max = q_center + VANE_HALF_SPAN;
    let band = [
        (q_min, -VANE_EMBED),
        (q_max, -VANE_EMBED),
        (q_max, TUBE_GAP + VANE_EMBED),
        (q_min, TUBE_GAP + VANE_EMBED),
    ];
    let above_bottom = clip_by_face(&band, |point| uphill_of(BOTTOM_FACE, point));
    let visible = clip_by_face(&above_bottom, |point| -uphill_of(TOP_FACE, point));
    if visible.len() < 3 {
        return None;
    }
    Some(visible.into_iter().map(|(q, t)| vane_point(q, t)).collect())
}

/// How far vane-coordinate point `(q, t)` lies uphill of `face` along the axis.
fn uphill_of(face: Cut, (q, t): (f32, f32)) -> f32 {
    q - face.offset + (t - TUBE_GAP / 2.0) * (VANE_TAN - face.lean_tan)
}

/// Keeps the part of a convex `(q, t)` polygon where `clearance` is not
/// negative (one Sutherland-Hodgman pass).
fn clip_by_face(polygon: &[(f32, f32)], clearance: impl Fn((f32, f32)) -> f32) -> Vec<(f32, f32)> {
    let mut kept = Vec::with_capacity(polygon.len() + 1);
    for index in 0..polygon.len() {
        let current = polygon[index];
        let next = polygon[(index + 1) % polygon.len()];
        let current_clearance = clearance(current);
        let next_clearance = clearance(next);
        if current_clearance >= 0.0 {
            kept.push(current);
        }
        if (current_clearance >= 0.0) != (next_clearance >= 0.0) {
            let along = current_clearance / (current_clearance - next_clearance);
            kept.push((
                current.0 + (next.0 - current.0) * along,
                current.1 + (next.1 - current.1) * along,
            ));
        }
    }
    kept
}

/// Each static wall polygon as a closed four-segment loop, in `wall_quads` order.
pub(super) fn wall_segments() -> Vec<RigidSegment> {
    let quads = wall_quads();
    let mut segments = Vec::with_capacity(quads.len() * 4);
    for quad in &quads {
        push_loop(&mut segments, quad);
    }
    segments
}

/// Appends `corners` as a closed loop of segments.
pub(super) fn push_loop(segments: &mut Vec<RigidSegment>, corners: &[Vec2]) {
    for index in 0..corners.len() {
        segments.push(RigidSegment {
            start: corners[index],
            end: corners[(index + 1) % corners.len()],
        });
    }
}

/// Whether `point` lies inside a counter-clockwise convex polygon, edges included.
pub(super) fn quad_contains(quad: &[Vec2; 4], point: Vec2) -> bool {
    (0..quad.len()).all(|index| {
        let start = quad[index];
        let end = quad[(index + 1) % quad.len()];
        (end - start).cross(point - start) >= 0.0
    })
}

/// Shortest distance from `point` to the boundary of `quad`.
pub(super) fn distance_to_boundary(quad: &[Vec2; 4], point: Vec2) -> f32 {
    (0..quad.len())
        .map(|index| segment_distance(point, quad[index], quad[(index + 1) % quad.len()]))
        .fold(f32::INFINITY, f32::min)
}

fn segment_distance(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let edge = end - start;
    let length_squared = edge.length_squared();
    if length_squared <= f32::EPSILON {
        return (point - start).length();
    }
    let along = ((point - start).dot(edge) / length_squared).clamp(0.0, 1.0);
    (point - (start + edge * along)).length()
}

/// Whether `point` is at least `clearance` outside `quad`.
fn clear_of(quad: &[Vec2; 4], point: Vec2, clearance: f32) -> bool {
    !quad_contains(quad, point) && distance_to_boundary(quad, point) >= clearance
}

/// Whether a pocket between vanes `index` and `index + 1` at phase 0 lies wholly over the floor slab.
fn prefilled_pocket(q: f32) -> bool {
    (0..VANE_COUNT - 1).any(|index| {
        let low = vane_center(index, 0.0) + VANE_HALF_SPAN;
        let high = vane_center(index + 1, 0.0) - VANE_HALF_SPAN;
        low >= INLET_LENGTH && high <= TUBE_LENGTH - OUTLET_LENGTH && (low..=high).contains(&q)
    })
}

/// Whether a grid point belongs to the initial water: the reservoir up to
/// its fill line (including the flooded inlet), plus the lower half of each
/// sealed pocket. The basin and chute start empty (D-08).
fn wanted_water(point: Vec2) -> bool {
    if in_tube(point) {
        let (s, t) = tube_coordinates(point);
        let q = vane_coordinate(s, t);
        return (q <= INLET_LENGTH && point.y <= RESERVOIR_FILL_TOP_Y)
            || (t <= TUBE_GAP / 2.0 && prefilled_pocket(q));
    }
    (RESERVOIR_LEFT_X..=CHUTE_INNER_X).contains(&point.x)
        && point.y >= reservoir_floor_y(point.x)
        && point.y <= RESERVOIR_FILL_TOP_Y
}

/// Water grid points in row-major order, each at least one particle radius
/// clear of every wall and every vane at phase 0 (research Pitfall 3).
pub(super) fn water_positions() -> Vec<Vec2> {
    let walls = wall_quads();
    let vanes = vane_quads().map(|quad| quad.map(|corner| corner + TUBE_ORIGIN));
    let grid_min = Vec2::new(
        RESERVOIR_LEFT_X + PARTICLE_RADIUS,
        RESERVOIR_FLOOR_Y + PARTICLE_RADIUS,
    );
    let mut positions = Vec::new();
    let mut row: u16 = 0;
    loop {
        let y = grid_min.y + f32::from(row) * PARTICLE_STRIDE;
        if y > WEIR_LIP_Y {
            break;
        }
        let mut column: u16 = 0;
        loop {
            let x = grid_min.x + f32::from(column) * PARTICLE_STRIDE;
            if x > CHUTE_INNER_X {
                break;
            }
            let point = Vec2::new(x, y);
            let clear = walls
                .iter()
                .chain(vanes.iter())
                .all(|quad| clear_of(quad, point, PARTICLE_RADIUS));
            if clear && wanted_water(point) {
                positions.push(point);
            }
            column += 1;
        }
        row += 1;
    }
    positions
}

/// Inside the open tube interior between the housing faces.
pub(super) fn in_tube(point: Vec2) -> bool {
    let (s, t) = tube_coordinates(point);
    (0.0..=TUBE_LENGTH).contains(&s) && (0.0..=TUBE_GAP).contains(&t)
}

/// Inside the basin, from its floor up past the lip toward the top housing,
/// below the tube floor line so outlet water falling out counts as basin.
#[cfg(test)]
pub(super) fn in_basin(point: Vec2) -> bool {
    let (_, t) = tube_coordinates(point);
    t < 0.0
        && (BASIN_FLOOR_LEFT_X..=WEIR_OUTER_X).contains(&point.x)
        && (BASIN_FLOOR_Y - PARTICLE_RADIUS..=CHUTE_TOP_Y).contains(&point.y)
}

/// Inside the reservoir trough below the basin floor and outside the tube interior.
#[cfg(test)]
pub(super) fn in_reservoir(point: Vec2) -> bool {
    !in_tube(point)
        && (RESERVOIR_LEFT_X - PARTICLE_RADIUS..=CHUTE_INNER_X + PARTICLE_RADIUS).contains(&point.x)
        && point.y >= reservoir_floor_y(point.x) - PARTICLE_RADIUS
        && point.y < BASIN_FLOOR_BOTTOM_Y
}

/// Inside the return chute between the weir and the chute wall, above the reservoir.
#[cfg(test)]
pub(super) fn in_chute(point: Vec2) -> bool {
    (WEIR_OUTER_X - PARTICLE_RADIUS..=CHUTE_INNER_X + PARTICLE_RADIUS).contains(&point.x)
        && (BASIN_FLOOR_BOTTOM_Y..=CHUTE_TOP_Y).contains(&point.y)
}

/// Tube-frame vane coordinate `q` of a world point.
#[cfg(test)]
pub(super) fn vane_q(point: Vec2) -> f32 {
    let (s, t) = tube_coordinates(point);
    vane_coordinate(s, t)
}
