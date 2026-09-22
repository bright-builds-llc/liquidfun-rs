//! Native Water Wheel: a motor-off revolute hub whose paddles follow the live pose.

use liquidfun::collision::{CircleShape, FilterData, PolygonShape, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, JointDef, ParticleColor, ParticleDef, ParticleFlags,
    ParticleSystemDef, ParticleSystemId, RevoluteJointDef, World,
};

use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
    attach_basin_fixture,
};
use crate::session::SessionError;

const HUB_POSITION: Vec2 = Vec2::new(0.0, 3.0);
const HUB_RADIUS: f32 = 0.35;
const PADDLE_INNER: f32 = 0.35;
const PADDLE_OUTER: f32 = 1.7;
const PADDLE_HALF_WIDTH: f32 = 0.14;
const WHEEL_DENSITY: f32 = 0.45;
const PARTICLE_RADIUS: f32 = 0.050_596;
const MAXIMUM_PARTICLE_COUNT: usize = 3200;
const PARTICLE_LIFETIME: f32 = 3.0;
const PARTICLE_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const SEED_POSITION: Vec2 = Vec2::new(-5.0, 0.3);
const JET_POSITION: Vec2 = Vec2::new(-3.9, 3.15);
const WEAK_JET_SPEED: f32 = 4.0;
const MEDIUM_JET_SPEED: f32 = 8.0;
const STRONG_JET_SPEED: f32 = 12.0;
const EMIT_PER_STEP: u8 = 20;
const EMIT_SPACING: f32 = 0.044_272;
const PADDLE_LOCAL_SEGMENTS: [[Vec2; 2]; 4] = [
    [Vec2::new(PADDLE_INNER, 0.0), Vec2::new(PADDLE_OUTER, 0.0)],
    [Vec2::new(0.0, PADDLE_INNER), Vec2::new(0.0, PADDLE_OUTER)],
    [Vec2::new(-PADDLE_INNER, 0.0), Vec2::new(-PADDLE_OUTER, 0.0)],
    [Vec2::new(0.0, -PADDLE_INNER), Vec2::new(0.0, -PADDLE_OUTER)],
];

#[derive(Debug, Clone, Copy, PartialEq)]
enum JetStrength {
    Weak,
    Medium,
    Strong,
}

impl JetStrength {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "weak" => Some(Self::Weak),
            "medium" => Some(Self::Medium),
            "strong" => Some(Self::Strong),
            _ => None,
        }
    }

    fn speed(self) -> f32 {
        match self {
            Self::Weak => WEAK_JET_SPEED,
            Self::Medium => MEDIUM_JET_SPEED,
            Self::Strong => STRONG_JET_SPEED,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Emission {
    On,
    Off,
}

impl Emission {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "on" => Some(Self::On),
            "off" => Some(Self::Off),
            _ => None,
        }
    }

    fn is_on(self) -> bool {
        matches!(self, Self::On)
    }
}

struct WaterWheelHooks {
    trough_segments: [RigidSegment; 3],
    wheel: BodyId,
    hub_radius: f32,
    jet_speed: f32,
    emission: Emission,
    maybe_aim_velocity: Option<Vec2>,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let _ = presets;
    build_wheel().map_err(|_error| SessionError::SceneConstruction)
}

fn build_wheel() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(Vec2::new(0.0, -10.0))
        .map_err(|_error| SceneError::Gravity)?;

    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    attach_basin_fixture(
        &mut world,
        ground,
        &[
            Vec2::new(-6.0, -1.0),
            Vec2::new(6.0, -1.0),
            Vec2::new(6.0, 0.0),
            Vec2::new(-6.0, 0.0),
        ],
    )?;
    attach_basin_fixture(
        &mut world,
        ground,
        &[
            Vec2::new(-6.0, 0.0),
            Vec2::new(-5.5, 0.0),
            Vec2::new(-5.5, 8.0),
            Vec2::new(-6.0, 8.0),
        ],
    )?;
    attach_basin_fixture(
        &mut world,
        ground,
        &[
            Vec2::new(5.5, 0.0),
            Vec2::new(6.0, 0.0),
            Vec2::new(6.0, 8.0),
            Vec2::new(5.5, 8.0),
        ],
    )?;

    let wheel = create_wheel(&mut world)?;
    pin_wheel(&mut world, ground, wheel)?;
    let particle_system = create_seed_system(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(WaterWheelHooks {
            trough_segments: [
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
            wheel,
            hub_radius: HUB_RADIUS,
            jet_speed: MEDIUM_JET_SPEED,
            emission: Emission::On,
            maybe_aim_velocity: None,
        }),
    })
}

