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
const PARTICLE_RADIUS: f32 = 0.16;
const MAXIMUM_PARTICLE_COUNT: usize = 320;
const PARTICLE_LIFETIME: f32 = 3.0;
const PARTICLE_COLOR: ParticleColor = ParticleColor::new(57, 211, 199, 255);
const SEED_POSITION: Vec2 = Vec2::new(-5.0, 0.3);
const JET_POSITION: Vec2 = Vec2::new(-3.9, 3.15);
const WEAK_JET_SPEED: f32 = 4.0;
const MEDIUM_JET_SPEED: f32 = 8.0;
const STRONG_JET_SPEED: f32 = 12.0;
const EMIT_PER_STEP: u8 = 2;
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

fn emit_jet(world: &mut World, system: ParticleSystemId, speed: f32) {
    for index in 0..EMIT_PER_STEP {
        let position = Vec2::new(JET_POSITION.x, JET_POSITION.y + f32::from(index) * 0.14);
        let Ok(definition) = ParticleDef::default()
            .with_flags(ParticleFlags::WATER)
            .with_color(PARTICLE_COLOR)
            .with_position(position)
            .and_then(|definition| definition.with_velocity(Vec2::new(speed, -0.4)))
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
        emit_jet(world, system, self.jet_speed);
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
        _world_x: f32,
        _world_y: f32,
    ) -> Result<(), SessionError> {
        match kind {
            PointerKind::Down | PointerKind::Move | PointerKind::Up | PointerKind::Cancel => Ok(()),
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
mod tests {
    use std::f32::consts::TAU;

    use liquidfun::math::Vec2;
    use liquidfun::{BodyType, JointDef, WorldObservationLimits};

    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::SessionCore;

    #[test]
    fn create_water_wheel_builds_hub_circle_and_paddle_segments() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::WaterWheel)
            .expect("Water Wheel should construct a native pinned wheel");
        let frame = capture(&session);

        // Assert
        assert!(
            frame.rigid_circles().len() >= 3,
            "hub circle reports at least x, y, radius"
        );
        assert!(
            frame.rigid_segments().len() >= 16,
            "at least four paddle segments report start/end floats"
        );
        assert!(session.rigid_shape_count() >= 5);
    }

    #[test]
    fn revolute_joint_is_created_without_a_motor() {
        // Arrange / Act
        let super::BuiltScene { world, .. } =
            super::build(&[]).expect("Water Wheel should construct");
        let observation = world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the revolute hub");
        let revolute: Vec<_> = observation
            .joints()
            .iter()
            .filter_map(|joint| match joint.snapshot().definition() {
                JointDef::Revolute(definition) => Some(definition),
                _ => None,
            })
            .collect();

        // Assert
        assert_eq!(revolute.len(), 1, "exactly one revolute pin should exist");
        assert!(
            !revolute[0].is_motor_enabled(),
            "D-09 forbids a motor-driven water-wheel hub"
        );
        assert_eq!(revolute[0].motor_speed().to_bits(), 0.0_f32.to_bits());
        assert_eq!(revolute[0].max_motor_torque().to_bits(), 0.0_f32.to_bits());
    }

    #[test]
    fn collect_segments_follow_a_forced_wheel_transform() {
        // Arrange
        let super::BuiltScene {
            mut world, hooks, ..
        } = super::build(&[]).expect("Water Wheel should construct");
        let before = flatten_segments(
            &hooks
                .collect_segments(&world)
                .expect("paddles should capture from the live pose"),
        );
        let wheel = world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the wheel body")
            .bodies()
            .iter()
            .find(|body| body.snapshot().body_type() == BodyType::Dynamic)
            .expect("the paddle wheel is the dynamic body")
            .id();

        // Act
        world
            .set_body_transform(wheel, Vec2::new(0.0, 3.0), TAU / 8.0)
            .expect("test-only pose change proves capture uses the live transform");
        let after = flatten_segments(
            &hooks
                .collect_segments(&world)
                .expect("rotated paddles should still capture"),
        );

        // Assert
        assert!(after.len() >= 16);
        assert_ne!(
            before, after,
            "paddle segments must come from BodySnapshot::transform().apply, not constants"
        );
    }

    #[test]
    fn medium_jet_with_emission_on_turns_the_wheel() {
        // Arrange
        let mut session = SessionCore::create(SceneId::WaterWheel)
            .expect("Water Wheel should construct a native pinned wheel");
        let recreated_strength = session
            .apply_control("jet-strength", "medium")
            .expect("medium jet should apply live");
        let recreated_emission = session
            .apply_control("emission", "on")
            .expect("emission on should apply live");
        let before = captured_paddle_angle(&session);

        // Act
        advance_steps(&mut session, 180);
        let after = captured_paddle_angle(&session);

        // Assert
        assert!(!recreated_strength);
        assert!(!recreated_emission);
        assert!(
            (after - before).abs() > 0.05,
            "native coupling should rotate the wheel; before={before} after={after}"
        );
    }

    #[test]
    fn emission_off_leaves_the_wheel_angle_nearly_unchanged() {
        // Arrange
        let mut session = SessionCore::create(SceneId::WaterWheel)
            .expect("Water Wheel should construct a native pinned wheel");
        session
            .apply_control("emission", "off")
            .expect("emission off should apply live");
        let before = captured_paddle_angle(&session);

        // Act
        advance_steps(&mut session, 180);
        let after = captured_paddle_angle(&session);

        // Assert
        assert!(
            (after - before).abs() < 0.01,
            "motor-off wheel must not spin without the jet; before={before} after={after}"
        );
    }

    #[test]
    fn two_hundred_forty_on_steps_plateau_at_or_below_the_particle_cap() {
        // Arrange
        let mut session = SessionCore::create(SceneId::WaterWheel)
            .expect("Water Wheel should construct a native pinned wheel");
        session
            .apply_control("jet-strength", "medium")
            .expect("medium jet should apply live");
        session
            .apply_control("emission", "on")
            .expect("emission on should apply live");

        // Act
        advance_steps(&mut session, 210);
        let mid_count = capture(&session).particle_count();
        advance_steps(&mut session, 30);
        let end_count = capture(&session).particle_count();

        // Assert
        assert!(end_count <= 320);
        assert!(
            end_count <= mid_count,
            "count should not keep climbing over the last 30 steps: {mid_count} -> {end_count}"
        );
    }

    #[test]
    fn pointer_aim_emits_positive_x_and_cancel_restores_default() {
        // Arrange
        let super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = super::build(&[]).expect("Water Wheel should construct");

        // Act
        hooks
            .apply_pointer(
                &mut world,
                particle_system,
                crate::scene::PointerKind::Down,
                0.0,
                3.15,
            )
            .expect("down should aim the jet toward the pointer");
        hooks
            .on_advance(&mut world, particle_system)
            .expect("aimed emit should succeed");
        let aimed_horizontal = {
            let view = world
                .particle_system_view(particle_system)
                .expect("aimed system should stay live");
            view.velocities()
                .iter()
                .any(|velocity| velocity.x > 1.0 && velocity.y.abs() < 0.15)
        };

        hooks
            .apply_pointer(
                &mut world,
                particle_system,
                crate::scene::PointerKind::Cancel,
                0.0,
                0.0,
            )
            .expect("cancel should clear the aim override");
        hooks
            .on_advance(&mut world, particle_system)
            .expect("default emit should succeed");
        let restored_default = {
            let view = world
                .particle_system_view(particle_system)
                .expect("restored system should stay live");
            view.velocities()
                .iter()
                .any(|velocity| velocity.x > 1.0 && (velocity.y + 0.4).abs() < 0.05)
        };

        // Assert
        assert!(
            aimed_horizontal,
            "pointer toward x=0 at nozzle height should emit a positive x velocity"
        );
        assert!(
            restored_default,
            "cancel must restore Vec2::new(speed, -0.4)"
        );
    }

    #[test]
    fn jet_strength_strong_sets_speed_without_a_motor_write() {
        // Arrange
        let super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = super::build(&[]).expect("Water Wheel should construct");
        let source = include_str!("water_wheel.rs");
        let impl_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("implementation precedes tests");

        // Act
        let strong = hooks
            .apply_control(&mut world, particle_system, "jet-strength", "strong")
            .expect("jet-strength=strong should apply live");
        hooks
            .on_advance(&mut world, particle_system)
            .expect("strong emit should succeed");
        let strong_speed = {
            let view = world
                .particle_system_view(particle_system)
                .expect("strong system should stay live");
            view.velocities()
                .iter()
                .any(|velocity| (velocity.x - 12.0).abs() < 0.05 && (velocity.y + 0.4).abs() < 0.05)
        };

        // Assert
        assert!(matches!(strong, super::ControlEffect::Live));
        assert!(
            strong_speed,
            "jet-strength=strong must still emit at speed 12.0"
        );
        assert!(
            impl_source.contains("maybe_aim_velocity"),
            "Water Wheel must store maybe_aim_velocity for pointer aim"
        );
        assert!(
            impl_source.contains("Vec2::new(speed, -0.4)")
                || impl_source.contains("Vec2::new(self.jet_speed, -0.4)"),
            "cancel default must remain Vec2::new(speed, -0.4)"
        );
        assert!(
            !impl_source.contains("enable_motor") && !impl_source.contains(".motor_speed"),
            "pointer aim must not write a revolute motor"
        );
    }

    #[test]
    fn jet_strength_and_emission_apply_live() {
        // Arrange
        let super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = super::build(&[]).expect("Water Wheel should construct");

        // Act
        let weak = hooks
            .apply_control(&mut world, particle_system, "jet-strength", "weak")
            .expect("weak jet should apply");
        let medium = hooks
            .apply_control(&mut world, particle_system, "jet-strength", "medium")
            .expect("medium jet should apply");
        let strong = hooks
            .apply_control(&mut world, particle_system, "jet-strength", "strong")
            .expect("strong jet should apply");
        let off = hooks
            .apply_control(&mut world, particle_system, "emission", "off")
            .expect("emission off should apply");
        let on = hooks
            .apply_control(&mut world, particle_system, "emission", "on")
            .expect("emission on should apply");

        // Assert
        assert!(matches!(weak, super::ControlEffect::Live));
        assert!(matches!(medium, super::ControlEffect::Live));
        assert!(matches!(strong, super::ControlEffect::Live));
        assert!(matches!(off, super::ControlEffect::Live));
        assert!(matches!(on, super::ControlEffect::Live));
    }

    #[test]
    fn unknown_jet_and_emission_tokens_fail_closed() {
        // Arrange
        let mut session = SessionCore::create(SceneId::WaterWheel)
            .expect("Water Wheel should construct a native pinned wheel");

        // Act
        let bad_strength = session.apply_control("jet-strength", "max");
        let bad_emission = session.apply_control("emission", "maybe");
        let unknown_name = session.apply_control("aim-angle", "0");

        // Assert
        assert_eq!(
            bad_strength,
            Err(crate::session::SessionError::UnknownControl)
        );
        assert_eq!(
            bad_emission,
            Err(crate::session::SessionError::UnknownControl)
        );
        assert_eq!(
            unknown_name,
            Err(crate::session::SessionError::UnknownControl)
        );
    }

    fn captured_paddle_angle(session: &SessionCore) -> f32 {
        let segments = capture(session).rigid_segments();
        assert!(
            segments.len() >= 16,
            "captured frame must include at least four paddle segments"
        );
        let first_paddle = segments.len().saturating_sub(16);
        let start_x = segments[first_paddle];
        let start_y = segments[first_paddle + 1];
        let end_x = segments[first_paddle + 2];
        let end_y = segments[first_paddle + 3];
        (end_y - start_y).atan2(end_x - start_x)
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

    fn flatten_segments(segments: &[crate::scene::RigidSegment]) -> Vec<f32> {
        let mut values = Vec::with_capacity(segments.len().saturating_mul(4));
        for segment in segments {
            values.extend_from_slice(&[
                segment.start.x,
                segment.start.y,
                segment.end.x,
                segment.end.y,
            ]);
        }
        values
    }

    fn capture(session: &SessionCore) -> ProofFrame {
        ProofFrame::from(
            session
                .capture_frame()
                .expect("Water Wheel should capture a frame"),
        )
    }
}
