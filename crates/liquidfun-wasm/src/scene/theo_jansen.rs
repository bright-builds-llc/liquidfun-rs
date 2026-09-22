//! Pinned `LiquidFun` Theo Jansen walker: soft distance legs and live motor reverse.
//!
//! ANTI-PATTERN: Do not copy Water Wheel's motor-off revolute.
//! Legs stay soft (`DistanceJointDef` frequency 10, damping 0.5) — do not weld.

use std::f32::consts::TAU;

use liquidfun::collision::{CircleShape, EdgeShape, FilterData, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{
    BodyDef, BodyId, BodyType, DistanceJointDef, FixtureDef, JointDef, JointId, ParticleSystemDef,
    ParticleSystemId, RevoluteJointDef, World,
};

use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.2;
const PARTICLE_DAMPING: f32 = 0.2;
const MAXIMUM_PARTICLE_COUNT: usize = 10240;
const GROUP_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);

const OFFSET: Vec2 = Vec2::new(0.0, 8.0);
const PIVOT: Vec2 = Vec2::new(0.0, 0.8);
const MOTOR_SPEED: f32 = 2.0;
const MAX_MOTOR_TORQUE: f32 = 400.0;
const CHASSIS_HALF: Vec2 = Vec2::new(2.5, 1.0);
const WHEEL_RADIUS: f32 = 1.6;
const BALL_RADIUS: f32 = 0.25;
const BALL_COUNT: usize = 40;
const LEG_ANGULAR_DAMPING: f32 = 10.0;
const WALKER_FILTER: FilterData = FilterData::new(0x0001, 0xffff, -1);
const MOTOR_DIRECTION_CONTROL: &str = "motor-direction";
const SLAB_HALF: Vec2 = Vec2::new(7.0, 0.5);
const SLAB_CENTER: Vec2 = Vec2::new(0.0, 15.0);

const GROUND_EDGES: [(Vec2, Vec2); 3] = [
    (Vec2::new(-50.0, 0.0), Vec2::new(50.0, 0.0)),
    (Vec2::new(-50.0, 0.0), Vec2::new(-50.0, 10.0)),
    (Vec2::new(50.0, 0.0), Vec2::new(50.0, 10.0)),
];

struct LegBody {
    body: BodyId,
    vertices: [Vec2; 3],
}

struct TheoJansenHooks {
    motor: JointId,
    ground_segments: [RigidSegment; 3],
    chassis: BodyId,
    wheel: BodyId,
    balls: Vec<BodyId>,
    legs: Vec<LegBody>,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_theo_jansen().map_err(|_error| SessionError::SceneConstruction)
}

fn build_theo_jansen() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let ground_segments = create_ground(&mut world)?;
    let balls = create_balls(&mut world)?;
    let chassis_position = PIVOT + OFFSET;
    let chassis = create_chassis(&mut world, chassis_position)?;
    let wheel = create_wheel(&mut world, chassis_position)?;
    let motor = pin_motor(&mut world, wheel, chassis, chassis_position)?;

    let wheel_anchor = PIVOT + Vec2::new(0.0, -0.8);
    let mut legs = Vec::with_capacity(12);
    create_leg_pair(&mut world, &mut legs, chassis, wheel, -1.0, wheel_anchor)?;
    create_leg_pair(&mut world, &mut legs, chassis, wheel, 1.0, wheel_anchor)?;

    world
        .set_body_transform(wheel, chassis_position, TAU / 3.0)
        .map_err(|_error| SceneError::Body)?;
    create_leg_pair(&mut world, &mut legs, chassis, wheel, -1.0, wheel_anchor)?;
    create_leg_pair(&mut world, &mut legs, chassis, wheel, 1.0, wheel_anchor)?;

    world
        .set_body_transform(wheel, chassis_position, -TAU / 3.0)
        .map_err(|_error| SceneError::Body)?;
    create_leg_pair(&mut world, &mut legs, chassis, wheel, -1.0, wheel_anchor)?;
    create_leg_pair(&mut world, &mut legs, chassis, wheel, 1.0, wheel_anchor)?;

    let particle_system = create_particle_slab(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(TheoJansenHooks {
            motor,
            ground_segments,
            chassis,
            wheel,
            balls,
            legs,
        }),
    })
}

