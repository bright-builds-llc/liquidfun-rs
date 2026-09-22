//! Shared Soup basin, water group, floating solids, and native carve.
//!
//! Geometry matches pinned `testSoup.js` / `Soup.h`. Soup Stirrer composes this
//! builder and keeps `ground` for the prismatic paddle rail.

use liquidfun::collision::{CircleShape, EdgeShape, FilterData, PolygonShape, Shape};
use liquidfun::math::Vec2;
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe,
    ParticleGroupSource,
};
use liquidfun::{
    BodyDef, BodyId, BodyMassData, BodyType, FixtureDef, ParticleSystemDef, ParticleSystemId,
    World,
};

use super::{RigidSegment, SceneError, attach_basin_fixture};

pub(crate) const PARTICLE_RADIUS: f32 = 0.035;
const MAXIMUM_PARTICLE_COUNT: usize = 10240;
const WATER_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const WATER_HALF_WIDTH: f32 = 2.0;
const WATER_HALF_HEIGHT: f32 = 1.0;
const WATER_CENTER: Vec2 = Vec2::new(0.0, 1.0);
const SOLID_DENSITY: f32 = 0.1;
const EDGE_DENSITY: f32 = 1.0;
const EDGE_MASS: f32 = 0.1;
const CIRCLE_LOCAL_CENTER: Vec2 = Vec2::new(0.0, 0.5);
const CIRCLE_RADIUS: f32 = 0.1;
const LEFT_BOX_CENTER: Vec2 = Vec2::new(-1.0, 0.5);
const RIGHT_BOX_CENTER: Vec2 = Vec2::new(1.0, 0.5);
const BOX_HALF: f32 = 0.1;
const RIGHT_BOX_ANGLE: f32 = 0.5;
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);

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
    Vec2::new(-4.0, 3.0),
];
const RIGHT_WALL_VERTICES: [Vec2; 4] = [
    Vec2::new(2.0, -0.1),
    Vec2::new(4.0, -0.1),
    Vec2::new(4.0, 3.0),
    Vec2::new(2.0, 2.0),
];

const EDGE_ENDPOINTS: [(Vec2, Vec2); 3] = [
    (Vec2::new(0.0, 2.0), Vec2::new(0.1, 2.1)),
    (Vec2::new(0.3, 2.0), Vec2::new(0.4, 2.1)),
    (Vec2::new(-0.3, 2.1), Vec2::new(-0.2, 2.0)),
];

/// Shared Soup world pieces returned for scene hooks and Soup Stirrer composition.
pub(crate) struct SoupFamilyBuilt {
    pub(crate) world: World,
    pub(crate) ground: BodyId,
    pub(crate) particle_system: ParticleSystemId,
    pub(crate) particle_radius: f32,
    pub(crate) basin_segments: [RigidSegment; 3],
    pub(crate) circle_body: BodyId,
    pub(crate) circle_radius: f32,
    pub(crate) box_bodies: [BodyId; 2],
    pub(crate) box_local_corners: [[Vec2; 4]; 2],
    pub(crate) edge_bodies: [BodyId; 3],
    pub(crate) edge_locals: [(Vec2, Vec2); 3],
}

pub(crate) fn build_soup_family() -> Result<SoupFamilyBuilt, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    attach_basin_fixture(&mut world, ground, &FLOOR_VERTICES)?;
    attach_basin_fixture(&mut world, ground, &LEFT_WALL_VERTICES)?;
    attach_basin_fixture(&mut world, ground, &RIGHT_WALL_VERTICES)?;

    let basin_segments = [
        RigidSegment {
            start: Vec2::new(-4.0, 0.0),
            end: Vec2::new(4.0, 0.0),
        },
        RigidSegment {
            start: Vec2::new(-2.0, 2.0),
            end: Vec2::new(-4.0, 3.0),
        },
        RigidSegment {
            start: Vec2::new(2.0, 2.0),
            end: Vec2::new(4.0, 3.0),
        },
    ];

    let particle_system = create_water_group(&mut world)?;
    let (circle_body, circle_shape) = create_floating_circle(&mut world)?;
    let left_box = create_floating_box(&mut world, LEFT_BOX_CENTER, 0.0)?;
    let right_box = create_floating_box(&mut world, RIGHT_BOX_CENTER, RIGHT_BOX_ANGLE)?;
    let (edge_bodies, edge_locals) = create_edge_noodles(&mut world)?;

    carve_under_shape(
        &mut world,
        particle_system,
        &circle_shape,
        circle_body,
    )?;
    carve_under_shape(&mut world, particle_system, &left_box.1, left_box.0)?;
    carve_under_shape(&mut world, particle_system, &right_box.1, right_box.0)?;

    Ok(SoupFamilyBuilt {
        world,
        ground,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        basin_segments,
        circle_body,
        circle_radius: CIRCLE_RADIUS,
        box_bodies: [left_box.0, right_box.0],
        box_local_corners: [left_box.2, right_box.2],
        edge_bodies,
        edge_locals,
    })
}

