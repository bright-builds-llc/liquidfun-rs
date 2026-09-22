//! Shared vertical-wall basin and falling-ball helpers for material flag-group scenes.
//!
//! Geometry matches pinned Surface Tension / Elastic / Rigid Particles JS tests:
//! floor plus side walls ending at `y=2` (not Particles' slanted `y=3` tops).

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{BodyDef, BodyId, BodyType, FixtureDef, World};

use super::{RigidSegment, SceneError, attach_basin_fixture};

const FLOOR_VERTICES: [Vec2; 4] = [
    Vec2::new(-4.0, -1.0),
    Vec2::new(4.0, -1.0),
    Vec2::new(4.0, 0.0),
    Vec2::new(-4.0, 0.0),
];

const LEFT_WALL_VERTICES: [Vec2; 4] = [
    Vec2::new(-4.0, -0.1),
    Vec2::new(-2.0, -0.1),
    Vec2::new(-2.0, 2.0),
    Vec2::new(-4.0, 2.0),
];

const RIGHT_WALL_VERTICES: [Vec2; 4] = [
    Vec2::new(2.0, -0.1),
    Vec2::new(4.0, -0.1),
    Vec2::new(4.0, 2.0),
    Vec2::new(2.0, 2.0),
];

const DYNAMIC_BALL_RADIUS: f32 = 0.5;
const DYNAMIC_BALL_POSITION: Vec2 = Vec2::new(0.0, 8.0);
const DYNAMIC_BALL_DENSITY: f32 = 0.5;

/// Attach the three vertical-wall basin polygons and return drawable outline segments.
pub(crate) fn attach_vertical_wall_basin(
    world: &mut World,
    body: BodyId,
) -> Result<[RigidSegment; 3], SceneError> {
    attach_basin_fixture(world, body, &FLOOR_VERTICES)?;
    attach_basin_fixture(world, body, &LEFT_WALL_VERTICES)?;
    attach_basin_fixture(world, body, &RIGHT_WALL_VERTICES)?;
    Ok([
        RigidSegment {
            start: Vec2::new(-4.0, 0.0),
            end: Vec2::new(4.0, 0.0),
        },
        RigidSegment {
            start: Vec2::new(-2.0, 0.0),
            end: Vec2::new(-2.0, 2.0),
        },
        RigidSegment {
            start: Vec2::new(2.0, 0.0),
            end: Vec2::new(2.0, 2.0),
        },
    ])
}

/// Spawn the pinned dynamic circle at `(0, 8)` with radius `0.5` and density `0.5`.
pub(crate) fn create_falling_ball(world: &mut World) -> Result<(BodyId, f32), SceneError> {
    let body_definition = BodyDef::new(BodyType::Dynamic, DYNAMIC_BALL_POSITION, 0.0, true)
        .map_err(|_error| SceneError::Body)?;
    let body = world
        .create_body(&body_definition)
        .map_err(|_error| SceneError::Body)?;
    let circle =
        CircleShape::new(Vec2::ZERO, DYNAMIC_BALL_RADIUS).map_err(|_error| SceneError::Geometry)?;
    let fixture_definition = FixtureDef::new(
        Shape::from(circle),
        DYNAMIC_BALL_DENSITY,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(body, &fixture_definition)
        .map_err(|_error| SceneError::Fixture)?;
    Ok((body, DYNAMIC_BALL_RADIUS))
}
