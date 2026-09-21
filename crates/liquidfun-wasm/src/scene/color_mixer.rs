//! Native Color Mixer: two contact-mixing particle-color groups in a bowl.

use liquidfun::collision::{CircleShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe,
    ParticleGroupSource,
};
use liquidfun::{BodyDef, ParticleSystemDef, ParticleSystemId, World};

use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
    attach_basin_fixture,
};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.05692;
const MAXIMUM_PARTICLE_COUNT: usize = 2200;
const BLUE_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const RED_COLOR: ParticleColor = ParticleColor::new(248, 113, 113, 255);
const GROUP_RADIUS: f32 = 1.15;
const BLUE_CENTER: Vec2 = Vec2::new(-1.0, 2.2);
const RED_CENTER: Vec2 = Vec2::new(1.0, 2.2);
const MIXING_FLAGS: ParticleFlags = ParticleFlags::from_bits_truncate(
    ParticleFlags::WATER.bits() | ParticleFlags::COLOR_MIXING.bits(),
);
const OFF_MIX_STRENGTH: f32 = 0.0;
const GENTLE_MIX_STRENGTH: f32 = 0.25;
const STRONG_MIX_STRENGTH: f32 = 0.5;
const SLOW_STIR_FORCE: Vec2 = Vec2::new(8.0, 0.0);
const FAST_STIR_FORCE: Vec2 = Vec2::new(18.0, 0.0);
const POINTER_STIR_RADIUS: f32 = 1.25;
const POINTER_STIR_FORCE: f32 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum MixStrength {
    Off,
    Gentle,
    Strong,
}

impl MixStrength {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "off" => Some(Self::Off),
            "gentle" => Some(Self::Gentle),
            "strong" => Some(Self::Strong),
            _ => None,
        }
    }

    fn strength(self) -> f32 {
        match self {
            Self::Off => OFF_MIX_STRENGTH,
            Self::Gentle => GENTLE_MIX_STRENGTH,
            Self::Strong => STRONG_MIX_STRENGTH,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum StirSpeed {
    Off,
    Slow,
    Fast,
}

impl StirSpeed {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "off" => Some(Self::Off),
            "slow" => Some(Self::Slow),
            "fast" => Some(Self::Fast),
            _ => None,
        }
    }

    fn maybe_force(self) -> Option<Vec2> {
        match self {
            Self::Off => None,
            Self::Slow => Some(SLOW_STIR_FORCE),
            Self::Fast => Some(FAST_STIR_FORCE),
        }
    }
}

struct ColorMixerHooks {
    basin_segments: [RigidSegment; 3],
    stir_speed: StirSpeed,
    maybe_pointer: Option<Vec2>,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let mix = preset_value(presets, "mix-strength")
        .map(MixStrength::parse)
        .map_or(Ok(MixStrength::Strong), |maybe_mix| {
            maybe_mix.ok_or(SessionError::UnknownControl)
        })?;
    build_mixer(mix).map_err(|_error| SessionError::SceneConstruction)
}

fn preset_value<'a>(presets: &'a [(String, String)], name: &str) -> Option<&'a str> {
    presets
        .iter()
        .rev()
        .find(|(key, _value)| key == name)
        .map(|(_key, value)| value.as_str())
}

fn build_mixer(mix: MixStrength) -> Result<BuiltScene, SceneError> {
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

    let particle_system = create_mixing_groups(&mut world, mix)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(ColorMixerHooks {
            stir_speed: StirSpeed::Slow,
            maybe_pointer: None,
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
        }),
    })
}

fn create_mixing_groups(
    world: &mut World,
    mix: MixStrength,
) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_color_mixing_strength(mix.strength())
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    create_colored_group(world, system, BLUE_CENTER, BLUE_COLOR)?;
    create_colored_group(world, system, RED_CENTER, RED_COLOR)?;
    Ok(system)
}

fn create_colored_group(
    world: &mut World,
    system: ParticleSystemId,
    center: Vec2,
    color: ParticleColor,
) -> Result<(), SceneError> {
    let filled = Shape::from(
        CircleShape::new(Vec2::ZERO, GROUP_RADIUS).map_err(|_error| SceneError::Geometry)?,
    );
    let source =
        ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(MIXING_FLAGS)
        .with_color(color)
        .with_transform(Transform::from_position_angle(center, 0.0))
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(())
}

fn stir_particles(
    world: &mut World,
    system: ParticleSystemId,
    force: Vec2,
) -> Result<(), SessionError> {
    let particles = {
        let view = world
            .particle_system_view(system)
            .map_err(|_error| SessionError::SceneConstruction)?;
        view.particle_ids().to_vec()
    };
    if particles.is_empty() {
        return Ok(());
    }
    world
        .apply_particle_force_range(system, &particles, force)
        .map_err(|_error| SessionError::SceneConstruction)?;
    Ok(())
}

fn apply_pointer_stir(
    world: &mut World,
    system: ParticleSystemId,
    origin: Vec2,
) -> Result<(), SessionError> {
    let (positions, ids) = {
        let view = world
            .particle_system_view(system)
            .map_err(|_error| SessionError::SceneConstruction)?;
        (view.positions().to_vec(), view.particle_ids().to_vec())
    };
    for (position, id) in positions.into_iter().zip(ids) {
        if (position - origin).length() > POINTER_STIR_RADIUS {
            continue;
        }
        let tangent = Vec2::new(-(position.y - origin.y), position.x - origin.x);
        let length = tangent.length();
        if length < 1e-4 {
            continue;
        }
        let force = tangent * (POINTER_STIR_FORCE / length);
        world
            .apply_particle_force(id, force)
            .map_err(|_error| SessionError::SceneConstruction)?;
    }
    Ok(())
}

impl SceneHooks for ColorMixerHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        if let Some(force) = self.stir_speed.maybe_force() {
            stir_particles(world, system, force)?;
        }
        let Some(origin) = self.maybe_pointer else {
            return Ok(());
        };
        apply_pointer_stir(world, system, origin)
    }

    fn apply_control(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        name: &str,
        value: &str,
    ) -> Result<ControlEffect, SessionError> {
        match name {
            "mix-strength" => {
                let Some(_mix) = MixStrength::parse(value) else {
                    return Err(SessionError::UnknownControl);
                };
                Ok(ControlEffect::Recreated)
            }
            "stir-speed" => {
                let Some(speed) = StirSpeed::parse(value) else {
                    return Err(SessionError::UnknownControl);
                };
                self.stir_speed = speed;
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
        kind: PointerKind,
        world_x: f32,
        world_y: f32,
    ) -> Result<(), SessionError> {
        match kind {
            PointerKind::Down | PointerKind::Move => {
                self.maybe_pointer = Some(Vec2::new(world_x, world_y));
                Ok(())
            }
            PointerKind::Up | PointerKind::Cancel => {
                self.maybe_pointer = None;
                Ok(())
            }
        }
    }

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        Ok(self.basin_segments.to_vec())
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests;
