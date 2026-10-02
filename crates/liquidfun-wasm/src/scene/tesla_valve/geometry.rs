//! Constant-width winding trunk and circular semicircular bypasses.
//!
//! All placements are rigid. Drawn borders are wall centerlines; overlapping
//! checked boxes give them the same physical thickness, including the islands.

use super::super::RigidSegment;
use liquidfun::math::Vec2;
use std::f32::consts::TAU;

pub(super) const MIRROR_Y: f32 = 2.15;
pub(super) const PARTICLE_RADIUS: f32 = 0.005;
pub(super) const WALL_HALF_THICKNESS: f32 = 0.03;
pub(super) const CLEAR_WIDTH: f32 = 0.06;
pub(super) const BORDER_HALF_WIDTH: f32 = CLEAR_WIDTH * 0.5 + WALL_HALF_THICKNESS;
pub(super) const BYPASS_RADIUS: f32 = 0.17;
pub(super) const TRUNK_RADIUS: f32 = 0.30;
const WALL_END_OVERLAP: f32 = 0.01;
const LEG_PITCH: f32 = 0.85;
const LEG_ANGLE: f32 = TAU / 18.0;
const BRANCH_ANGLE: f32 = TAU / 6.0;
const OUTER_SAMPLES: u16 = 56;
const INNER_SAMPLES: u16 = 24;
const CORNER_SAMPLES: u16 = 12;
pub(super) const DRAIN_TOP_Y: f32 = 0.32;
pub(super) const DRAIN_HALF_WIDTH: f32 = 0.7;
pub(super) const DRAIN_HALF_HEIGHT: f32 = 0.22;
pub(super) const DRAIN_CENTER: Vec2 = Vec2::new(0.0, DRAIN_TOP_Y - DRAIN_HALF_HEIGHT);
const SOURCE_COLUMNS: u32 = 4;
const SOURCE_SLOTS: u32 = SOURCE_COLUMNS * 12;
pub(super) const SOURCE_SPACING: f32 = 0.0102;
#[cfg(test)]
pub(super) const FRAME_MIN_X: f32 = -0.70;
#[cfg(test)]
pub(super) const FRAME_MAX_X: f32 = 0.58;
#[cfg(test)]
pub(super) const FRAME_MIN_Y: f32 = -0.12;
#[cfg(test)]
pub(super) const FRAME_MAX_Y: f32 = 4.15;

#[derive(Clone, Copy)]
pub(super) struct CircularArc {
    pub(super) center: Vec2,
    pub(super) radius: f32,
    pub(super) start: f32,
    pub(super) sweep: f32,
    pub(super) samples: u16,
}

impl CircularArc {
    pub(super) fn point(self, fraction: f32) -> Vec2 {
        let angle = self.start + self.sweep * fraction;
        self.center + Vec2::new(angle.cos(), angle.sin()) * self.radius
    }

    pub(super) fn points(self) -> Vec<Vec2> {
        (0..=self.samples)
            .map(|step| self.point(f32::from(step) / f32::from(self.samples)))
            .collect()
    }

    #[cfg(test)]
    fn reflected(self) -> Self {
        Self {
            center: mirror_y(self.center),
            start: -self.start,
            sweep: -self.sweep,
            ..self
        }
    }

    fn border(self, radius: f32, samples: u16) -> Self {
        Self {
            radius,
            samples,
            ..self
        }
    }
}

pub(super) struct SplitterIsland {
    pub(super) center: Vec2,
    pub(super) radius: f32,
    pub(super) stem: [Vec2; 4],
    pub(super) outline: Vec<Vec2>,
}

pub(super) struct ValveGeometry {
    pub(super) outer_paths: [Vec<Vec2>; 2],
    pub(super) splitters: [SplitterIsland; 4],
    #[cfg(test)]
    pub(super) bypass_arcs: [CircularArc; 4],
    #[cfg(test)]
    pub(super) trunk_arcs: [CircularArc; 3],
}

#[derive(Clone, Copy)]
struct Leg {
    vertex: Vec2,
    direction: Vec2,
    normal: Vec2,
    outward: Vec2,
    side: f32,
}

