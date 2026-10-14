//! Fixed siphon spike geometry (Phase 45 D-19): walls, interior regions and
//! the primed fill, recorded in `45-SIPHON-SPIKE.md` before any measurement.

use liquidfun::math::Vec2;

/// Particle radius (D-15).
pub(super) const RADIUS: f32 = 0.05;
/// Fill spacing, `0.75 × diameter`.
pub(super) const SPACING: f32 = 0.075;
/// Inner lower wall height at the tube apex: the highest point water clears.
pub(super) const CREST: f32 = 2.2;
/// Top of the inlet mouth gap below the inlet divider.
pub(super) const INLET_MOUTH_TOP: f32 = 0.5;
/// Leak margin around the union of interior regions.
const LEAK_MARGIN: f32 = 0.1;
/// Primed source fill level.
const SOURCE_FILL_TOP: f32 = 2.0;
/// Primed destination fill level (outlet mouth top is -0.5, so it starts
/// submerged).
const DEST_FILL_TOP: f32 = -0.4;

/// Closed axis-aligned rectangle `min..=max`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Rect {
    pub(super) min: Vec2,
    pub(super) max: Vec2,
}

impl Rect {
    const fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
        }
    }

    pub(super) fn contains(self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    const fn expanded(self, margin: f32) -> Self {
        Self::new(
            self.min.x - margin,
            self.min.y - margin,
            self.max.x + margin,
            self.max.y + margin,
        )
    }

    const fn capped(self, top: f32) -> Self {
        Self::new(self.min.x, self.min.y, self.max.x, top)
    }

    pub(super) fn center_x(self) -> f32 {
        0.5 * (self.min.x + self.max.x)
    }

    pub(super) fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    /// Counter-clockwise corners.
    pub(super) const fn quad(self) -> [Vec2; 4] {
        [
            Vec2::new(self.min.x, self.min.y),
            Vec2::new(self.max.x, self.min.y),
            Vec2::new(self.max.x, self.max.y),
            Vec2::new(self.min.x, self.max.y),
        ]
    }
}

/// Thick overlapping wall quads on one static body (all 0.3 m = 3 diameters).
pub(super) const WALLS: [Rect; 10] = [
    // Source floor
    Rect::new(-4.6, -0.3, -0.7, 0.0),
    // Source left wall
    Rect::new(-4.6, -0.3, -4.3, 3.3),
    // Inlet divider (tube inner wall; mouth gap below it)
    Rect::new(-2.1, 0.5, -1.8, 3.0),
    // Source right wall = inlet leg outer wall
    Rect::new(-1.3, -0.3, -1.0, 2.2),
    // Crest lower wall
    Rect::new(-1.3, 1.9, 1.3, 2.2),
    // Crest upper wall
    Rect::new(-2.1, 2.7, 2.1, 3.0),
    // Dest left wall = outlet leg outer wall
    Rect::new(1.0, -1.3, 1.3, 2.2),
    // Outlet divider (mouth gap below it)
    Rect::new(1.8, -0.5, 2.1, 3.0),
    // Dest right wall
    Rect::new(4.3, -1.3, 4.6, 3.0),
    // Dest floor
    Rect::new(0.7, -1.3, 4.6, -1.0),
];

/// Source open tank `S`.
pub(super) const SOURCE: Rect = Rect::new(-4.3, 0.0, -2.1, 3.3);
/// Gap below the inlet divider joining `S` to the inlet leg.
const INLET_MOUTH: Rect = Rect::new(-2.1, 0.0, -1.8, INLET_MOUTH_TOP);
/// Inlet leg of the inverted U.
pub(super) const INLET_LEG: Rect = Rect::new(-1.8, 0.0, -1.3, CREST);
/// Crest channel across the top of the U.
pub(super) const CREST_CHANNEL: Rect = Rect::new(-1.8, CREST, 1.8, 2.7);
/// Outlet leg of the inverted U.
pub(super) const OUTLET_LEG: Rect = Rect::new(1.3, -1.0, 1.8, CREST);
/// Gap below the outlet divider joining the outlet leg to `D`.
const OUTLET_MOUTH: Rect = Rect::new(1.8, -1.0, 2.1, -0.5);
/// Destination open tank `D`.
pub(super) const DEST: Rect = Rect::new(2.1, -1.0, 4.3, 3.0);

/// Every interior region; leaks are particles outside all of them, expanded
/// by `LEAK_MARGIN`.
const INTERIOR: [Rect; 7] = [
    SOURCE,
    INLET_MOUTH,
    INLET_LEG,
    CREST_CHANNEL,
    OUTLET_LEG,
    OUTLET_MOUTH,
    DEST,
];

/// Regions filled at step 0: tanks to their levels, the whole tube full.
const FILL: [Rect; 7] = [
    SOURCE.capped(SOURCE_FILL_TOP),
    INLET_MOUTH,
    INLET_LEG,
    CREST_CHANNEL,
    OUTLET_LEG,
    OUTLET_MOUTH,
    DEST.capped(DEST_FILL_TOP),
];

/// Grid origin; every fill point is `origin + (i, j) × SPACING`.
const GRID_ORIGIN: Vec2 = Vec2::new(-4.25, -0.95);
const GRID_COLUMNS: u32 = 114;
const GRID_ROWS: u32 = 50;

/// Primed fill points on the shared 0.075 m grid, inset by `RADIUS` from
/// every wall.
pub(super) fn primed_fill() -> Vec<Vec2> {
    let mut points = Vec::new();
    for row in 0..GRID_ROWS {
        for column in 0..GRID_COLUMNS {
            let point = Vec2::new(
                GRID_ORIGIN.x + column as f32 * SPACING,
                GRID_ORIGIN.y + row as f32 * SPACING,
            );
            let filled = FILL.iter().any(|region| region.contains(point));
            let clear = WALLS
                .iter()
                .all(|wall| !wall.expanded(RADIUS).contains(point));
            if filled && clear {
                points.push(point);
            }
        }
    }
    points
}

/// Whether `point` is outside every interior region expanded by the leak
/// margin.
pub(super) fn is_leak(point: Vec2) -> bool {
    !INTERIOR
        .iter()
        .any(|region| region.expanded(LEAK_MARGIN).contains(point))
}
