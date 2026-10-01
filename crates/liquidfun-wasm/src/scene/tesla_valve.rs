//! Tesla valve: gravity pulls water down through a one-way conduit.
//!
//! A source at the top pours particles into the valve. A drain under the
//! outlet destroys whatever gets through, so the conduit does not fill up.
//! Forward leaves each curved head turning back downstream into the tube.
//! Reverse flips the valve about a horizontal axis so those heads turn the
//! water back upstream and hold it.

mod geometry;

#[cfg(test)]
mod tests;

use liquidfun::collision::{FilterData, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::{
    BodyDef, BodyId, FixtureDef, ParticleColor, ParticleDef, ParticleFlags, ParticleSystemDef,
    ParticleSystemId, World,
};

use super::{BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks};
use crate::session::SessionError;
use geometry::{
    DRAIN_CENTER, DRAIN_HALF_HEIGHT, DRAIN_HALF_WIDTH, MIRROR_Y, PARTICLE_RADIUS, SPAWN_XS,
    SPAWN_Y, collision_quads, outline_segments,
};

/// Extra substeps so particles stay inside the curved heads.
pub(crate) const PARTICLE_ITERATIONS: u32 = 4;

const FLOW_RATE_CONTROL: &str = "flow-rate";
const FLOW_DIRECTION_CONTROL: &str = "flow-direction";
const FLOW_RATE_MAX: u16 = 720;
const FLOW_RATE_STEP: u16 = 30;
const DEFAULT_FLOW_RATE: u16 = 180;
const MAXIMUM_PARTICLE_COUNT: usize = 4_096;
const PARTICLE_DAMPING: f32 = 0.2;
const WALL_FRICTION: f32 = 0.05;
const SIM_DT: f32 = 1.0 / 60.0;
const SPAWN_SPEED: f32 = 2.0;
const PARTICLE_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const MAX_EMIT_PER_STEP: u32 = 24;

struct TeslaValveHooks {
    forward: bool,
    per_second: f32,
    credit: f32,
    cursor: u32,
    valve_body: BodyId,
    segments: Vec<RigidSegment>,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_tesla_valve().map_err(|_error| SessionError::SceneConstruction)
}

fn build_tesla_valve() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let quads = collision_quads(true);
    let valve_body = create_valve_body(&mut world, &quads)?;
    let particle_system = create_particle_system(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(TeslaValveHooks {
            forward: true,
            per_second: f32::from(DEFAULT_FLOW_RATE),
            credit: 0.0,
            cursor: 0,
            valve_body,
            segments: outline_segments(true),
        }),
    })
}

fn create_valve_body(world: &mut World, quads: &[[Vec2; 4]]) -> Result<BodyId, SceneError> {
    let body = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    for quad in quads {
        attach_quad(world, body, quad)?;
    }
    Ok(body)
}

