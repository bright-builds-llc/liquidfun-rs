//! Sparky is an experimental native Rust port of the pinned `LiquidFun` tests.
//! The playground shows recognizable behavior. Catalog previews are static illustrations.

#[cfg(test)]
mod tests;

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, ParticleSystemDef, ParticleSystemId, World,
};

use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
    attach_basin_fixture,
};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.25;
const MAXIMUM_PARTICLE_COUNT: usize = 10240;
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const CIRCLE_RADIUS: f32 = 2.0;
const CIRCLE_DENSITY: f32 = 0.5;
const CIRCLE_COUNT: usize = 6;

const FLOOR: [Vec2; 4] = [
    Vec2::new(-40.0, -10.0),
    Vec2::new(40.0, -10.0),
    Vec2::new(40.0, 0.0),
    Vec2::new(-40.0, 0.0),
];
const CEILING: [Vec2; 4] = [
    Vec2::new(-40.0, 40.0),
    Vec2::new(40.0, 40.0),
    Vec2::new(40.0, 50.0),
    Vec2::new(-40.0, 50.0),
];
const LEFT_WALL: [Vec2; 4] = [
    Vec2::new(-40.0, -1.0),
    Vec2::new(-20.0, -1.0),
    Vec2::new(-20.0, 40.0),
    Vec2::new(-40.0, 40.0),
];
const RIGHT_WALL: [Vec2; 4] = [
    Vec2::new(20.0, -1.0),
    Vec2::new(40.0, -1.0),
    Vec2::new(40.0, 40.0),
    Vec2::new(20.0, 40.0),
];
const WALLS: [[Vec2; 4]; 4] = [FLOOR, CEILING, LEFT_WALL, RIGHT_WALL];

struct SparkyHooks {
    wall_segments: [RigidSegment; 16],
    sparkable_bodies: [BodyId; CIRCLE_COUNT],
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_sparky().map_err(|_error| SessionError::SceneConstruction)
}

fn build_sparky() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;
    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    for vertices in WALLS {
        attach_basin_fixture(&mut world, ground, &vertices)?;
    }
    let mut sparkable_bodies = [ground; CIRCLE_COUNT];
    for index in 0..CIRCLE_COUNT {
        sparkable_bodies[index] = create_sparkable_circle(&mut world, index)?;
    }
    let particle_system = create_particle_system(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(SparkyHooks {
            wall_segments: wall_segments(),
            sparkable_bodies,
        }),
    })
}

fn create_particle_system(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_destruction_by_age(false);
    world
        .create_particle_system_with_def(&definition)
        .map_err(|_error| SceneError::ParticleSystem)
}

fn create_sparkable_circle(world: &mut World, index: usize) -> Result<BodyId, SceneError> {
    let center = Vec2::new(circle_x(index), circle_y(index));
    let definition =
        BodyDef::new(BodyType::Dynamic, center, 0.0, true).map_err(|_error| SceneError::Body)?;
    let body = world
        .create_body(&definition)
        .map_err(|_error| SceneError::Body)?;
    let circle =
        CircleShape::new(Vec2::ZERO, CIRCLE_RADIUS).map_err(|_error| SceneError::Geometry)?;
    let fixture = FixtureDef::new(
        Shape::from(circle),
        CIRCLE_DENSITY,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(body, &fixture)
        .map_err(|_error| SceneError::Fixture)?;
    Ok(body)
}

fn circle_x(index: usize) -> f32 {
    if index % 2 == 0 { -1.5 } else { 1.5 }
}

fn circle_y(index: usize) -> f32 {
    7.0 + 4.5 * index as f32
}

fn wall_segments() -> [RigidSegment; 16] {
    let mut segments = [RigidSegment {
        start: Vec2::ZERO,
        end: Vec2::ZERO,
    }; 16];
    let mut index = 0usize;
    for wall in WALLS {
        for edge in 0..4 {
            segments[index] = RigidSegment {
                start: wall[edge],
                end: wall[(edge + 1) % 4],
            };
            index += 1;
        }
    }
    segments
}

impl SceneHooks for SparkyHooks {
    fn on_advance(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        Ok(())
    }

    fn apply_control(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        _name: &str,
        _value: &str,
    ) -> Result<ControlEffect, SessionError> {
        Err(SessionError::UnknownControl)
    }

    fn apply_action(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        _name: &str,
    ) -> Result<(), SessionError> {
        Err(SessionError::UnknownControl)
    }

    fn apply_pointer(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        _kind: PointerKind,
        _world_x: f32,
        _world_y: f32,
    ) -> Result<(), SessionError> {
        Ok(())
    }

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        Ok(self.wall_segments.to_vec())
    }

    fn collect_circles(&self, world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        let mut circles = Vec::with_capacity(self.sparkable_bodies.len());
        for &body in &self.sparkable_bodies {
            let position = world
                .body_snapshot(body)
                .map_err(|_error| SessionError::FrameCaptureFailed)?
                .transform()
                .position();
            circles.push((position, CIRCLE_RADIUS));
        }
        Ok(circles)
    }
}
