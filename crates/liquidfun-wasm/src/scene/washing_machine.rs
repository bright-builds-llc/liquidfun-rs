//! Spinning drum with rim ribs and a partial fill of water.
//!
//! The drum is a motorized revolute body. Four ribs are fixtures on that same
//! body, so they turn with the wall and lift the water. `drum-speed` is live:
//! it sets the motor in revolutions per minute and leaves the water in place.

mod geometry;

use std::f32::consts::TAU;

use liquidfun::collision::{FilterData, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe,
    ParticleGroupSource,
};
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, JointDef, JointId, ParticleSystemDef, ParticleSystemId,
    RevoluteJointDef, World,
};

use self::geometry::{DRUM_CENTER, Quad, RIB_COUNT, SEGMENT_COUNT};
use super::{BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.03;
const PARTICLE_DAMPING: f32 = 0.25;
const MAXIMUM_PARTICLE_COUNT: usize = 4_096;
const WATER_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const DRUM_DENSITY: f32 = 6.0;
const DRUM_FRICTION: f32 = 0.6;
const MAX_MOTOR_TORQUE: f32 = 1.0e7;
const DRUM_SPEED_CONTROL: &str = "drum-speed";
/// Resting tumble. Twenty turns a minute is slow enough to watch the ribs lift.
const DEFAULT_DRUM_RPM: u16 = 20;
/// Fast spin. One step of wall motion stays inside the wall thickness.
const MAX_DRUM_RPM: u16 = 48;
/// Water box in the lower part of the drum, clear of the wall and the ribs.
const FILL_HALF_WIDTH: f32 = 0.58;
const FILL_HALF_HEIGHT: f32 = 0.36;
const FILL_CENTER: Vec2 = Vec2::new(0.0, -0.55);

struct WashingMachineHooks {
    drum: BodyId,
    joint: JointId,
    wall_quads: [Quad; SEGMENT_COUNT],
    rib_quads: [Quad; RIB_COUNT],
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_washing_machine().map_err(|_error| SessionError::SceneConstruction)
}

fn build_washing_machine() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    let drum_definition = BodyDef::new(BodyType::Dynamic, DRUM_CENTER, 0.0, true)
        .map_err(|_error| SceneError::Body)?
        .with_sleeping_allowed(false);
    let drum = world
        .create_body(&drum_definition)
        .map_err(|_error| SceneError::Body)?;

    let wall_quads = geometry::wall_quads();
    let rib_quads = geometry::rib_quads();
    attach_quads(&mut world, drum, &wall_quads)?;
    attach_quads(&mut world, drum, &rib_quads)?;

    let joint = pin_drum(&mut world, ground, drum, rad_per_sec(DEFAULT_DRUM_RPM))?;
    let particle_system = create_water(&mut world)?;
    carve_quads(&mut world, particle_system, drum, &wall_quads)?;
    carve_quads(&mut world, particle_system, drum, &rib_quads)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(WashingMachineHooks {
            drum,
            joint,
            wall_quads,
            rib_quads,
        }),
    })
}

fn attach_quads(world: &mut World, body: BodyId, quads: &[Quad]) -> Result<(), SceneError> {
    for quad in quads {
        let polygon = PolygonShape::new(quad).map_err(|_error| SceneError::Geometry)?;
        let definition = FixtureDef::new(
            Shape::from(polygon),
            DRUM_DENSITY,
            DRUM_FRICTION,
            0.0,
            false,
            FilterData::default(),
        )
        .map_err(|_error| SceneError::Fixture)?;
        world
            .create_fixture(body, &definition)
            .map_err(|_error| SceneError::Fixture)?;
    }
    Ok(())
}

fn pin_drum(
    world: &mut World,
    ground: BodyId,
    drum: BodyId,
    speed: f32,
) -> Result<JointId, SceneError> {
    let joint = RevoluteJointDef::new(ground, drum)
        .map_err(|_error| SceneError::Body)?
        .with_frame(DRUM_CENTER, Vec2::ZERO, 0.0)
        .map_err(|_error| SceneError::Body)?
        .with_motor(true, speed, MAX_MOTOR_TORQUE)
        .map_err(|_error| SceneError::Body)?;
    world
        .create_joint(JointDef::from(joint))
        .map_err(|_error| SceneError::Body)
}

fn create_water(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_damping(PARTICLE_DAMPING)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    let filled = Shape::from(
        PolygonShape::oriented_box(FILL_HALF_WIDTH, FILL_HALF_HEIGHT, FILL_CENTER, 0.0)
            .map_err(|_error| SceneError::Geometry)?,
    );
    let source =
        ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(ParticleFlags::WATER)
        .with_color(WATER_COLOR)
        .with_default_stride()
        .with_transform(Transform::IDENTITY)
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(system)
}

fn carve_quads(
    world: &mut World,
    system: ParticleSystemId,
    body: BodyId,
    quads: &[Quad],
) -> Result<(), SceneError> {
    let transform = world
        .body_snapshot(body)
        .map_err(|_error| SceneError::Body)?
        .transform();
    for quad in quads {
        let shape = Shape::from(PolygonShape::new(quad).map_err(|_error| SceneError::Geometry)?);
        world
            .destroy_particles_in_shape(system, &shape, transform)
            .map_err(|_error| SceneError::Particle)?;
    }
    Ok(())
}

/// Whole-number rpm tokens from `0` through [`MAX_DRUM_RPM`].
fn parse_drum_rpm(value: &str) -> Option<u16> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if value.len() > 1 && value.starts_with('0') {
        return None;
    }

    let rpm = value.parse::<u16>().ok()?;
    if rpm > MAX_DRUM_RPM {
        return None;
    }
    Some(rpm)
}

fn rad_per_sec(rpm: u16) -> f32 {
    f32::from(rpm) * TAU / 60.0
}

fn write_motor_speed(world: &mut World, joint: JointId, speed: f32) -> Result<(), SessionError> {
    world
        .set_revolute_motor_speed(joint, speed)
        .map_err(|_error| SessionError::StepFailed)
}

impl SceneHooks for WashingMachineHooks {
    fn on_advance(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        Ok(())
    }

    fn apply_control(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
        name: &str,
        value: &str,
    ) -> Result<ControlEffect, SessionError> {
        if name != DRUM_SPEED_CONTROL {
            return Err(SessionError::UnknownControl);
        }
        let Some(rpm) = parse_drum_rpm(value) else {
            return Err(SessionError::UnknownControl);
        };
        write_motor_speed(world, self.joint, rad_per_sec(rpm))?;
        Ok(ControlEffect::Live)
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

    fn collect_segments(&self, world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        let transform = world
            .body_snapshot(self.drum)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .transform();
        let mut segments = Vec::with_capacity((SEGMENT_COUNT + RIB_COUNT) * 4);
        for quad in self.wall_quads.iter().chain(self.rib_quads.iter()) {
            push_quad(&mut segments, transform, quad);
        }
        Ok(segments)
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

fn push_quad(segments: &mut Vec<RigidSegment>, transform: Transform, quad: &Quad) {
    let world_corners = [
        transform.apply(quad[0]),
        transform.apply(quad[1]),
        transform.apply(quad[2]),
        transform.apply(quad[3]),
    ];
    for index in 0..4 {
        segments.push(RigidSegment {
            start: world_corners[index],
            end: world_corners[(index + 1) % 4],
        });
    }
}

#[cfg(test)]
mod tests;