fn create_ground(world: &mut World) -> Result<[RigidSegment; 3], SceneError> {
    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    let mut segments = [RigidSegment {
        start: Vec2::ZERO,
        end: Vec2::ZERO,
    }; 3];
    for (index, &(start, end)) in GROUND_EDGES.iter().enumerate() {
        let edge = EdgeShape::new(start, end).map_err(|_error| SceneError::Geometry)?;
        let definition = FixtureDef::new(
            Shape::from(edge),
            0.0,
            0.2,
            0.0,
            false,
            FilterData::default(),
        )
        .map_err(|_error| SceneError::Fixture)?;
        world
            .create_fixture(ground, &definition)
            .map_err(|_error| SceneError::Fixture)?;
        segments[index] = RigidSegment { start, end };
    }
    Ok(segments)
}

fn create_balls(world: &mut World) -> Result<Vec<BodyId>, SceneError> {
    let mut balls = Vec::with_capacity(BALL_COUNT);
    for index in 0..BALL_COUNT {
        let position = Vec2::new(-40.0 + 2.0 * index as f32, 0.5);
        let definition = BodyDef::new(BodyType::Dynamic, position, 0.0, true)
            .map_err(|_error| SceneError::Body)?;
        let body = world
            .create_body(&definition)
            .map_err(|_error| SceneError::Body)?;
        let circle =
            CircleShape::new(Vec2::ZERO, BALL_RADIUS).map_err(|_error| SceneError::Geometry)?;
        let fixture = FixtureDef::new(
            Shape::from(circle),
            1.0,
            0.2,
            0.0,
            false,
            FilterData::default(),
        )
        .map_err(|_error| SceneError::Fixture)?;
        world
            .create_fixture(body, &fixture)
            .map_err(|_error| SceneError::Fixture)?;
        balls.push(body);
    }
    Ok(balls)
}

fn create_chassis(world: &mut World, position: Vec2) -> Result<BodyId, SceneError> {
    let definition = BodyDef::new(BodyType::Dynamic, position, 0.0, true)
        .map_err(|_error| SceneError::Body)?;
    let body = world
        .create_body(&definition)
        .map_err(|_error| SceneError::Body)?;
    let polygon = PolygonShape::oriented_box(CHASSIS_HALF.x, CHASSIS_HALF.y, Vec2::ZERO, 0.0)
        .map_err(|_error| SceneError::Geometry)?;
    let fixture = FixtureDef::new(Shape::from(polygon), 1.0, 0.2, 0.0, false, WALKER_FILTER)
        .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(body, &fixture)
        .map_err(|_error| SceneError::Fixture)?;
    Ok(body)
}

fn create_wheel(world: &mut World, position: Vec2) -> Result<BodyId, SceneError> {
    let definition = BodyDef::new(BodyType::Dynamic, position, 0.0, true)
        .map_err(|_error| SceneError::Body)?;
    let body = world
        .create_body(&definition)
        .map_err(|_error| SceneError::Body)?;
    let circle =
        CircleShape::new(Vec2::ZERO, WHEEL_RADIUS).map_err(|_error| SceneError::Geometry)?;
    let fixture = FixtureDef::new(Shape::from(circle), 1.0, 0.2, 0.0, false, WALKER_FILTER)
        .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(body, &fixture)
        .map_err(|_error| SceneError::Fixture)?;
    Ok(body)
}

fn pin_motor(
    world: &mut World,
    wheel: BodyId,
    chassis: BodyId,
    _world_anchor: Vec2,
) -> Result<JointId, SceneError> {
    // Both bodies share the chassis/wheel origin, so local anchors are zero.
    let joint = RevoluteJointDef::new(wheel, chassis)
        .map_err(|_error| SceneError::Body)?
        .with_collide_connected(false)
        .with_frame(Vec2::ZERO, Vec2::ZERO, 0.0)
        .map_err(|_error| SceneError::Body)?
        .with_motor(true, MOTOR_SPEED, MAX_MOTOR_TORQUE)
        .map_err(|_error| SceneError::Body)?;
    world
        .create_joint(JointDef::from(joint))
        .map_err(|_error| SceneError::Body)
}

fn create_leg_pair(
    world: &mut World,
    legs: &mut Vec<LegBody>,
    chassis: BodyId,
    wheel: BodyId,
    side: f32,
    wheel_anchor: Vec2,
) -> Result<(), SceneError> {
    create_leg(world, legs, chassis, wheel, side, wheel_anchor)
}

