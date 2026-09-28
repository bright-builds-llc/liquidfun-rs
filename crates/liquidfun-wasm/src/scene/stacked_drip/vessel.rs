//! Static walls and tray boxes for Stacked Drip.
//!
//! The shaft is the right-hand chamber. Each tray deck extends to local
//! negative x so a positive pour angle drops that lip away from the shaft.

use liquidfun::math::Vec2;

use super::{COUNTERWEIGHT_DENSITY, DECK_DENSITY, DIVIDER_CENTER_X, WALL_HALF};

#[derive(Clone, Copy)]
pub(super) struct BoxSpec {
    pub(super) half_width: f32,
    pub(super) half_height: f32,
    pub(super) center: Vec2,
}

#[derive(Clone, Copy)]
pub(super) struct LocalBox {
    pub(super) half_width: f32,
    pub(super) half_height: f32,
    pub(super) center: Vec2,
    pub(super) density: f32,
}

pub(super) fn wall_boxes() -> [BoxSpec; 4] {
    [
        BoxSpec {
            half_width: WALL_HALF,
            half_height: 1.05,
            center: Vec2::new(-0.74, 1.05),
        },
        BoxSpec {
            half_width: 1.19,
            half_height: WALL_HALF,
            center: Vec2::new(0.41, -0.04),
        },
        BoxSpec {
            half_width: WALL_HALF,
            half_height: 1.05,
            center: Vec2::new(1.56, 1.05),
        },
        BoxSpec {
            half_width: WALL_HALF,
            half_height: 1.05,
            center: Vec2::new(DIVIDER_CENTER_X, 1.05),
        },
    ]
}

pub(super) fn tray_fixtures() -> [LocalBox; 2] {
    [
        LocalBox {
            half_width: 0.24,
            half_height: 0.01,
            center: Vec2::new(-0.16, 0.0),
            density: DECK_DENSITY,
        },
        LocalBox {
            half_width: 0.06,
            half_height: 0.04,
            center: Vec2::new(0.16, 0.0),
            density: COUNTERWEIGHT_DENSITY,
        },
    ]
}

pub(super) fn local_corners(fixture: LocalBox) -> [Vec2; 4] {
    let left = fixture.center.x - fixture.half_width;
    let right = fixture.center.x + fixture.half_width;
    let bottom = fixture.center.y - fixture.half_height;
    let top = fixture.center.y + fixture.half_height;
    [
        Vec2::new(left, bottom),
        Vec2::new(right, bottom),
        Vec2::new(right, top),
        Vec2::new(left, top),
    ]
}