fn legs() -> [Leg; 4] {
    let amplitude = 0.5 * LEG_PITCH * LEG_ANGLE.tan();
    std::array::from_fn(|index| {
        let y = [3.85, 3.0, 2.15, 1.30][index];
        let side = if index.is_multiple_of(2) { 1.0 } else { -1.0 };
        let direction = Vec2::new(side * LEG_ANGLE.sin(), -LEG_ANGLE.cos());
        let normal = Vec2::new(-direction.y, direction.x);
        Leg {
            vertex: Vec2::new(-side * amplitude, y),
            direction,
            normal,
            outward: normal * side,
            side,
        }
    })
}

fn leg_length() -> f32 {
    LEG_PITCH / LEG_ANGLE.cos()
}
fn trim() -> f32 {
    TRUNK_RADIUS * LEG_ANGLE.tan()
}
fn entry_length() -> f32 {
    BORDER_HALF_WIDTH * (1.0 + BRANCH_ANGLE.cos()) / BRANCH_ANGLE.sin()
}
fn return_span() -> f32 {
    2.0 * BYPASS_RADIUS / BRANCH_ANGLE.sin() + BORDER_HALF_WIDTH / BRANCH_ANGLE.tan()
}
fn anchor_distance() -> f32 {
    (leg_length() - entry_length() - return_span()) * 0.5 + entry_length()
}

pub(super) fn inlet_direction() -> Vec2 {
    legs()[0].direction
}
pub(super) fn inlet_center() -> Vec2 {
    let leg = legs()[0];
    leg.vertex - leg.direction * 0.20
}

pub(super) fn source_position(cursor: u32) -> Vec2 {
    let leg = legs()[0];
    let slot = cursor % SOURCE_SLOTS;
    let column = u16::try_from(slot % SOURCE_COLUMNS).expect("source columns fit u16");
    let row = u16::try_from(slot / SOURCE_COLUMNS).expect("source rows fit u16");
    inlet_center()
        + leg.direction * (0.15 - f32::from(row) * SOURCE_SPACING)
        + leg.normal * ((f32::from(column) - 1.5) * SOURCE_SPACING)
}

fn bypass(leg: Leg) -> CircularArc {
    let upstream = -leg.direction;
    let center_x = (BYPASS_RADIUS + BORDER_HALF_WIDTH) * BRANCH_ANGLE.cos() + BORDER_HALF_WIDTH;
    let center_y = entry_length() * BRANCH_ANGLE.cos() - BYPASS_RADIUS * BRANCH_ANGLE.sin();
    let center = leg.vertex
        + leg.direction * anchor_distance()
        + leg.outward * center_x
        + upstream * center_y;
    let radial = -leg.outward * BRANCH_ANGLE.cos() + upstream * BRANCH_ANGLE.sin();
    CircularArc {
        center,
        radius: BYPASS_RADIUS,
        start: radial.y.atan2(radial.x),
        sweep: -leg.side * TAU * 0.5,
        samples: OUTER_SAMPLES,
    }
}

fn trunk_corners(legs: &[Leg; 4]) -> [CircularArc; 3] {
    std::array::from_fn(|index| {
        let vertex = legs[index + 1].vertex;
        let sign = vertex.x.signum();
        let center = vertex - Vec2::new(sign * TRUNK_RADIUS / LEG_ANGLE.cos(), 0.0);
        let tangent = vertex - legs[index].direction * trim();
        let radial = tangent - center;
        CircularArc {
            center,
            radius: TRUNK_RADIUS,
            start: radial.y.atan2(radial.x),
            sweep: -sign * 2.0 * LEG_ANGLE,
            samples: CORNER_SAMPLES,
        }
    })
}

fn append(path: &mut Vec<Vec2>, point: Vec2) {
    if path
        .last()
        .is_none_or(|previous| (point - *previous).length_squared() > 0.000_000_000_1)
    {
        path.push(point);
    }
}

