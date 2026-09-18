//! Native Fountain: a lifetime-capped stream that plateaus instead of hitting the frame cap.

use std::f32::consts::TAU;

use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, ParticleColor, ParticleDef, ParticleFlags, ParticleSystemDef, ParticleSystemId, World,
};

use super::{
    BuiltScene, ControlEffect, RigidSegment, SceneError, SceneHooks, attach_basin_fixture,
};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.18;
const MAXIMUM_PARTICLE_COUNT: usize = 320;
const PARTICLE_LIFETIME: f32 = 3.0;
const PARTICLE_COLOR: ParticleColor = ParticleColor::new(57, 211, 199, 255);
const NOZZLE_POSITION: Vec2 = Vec2::new(0.0, 0.5);
const SLOW_LAUNCH_SPEED: f32 = 4.0;
const MEDIUM_LAUNCH_SPEED: f32 = 8.0;
const FAST_LAUNCH_SPEED: f32 = 12.0;
const LEFT_AIM: f32 = -TAU / 8.0;
const UP_AIM: f32 = 0.0;
const RIGHT_AIM: f32 = TAU / 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmissionRate {
    Off,
    Low,
    Medium,
    High,
}

impl EmissionRate {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "off" => Some(Self::Off),
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            _ => None,
        }
    }

    fn particles_per_step(self) -> u8 {
        match self {
            Self::Off => 0,
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LaunchSpeed {
    Slow,
    Medium,
    Fast,
}

impl LaunchSpeed {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "slow" => Some(Self::Slow),
            "medium" => Some(Self::Medium),
            "fast" => Some(Self::Fast),
            _ => None,
        }
    }

    fn speed(self) -> f32 {
        match self {
            Self::Slow => SLOW_LAUNCH_SPEED,
            Self::Medium => MEDIUM_LAUNCH_SPEED,
            Self::Fast => FAST_LAUNCH_SPEED,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AimAngle {
    Left,
    Up,
    Right,
}

impl AimAngle {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "left" => Some(Self::Left),
            "up" => Some(Self::Up),
            "right" => Some(Self::Right),
            _ => None,
        }
    }

    fn angle_from_up(self) -> f32 {
        match self {
            Self::Left => LEFT_AIM,
            Self::Up => UP_AIM,
            Self::Right => RIGHT_AIM,
        }
    }
}

struct FountainHooks {
    basin_segments: [RigidSegment; 3],
    emission_rate: EmissionRate,
    launch_speed: f32,
    aim_from_up: f32,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let _ = presets;
    build_fountain().map_err(|_error| SessionError::SceneConstruction)
}

fn build_fountain() -> Result<BuiltScene, SceneError> {
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

    let particle_system = create_stream_system(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(FountainHooks {
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
            emission_rate: EmissionRate::Medium,
            launch_speed: MEDIUM_LAUNCH_SPEED,
            aim_from_up: UP_AIM,
        }),
    })
}

fn create_stream_system(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_destruction_by_age(true);
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let definition = ParticleDef::default()
        .with_flags(ParticleFlags::WATER)
        .with_color(PARTICLE_COLOR)
        .with_position(NOZZLE_POSITION)
        .map_err(|_error| SceneError::Particle)?
        .with_lifetime(PARTICLE_LIFETIME)
        .map_err(|_error| SceneError::Particle)?;
    let receipt = world
        .create_particle_with_def(system, None, &definition)
        .map_err(|_error| SceneError::Particle)?;
    if !receipt.destruction_occurrences().is_empty() {
        return Err(SceneError::Particle);
    }
    Ok(system)
}

fn aim_velocity(aim_from_up: f32, speed: f32) -> Vec2 {
    Vec2::new(aim_from_up.sin() * speed, aim_from_up.cos() * speed)
}

