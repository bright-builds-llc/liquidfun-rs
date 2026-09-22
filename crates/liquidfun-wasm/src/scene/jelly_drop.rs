//! Native Jelly Drop scene: a bounded elastic particle group on two rigid bars.

use liquidfun::collision::{CircleShape, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe,
    ParticleGroupSource,
};
use liquidfun::{ParticleGroupId, ParticleSystemDef, ParticleSystemId, World};

use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
    attach_basin_fixture,
};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.050_596;
const MAXIMUM_PARTICLE_COUNT: usize = 2200;
const JELLY_COLOR: ParticleColor = ParticleColor::new(244, 114, 182, 255);
const JELLY_HALF_EXTENT: f32 = 1.2;
const JELLY_CENTER: Vec2 = Vec2::new(0.0, 3.6);
const SOFT_STRENGTH: f32 = 0.4;
const MEDIUM_STRENGTH: f32 = 1.0;
const FIRM_STRENGTH: f32 = 2.0;
/// Downward labeled poke; applied to the contiguous group member range.
const POKE_IMPULSE: Vec2 = Vec2::new(0.0, -8.0);
const POINTER_POKE_RADIUS: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JellyShape {
    Circle,
    Square,
}

impl JellyShape {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "circle" => Some(Self::Circle),
            "square" => Some(Self::Square),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Softness {
    Soft,
    Medium,
    Firm,
}

impl Softness {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "soft" => Some(Self::Soft),
            "medium" => Some(Self::Medium),
            "firm" => Some(Self::Firm),
            _ => None,
        }
    }

    fn strength(self) -> f32 {
        match self {
            Self::Soft => SOFT_STRENGTH,
            Self::Medium => MEDIUM_STRENGTH,
            Self::Firm => FIRM_STRENGTH,
        }
    }
}

struct JellyDropHooks {
    bar_segments: [RigidSegment; 4],
    group: ParticleGroupId,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let shape = preset_value(presets, "shape").and_then(JellyShape::parse);
    let softness = preset_value(presets, "softness").and_then(Softness::parse);
    build_jelly(
        shape.unwrap_or(JellyShape::Circle),
        softness.unwrap_or(Softness::Medium),
    )
    .map_err(|_error| SessionError::SceneConstruction)
}

fn preset_value<'a>(presets: &'a [(String, String)], name: &str) -> Option<&'a str> {
    presets
        .iter()
        .rev()
        .find(|(key, _value)| key == name)
        .map(|(_key, value)| value.as_str())
}

fn build_jelly(shape: JellyShape, softness: Softness) -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(Vec2::new(0.0, -10.0))
        .map_err(|_error| SceneError::Gravity)?;

    let ground = world
        .create_body(&liquidfun::BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    attach_basin_fixture(
        &mut world,
        ground,
        &[
            Vec2::new(-3.2, 0.85),
            Vec2::new(0.6, 0.85),
            Vec2::new(0.6, 1.15),
            Vec2::new(-3.2, 1.15),
        ],
    )?;
    attach_basin_fixture(
        &mut world,
        ground,
        &[
            Vec2::new(-0.6, 0.85),
            Vec2::new(3.2, 0.85),
            Vec2::new(3.2, 1.15),
            Vec2::new(-0.6, 1.15),
        ],
    )?;
    attach_basin_fixture(
        &mut world,
        ground,
        &[
            Vec2::new(-3.55, 0.85),
            Vec2::new(-3.2, 0.85),
            Vec2::new(-3.2, 7.2),
            Vec2::new(-3.55, 7.2),
        ],
    )?;
    attach_basin_fixture(
        &mut world,
        ground,
        &[
            Vec2::new(3.2, 0.85),
            Vec2::new(3.55, 0.85),
            Vec2::new(3.55, 7.2),
            Vec2::new(3.2, 7.2),
        ],
    )?;

    let (particle_system, group) = create_jelly_group(&mut world, shape, softness)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(JellyDropHooks {
            bar_segments: [
                RigidSegment {
                    start: Vec2::new(-3.2, 1.0),
                    end: Vec2::new(0.6, 1.0),
                },
                RigidSegment {
                    start: Vec2::new(-0.6, 1.0),
                    end: Vec2::new(3.2, 1.0),
                },
                RigidSegment {
                    start: Vec2::new(-3.2, 1.15),
                    end: Vec2::new(-3.2, 7.2),
                },
                RigidSegment {
                    start: Vec2::new(3.2, 1.15),
                    end: Vec2::new(3.2, 7.2),
                },
            ],
            group,
        }),
    })
}

