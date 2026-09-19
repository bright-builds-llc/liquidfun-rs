//! Native Color Mixer: two contact-mixing particle-color groups in a bowl.

use liquidfun::collision::{CircleShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe,
    ParticleGroupSource,
};
use liquidfun::{BodyDef, ParticleSystemDef, ParticleSystemId, World};

use super::{
    attach_basin_fixture, BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError,
    SceneHooks,
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
const OFF_MIX_STRENGTH: f32 = 0.0;
const GENTLE_MIX_STRENGTH: f32 = 0.25;
const STRONG_MIX_STRENGTH: f32 = 0.5;
const SLOW_STIR_FORCE: Vec2 = Vec2::new(8.0, 0.0);
const FAST_STIR_FORCE: Vec2 = Vec2::new(18.0, 0.0);

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

impl SceneHooks for ColorMixerHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        let Some(force) = self.stir_speed.maybe_force() else {
            return Ok(());
        };
        stir_particles(world, system, force)
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
        _world_x: f32,
        _world_y: f32,
    ) -> Result<(), SessionError> {
        match kind {
            PointerKind::Down | PointerKind::Move | PointerKind::Up | PointerKind::Cancel => {
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
mod tests {
    use liquidfun::particle::ParticleFlags;

    use crate::scene::SceneId;
    use crate::session::SessionCore;
    use crate::ProofFrame;

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

    #[test]
    fn mix_strength_off_keeps_color_lanes_stable_after_stirred_steps() {
        // Arrange
        let mut session = SessionCore::create(SceneId::ColorMixer)
            .expect("Color Mixer should construct two mixing groups");
        let recreated = session
            .apply_control("mix-strength", "off")
            .expect("mix-strength=off should recreate");
        let before = capture(&session);
        let before_colors = before.particle_colors();
        let before_count = before.particle_count();

        // Act
        advance_steps(&mut session, 120);
        let after = capture(&session);

        // Assert
        assert!(recreated, "mix-strength must return Recreated");
        assert_eq!(after.particle_count(), before_count);
        assert_eq!(
            after.particle_colors().as_ref(),
            before_colors.as_ref(),
            "Off mix-strength must skip the engine color-mixing pass"
        );
    }

    #[test]
    fn default_strong_changes_color_lanes_after_contact() {
        // Arrange
        let mut session = SessionCore::create(SceneId::ColorMixer)
            .expect("Color Mixer should construct two mixing groups");
        let before = capture(&session);
        let before_colors = before.particle_colors();
        let before_count = before.particle_count();

        // Act
        advance_steps(&mut session, 120);
        let after = capture(&session);

        // Assert
        assert_eq!(after.particle_count(), before_count);
        assert_ne!(
            after.particle_colors().as_ref(),
            before_colors.as_ref(),
            "default Strong mix-strength must change captured color-lane bytes"
        );
    }

    #[test]
    fn stir_speed_presets_apply_live() {
        // Arrange
        let super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = super::build(&[]).expect("Color Mixer should construct");

        // Act
        let off = hooks
            .apply_control(&mut world, particle_system, "stir-speed", "off")
            .expect("stir-speed=off should apply");
        let slow = hooks
            .apply_control(&mut world, particle_system, "stir-speed", "slow")
            .expect("stir-speed=slow should apply");
        let fast = hooks
            .apply_control(&mut world, particle_system, "stir-speed", "fast")
            .expect("stir-speed=fast should apply");

        // Assert
        assert!(matches!(off, crate::scene::ControlEffect::Live));
        assert!(matches!(slow, crate::scene::ControlEffect::Live));
        assert!(matches!(fast, crate::scene::ControlEffect::Live));
    }

    #[test]
    fn unknown_mix_and_stir_tokens_fail_closed() {
        // Arrange
        let mut session = SessionCore::create(SceneId::ColorMixer)
            .expect("Color Mixer should construct two mixing groups");

        // Act
        let bad_mix = session.apply_control("mix-strength", "paint");
        let bad_stir = session.apply_control("stir-speed", "spin");
        let unknown_name = session.apply_control("water-amount", "medium");
        let unknown_action = session.apply_action("poke-jelly");

        // Assert
        assert_eq!(bad_mix, Err(crate::session::SessionError::UnknownControl));
        assert_eq!(bad_stir, Err(crate::session::SessionError::UnknownControl));
        assert_eq!(
            unknown_name,
            Err(crate::session::SessionError::UnknownControl)
        );
        assert_eq!(
            unknown_action,
            Err(crate::session::SessionError::UnknownControl)
        );
    }

    fn colors_contain(colors: &[u8], expected: [u8; 4]) -> bool {
        colors.chunks_exact(4).any(|chunk| chunk == expected)
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
                .expect("Color Mixer should capture a frame"),
        )
    }
}
