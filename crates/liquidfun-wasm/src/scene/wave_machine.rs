//! Pinned `LiquidFun` Wave Machine: motorized four-wall tank rocking from sim time.
//!
//! ANTI-PATTERN: Do not copy Water Wheel's motor-off revolute / empty motor writes.
//! Motor speed is written only from `on_advance` using `0.05 * cos(t) * π`.

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

use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.025;
const PARTICLE_DAMPING: f32 = 0.2;
const MAXIMUM_PARTICLE_COUNT: usize = 10240;
const GROUP_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);

const TANK_POSITION: Vec2 = Vec2::new(0.0, 1.0);
const WALL_DENSITY: f32 = 5.0;
const MOTOR_SPEED_SCALE: f32 = 0.05;
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

    let joint = pin_tank(&mut world, ground, tank)?;
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

fn pin_tank(world: &mut World, ground: BodyId, tank: BodyId) -> Result<JointId, SceneError> {
    let initial_speed = MOTOR_SPEED_SCALE * PI;
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
        // Match testWaveMachine.js Step: t += 1/60 then SetMotorSpeed(0.05 * cos(t) * π).
        self.time += SIM_DT;
        let speed = MOTOR_SPEED_SCALE * self.time.cos() * PI;
        world
            .set_revolute_motor_speed(self.joint, speed)
            .map_err(|_error| SessionError::StepFailed)?;
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
mod tests {
    use std::f32::consts::PI;

    use liquidfun::{JointDef, World, WorldObservationLimits};

    use super::super::SceneId;
    use super::build;
    use crate::session::{SessionCore, SessionError};

    #[test]
    fn create_builds_wave_machine_with_particles() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::WaveMachine)
            .expect("Wave Machine should construct the pinned motorized tank");

        // Assert
        assert!(
            session.particle_count() > 0,
            "fill box must create at least one particle"
        );
    }

    #[test]
    fn motor_speed_follows_sim_time_after_advances() {
        // Arrange — JS Step order: t += 1/60 then set_revolute_motor_speed(0.05 * cos(t) * π)
        // Water Wheel motor-off is NOT the template.
        let super::super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = build(&[]).expect("Wave Machine should construct the pinned motorized tank");
        let advances = 7_u32;

        // Act
        for _ in 0..advances {
            hooks
                .on_advance(&mut world, particle_system)
                .expect("on_advance must update motor from sim time");
        }
        let speed = revolute_motor_speed(&world);
        let expected = 0.05 * (advances as f32 / 60.0).cos() * PI;

        // Assert
        assert!(
            (speed - expected).abs() < 1.0e-5,
            "motor speed must track sim time (got {speed}, expected {expected}); \
             set_revolute_motor_speed writes 0.05 * cos(t) * PI"
        );
    }

    #[test]
    fn first_advance_changes_motor_speed_from_initial() {
        // Arrange
        let super::super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = build(&[]).expect("Wave Machine should construct");
        let initial = revolute_motor_speed(&world);

        // Act
        hooks
            .on_advance(&mut world, particle_system)
            .expect("first advance must update motor from sim time");
        let after = revolute_motor_speed(&world);
        let expected = 0.05 * (1.0_f32 / 60.0).cos() * PI;

        // Assert — not stalled at create speed forever; matches increment-then-set
        assert!(
            (initial - 0.05 * PI).abs() < 1.0e-5,
            "initial motor speed must be 0.05 * PI before any advance (got {initial})"
        );
        assert!(
            (after - expected).abs() < 1.0e-5,
            "after one advance speed must be 0.05 * cos(1/60) * PI (got {after})"
        );
        assert!(
            (after - initial).abs() > 1.0e-6,
            "motor speed must change from initial after the first advance"
        );
    }

    #[test]
    fn unknown_control_is_rejected_and_pointer_is_noop() {
        // Arrange
        let mut session = SessionCore::create(SceneId::WaveMachine)
            .expect("Wave Machine should construct the pinned motorized tank");
        let before = session.particle_count();

        // Act
        let control = session.apply_control("stiffness", "high");
        let action = session.apply_action("refill");
        let pointer = session.apply_pointer("up", 0.0, 1.0);

        // Assert
        assert_eq!(control, Err(SessionError::UnknownControl));
        assert_eq!(action, Err(SessionError::UnknownControl));
        assert_eq!(pointer, Ok(()));
        assert_eq!(session.particle_count(), before);
    }

    #[test]
    fn motor_on_pattern_is_not_water_wheel_motor_off() {
        // Arrange
        let source = include_str!("wave_machine.rs");
        let impl_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("implementation precedes tests");

        // Assert — production path must use live motor writes (Task 2)
        assert!(
            impl_source.contains("set_revolute_motor_speed"),
            "on_advance must call set_revolute_motor_speed"
        );
        assert!(
            impl_source.contains("0.05"),
            "motor formula constant 0.05 must be present"
        );
        assert!(
            !impl_source.contains("enable_motor: false")
                && !impl_source.contains("with_motor(false"),
            "Wave Machine must not copy Water Wheel motor-off"
        );
    }

    fn revolute_motor_speed(world: &World) -> f32 {
        let observation = world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the motorized revolute");
        let revolute: Vec<_> = observation
            .joints()
            .iter()
            .filter_map(|joint| match joint.snapshot().definition() {
                JointDef::Revolute(definition) => Some(definition),
                _ => None,
            })
            .collect();
        assert_eq!(revolute.len(), 1, "exactly one revolute joint");
        assert!(
            revolute[0].is_motor_enabled(),
            "Wave Machine revolute must enable_motor (not Water Wheel motor-off)"
        );
        revolute[0].motor_speed()
    }
}