fn create_jelly_group(
    world: &mut World,
    shape: JellyShape,
    softness: Softness,
) -> Result<(ParticleSystemId, ParticleGroupId), SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_damping(1.2)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_elastic_strength(0.75)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_spring_strength(0.75)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    let source = jelly_source(shape)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(ParticleFlags::ELASTIC | ParticleFlags::SPRING)
        .with_strength(softness.strength())
        .map_err(|_error| SceneError::Particle)?
        .with_color(JELLY_COLOR)
        .with_transform(Transform::from_position_angle(JELLY_CENTER, 0.0))
        .map_err(|_error| SceneError::Particle)?;
    let group = world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok((system, group))
}

fn jelly_source(shape: JellyShape) -> Result<ParticleGroupSource, SceneError> {
    let filled = match shape {
        JellyShape::Circle => Shape::from(
            CircleShape::new(Vec2::ZERO, JELLY_HALF_EXTENT)
                .map_err(|_error| SceneError::Geometry)?,
        ),
        JellyShape::Square => Shape::from(
            PolygonShape::new(&[
                Vec2::new(-JELLY_HALF_EXTENT, -JELLY_HALF_EXTENT),
                Vec2::new(JELLY_HALF_EXTENT, -JELLY_HALF_EXTENT),
                Vec2::new(JELLY_HALF_EXTENT, JELLY_HALF_EXTENT),
                Vec2::new(-JELLY_HALF_EXTENT, JELLY_HALF_EXTENT),
            ])
            .map_err(|_error| SceneError::Geometry)?,
        ),
    };
    ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)
}

fn poke_jelly(
    world: &mut World,
    system: ParticleSystemId,
    group: ParticleGroupId,
) -> Result<(), SessionError> {
    let members = {
        let view = world
            .particle_group_view(group)
            .map_err(|_error| SessionError::SceneConstruction)?;
        view.member_ids().to_vec()
    };
    let poke_len = members.len().div_ceil(3).max(1).min(members.len());
    let poked = &members[..poke_len];
    world
        .apply_particle_linear_impulse_range(system, poked, POKE_IMPULSE)
        .map_err(|_error| SessionError::SceneConstruction)?;
    Ok(())
}

fn poke_jelly_at(
    world: &mut World,
    system: ParticleSystemId,
    group: ParticleGroupId,
    origin: Vec2,
) -> Result<(), SessionError> {
    let members = {
        let view = world
            .particle_group_view(group)
            .map_err(|_error| SessionError::SceneConstruction)?;
        view.member_ids().to_vec()
    };
    let (positions, ids) = {
        let view = world
            .particle_system_view(system)
            .map_err(|_error| SessionError::SceneConstruction)?;
        (view.positions().to_vec(), view.particle_ids().to_vec())
    };
    for member in members {
        let Some(index) = ids.iter().position(|id| *id == member) else {
            continue;
        };
        let Some(position) = positions.get(index).copied() else {
            continue;
        };
        if (position - origin).length() > POINTER_POKE_RADIUS {
            continue;
        }
        world
            .apply_particle_linear_impulse(member, POKE_IMPULSE)
            .map_err(|_error| SessionError::SceneConstruction)?;
    }
    Ok(())
}

impl SceneHooks for JellyDropHooks {
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
        name: &str,
        value: &str,
    ) -> Result<ControlEffect, SessionError> {
        match name {
            "shape" => {
                let Some(_shape) = JellyShape::parse(value) else {
                    return Err(SessionError::UnknownControl);
                };
                Ok(ControlEffect::Recreated)
            }
            "softness" => {
                let Some(_softness) = Softness::parse(value) else {
                    return Err(SessionError::UnknownControl);
                };
                Ok(ControlEffect::Recreated)
            }
            _ => Err(SessionError::UnknownControl),
        }
    }

    fn apply_action(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
        name: &str,
    ) -> Result<(), SessionError> {
        if name != "poke-jelly" {
            return Err(SessionError::UnknownControl);
        }
        poke_jelly(world, system, self.group)
    }

    fn apply_pointer(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
        kind: PointerKind,
        world_x: f32,
        world_y: f32,
    ) -> Result<(), SessionError> {
        match kind {
            PointerKind::Down => {
                poke_jelly_at(world, system, self.group, Vec2::new(world_x, world_y))
            }
            PointerKind::Move | PointerKind::Up | PointerKind::Cancel => Ok(()),
        }
    }

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        Ok(self.bar_segments.to_vec())
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests;
