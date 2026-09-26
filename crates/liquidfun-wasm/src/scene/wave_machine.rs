//! Pinned `LiquidFun` Wave Machine: motorized four-wall tank rocking from sim time.
//!
//! ANTI-PATTERN: Do not copy Water Wheel's motor-off revolute / empty motor writes.
//! At 1×, motor speed is `0.05 * cos(t) * π`, matching testWaveMachine.js.
//! The `wave-speed` control starts at 1. Higher multipliers raise the rocking
//! frequency and the speed setpoint together, so the tank keeps that tilt.

use std::f32::consts::PI;

use liquidfun::collision::{FilterData, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, JointDef, JointId, ParticleSystemDef, ParticleSystemId,
    RevoluteJointDef, World,
};

use super::{BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.025;
const PARTICLE_DAMPING: f32 = 0.2;
const MAXIMUM_PARTICLE_COUNT: usize = 10240;
const GROUP_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);

const TANK_POSITION: Vec2 = Vec2::new(0.0, 1.0);
const WALL_DENSITY: f32 = 5.0;
const MOTOR_SPEED_SCALE: f32 = 0.05;
const WAVE_SPEED_CONTROL: &str = "wave-speed";
/// Slider default. `1` is the pinned Wave Machine motor.
const DEFAULT_WAVE_SPEED: f32 = 1.0;
/// Slider tenths. 100 is ten times the original rocking frequency.
const MAX_WAVE_SPEED_TENTHS: u16 = 100;
const MAX_MOTOR_TORQUE: f32 = 1.0e7;
const SIM_DT: f32 = 1.0 / 60.0;
const FILL_HALF: f32 = 0.9;

/// Wall half-extents and local centers matching testWaveMachine.js SetAsBoxXYCenterAngle.
const WALLS: [(f32, f32, Vec2); 4] = [
    (0.05, 1.0, Vec2::new(2.0, 0.0)),
    (0.05, 1.0, Vec2::new(-2.0, 0.0)),
    (2.0, 0.05, Vec2::new(0.0, 1.0)),
    (2.0, 0.05, Vec2::new(0.0, -1.0)),
];

struct WaveMachineHooks {
    tank: BodyId,
    joint: JointId,
    wall_local_corners: [[Vec2; 4]; 4],
    time: f32,
    speed_multiplier: f32,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_wave_machine().map_err(|_error| SessionError::SceneConstruction)
}

fn build_wave_machine() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;

    let tank_definition = BodyDef::new(BodyType::Dynamic, TANK_POSITION, 0.0, true)
        .map_err(|_error| SceneError::Body)?
        .with_sleeping_allowed(false);
    let tank = world
        .create_body(&tank_definition)
        .map_err(|_error| SceneError::Body)?;

    let mut wall_local_corners = [[Vec2::ZERO; 4]; 4];
    for (index, &(half_width, half_height, center)) in WALLS.iter().enumerate() {
        wall_local_corners[index] = wall_corners(half_width, half_height, center);
        attach_wall_fixture(&mut world, tank, half_width, half_height, center)?;
    }

    let joint = pin_tank(
        &mut world,
        ground,
        tank,
        scaled_motor_speed(0.0, DEFAULT_WAVE_SPEED),
    )?;
    let particle_system = create_particle_fill(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(WaveMachineHooks {
            tank,
            joint,
            wall_local_corners,
            time: 0.0,
            speed_multiplier: DEFAULT_WAVE_SPEED,
        }),
    })
}

fn wall_corners(half_width: f32, half_height: f32, center: Vec2) -> [Vec2; 4] {
    [
        center + Vec2::new(-half_width, -half_height),
        center + Vec2::new(half_width, -half_height),
        center + Vec2::new(half_width, half_height),
        center + Vec2::new(-half_width, half_height),
    ]
}