fn emit_stream(world: &mut World, system: ParticleSystemId, count: u8, speed: f32, aim_from_up: f32) {
    let velocity = aim_velocity(aim_from_up, speed);
    for index in 0..count {
        let position = Vec2::new(
            NOZZLE_POSITION.x + f32::from(index) * 0.08,
            NOZZLE_POSITION.y,
        );
        let Ok(definition) = ParticleDef::default()
            .with_flags(ParticleFlags::WATER)
            .with_color(PARTICLE_COLOR)
            .with_position(position)
            .and_then(|definition| definition.with_velocity(velocity))
            .and_then(|definition| definition.with_lifetime(PARTICLE_LIFETIME))
        else {
            return;
        };
        match world.create_particle_with_def(system, None, &definition) {
            Ok(_receipt) => {}
            Err(_error) => return,
        }
    }
}

impl SceneHooks for FountainHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        let count = self.emission_rate.particles_per_step();
        if count == 0 {
            return Ok(());
        }
        emit_stream(world, system, count, self.launch_speed, self.aim_from_up);
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
            "emission-rate" => {
                let Some(rate) = EmissionRate::parse(value) else {
                    return Err(SessionError::UnknownControl);
                };
                self.emission_rate = rate;
                Ok(ControlEffect::Live)
            }
            "launch-speed" => {
                let Some(speed) = LaunchSpeed::parse(value) else {
                    return Err(SessionError::UnknownControl);
                };
                self.launch_speed = speed.speed();
                Ok(ControlEffect::Live)
            }
            "aim-angle" => {
                let Some(aim) = AimAngle::parse(value) else {
                    return Err(SessionError::UnknownControl);
                };
                self.aim_from_up = aim.angle_from_up();
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

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        Ok(self.basin_segments.to_vec())
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::ProofFrame;
    use crate::scene::{ControlEffect, SceneId};
    use crate::session::SessionCore;

    #[test]
    fn create_fountain_succeeds() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::Fountain);

        // Assert
        assert!(
            session.is_ok(),
            "Fountain should construct a bounded native stream"
        );
    }

    #[test]
    fn default_medium_stream_plateaus_at_or_below_three_hundred_twenty() {
        // Arrange
        let mut session = SessionCore::create(SceneId::Fountain)
            .expect("Fountain should construct a bounded native stream");

        // Act
        advance_steps(&mut session, 210);
        let mid_count = capture(&session).particle_count();
        advance_steps(&mut session, 30);
        let end_count = capture(&session).particle_count();

        // Assert
        assert!(end_count <= 320);
        assert!(
            end_count <= mid_count + 2,
            "count should plateau instead of climbing toward 512: {mid_count} -> {end_count}"
        );
    }

    #[test]
    fn emission_rate_off_does_not_increase_count() {
        // Arrange
        let mut session = SessionCore::create(SceneId::Fountain)
            .expect("Fountain should construct a bounded native stream");
        let off = session
            .apply_control("emission-rate", "off")
            .expect("emission-rate=off should apply live");
        let before = capture(&session).particle_count();

        // Act
        advance_steps(&mut session, 60);
        let after = capture(&session).particle_count();

        // Assert
        assert!(!off, "emission-rate must return Live");
        assert_eq!(after, before, "off emission must not add particles");
    }

    #[test]
    fn fountain_controls_apply_live() {
        // Arrange
        let super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = super::build(&[]).expect("Fountain should construct");

        // Act
        let rate = hooks
            .apply_control(&mut world, particle_system, "emission-rate", "high")
            .expect("emission-rate should apply");
        let speed = hooks
            .apply_control(&mut world, particle_system, "launch-speed", "fast")
            .expect("launch-speed should apply");
        let aim = hooks
            .apply_control(&mut world, particle_system, "aim-angle", "left")
            .expect("aim-angle should apply");

        // Assert
        assert!(matches!(rate, ControlEffect::Live));
        assert!(matches!(speed, ControlEffect::Live));
        assert!(matches!(aim, ControlEffect::Live));
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
                .expect("Fountain should capture a frame"),
        )
    }
}
