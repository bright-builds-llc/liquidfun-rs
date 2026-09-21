//! Named Dam Break scene using the documented Medium / Normal defaults.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, ParticleColor, ParticleDef, ParticleFlags,
    ParticleSystemDef, ParticleSystemId, WakePolicy, World,
};

use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
    attach_basin_fixture,
};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.063_245_55;
const PARTICLE_COLUMNS: u8 = 48;
const PARTICLE_ROWS: u8 = 40;
/// Medium water amount remains the documented 48 * 40 basin.
const PARTICLE_COUNT: usize = 48 * 40;
const SMALL_PARTICLE_COLUMNS: u8 = 25;
const SMALL_PARTICLE_ROWS: u8 = 26;
const LARGE_PARTICLE_COLUMNS: u8 = 63;
const LARGE_PARTICLE_ROWS: u8 = 44;
const PARTICLE_SPACING: f32 = 0.101_193;
const PARTICLE_ORIGIN: Vec2 = Vec2::new(-4.7, 0.4);
const PARTICLE_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const DYNAMIC_CIRCLE_RADIUS: f32 = 0.75;
const DYNAMIC_CIRCLE_POSITION: Vec2 = Vec2::new(2.5, 5.5);
const DROP_CIRCLE_POSITION: Vec2 = Vec2::new(2.5, 7.2);
const DROP_WAKE_IMPULSE: Vec2 = Vec2::new(0.0, -0.1);
const MAXIMUM_PARTICLE_COUNT: usize = 10240;
const LOW_GRAVITY: Vec2 = Vec2::new(0.0, -6.0);
const NORMAL_GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const HIGH_GRAVITY: Vec2 = Vec2::new(0.0, -16.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WaterAmount {
    Small,
    Medium,
    Large,
}

impl WaterAmount {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "small" => Some(Self::Small),
            "medium" => Some(Self::Medium),
            "large" => Some(Self::Large),
            _ => None,
        }
    }

    fn grid(self) -> (u8, u8) {
        match self {
            Self::Small => (SMALL_PARTICLE_COLUMNS, SMALL_PARTICLE_ROWS),
            Self::Medium => (PARTICLE_COLUMNS, PARTICLE_ROWS),
            Self::Large => (LARGE_PARTICLE_COLUMNS, LARGE_PARTICLE_ROWS),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum GravityPreset {
    Low,
    Normal,
    High,
}

impl GravityPreset {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "low" => Some(Self::Low),
            "normal" => Some(Self::Normal),
            "high" => Some(Self::High),
            _ => None,
        }
    }

    fn vector(self) -> Vec2 {
        match self {
            Self::Low => LOW_GRAVITY,
            Self::Normal => NORMAL_GRAVITY,
            Self::High => HIGH_GRAVITY,
        }
    }
}

struct DamBreakHooks {
    basin_segments: [RigidSegment; 3],
    circle_body: BodyId,
    circle_radius: f32,
    dragging: bool,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let water = preset_value(presets, "water-amount")
        .map(WaterAmount::parse)
        .map_or(Ok(WaterAmount::Medium), |maybe_water| {
            maybe_water.ok_or(SessionError::UnknownControl)
        })?;
    let gravity = preset_value(presets, "gravity")
        .map(GravityPreset::parse)
        .map_or(Ok(GravityPreset::Normal), |maybe_gravity| {
            maybe_gravity.ok_or(SessionError::UnknownControl)
        })?;
    debug_assert_eq!(
        PARTICLE_COUNT,
        usize::from(PARTICLE_COLUMNS) * usize::from(PARTICLE_ROWS)
    );
    build_basin(water, gravity).map_err(|_error| SessionError::SceneConstruction)
}

fn preset_value<'a>(presets: &'a [(String, String)], name: &str) -> Option<&'a str> {
    presets
        .iter()
        .rev()
        .find(|(key, _value)| key == name)
        .map(|(_key, value)| value.as_str())
}

fn build_basin(water: WaterAmount, gravity: GravityPreset) -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(gravity.vector())
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

    let circle_body = create_dynamic_circle(&mut world)?;
    let particle_system = create_particles(&mut world, water)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(DamBreakHooks {
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
            circle_body,
            circle_radius: DYNAMIC_CIRCLE_RADIUS,
            dragging: false,
        }),
    })
}