fn attach_wall_fixture(
    world: &mut World,
    body: BodyId,
    half_width: f32,
    half_height: f32,
    center: Vec2,
) -> Result<(), SceneError> {
    let polygon = PolygonShape::oriented_box(half_width, half_height, center, 0.0)
        .map_err(|_error| SceneError::Geometry)?;
    let definition = FixtureDef::new(
        Shape::from(polygon),
        WALL_DENSITY,
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

fn pin_tank(
    world: &mut World,
    ground: BodyId,
    tank: BodyId,
    initial_speed: f32,
) -> Result<JointId, SceneError> {
    let joint = RevoluteJointDef::new(ground, tank)
        .map_err(|_error| SceneError::Body)?
        .with_frame(TANK_POSITION, Vec2::ZERO, 0.0)
        .map_err(|_error| SceneError::Body)?
        .with_motor(true, initial_speed, MAX_MOTOR_TORQUE)
        .map_err(|_error| SceneError::Body)?;
    world
        .create_joint(JointDef::from(joint))
        .map_err(|_error| SceneError::Body)
}

/// Accepts one-decimal tokens from `0.0` through `10.0`, matching the HUD slider.
fn parse_wave_speed(value: &str) -> Option<f32> {
    let (whole, fraction) = value.split_once('.')?;
    if fraction.len() != 1 || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if whole.is_empty() || !whole.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if whole.len() > 1 && whole.starts_with('0') {
        return None;
    }

    let whole_value = whole.parse::<u16>().ok()?;
    let tenth_byte = fraction.as_bytes().first().copied()?;
    let tenth = u16::from(tenth_byte.checked_sub(b'0')?);
    let tenths = whole_value.checked_mul(10)?.checked_add(tenth)?;
    if tenths > MAX_WAVE_SPEED_TENTHS {
        return None;
    }

    Some(f32::from(tenths) / 10.0)
}

/// Motor speed whose integral keeps the pinned tilt.
///
/// `testWaveMachine.js` commands `0.05 * cos(t) * π`. A strong motor tracks
/// that, so the tank angle is about `0.05 * π * sin(t)`. Multiplying only the
/// speed would multiply that tilt. Scaling time and speed together keeps the
/// peak: `θ ≈ 0.05 * π * sin(multiplier * t)`.
fn scaled_motor_speed(time: f32, speed_multiplier: f32) -> f32 {
    MOTOR_SPEED_SCALE * speed_multiplier * (speed_multiplier * time).cos() * PI
}

fn write_motor_speed(
    world: &mut World,
    joint: JointId,
    time: f32,
    speed_multiplier: f32,
) -> Result<(), SessionError> {
    world
        .set_revolute_motor_speed(joint, scaled_motor_speed(time, speed_multiplier))
        .map_err(|_error| SessionError::StepFailed)
}

fn create_particle_fill(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_damping(PARTICLE_DAMPING)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    let filled = Shape::from(
        PolygonShape::oriented_box(FILL_HALF, FILL_HALF, TANK_POSITION, 0.0)
            .map_err(|_error| SceneError::Geometry)?,
    );
    let source =
        ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_color(GROUP_COLOR)
        .with_transform(Transform::IDENTITY)
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(system)
}

impl SceneHooks for WaveMachineHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        // Match testWaveMachine.js Step: t += 1/60, then set the motor.
        // At 1× that speed is `0.05 * cos(t) * π`.
        self.time += SIM_DT;
        write_motor_speed(world, self.joint, self.time, self.speed_multiplier)
    }

    fn apply_control(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
        name: &str,
        value: &str,
    ) -> Result<ControlEffect, SessionError> {
        if name != WAVE_SPEED_CONTROL {
            return Err(SessionError::UnknownControl);
        }
        let Some(speed_multiplier) = parse_wave_speed(value) else {
            return Err(SessionError::UnknownControl);
        };

        self.speed_multiplier = speed_multiplier;
        write_motor_speed(world, self.joint, self.time, speed_multiplier)?;
        Ok(ControlEffect::Live)
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

    fn collect_segments(&self, world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        let transform = world
            .body_snapshot(self.tank)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .transform();
        let mut segments = Vec::with_capacity(16);
        for corners in &self.wall_local_corners {
            let world_corners: [Vec2; 4] = [
                transform.apply(corners[0]),
                transform.apply(corners[1]),
                transform.apply(corners[2]),
                transform.apply(corners[3]),
            ];
            for index in 0..4 {
                segments.push(RigidSegment {
                    start: world_corners[index],
                    end: world_corners[(index + 1) % 4],
                });
            }
        }
        Ok(segments)
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests;