fn create_wheel(world: &mut World) -> Result<BodyId, SceneError> {
    let body_definition = BodyDef::new(BodyType::Dynamic, HUB_POSITION, 0.0, true)
        .map_err(|_error| SceneError::Body)?
        .with_angular_damping(0.05)
        .map_err(|_error| SceneError::Body)?;
    let wheel = world
        .create_body(&body_definition)
        .map_err(|_error| SceneError::Body)?;
    attach_circle_fixture(world, wheel, Vec2::ZERO, HUB_RADIUS)?;
    attach_paddle_fixture(
        world,
        wheel,
        &[
            Vec2::new(PADDLE_INNER, -PADDLE_HALF_WIDTH),
            Vec2::new(PADDLE_OUTER, -PADDLE_HALF_WIDTH),
            Vec2::new(PADDLE_OUTER, PADDLE_HALF_WIDTH),
            Vec2::new(PADDLE_INNER, PADDLE_HALF_WIDTH),
        ],
    )?;
    attach_paddle_fixture(
        world,
        wheel,
        &[
            Vec2::new(-PADDLE_HALF_WIDTH, PADDLE_INNER),
            Vec2::new(PADDLE_HALF_WIDTH, PADDLE_INNER),
            Vec2::new(PADDLE_HALF_WIDTH, PADDLE_OUTER),
            Vec2::new(-PADDLE_HALF_WIDTH, PADDLE_OUTER),
        ],
    )?;
    attach_paddle_fixture(
        world,
        wheel,
        &[
            Vec2::new(-PADDLE_OUTER, -PADDLE_HALF_WIDTH),
            Vec2::new(-PADDLE_INNER, -PADDLE_HALF_WIDTH),
            Vec2::new(-PADDLE_INNER, PADDLE_HALF_WIDTH),
            Vec2::new(-PADDLE_OUTER, PADDLE_HALF_WIDTH),
        ],
    )?;
    attach_paddle_fixture(
        world,
        wheel,
        &[
            Vec2::new(-PADDLE_HALF_WIDTH, -PADDLE_OUTER),
            Vec2::new(PADDLE_HALF_WIDTH, -PADDLE_OUTER),
            Vec2::new(PADDLE_HALF_WIDTH, -PADDLE_INNER),
            Vec2::new(-PADDLE_HALF_WIDTH, -PADDLE_INNER),
        ],
    )?;
    Ok(wheel)
}

fn attach_circle_fixture(
    world: &mut World,
    body: BodyId,
    center: Vec2,
    radius: f32,
) -> Result<(), SceneError> {
    let circle = CircleShape::new(center, radius).map_err(|_error| SceneError::Geometry)?;
    let definition = FixtureDef::new(
        Shape::from(circle),
        WHEEL_DENSITY,
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

fn attach_paddle_fixture(
    world: &mut World,
    body: BodyId,
    vertices: &[Vec2; 4],
) -> Result<(), SceneError> {
    let polygon = PolygonShape::new(vertices).map_err(|_error| SceneError::Geometry)?;
    let definition = FixtureDef::new(
        Shape::from(polygon),
        WHEEL_DENSITY,
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

fn pin_wheel(world: &mut World, ground: BodyId, wheel: BodyId) -> Result<(), SceneError> {
    let joint = RevoluteJointDef::new(ground, wheel)
        .map_err(|_error| SceneError::Body)?
        .with_frame(HUB_POSITION, Vec2::ZERO, 0.0)
        .map_err(|_error| SceneError::Body)?;
    world
        .create_joint(JointDef::from(joint))
        .map_err(|_error| SceneError::Body)?;
    Ok(())
}

fn create_seed_system(world: &mut World) -> Result<ParticleSystemId, SceneError> {
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
        .with_position(SEED_POSITION)
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

fn emit_jet(world: &mut World, system: ParticleSystemId, velocity: Vec2) {
    if world
        .reserve_particle_creations(system, usize::from(EMIT_PER_STEP))
        .is_err()
    {
        return;
    }
    for index in 0..EMIT_PER_STEP {
        let position = Vec2::new(
            JET_POSITION.x,
            JET_POSITION.y + f32::from(index) * EMIT_SPACING,
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

impl SceneHooks for WaterWheelHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        if !self.emission.is_on() {
            return Ok(());
        }
        let speed = self.jet_speed;
        let velocity = self.maybe_aim_velocity.unwrap_or(Vec2::new(speed, -0.4));
        emit_jet(world, system, velocity);
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
            "jet-strength" => {
                let Some(strength) = JetStrength::parse(value) else {
                    return Err(SessionError::UnknownControl);
                };
                self.jet_speed = strength.speed();
                Ok(ControlEffect::Live)
            }
            "emission" => {
                let Some(emission) = Emission::parse(value) else {
                    return Err(SessionError::UnknownControl);
                };
                self.emission = emission;
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
                let delta = Vec2::new(world_x + 3.9, world_y - 3.15);
                let length = delta.length();
                if length.is_finite() && length > 1e-4 {
                    self.maybe_aim_velocity = Some(delta * (self.jet_speed / length));
                }
                Ok(())
            }
            PointerKind::Up => Ok(()),
            PointerKind::Cancel => {
                self.maybe_aim_velocity = None;
                Ok(())
            }
        }
    }

    fn collect_segments(&self, world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        let pose = world
            .body_snapshot(self.wheel)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .transform();
        let mut segments = self.trough_segments.to_vec();
        for [start, end] in PADDLE_LOCAL_SEGMENTS {
            segments.push(RigidSegment {
                start: pose.apply(start),
                end: pose.apply(end),
            });
        }
        Ok(segments)
    }

    fn collect_circles(&self, world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        let position = world
            .body_snapshot(self.wheel)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .position();
        Ok(vec![(position, self.hub_radius)])
    }
}

#[cfg(test)]
mod tests;