fn create_dynamic_circle(world: &mut World) -> Result<BodyId, SceneError> {
    let body_definition = BodyDef::new(BodyType::Dynamic, DYNAMIC_CIRCLE_POSITION, 0.0, true)
        .map_err(|_error| SceneError::Body)?;
    let body = world
        .create_body(&body_definition)
        .map_err(|_error| SceneError::Body)?;
    let circle = CircleShape::new(Vec2::ZERO, DYNAMIC_CIRCLE_RADIUS)
        .map_err(|_error| SceneError::Geometry)?;
    let fixture_definition = FixtureDef::new(
        Shape::from(circle),
        1.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(body, &fixture_definition)
        .map_err(|_error| SceneError::Fixture)?;
    Ok(body)
}

fn create_particles(world: &mut World, water: WaterAmount) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let (columns, rows) = water.grid();

    for row in 0..rows {
        for column in 0..columns {
            let position = Vec2::new(
                PARTICLE_ORIGIN.x + f32::from(column) * PARTICLE_SPACING,
                PARTICLE_ORIGIN.y + f32::from(row) * PARTICLE_SPACING,
            );
            let definition = ParticleDef::default()
                .with_flags(ParticleFlags::WATER)
                .with_color(PARTICLE_COLOR)
                .with_position(position)
                .map_err(|_error| SceneError::Particle)?;
            let receipt = world
                .create_particle_with_def(system, None, &definition)
                .map_err(|_error| SceneError::Particle)?;
            if !receipt.destruction_occurrences().is_empty() {
                return Err(SceneError::Particle);
            }
        }
    }

    Ok(system)
}

fn drop_obstacle(world: &mut World, circle_body: BodyId) -> Result<(), SessionError> {
    world
        .set_body_transform(circle_body, DROP_CIRCLE_POSITION, 0.0)
        .map_err(|_error| SessionError::SceneConstruction)?;
    world
        .apply_body_linear_impulse_to_center(circle_body, DROP_WAKE_IMPULSE, WakePolicy::Wake)
        .map_err(|_error| SessionError::SceneConstruction)?;
    Ok(())
}

fn reset_obstacle(world: &mut World, circle_body: BodyId) -> Result<(), SessionError> {
    world
        .set_body_transform(circle_body, DYNAMIC_CIRCLE_POSITION, 0.0)
        .map_err(|_error| SessionError::SceneConstruction)?;
    Ok(())
}

impl SceneHooks for DamBreakHooks {
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
            "water-amount" => {
                let Some(_water) = WaterAmount::parse(value) else {
                    return Err(SessionError::UnknownControl);
                };
                Ok(ControlEffect::Recreated)
            }
            "gravity" => {
                let Some(_gravity) = GravityPreset::parse(value) else {
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
        _system: ParticleSystemId,
        name: &str,
    ) -> Result<(), SessionError> {
        match name {
            "drop-obstacle" => drop_obstacle(world, self.circle_body),
            "reset-obstacle" => reset_obstacle(world, self.circle_body),
            _ => Err(SessionError::UnknownControl),
        }
    }

    fn apply_pointer(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
        kind: PointerKind,
        world_x: f32,
        world_y: f32,
    ) -> Result<(), SessionError> {
        match kind {
            PointerKind::Down | PointerKind::Move => {
                self.dragging = true;
                let position = Vec2::new(world_x.clamp(-5.5, 5.5), world_y.clamp(0.75, 7.25));
                world
                    .set_body_transform(self.circle_body, position, 0.0)
                    .map_err(|_error| SessionError::SceneConstruction)?;
                Ok(())
            }
            PointerKind::Up => {
                self.dragging = false;
                world
                    .apply_body_linear_impulse_to_center(
                        self.circle_body,
                        DROP_WAKE_IMPULSE,
                        WakePolicy::Wake,
                    )
                    .map_err(|_error| SessionError::SceneConstruction)?;
                Ok(())
            }
            PointerKind::Cancel => {
                self.dragging = false;
                Ok(())
            }
        }
    }

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        Ok(self.basin_segments.to_vec())
    }

    fn collect_circles(&self, world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        let position = world
            .body_snapshot(self.circle_body)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .position();
        Ok(vec![(position, self.circle_radius)])
    }
}

#[cfg(test)]
mod tests;