fn create_water_group(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    let filled = Shape::from(
        PolygonShape::oriented_box(WATER_HALF_WIDTH, WATER_HALF_HEIGHT, WATER_CENTER, 0.0)
            .map_err(|_error| SceneError::Geometry)?,
    );
    let source =
        ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(ParticleFlags::WATER)
        .with_color(WATER_COLOR);
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(system)
}

fn create_floating_circle(world: &mut World) -> Result<(BodyId, Shape), SceneError> {
    let body_definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
        .map_err(|_error| SceneError::Body)?;
    let body = world
        .create_body(&body_definition)
        .map_err(|_error| SceneError::Body)?;
    let circle =
        CircleShape::new(CIRCLE_LOCAL_CENTER, CIRCLE_RADIUS).map_err(|_error| SceneError::Geometry)?;
    let shape = Shape::from(circle);
    let fixture_definition = FixtureDef::new(
        shape.clone(),
        SOLID_DENSITY,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(body, &fixture_definition)
        .map_err(|_error| SceneError::Fixture)?;
    Ok((body, shape))
}

fn create_floating_box(
    world: &mut World,
    center: Vec2,
    angle: f32,
) -> Result<(BodyId, Shape, [Vec2; 4]), SceneError> {
    let body_definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
        .map_err(|_error| SceneError::Body)?;
    let body = world
        .create_body(&body_definition)
        .map_err(|_error| SceneError::Body)?;
    let polygon = PolygonShape::oriented_box(BOX_HALF, BOX_HALF, center, angle)
        .map_err(|_error| SceneError::Geometry)?;
    let corners = [
        polygon.vertices()[0],
        polygon.vertices()[1],
        polygon.vertices()[2],
        polygon.vertices()[3],
    ];
    let shape = Shape::from(polygon);
    let fixture_definition = FixtureDef::new(
        shape.clone(),
        SOLID_DENSITY,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(body, &fixture_definition)
        .map_err(|_error| SceneError::Fixture)?;
    Ok((body, shape, corners))
}

fn create_edge_noodles(
    world: &mut World,
) -> Result<([BodyId; 3], [(Vec2, Vec2); 3]), SceneError> {
    let mut built_bodies = Vec::with_capacity(3);
    let mut locals = [(Vec2::ZERO, Vec2::ZERO); 3];
    for (index, (start, end)) in EDGE_ENDPOINTS.iter().copied().enumerate() {
        let body_definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
            .map_err(|_error| SceneError::Body)?;
        let body = world
            .create_body(&body_definition)
            .map_err(|_error| SceneError::Body)?;
        let edge = EdgeShape::new(start, end).map_err(|_error| SceneError::Geometry)?;
        let fixture_definition = FixtureDef::new(
            Shape::from(edge),
            EDGE_DENSITY,
            0.2,
            0.0,
            false,
            FilterData::default(),
        )
        .map_err(|_error| SceneError::Fixture)?;
        world
            .create_fixture(body, &fixture_definition)
            .map_err(|_error| SceneError::Fixture)?;
        let midpoint = 0.5 * (start + end);
        let mass = BodyMassData::new(EDGE_MASS, midpoint, 0.0).map_err(|_error| SceneError::Body)?;
        world
            .set_body_mass_data(body, mass)
            .map_err(|_error| SceneError::Body)?;
        built_bodies.push(body);
        locals[index] = (start, end);
    }
    let bodies = [
        built_bodies[0],
        built_bodies[1],
        built_bodies[2],
    ];
    Ok((bodies, locals))
}

fn carve_under_shape(
    world: &mut World,
    system: ParticleSystemId,
    shape: &Shape,
    body: BodyId,
) -> Result<(), SceneError> {
    let transform = world
        .body_snapshot(body)
        .map_err(|_error| SceneError::Body)?
        .transform();
    world
        .destroy_particles_in_shape(system, shape, transform)
        .map_err(|_error| SceneError::Particle)?;
    Ok(())
}
