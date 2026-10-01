//! Side lips on the elevator deck. They sit inboard of the shaft faces so the
//! spillway can still reach the deck, and they rise high enough that liquid
//! riding the slant cannot slide off the side seams.

use liquidfun::collision::{FilterData, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::{BodyId, FixtureDef, World};

use super::{
    PLATE_DENSITY, PLATE_HALF_HEIGHT, PLATE_HALF_WIDTH, PLATE_SLANT, RigidSegment, SceneError,
    WALL_FRICTION,
};

const CHEEK_HALF_HEIGHT: f32 = 0.05;
const CHEEK_CENTER_Y: f32 = 0.05;
const LEFT_CHEEK_HALF_WIDTH: f32 = 0.020;
const RIGHT_CHEEK_HALF_WIDTH: f32 = 0.020;
/// Local centers. The left lip leaves a gutter at the spillway; the right lip
/// stops just short of the shaft wall.
const LEFT_CHEEK_CENTER_X: f32 = -0.162;
const RIGHT_CHEEK_CENTER_X: f32 = 0.176;

pub(super) fn attach_plate_fixtures(world: &mut World, plate: BodyId) -> Result<(), SceneError> {
    attach_oriented_box(
        world,
        plate,
        PLATE_HALF_WIDTH,
        PLATE_HALF_HEIGHT,
        Vec2::ZERO,
        PLATE_SLANT,
    )?;
    attach_oriented_box(
        world,
        plate,
        LEFT_CHEEK_HALF_WIDTH,
        CHEEK_HALF_HEIGHT,
        Vec2::new(LEFT_CHEEK_CENTER_X, CHEEK_CENTER_Y),
        0.0,
    )?;
    attach_oriented_box(
        world,
        plate,
        RIGHT_CHEEK_HALF_WIDTH,
        CHEEK_HALF_HEIGHT,
        Vec2::new(RIGHT_CHEEK_CENTER_X, CHEEK_CENTER_Y),
        0.0,
    )?;
    Ok(())
}

pub(super) fn push_plate_segments(segments: &mut Vec<RigidSegment>, transform: Transform) {
    push_loop(segments, transform, &super::plate_local_corners());
    // Two edges per lip. The frame budget is 64 segments, and the shelves already use most of it.
    push_lip(
        segments,
        transform,
        LEFT_CHEEK_CENTER_X,
        LEFT_CHEEK_HALF_WIDTH,
        true,
    );
    push_lip(
        segments,
        transform,
        RIGHT_CHEEK_CENTER_X,
        RIGHT_CHEEK_HALF_WIDTH,
        false,
    );
}

/// World-space corners of the left lip, then the right lip, at a vertical offset.
#[cfg(test)]
pub(super) fn cheek_world_corners(offset_y: f32) -> [[Vec2; 4]; 2] {
    let origin_y = super::PLATE_CENTER.y + offset_y;
    [
        axis_box(
            super::PLATE_CENTER.x + LEFT_CHEEK_CENTER_X,
            origin_y + CHEEK_CENTER_Y,
            LEFT_CHEEK_HALF_WIDTH,
            CHEEK_HALF_HEIGHT,
        ),
        axis_box(
            super::PLATE_CENTER.x + RIGHT_CHEEK_CENTER_X,
            origin_y + CHEEK_CENTER_Y,
            RIGHT_CHEEK_HALF_WIDTH,
            CHEEK_HALF_HEIGHT,
        ),
    ]
}

#[cfg(test)]
fn axis_box(center_x: f32, center_y: f32, half_width: f32, half_height: f32) -> [Vec2; 4] {
    [
        Vec2::new(center_x - half_width, center_y - half_height),
        Vec2::new(center_x + half_width, center_y - half_height),
        Vec2::new(center_x + half_width, center_y + half_height),
        Vec2::new(center_x - half_width, center_y + half_height),
    ]
}

fn push_lip(
    segments: &mut Vec<RigidSegment>,
    transform: Transform,
    center_x: f32,
    half_width: f32,
    inner_is_positive_x: bool,
) {
    let inner_x = if inner_is_positive_x {
        center_x + half_width
    } else {
        center_x - half_width
    };
    let top = CHEEK_CENTER_Y + CHEEK_HALF_HEIGHT;
    let bottom = CHEEK_CENTER_Y - CHEEK_HALF_HEIGHT;
    let outer_x = if inner_is_positive_x {
        center_x - half_width
    } else {
        center_x + half_width
    };
    push_edge(
        segments,
        transform,
        Vec2::new(inner_x, bottom),
        Vec2::new(inner_x, top),
    );
    push_edge(
        segments,
        transform,
        Vec2::new(outer_x, top),
        Vec2::new(inner_x, top),
    );
}

fn push_edge(segments: &mut Vec<RigidSegment>, transform: Transform, start: Vec2, end: Vec2) {
    segments.push(RigidSegment {
        start: transform.apply(start),
        end: transform.apply(end),
    });
}

fn push_loop(segments: &mut Vec<RigidSegment>, transform: Transform, corners: &[Vec2; 4]) {
    for index in 0..corners.len() {
        let start = transform.apply(corners[index]);
        let end = transform.apply(corners[(index + 1) % corners.len()]);
        segments.push(RigidSegment { start, end });
    }
}

fn attach_oriented_box(
    world: &mut World,
    body: BodyId,
    half_width: f32,
    half_height: f32,
    center: Vec2,
    angle: f32,
) -> Result<(), SceneError> {
    let polygon = PolygonShape::oriented_box(half_width, half_height, center, angle)
        .map_err(|_error| SceneError::Geometry)?;
    let definition = FixtureDef::new(
        Shape::from(polygon),
        PLATE_DENSITY,
        WALL_FRICTION,
        0.0,
        false,
        FilterData::default(),
    )
    .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(body, &definition)
        .map_err(|_error| SceneError::Fixture)?;
    Ok(())
}
