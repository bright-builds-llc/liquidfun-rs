//! Periodic hydraulic fountain: a dynamic piston squeezes one water group through a throat.
//!
//! Motor speed is a square wave of simulation time. Limits hold the stroke at each end.
//! The fountain chamber starts empty, and the existing Fountain emitter is not reused.

use liquidfun::collision::{FilterData, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, JointDef, JointId, ParticleSystemDef, ParticleSystemId,
    PrismaticJointDef, World,
};

use super::{BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.025;
const GROUP_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const SIM_DT: f32 = 1.0 / 60.0;
const ADVANCE_SPEED: f32 = 0.6;
const STROKE: f32 = 0.35;
const PERIOD: f32 = 2.0;
const MAX_MOTOR_FORCE: f32 = 1.0e6;
const PISTON_DENSITY: f32 = 1.0;
const PISTON_CENTER_RETRACTED: Vec2 = Vec2::new(-0.90, 0.50);
const PISTON_HALF_WIDTH: f32 = 0.04;
const PISTON_HALF_HEIGHT: f32 = 0.45;
const WALL_FRICTION: f32 = 0.2;
const WALL_HALF_THICKNESS: f32 = 0.04;

const WATER_POLYGON: [Vec2; 4] = [
    Vec2::new(-0.78, 0.04),
    Vec2::new(-0.16, 0.04),
    Vec2::new(-0.16, 0.48),
    Vec2::new(-0.78, 0.48),
];

/// Floor top, side inner faces, and the divider above the floor throat.
const WALL_SEGMENTS: [RigidSegment; 4] = [
    RigidSegment {
        start: Vec2::new(-1.18, 0.0),
        end: Vec2::new(1.18, 0.0),
    },
    RigidSegment {
        start: Vec2::new(-1.14, 0.0),
        end: Vec2::new(-1.14, 1.39),
    },
    RigidSegment {
        start: Vec2::new(1.14, 0.0),
        end: Vec2::new(1.14, 1.39),
    },
    RigidSegment {
        start: Vec2::new(0.0, 0.12),
        end: Vec2::new(0.0, 1.39),
    },
];

struct HydraulicFountainHooks {
    elapsed: f32,
    joint: JointId,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_hydraulic_fountain().map_err(|_error| SessionError::SceneConstruction)
}

fn build_hydraulic_fountain() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    attach_wall_boxes(&mut world, ground)?;

    let piston = create_piston(&mut world)?;
    let joint = create_piston_joint(&mut world, ground, piston)?;
    let particle_system = create_water_group(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(HydraulicFountainHooks {
            elapsed: 0.0,
            joint,
        }),
    })
}

fn scheduled_motor_speed(elapsed: f32) -> f32 {
    let squeezing = elapsed % PERIOD < PERIOD * 0.5;
    if squeezing {
        ADVANCE_SPEED
    } else {
        -ADVANCE_SPEED
    }
}

fn attach_wall_boxes(world: &mut World, ground: BodyId) -> Result<(), SceneError> {
    let boxes = [
        (
            1.18,
            WALL_HALF_THICKNESS,
            Vec2::new(0.0, -WALL_HALF_THICKNESS),
        ),
        (WALL_HALF_THICKNESS, 0.695, Vec2::new(-1.18, 0.695)),
        (WALL_HALF_THICKNESS, 0.695, Vec2::new(1.18, 0.695)),
        (WALL_HALF_THICKNESS, 0.635, Vec2::new(0.0, 0.755)),
    ];
    for (half_width, half_height, center) in boxes {
        attach_box(world, ground, half_width, half_height, center, 0.0)?;
    }
    Ok(())
}

fn create_piston(world: &mut World) -> Result<BodyId, SceneError> {
    let definition = BodyDef::new(BodyType::Dynamic, PISTON_CENTER_RETRACTED, 0.0, true)
        .map_err(|_error| SceneError::Body)?
        .with_sleeping_allowed(false);
    let piston = world
        .create_body(&definition)
        .map_err(|_error| SceneError::Body)?;
    attach_box(
        world,
        piston,
        PISTON_HALF_WIDTH,
        PISTON_HALF_HEIGHT,
        Vec2::ZERO,
        PISTON_DENSITY,
    )?;
    Ok(piston)
}

fn create_piston_joint(
    world: &mut World,
    ground: BodyId,
    piston: BodyId,
) -> Result<JointId, SceneError> {
    let definition = PrismaticJointDef::new(ground, piston)
        .map_err(|_error| SceneError::Body)?
        .with_collide_connected(true)
        .with_frame(
            PISTON_CENTER_RETRACTED,
            Vec2::ZERO,
            Vec2::new(1.0, 0.0),
            0.0,
        )
        .map_err(|_error| SceneError::Body)?
        .with_limits(true, 0.0, STROKE)
        .map_err(|_error| SceneError::Body)?
        .with_motor(true, ADVANCE_SPEED, MAX_MOTOR_FORCE)
        .map_err(|_error| SceneError::Body)?;
    world
        .create_joint(JointDef::from(definition))
        .map_err(|_error| SceneError::Body)
}

fn attach_box(
    world: &mut World,
    body: BodyId,
    half_width: f32,
    half_height: f32,
    center: Vec2,
    density: f32,
) -> Result<(), SceneError> {
    let polygon = PolygonShape::oriented_box(half_width, half_height, center, 0.0)
        .map_err(|_error| SceneError::Geometry)?;
    let definition = FixtureDef::new(
        Shape::from(polygon),
        density,
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

fn create_water_group(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    let filled =
        Shape::from(PolygonShape::new(&WATER_POLYGON).map_err(|_error| SceneError::Geometry)?);
    let source =
        ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_color(GROUP_COLOR)
        .with_transform(Transform::IDENTITY)
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(system)
}

impl SceneHooks for HydraulicFountainHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        self.elapsed += SIM_DT;
        world
            .set_prismatic_motor_speed(self.joint, scheduled_motor_speed(self.elapsed))
            .map_err(|_error| SessionError::StepFailed)
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
        Ok(WALL_SEGMENTS.to_vec())
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests;