fn create_leg(
    world: &mut World,
    legs: &mut Vec<LegBody>,
    chassis: BodyId,
    wheel: BodyId,
    side: f32,
    wheel_anchor: Vec2,
) -> Result<(), SceneError> {
    let p1 = Vec2::new(5.4 * side, -6.1);
    let p2 = Vec2::new(7.2 * side, -1.2);
    let p3 = Vec2::new(4.3 * side, -1.9);
    let p4 = Vec2::new(3.1 * side, 0.8);
    let p5 = Vec2::new(6.0 * side, 1.5);
    let p6 = Vec2::new(2.5 * side, 3.7);

    let (poly1, poly2) = if side > 0.0 {
        ([p1, p2, p3], [Vec2::ZERO, p5 - p4, p6 - p4])
    } else {
        ([p1, p3, p2], [Vec2::ZERO, p6 - p4, p5 - p4])
    };

    let body1_definition = BodyDef::new(BodyType::Dynamic, OFFSET, 0.0, true)
        .map_err(|_error| SceneError::Body)?
        .with_angular_damping(LEG_ANGULAR_DAMPING)
        .map_err(|_error| SceneError::Body)?;
    let body1 = world
        .create_body(&body1_definition)
        .map_err(|_error| SceneError::Body)?;
    attach_leg_polygon(world, body1, &poly1)?;

    let body2_definition = BodyDef::new(BodyType::Dynamic, p4 + OFFSET, 0.0, true)
        .map_err(|_error| SceneError::Body)?
        .with_angular_damping(LEG_ANGULAR_DAMPING)
        .map_err(|_error| SceneError::Body)?;
    let body2 = world
        .create_body(&body2_definition)
        .map_err(|_error| SceneError::Body)?;
    attach_leg_polygon(world, body2, &poly2)?;

    soft_distance(world, body1, body2, p2 + OFFSET, p5 + OFFSET)?;
    soft_distance(world, body1, body2, p3 + OFFSET, p4 + OFFSET)?;
    soft_distance(world, body1, wheel, p3 + OFFSET, wheel_anchor + OFFSET)?;
    soft_distance(world, body2, wheel, p6 + OFFSET, wheel_anchor + OFFSET)?;

    revolute_at_world(world, body2, chassis, p4 + OFFSET)?;

    legs.push(LegBody {
        body: body1,
        vertices: poly1,
    });
    legs.push(LegBody {
        body: body2,
        vertices: poly2,
    });
    Ok(())
}

fn attach_leg_polygon(world: &mut World, body: BodyId, vertices: &[Vec2; 3]) -> Result<(), SceneError> {
    let polygon = PolygonShape::new(vertices).map_err(|_error| SceneError::Geometry)?;
    let fixture = FixtureDef::new(Shape::from(polygon), 1.0, 0.2, 0.0, false, WALKER_FILTER)
        .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(body, &fixture)
        .map_err(|_error| SceneError::Fixture)?;
    Ok(())
}

fn soft_distance(
    world: &mut World,
    body_a: BodyId,
    body_b: BodyId,
    world_anchor_a: Vec2,
    world_anchor_b: Vec2,
) -> Result<(), SceneError> {
    let transform_a = world
        .body_snapshot(body_a)
        .map_err(|_error| SceneError::Body)?
        .transform();
    let transform_b = world
        .body_snapshot(body_b)
        .map_err(|_error| SceneError::Body)?
        .transform();
    let local_a = transform_a.inverse_apply(world_anchor_a);
    let local_b = transform_b.inverse_apply(world_anchor_b);
    let length = (world_anchor_b - world_anchor_a).length();
    let definition = DistanceJointDef::new(body_a, body_b)
        .map_err(|_error| SceneError::Body)?
        .with_anchors(local_a, local_b)
        .map_err(|_error| SceneError::Body)?
        .with_length(length)
        .map_err(|_error| SceneError::Body)?
        // Pinned CreateLeg soft suspension: frequencyHz=10, dampingRatio=0.5
        .with_frequency(10.0)
        .map_err(|_error| SceneError::Body)?
        .with_damping_ratio(0.5)
        .map_err(|_error| SceneError::Body)?;
    world
        .create_joint(JointDef::from(definition))
        .map_err(|_error| SceneError::Body)?;
    Ok(())
}

