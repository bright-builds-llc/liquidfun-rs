//! Static walls and tray boxes for Stacked Drip.
//!
//! The shaft is the right-hand chamber. Each tray deck extends to local
//! negative x so a positive pour angle drops that lip away from the shaft.

use liquidfun::math::Vec2;

use super::{
    COUNTERWEIGHT_DENSITY, DECK_DENSITY, DIVIDER_CENTER_X, DIVIDER_CENTER_Y, DIVIDER_HALF_HEIGHT,
    DIVIDER_INNER_X, FLOOR_BOTTOM_Y, FLOOR_OUTER_LEFT_X, FLOOR_OUTER_RIGHT_X, RIGHT_WALL_CENTER_X,
    SLOPE_HIGH_X, SLOPE_HIGH_Y, SLOPE_LOW_Y, WALL_CENTER_Y, WALL_HALF, WALL_HALF_HEIGHT,
};

#[derive(Clone, Copy)]
pub(super) struct BoxSpec {
    pub(super) half_width: f32,
    pub(super) half_height: f32,
    pub(super) center: Vec2,
    /// Shares the plate's negative collision group so the plate can overlap it.
    pub(super) shares_plate_group: bool,
}

#[derive(Clone, Copy)]
pub(super) struct LocalBox {
    pub(super) half_width: f32,
    pub(super) half_height: f32,
    pub(super) center: Vec2,
    pub(super) density: f32,
}

pub(super) fn wall_boxes() -> [BoxSpec; 5] {
    [
        BoxSpec {
            half_width: WALL_HALF,
            half_height: WALL_HALF_HEIGHT,
            center: Vec2::new(-0.74, WALL_CENTER_Y),
            shares_plate_group: false,
        },
        BoxSpec {
            half_width: (FLOOR_OUTER_RIGHT_X - DIVIDER_INNER_X) * 0.5,
            half_height: WALL_HALF,
            center: Vec2::new(
                (DIVIDER_INNER_X + FLOOR_OUTER_RIGHT_X) * 0.5,
                FLOOR_BOTTOM_Y + WALL_HALF,
            ),
            shares_plate_group: false,
        },
        BoxSpec {
            half_width: WALL_HALF,
            half_height: WALL_HALF_HEIGHT,
            center: Vec2::new(RIGHT_WALL_CENTER_X, WALL_CENTER_Y),
            shares_plate_group: true,
        },
        BoxSpec {
            half_width: WALL_HALF,
            half_height: DIVIDER_HALF_HEIGHT,
            center: Vec2::new(DIVIDER_CENTER_X, DIVIDER_CENTER_Y),
            shares_plate_group: true,
        },
        BoxSpec {
            half_width: (FLOOR_OUTER_RIGHT_X - FLOOR_OUTER_LEFT_X) * 0.5,
            half_height: WALL_HALF,
            center: Vec2::new(
                (FLOOR_OUTER_LEFT_X + FLOOR_OUTER_RIGHT_X) * 0.5,
                super::CEILING_CENTER_Y,
            ),
            shares_plate_group: false,
        },
    ]
}

pub(super) fn floor_wedge() -> [Vec2; 4] {
    [
        Vec2::new(FLOOR_OUTER_LEFT_X, FLOOR_BOTTOM_Y),
        Vec2::new(DIVIDER_INNER_X, FLOOR_BOTTOM_Y),
        Vec2::new(DIVIDER_INNER_X, SLOPE_LOW_Y),
        Vec2::new(SLOPE_HIGH_X, SLOPE_HIGH_Y),
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
