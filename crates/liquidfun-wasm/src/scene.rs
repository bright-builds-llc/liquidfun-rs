//! Construction of the named Dam Break scene (Phase 16 basin proof).
//!
//! Documented reset constants — do not retune without updating tests and copy:
//! gravity `(0, -10)`, 16×12 = 192 particles, radius `0.2`, spacing `0.32`,
//! origin `(-4.7, 0.4)`, color `(57, 211, 199, 255)`, basin floor `y=0` from
//! `x=-5.5..5.5` with walls to `y=8`, dynamic circle `(2.5, 5.5)` radius
//! `0.75`, particle cap 512, timestep `1/60`.

use liquidfun::collision::{CircleShape, FilterData, PolygonShape, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, ParticleColor, ParticleDef, ParticleFlags,
    ParticleSystemDef, ParticleSystemId, World,
};

pub(crate) const PARTICLE_RADIUS: f32 = 0.2;
pub(crate) const PARTICLE_COUNT: usize = 16 * 12;

const PARTICLE_COLUMNS: u8 = 16;
const PARTICLE_ROWS: u8 = 12;
const PARTICLE_SPACING: f32 = 0.32;
const PARTICLE_ORIGIN: Vec2 = Vec2::new(-4.7, 0.4);
const PARTICLE_COLOR: ParticleColor = ParticleColor::new(57, 211, 199, 255);
const DYNAMIC_CIRCLE_RADIUS: f32 = 0.75;
const DYNAMIC_CIRCLE_POSITION: Vec2 = Vec2::new(2.5, 5.5);
const MAXIMUM_PARTICLE_COUNT: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SceneError {
    World,
    Gravity,
    Geometry,
    Body,
    Fixture,
    ParticleSystem,
    Particle,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RigidSegment {
    pub(crate) start: Vec2,
    pub(crate) end: Vec2,
}

pub(crate) struct ProofScene {
    pub(crate) world: World,
    pub(crate) particle_system: ParticleSystemId,
    pub(crate) dynamic_circle: BodyId,
    pub(crate) basin_segments: [RigidSegment; 3],
    pub(crate) circle_radius: f32,
}

pub(crate) fn build_proof_scene() -> Result<ProofScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(Vec2::new(0.0, -10.0))
        .map_err(|_error| SceneError::Gravity)?;

    let basin_body = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    attach_basin_fixture(
        &mut world,
        basin_body,
        &[
            Vec2::new(-6.0, -1.0),
            Vec2::new(6.0, -1.0),
            Vec2::new(6.0, 0.0),
            Vec2::new(-6.0, 0.0),
        ],
    )?;
    attach_basin_fixture(
        &mut world,
        basin_body,
        &[
            Vec2::new(-6.0, 0.0),
            Vec2::new(-5.5, 0.0),
            Vec2::new(-5.5, 8.0),
            Vec2::new(-6.0, 8.0),
        ],
    )?;
    attach_basin_fixture(
        &mut world,
        basin_body,
        &[
            Vec2::new(5.5, 0.0),
            Vec2::new(6.0, 0.0),
            Vec2::new(6.0, 8.0),
            Vec2::new(5.5, 8.0),
        ],
    )?;

    let dynamic_circle = create_dynamic_circle(&mut world)?;
    let particle_system = create_particles(&mut world)?;

    Ok(ProofScene {
        world,
        particle_system,
        dynamic_circle,
        basin_segments: [
            RigidSegment {
                start: Vec2::new(-5.5, 0.0),
                end: Vec2::new(5.5, 0.0),
            },
            RigidSegment {
                start: Vec2::new(-5.5, 0.0),
                end: Vec2::new(-5.5, 8.0),
            },
            RigidSegment {
                start: Vec2::new(5.5, 0.0),
                end: Vec2::new(5.5, 8.0),
            },
        ],
        circle_radius: DYNAMIC_CIRCLE_RADIUS,
    })
}

fn attach_basin_fixture(
    world: &mut World,
    body: BodyId,
    vertices: &[Vec2; 4],
) -> Result<(), SceneError> {
    let polygon = PolygonShape::new(vertices).map_err(|_error| SceneError::Geometry)?;
    let definition = FixtureDef::new(
        Shape::from(polygon),
        0.0,
        0.2,
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

fn create_dynamic_circle(world: &mut World) -> Result<BodyId, SceneError> {
    let body_definition = BodyDef::new(BodyType::Dynamic, DYNAMIC_CIRCLE_POSITION, 0.0, true)
        .map_err(|_error| SceneError::Body)?;
    let body = world
        .create_body(&body_definition)
        .map_err(|_error| SceneError::Body)?;
    let circle = CircleShape::new(Vec2::ZERO, DYNAMIC_CIRCLE_RADIUS)
        .map_err(|_error| SceneError::Geometry)?;
    let fixture_definition = FixtureDef::new(
        Shape::from(circle),
        1.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(body, &fixture_definition)
        .map_err(|_error| SceneError::Fixture)?;
    Ok(body)
}

fn create_particles(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    for row in 0..PARTICLE_ROWS {
        for column in 0..PARTICLE_COLUMNS {
            let position = Vec2::new(
                PARTICLE_ORIGIN.x + f32::from(column) * PARTICLE_SPACING,
                PARTICLE_ORIGIN.y + f32::from(row) * PARTICLE_SPACING,
            );
            let definition = ParticleDef::default()
                .with_flags(ParticleFlags::WATER)
                .with_color(PARTICLE_COLOR)
                .with_position(position)
                .map_err(|_error| SceneError::Particle)?;
            let receipt = world
                .create_particle_with_def(system, None, &definition)
                .map_err(|_error| SceneError::Particle)?;
            if !receipt.destruction_occurrences().is_empty() {
                return Err(SceneError::Particle);
            }
        }
    }

    Ok(system)
}
