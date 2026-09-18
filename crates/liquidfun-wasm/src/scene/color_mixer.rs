//! Native Color Mixer: two contact-mixing particle-color groups in a bowl.

use liquidfun::collision::{CircleShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{BodyDef, ParticleSystemDef, ParticleSystemId, World};

use super::{
    BuiltScene, ControlEffect, RigidSegment, SceneError, SceneHooks, attach_basin_fixture,
};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.18;
const MAXIMUM_PARTICLE_COUNT: usize = 220;
const TEAL_COLOR: ParticleColor = ParticleColor::new(57, 211, 199, 255);
const RED_COLOR: ParticleColor = ParticleColor::new(248, 113, 113, 255);
const GROUP_RADIUS: f32 = 1.15;
const TEAL_CENTER: Vec2 = Vec2::new(-1.0, 2.2);
const RED_CENTER: Vec2 = Vec2::new(1.0, 2.2);
const MIXING_FLAGS: ParticleFlags = ParticleFlags::from_bits_truncate(
    ParticleFlags::WATER.bits() | ParticleFlags::COLOR_MIXING.bits(),
);

struct ColorMixerHooks {
    basin_segments: [RigidSegment; 3],
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let _ = presets;
    build_mixer().map_err(|_error| SessionError::SceneConstruction)
}

fn build_mixer() -> Result<BuiltScene, SceneError> {
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

    let particle_system = create_mixing_groups(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(ColorMixerHooks {
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

fn create_mixing_groups(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    create_colored_group(world, system, TEAL_CENTER, TEAL_COLOR)?;
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
    let source = ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)?;
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

impl SceneHooks for ColorMixerHooks {
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

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        Ok(self.basin_segments.to_vec())
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

    const TEAL: [u8; 4] = [57, 211, 199, 255];
    const RED: [u8; 4] = [248, 113, 113, 255];

    #[test]
    fn create_succeeds_with_two_distinct_mixing_colors() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::ColorMixer)
            .expect("Color Mixer should construct two mixing groups");
        let frame = capture(&session);
        let colors = frame.particle_colors();

        // Assert
        assert!((40..=220).contains(&session.particle_count()));
        assert!(
            colors_contain(colors.as_ref(), TEAL),
            "captured colors should include teal (57, 211, 199, 255)"
        );
        assert!(
            colors_contain(colors.as_ref(), RED),
            "captured colors should include destructive red (248, 113, 113, 255)"
        );
    }

    #[test]
    fn every_constructed_particle_carries_color_mixing() {
        // Arrange / Act
        let super::BuiltScene {
            world,
            particle_system,
            ..
        } = super::build(&[]).expect("Color Mixer should construct");
        let view = world
            .particle_system_view(particle_system)
            .expect("constructed system should stay live");

        // Assert
        assert!(!view.flags().is_empty());
        assert!(
            view.flags()
                .iter()
                .all(|flags| flags.contains(ParticleFlags::COLOR_MIXING)),
            "every particle must carry COLOR_MIXING so contact mixing can run"
        );
    }

    fn colors_contain(colors: &[u8], expected: [u8; 4]) -> bool {
        colors.chunks_exact(4).any(|chunk| chunk == expected)
    }

    fn capture(session: &SessionCore) -> ProofFrame {
        ProofFrame::from(
            session
                .capture_frame()
                .expect("Color Mixer should capture a frame"),
        )
    }
}