fn attach_quad(world: &mut World, body: BodyId, corners: &[Vec2; 4]) -> Result<(), SceneError> {
    let polygon = PolygonShape::new(corners).map_err(|_error| SceneError::Geometry)?;
    let definition = FixtureDef::new(
        Shape::from(polygon),
        0.0,
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

fn create_particle_system(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_damping(PARTICLE_DAMPING)
        .map_err(|_error| SceneError::ParticleSystem)?;
    world
        .create_particle_system_with_def(&definition)
        .map_err(|_error| SceneError::ParticleSystem)
}

fn parse_flow_rate(value: &str) -> Option<u16> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if value.len() > 1 && value.starts_with('0') {
        return None;
    }
    let rate = value.parse::<u16>().ok()?;
    if rate > FLOW_RATE_MAX || rate % FLOW_RATE_STEP != 0 {
        return None;
    }
    Some(rate)
}

fn parse_forward(value: &str) -> Option<bool> {
    match value {
        "1" => Some(true),
        "-1" => Some(false),
        _ => None,
    }
}

fn emit_due(hooks: &mut TeslaValveHooks, world: &mut World, system: ParticleSystemId) {
    hooks.credit += hooks.per_second * SIM_DT;
    let mut spawned = 0u32;
    while hooks.credit >= 1.0 && spawned < MAX_EMIT_PER_STEP {
        if !spawn_one(world, system, hooks.cursor) {
            return;
        }
        hooks.credit -= 1.0;
        hooks.cursor = hooks.cursor.wrapping_add(1);
        spawned += 1;
    }
}

fn spawn_one(world: &mut World, system: ParticleSystemId, cursor: u32) -> bool {
    if world.reserve_particle_creations(system, 1).is_err() {
        return false;
    }
    let column = usize::try_from(cursor).map_or(0, |index| index % SPAWN_XS.len());
    let position = Vec2::new(SPAWN_XS[column], SPAWN_Y);
    let velocity = Vec2::new(0.0, -SPAWN_SPEED);
    let Ok(definition) = ParticleDef::default()
        .with_flags(ParticleFlags::WATER)
        .with_color(PARTICLE_COLOR)
        .with_position(position)
        .and_then(|definition| definition.with_velocity(velocity))
    else {
        return false;
    };
    world
        .create_particle_with_def(system, None, &definition)
        .is_ok()
}

fn drain(world: &mut World, system: ParticleSystemId) -> Result<(), SessionError> {
    let polygon =
        PolygonShape::oriented_box(DRAIN_HALF_WIDTH, DRAIN_HALF_HEIGHT, DRAIN_CENTER, 0.0)
            .map_err(|_error| SessionError::StepFailed)?;
    world
        .destroy_particles_in_shape(system, &Shape::from(polygon), Transform::IDENTITY)
        .map_err(|_error| SessionError::StepFailed)?;
    Ok(())
}

fn clear_playfield(world: &mut World, system: ParticleSystemId) -> Result<(), SessionError> {
    let polygon = PolygonShape::oriented_box(2.0, 4.0, Vec2::new(0.0, MIRROR_Y), 0.0)
        .map_err(|_error| SessionError::SceneConstruction)?;
    world
        .destroy_particles_in_shape(system, &Shape::from(polygon), Transform::IDENTITY)
        .map_err(|_error| SessionError::SceneConstruction)?;
    Ok(())
}

fn flip_valve(
    hooks: &mut TeslaValveHooks,
    world: &mut World,
    system: ParticleSystemId,
    forward: bool,
) -> Result<(), SessionError> {
    if forward == hooks.forward {
        return Ok(());
    }
    let quads = collision_quads(forward);
    let new_body =
        create_valve_body(world, &quads).map_err(|_error| SessionError::SceneConstruction)?;
    clear_playfield(world, system)?;
    world
        .destroy_body(hooks.valve_body)
        .map_err(|_error| SessionError::SceneConstruction)?;
    hooks.valve_body = new_body;
    hooks.segments = outline_segments(forward);
    hooks.forward = forward;
    hooks.credit = 0.0;
    Ok(())
}

impl SceneHooks for TeslaValveHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        emit_due(self, world, system);
        Ok(())
    }

    fn on_after_step(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
        _transitions: &[liquidfun::ContactTransition],
    ) -> Result<(), SessionError> {
        drain(world, system)
    }

    fn apply_control(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
        name: &str,
        value: &str,
    ) -> Result<ControlEffect, SessionError> {
        match name {
            FLOW_RATE_CONTROL => {
                let Some(rate) = parse_flow_rate(value) else {
                    return Err(SessionError::UnknownControl);
                };
                self.per_second = f32::from(rate);
                Ok(ControlEffect::Live)
            }
            FLOW_DIRECTION_CONTROL => {
                let Some(forward) = parse_forward(value) else {
                    return Err(SessionError::UnknownControl);
                };
                flip_valve(self, world, system, forward)?;
                Ok(ControlEffect::Live)
            }
            _ => Err(SessionError::UnknownControl),
        }
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
        Ok(self.segments.clone())
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}