pub(super) fn valve_geometry(forward: bool) -> ValveGeometry {
    let legs = legs();
    let bypass_arcs = legs.map(bypass);
    let trunk_arcs = trunk_corners(&legs);
    let mut splitters = std::array::from_fn(|index| splitter(legs[index], bypass_arcs[index]));
    let mut outer_paths = std::array::from_fn(|path_index| {
        let side = if path_index == 0 { 1.0 } else { -1.0 };
        let mut path = Vec::new();
        for (index, leg) in legs.iter().copied().enumerate() {
            let start = if index == 0 { -0.20 } else { trim() };
            let end = if index == 3 {
                leg_length() + 0.20
            } else {
                leg_length() - trim()
            };
            append(
                &mut path,
                leg.vertex + leg.direction * start + leg.normal * (side * BORDER_HALF_WIDTH),
            );
            if (side - leg.side).abs() < 0.1 {
                for point in bypass_arcs[index]
                    .border(BYPASS_RADIUS + BORDER_HALF_WIDTH, OUTER_SAMPLES)
                    .points()
                {
                    append(&mut path, point);
                }
                append(
                    &mut path,
                    leg.vertex
                        + leg.direction * (anchor_distance() + return_span())
                        + leg.normal * (side * BORDER_HALF_WIDTH),
                );
            }
            append(
                &mut path,
                leg.vertex + leg.direction * end + leg.normal * (side * BORDER_HALF_WIDTH),
            );
            if index < 3 {
                let arc = trunk_arcs[index];
                let radius = TRUNK_RADIUS - side * arc.sweep.signum() * BORDER_HALF_WIDTH;
                for point in arc.border(radius, CORNER_SAMPLES).points() {
                    append(&mut path, point);
                }
            }
        }
        path
    });
    if !forward {
        for path in &mut outer_paths {
            for point in path.iter_mut() {
                *point = mirror_y(*point);
            }
            path.reverse();
        }
        for island in &mut splitters {
            island.center = mirror_y(island.center);
            island.stem = island.stem.map(mirror_y);
            for point in &mut island.outline {
                *point = mirror_y(*point);
            }
        }
    }
    ValveGeometry {
        outer_paths,
        splitters,
        #[cfg(test)]
        bypass_arcs: if forward {
            bypass_arcs
        } else {
            bypass_arcs.map(CircularArc::reflected)
        },
        #[cfg(test)]
        trunk_arcs: if forward {
            trunk_arcs
        } else {
            trunk_arcs.map(CircularArc::reflected)
        },
    }
}

fn splitter(leg: Leg, arc: CircularArc) -> SplitterIsland {
    let radius = BYPASS_RADIUS - BORDER_HALF_WIDTH;
    let inner = arc.border(radius, INNER_SAMPLES);
    let mut outline = inner.points();
    let first = outline[0];
    let last = outline[outline.len() - 1];
    let direction = leg.outward * BRANCH_ANGLE.sin() - leg.direction * BRANCH_ANGLE.cos();
    let center_x = (BYPASS_RADIUS + BORDER_HALF_WIDTH) * BRANCH_ANGLE.cos() + BORDER_HALF_WIDTH;
    let upper = first
        - direction
            * ((center_x - radius * BRANCH_ANGLE.cos() - BORDER_HALF_WIDTH) / BRANCH_ANGLE.sin());
    let lower = last
        - direction
            * ((center_x + radius * BRANCH_ANGLE.cos() - BORDER_HALF_WIDTH) / BRANCH_ANGLE.sin());
    outline.push(lower);
    outline.push(upper);
    SplitterIsland {
        center: arc.center,
        radius,
        stem: [upper, first, last, lower],
        outline,
    }
}

pub(super) struct WallBox {
    pub(super) center: Vec2,
    pub(super) half_length: f32,
    pub(super) angle: f32,
}

pub(super) fn wall_boxes(paths: &[Vec<Vec2>]) -> Vec<WallBox> {
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

pub(super) fn island_wall_path(island: &SplitterIsland) -> Vec<Vec2> {
    let mut points = island.outline.clone();
    points.push(points[0]);
    points
}

pub(super) fn mirror_y(point: Vec2) -> Vec2 {
    Vec2::new(point.x, 2.0 * MIRROR_Y - point.y)
}

pub(super) fn outline_segments(forward: bool) -> Vec<RigidSegment> {
    let geometry = valve_geometry(forward);
    geometry
        .outer_paths
        .iter()
        .flat_map(|path| polyline(path))
        .chain(
            geometry
                .splitters
                .iter()
                .flat_map(|island| polyline(&island_wall_path(island))),
        )
        .collect()
}

fn polyline(points: &[Vec2]) -> Vec<RigidSegment> {
    points
        .windows(2)
        .map(|edge| RigidSegment {
            start: edge[0],
            end: edge[1],
        })
        .collect()
}