fn revolute_at_world(
    world: &mut World,
    body_a: BodyId,
    body_b: BodyId,
    world_anchor: Vec2,
) -> Result<(), SceneError> {
    let transform_a = world
        .body_snapshot(body_a)
        .map_err(|_error| SceneError::Body)?
        .transform();
    let transform_b = world
        .body_snapshot(body_b)
        .map_err(|_error| SceneError::Body)?
        .transform();
    let local_a = transform_a.inverse_apply(world_anchor);
    let local_b = transform_b.inverse_apply(world_anchor);
    let reference = transform_b.rotation().angle() - transform_a.rotation().angle();
    let definition = RevoluteJointDef::new(body_a, body_b)
        .map_err(|_error| SceneError::Body)?
        .with_frame(local_a, local_b, reference)
        .map_err(|_error| SceneError::Body)?;
    world
        .create_joint(JointDef::from(definition))
        .map_err(|_error| SceneError::Body)?;
    Ok(())
}

fn create_particle_slab(world: &mut World) -> Result<ParticleSystemId, SceneError> {
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
        PolygonShape::oriented_box(SLAB_HALF.x, SLAB_HALF.y, SLAB_CENTER, 0.0)
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

impl SceneHooks for TheoJansenHooks {
    fn on_advance(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        Ok(())
    }

    fn apply_control(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
        name: &str,
        value: &str,
    ) -> Result<ControlEffect, SessionError> {
        match (name, value) {
            (MOTOR_DIRECTION_CONTROL, "forward") => {
                world
                    .set_revolute_motor_speed(self.motor, MOTOR_SPEED)
                    .map_err(|_error| SessionError::StepFailed)?;
                Ok(ControlEffect::Live)
            }
            (MOTOR_DIRECTION_CONTROL, "reverse") => {
                world
                    .set_revolute_motor_speed(self.motor, -MOTOR_SPEED)
                    .map_err(|_error| SessionError::StepFailed)?;
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
        _kind: PointerKind,
        _world_x: f32,
        _world_y: f32,
    ) -> Result<(), SessionError> {
        Ok(())
    }

    fn collect_segments(&self, world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        let mut segments = Vec::with_capacity(3 + 4 + self.legs.len() * 3);
        segments.extend_from_slice(&self.ground_segments);

        let chassis_transform = world
            .body_snapshot(self.chassis)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .transform();
        let chassis_corners = [
            Vec2::new(-CHASSIS_HALF.x, -CHASSIS_HALF.y),
            Vec2::new(CHASSIS_HALF.x, -CHASSIS_HALF.y),
            Vec2::new(CHASSIS_HALF.x, CHASSIS_HALF.y),
            Vec2::new(-CHASSIS_HALF.x, CHASSIS_HALF.y),
        ];
        let world_corners: [Vec2; 4] = [
            chassis_transform.apply(chassis_corners[0]),
            chassis_transform.apply(chassis_corners[1]),
            chassis_transform.apply(chassis_corners[2]),
            chassis_transform.apply(chassis_corners[3]),
        ];
        for index in 0..4 {
            segments.push(RigidSegment {
                start: world_corners[index],
                end: world_corners[(index + 1) % 4],
            });
        }

        for leg in &self.legs {
            let transform = world
                .body_snapshot(leg.body)
                .map_err(|_error| SessionError::FrameCaptureFailed)?
                .transform();
            let corners = [
                transform.apply(leg.vertices[0]),
                transform.apply(leg.vertices[1]),
                transform.apply(leg.vertices[2]),
            ];
            for index in 0..3 {
                segments.push(RigidSegment {
                    start: corners[index],
                    end: corners[(index + 1) % 3],
                });
            }
        }
        Ok(segments)
    }

    fn collect_circles(&self, world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        let mut circles = Vec::with_capacity(1 + BALL_COUNT);
        let wheel_position = world
            .body_snapshot(self.wheel)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .transform()
            .position();
        circles.push((wheel_position, WHEEL_RADIUS));
        for &ball in &self.balls {
            let position = world
                .body_snapshot(ball)
                .map_err(|_error| SessionError::FrameCaptureFailed)?
                .transform()
                .position();
            circles.push((position, BALL_RADIUS));
        }
        Ok(circles)
    }
}

#[cfg(test)]
mod tests;
