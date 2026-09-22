//! Allowlisted scene-id factory and shared basin helpers.
//!
//! Dam Break Medium water / Normal gravity remains the documented default:
//! gravity `(0, -10)`, 48×40 = 1920 particles, radius `0.06324555`, spacing `0.101193`,
//! origin `(-4.7, 0.4)`, color `(77, 163, 255, 255)`, basin floor `y=0` from
//! `x=-5.5..5.5` with walls to `y=8`, dynamic circle `(2.5, 5.5)` radius
//! `0.75`, particle cap 10240, timestep `1/60`.

mod basin_family;
mod color_mixer;
mod dam_break;
mod elastic_particles;
mod float_or_sink;
mod fountain;
mod jelly_drop;
mod liquid_timer;
mod particles;
mod rigid_particles;
mod soup;
mod surface_tension;
mod water_wheel;

use liquidfun::collision::{FilterData, PolygonShape, Shape};
use liquidfun::math::Vec2;
use liquidfun::{BodyId, FixtureDef, ParticleSystemId, World};

use crate::session::SessionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SceneId {
    DamBreak,
    Fountain,
    FloatOrSink,
    ColorMixer,
    JellyDrop,
    WaterWheel,
    Particles,
    LiquidTimer,
    SurfaceTension,
    ElasticParticles,
    RigidParticles,
    Soup,
}

pub(crate) fn parse_scene_id(raw: &str) -> Result<SceneId, SessionError> {
    match raw {
        "dam-break" => Ok(SceneId::DamBreak),
        "fountain" => Ok(SceneId::Fountain),
        "float-or-sink" => Ok(SceneId::FloatOrSink),
        "color-mixer" => Ok(SceneId::ColorMixer),
        "jelly-drop" => Ok(SceneId::JellyDrop),
        "water-wheel" => Ok(SceneId::WaterWheel),
        "particles" => Ok(SceneId::Particles),
        "liquid-timer" => Ok(SceneId::LiquidTimer),
        "surface-tension" => Ok(SceneId::SurfaceTension),
        "elastic-particles" => Ok(SceneId::ElasticParticles),
        "rigid-particles" => Ok(SceneId::RigidParticles),
        "soup" => Ok(SceneId::Soup),
        _ => Err(SessionError::UnknownScene),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PointerKind {
    Down,
    Move,
    Up,
    Cancel,
}

pub(crate) fn parse_pointer_kind(raw: &str) -> Result<PointerKind, SessionError> {
    match raw {
        "down" => Ok(PointerKind::Down),
        "move" => Ok(PointerKind::Move),
        "up" => Ok(PointerKind::Up),
        "cancel" => Ok(PointerKind::Cancel),
        _ => Err(SessionError::UnknownControl),
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RigidSegment {
    pub(crate) start: Vec2,
    pub(crate) end: Vec2,
}

#[allow(dead_code)] // Later scene plans return Live or Recreated from apply_control.
pub(crate) enum ControlEffect {
    Live,
    Recreated,
}

pub(crate) struct BuiltScene {
    pub(crate) world: World,
    pub(crate) particle_system: ParticleSystemId,
    pub(crate) particle_radius: f32,
    pub(crate) hooks: Box<dyn SceneHooks>,
}

pub(crate) trait SceneHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
    ) -> Result<(), SessionError>;

    fn apply_control(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
        name: &str,
        value: &str,
    ) -> Result<ControlEffect, SessionError>;

    fn apply_action(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
        name: &str,
    ) -> Result<(), SessionError>;

    fn apply_pointer(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
        kind: PointerKind,
        world_x: f32,
        world_y: f32,
    ) -> Result<(), SessionError>;

    fn collect_segments(&self, world: &World) -> Result<Vec<RigidSegment>, SessionError>;

    fn collect_circles(&self, world: &World) -> Result<Vec<(Vec2, f32)>, SessionError>;

    fn collect_circle_labels(&self, _world: &World) -> Result<Vec<String>, SessionError> {
        Ok(Vec::new())
    }
}

pub(crate) fn build_scene(
    id: SceneId,
    presets: &[(String, String)],
) -> Result<BuiltScene, SessionError> {
    match id {
        SceneId::DamBreak => dam_break::build(presets),
        SceneId::Fountain => fountain::build(presets),
        SceneId::FloatOrSink => float_or_sink::build(presets),
        SceneId::ColorMixer => color_mixer::build(presets),
        SceneId::JellyDrop => jelly_drop::build(presets),
        SceneId::WaterWheel => water_wheel::build(presets),
        SceneId::Particles => particles::build(presets),
        SceneId::LiquidTimer => liquid_timer::build(presets),
        SceneId::SurfaceTension => surface_tension::build(presets),
        SceneId::ElasticParticles => elastic_particles::build(presets),
        SceneId::RigidParticles => rigid_particles::build(presets),
        SceneId::Soup => soup::build(presets),
    }
}

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

pub(crate) fn attach_basin_fixture(
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

#[cfg(test)]
mod tests {
    use super::{PointerKind, parse_pointer_kind};
    use crate::session::SessionError;

    #[test]
    fn parse_pointer_kind_maps_lowercase_tokens() {
        // Arrange
        let tokens = [
            ("down", PointerKind::Down),
            ("move", PointerKind::Move),
            ("up", PointerKind::Up),
            ("cancel", PointerKind::Cancel),
        ];

        for (raw, expected) in tokens {
            // Act
            let parsed = parse_pointer_kind(raw);

            // Assert
            assert_eq!(parsed, Ok(expected));
        }
    }

    #[test]
    fn parse_pointer_kind_rejects_unknown_tokens() {
        // Arrange
        let rejected = ["Down", "click"];

        for raw in rejected {
            // Act
            let parsed = parse_pointer_kind(raw);

            // Assert
            assert_eq!(parsed, Err(SessionError::UnknownControl));
            assert_eq!(
                SessionError::UnknownControl.message(),
                "Rust/WASM control is not allowlisted"
            );
        }
    }
}
