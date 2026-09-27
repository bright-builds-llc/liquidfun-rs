//! Sinusoidal wave tank: a dynamic end platform lifts one end of a still pool.
//!
//! The construction in this file is a compile stub until the platform, walls,
//! and water group land. Tests lock the still band and the vertical joint.

use liquidfun::math::Vec2;
use liquidfun::{ParticleSystemDef, ParticleSystemId, World};

use super::{BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneHooks};
use crate::session::SessionError;

struct WaveTankHooks;

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }

    let mut world = World::new().map_err(|_error| SessionError::SceneConstruction)?;
    world
        .set_gravity(Vec2::new(0.0, -10.0))
        .map_err(|_error| SessionError::SceneConstruction)?;
    let system_definition = ParticleSystemDef::default()
        .with_radius(0.025)
        .map_err(|_error| SessionError::SceneConstruction)?;
    let particle_system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SessionError::SceneConstruction)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: 0.025,
        hooks: Box::new(WaveTankHooks),
    })
}

impl SceneHooks for WaveTankHooks {
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
        Ok(Vec::new())
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests;
