//! Native Jelly Drop scene: a bounded elastic particle group on two rigid bars.

use liquidfun::collision::{CircleShape, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{ParticleSystemDef, ParticleSystemId, World};

use super::{BuiltScene, RigidSegment, SceneError, SceneHooks, attach_basin_fixture};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.16;
const MAXIMUM_PARTICLE_COUNT: usize = 220;
const JELLY_COLOR: ParticleColor = ParticleColor::new(244, 114, 182, 255);
const JELLY_HALF_EXTENT: f32 = 1.2;
const JELLY_CENTER: Vec2 = Vec2::new(0.0, 3.6);
const SOFT_STRENGTH: f32 = 0.4;
const MEDIUM_STRENGTH: f32 = 1.0;
const FIRM_STRENGTH: f32 = 2.0;

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
    bar_segments: [RigidSegment; 2],
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
            Vec2::new(-0.4, 0.85),
            Vec2::new(-0.4, 1.15),
            Vec2::new(-3.2, 1.15),
        ],
    )?;
    attach_basin_fixture(
        &mut world,
        ground,
        &[
            Vec2::new(0.4, 0.85),
            Vec2::new(3.2, 0.85),
            Vec2::new(3.2, 1.15),
            Vec2::new(0.4, 1.15),
        ],
    )?;

    let particle_system = create_jelly_group(&mut world, shape, softness)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(JellyDropHooks {
            bar_segments: [
                RigidSegment {
                    start: Vec2::new(-3.2, 1.0),
                    end: Vec2::new(-0.4, 1.0),
                },
                RigidSegment {
                    start: Vec2::new(0.4, 1.0),
                    end: Vec2::new(3.2, 1.0),
                },
            ],
        }),
    })
}

fn create_jelly_group(
    world: &mut World,
    shape: JellyShape,
    softness: Softness,
) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
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
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(system)
}

fn jelly_source(shape: JellyShape) -> Result<ParticleGroupSource, SceneError> {
    let filled = match shape {
        JellyShape::Circle => Shape::from(
            CircleShape::new(Vec2::ZERO, JELLY_HALF_EXTENT).map_err(|_error| SceneError::Geometry)?,
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
        _name: &str,
        _value: &str,
    ) -> Result<super::ControlEffect, SessionError> {
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

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        Ok(self.bar_segments.to_vec())
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use liquidfun::particle::ParticleFlags;

    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::SessionCore;

    #[test]
    fn create_jelly_drop_builds_a_bounded_elastic_group_on_two_bars() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::JellyDrop)
            .expect("Jelly Drop should construct a native elastic group");
        let frame = capture(&session);

        // Assert
        assert!((8..=220).contains(&session.particle_count()));
        assert!(session.particle_count() <= 512);
        assert!(
            frame.rigid_segments().len() >= 8,
            "two rigid bars report at least eight segment floats"
        );
    }

    #[test]
    fn constructed_group_flags_include_elastic_and_spring() {
        // Arrange / Act
        let super::BuiltScene {
            world,
            particle_system,
            ..
        } = super::build(&[]).expect("Jelly Drop should construct");
        let view = world
            .particle_system_view(particle_system)
            .expect("constructed system should stay live");
        let expected = ParticleFlags::ELASTIC | ParticleFlags::SPRING;

        // Assert
        assert!(
            view.flags()
                .iter()
                .any(|flags| flags.contains(ParticleFlags::ELASTIC)
                    && flags.contains(ParticleFlags::SPRING)),
            "group particles should carry {expected:?}"
        );
    }

    #[test]
    fn thirty_steps_keep_the_particle_count_constant() {
        // Arrange
        let mut session = SessionCore::create(SceneId::JellyDrop)
            .expect("Jelly Drop should construct a native elastic group");
        let before = capture(&session).particle_count();

        // Act
        advance_steps(&mut session, 30);
        let after = capture(&session).particle_count();

        // Assert
        assert_eq!(after, before, "Jelly Drop must not emit extra particles");
        assert!((8..=220).contains(&after));
    }

    fn advance_steps(session: &mut SessionCore, steps: u32) {
        let mut remaining = steps;
        while remaining > 0 {
            let chunk = remaining.min(4);
            session
                .advance(chunk)
                .expect("bounded native steps should succeed");
            remaining -= chunk;
        }
    }

    fn capture(session: &SessionCore) -> ProofFrame {
        ProofFrame::from(
            session
                .capture_frame()
                .expect("Jelly Drop should capture a frame"),
        )
    }
}
