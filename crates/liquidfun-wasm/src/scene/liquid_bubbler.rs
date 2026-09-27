//! Liquid Bubbler: colored water drips through a static waist and turns a motor-off wheel.
//!
//! A side-shaft plate stays down through the opening, then creeps upward to return the liquid.
//! Water Wheel stays the jet-driven wheel.

use liquidfun::collision::{CircleShape, FilterData, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, JointDef, JointId, ParticleSystemDef, ParticleSystemId,
    PrismaticJointDef, RevoluteJointDef, World,
};

use super::{BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.025;
const PARTICLE_DAMPING: f32 = 0.2;
const DRIP_COLOR: ParticleColor = ParticleColor::new(242, 176, 64, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const SIM_DT: f32 = 1.0 / 60.0;
const WALL_HALF: f32 = 0.04;
const WALL_FRICTION: f32 = 0.2;
const WAIST_GAP: f32 = 0.14;
const WAIST_BOTTOM: f32 = 0.90;
const WAIST_TOP: f32 = 0.98;
const HUB: Vec2 = Vec2::new(0.0, 0.40);
const HUB_RADIUS: f32 = 0.12;
const PADDLE_INNER: f32 = 0.12;
const PADDLE_OUTER: f32 = 0.32;
const PADDLE_HALF_WIDTH: f32 = 0.035;
const WHEEL_DENSITY: f32 = 0.03;
const ANGULAR_DAMPING: f32 = 0.05;
/// Rests one polygon-skin pair above the floor so a flush contact does not
/// pop the plate off translation 0 during the dwell.
const PLATE_FLOOR_CLEARANCE: f32 = 2.0 * 0.01;
const PLATE_CENTER: Vec2 = Vec2::new(0.78, 0.04 + PLATE_FLOOR_CLEARANCE);
const PLATE_HALF_WIDTH: f32 = 0.12;
const PLATE_HALF_HEIGHT: f32 = 0.04;
const PLATE_DENSITY: f32 = 1.0;
const STROKE: f32 = 1.64;
const PLATE_SPEED: f32 = 0.15;
const DWELL: f32 = 3.0;
const RISE_SECONDS: f32 = STROKE / PLATE_SPEED;
const CYCLE: f32 = DWELL + RISE_SECONDS + RISE_SECONDS;
const LIMIT_LOW: f32 = -0.02;
const LIMIT_HIGH: f32 = STROKE + 0.02;
const MAX_MOTOR_FORCE: f32 = 1.0e6;

const GAP_HALF: f32 = WAIST_GAP * 0.5;
const LEFT_CHAMBER_INNER_X: f32 = -0.55;
const DIVIDER_LEFT_X: f32 = 0.48;
const LIP_HALF_HEIGHT: f32 = (WAIST_TOP - WAIST_BOTTOM) * 0.5;
const LIP_CENTER_Y: f32 = (WAIST_BOTTOM + WAIST_TOP) * 0.5;
const LEFT_LIP_HALF_WIDTH: f32 = (-GAP_HALF - LEFT_CHAMBER_INNER_X) * 0.5;
const LEFT_LIP_CENTER_X: f32 = (LEFT_CHAMBER_INNER_X - GAP_HALF) * 0.5;
const RIGHT_LIP_HALF_WIDTH: f32 = (DIVIDER_LEFT_X - GAP_HALF) * 0.5;
const RIGHT_LIP_CENTER_X: f32 = (GAP_HALF + DIVIDER_LEFT_X) * 0.5;

const WATER_POLYGON: [Vec2; 4] = [
    Vec2::new(-0.48, 1.12),
    Vec2::new(0.40, 1.12),
    Vec2::new(0.40, 1.50),
    Vec2::new(-0.48, 1.50),
];

const PADDLE_POLYGONS: [[Vec2; 4]; 4] = [
    [
        Vec2::new(PADDLE_INNER, -PADDLE_HALF_WIDTH),
        Vec2::new(PADDLE_OUTER, -PADDLE_HALF_WIDTH),
        Vec2::new(PADDLE_OUTER, PADDLE_HALF_WIDTH),
        Vec2::new(PADDLE_INNER, PADDLE_HALF_WIDTH),
    ],
    [
        Vec2::new(-PADDLE_HALF_WIDTH, PADDLE_INNER),
        Vec2::new(PADDLE_HALF_WIDTH, PADDLE_INNER),
        Vec2::new(PADDLE_HALF_WIDTH, PADDLE_OUTER),
        Vec2::new(-PADDLE_HALF_WIDTH, PADDLE_OUTER),
    ],
    [
        Vec2::new(-PADDLE_OUTER, -PADDLE_HALF_WIDTH),
        Vec2::new(-PADDLE_INNER, -PADDLE_HALF_WIDTH),
        Vec2::new(-PADDLE_INNER, PADDLE_HALF_WIDTH),
        Vec2::new(-PADDLE_OUTER, PADDLE_HALF_WIDTH),
    ],
    [
        Vec2::new(-PADDLE_HALF_WIDTH, -PADDLE_OUTER),
        Vec2::new(PADDLE_HALF_WIDTH, -PADDLE_OUTER),
        Vec2::new(PADDLE_HALF_WIDTH, -PADDLE_INNER),
        Vec2::new(-PADDLE_HALF_WIDTH, -PADDLE_INNER),
    ],
];

const PADDLE_CENTERLINES: [(Vec2, Vec2); 4] = [
    (Vec2::new(PADDLE_INNER, 0.0), Vec2::new(PADDLE_OUTER, 0.0)),
    (Vec2::new(0.0, PADDLE_INNER), Vec2::new(0.0, PADDLE_OUTER)),
    (Vec2::new(-PADDLE_OUTER, 0.0), Vec2::new(-PADDLE_INNER, 0.0)),
    (Vec2::new(0.0, -PADDLE_OUTER), Vec2::new(0.0, -PADDLE_INNER)),
];

const PLATE_LOCAL_CORNERS: [Vec2; 4] = [
    Vec2::new(-PLATE_HALF_WIDTH, -PLATE_HALF_HEIGHT),
    Vec2::new(PLATE_HALF_WIDTH, -PLATE_HALF_HEIGHT),
    Vec2::new(PLATE_HALF_WIDTH, PLATE_HALF_HEIGHT),
    Vec2::new(-PLATE_HALF_WIDTH, PLATE_HALF_HEIGHT),
];

struct BoxSpec {
    half_width: f32,
    half_height: f32,
    center: Vec2,
}

struct LiquidBubblerHooks {
    elapsed: f32,
    revolute: JointId,
    prismatic: JointId,
    wheel: BodyId,
    plate: BodyId,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_liquid_bubbler().map_err(|_error| SessionError::SceneConstruction)
}

fn build_liquid_bubbler() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    attach_wall_boxes(&mut world, ground)?;

    let wheel = create_wheel(&mut world)?;
    let revolute = pin_wheel(&mut world, ground, wheel)?;
    let plate = create_plate(&mut world)?;
    let prismatic = create_plate_joint(&mut world, ground, plate)?;
    let particle_system = create_water_group(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(LiquidBubblerHooks {
            elapsed: 0.0,
            revolute,
            prismatic,
            wheel,
            plate,
        }),
    })
}

fn wall_boxes() -> [BoxSpec; 6] {
    [
        BoxSpec {
            half_width: WALL_HALF,
            half_height: 0.95,
            center: Vec2::new(-0.59, 0.95),
        },
        BoxSpec {
            half_width: 0.835,
            half_height: WALL_HALF,
            center: Vec2::new(0.205, -WALL_HALF),
        },
        BoxSpec {
            half_width: WALL_HALF,
            half_height: 0.95,
            center: Vec2::new(1.00, 0.95),
        },
        BoxSpec {
            half_width: WALL_HALF,
            half_height: 0.65,
            center: Vec2::new(0.52, 0.97),
        },
        BoxSpec {
            half_width: LEFT_LIP_HALF_WIDTH,
            half_height: LIP_HALF_HEIGHT,
            center: Vec2::new(LEFT_LIP_CENTER_X, LIP_CENTER_Y),
        },
        BoxSpec {
            half_width: RIGHT_LIP_HALF_WIDTH,
            half_height: LIP_HALF_HEIGHT,
            center: Vec2::new(RIGHT_LIP_CENTER_X, LIP_CENTER_Y),
        },
    ]
}

fn attach_wall_boxes(world: &mut World, ground: BodyId) -> Result<(), SceneError> {
    for wall in wall_boxes() {
        attach_box(
            world,
            ground,
            wall.half_width,
            wall.half_height,
            wall.center,
            0.0,
        )?;
    }
    Ok(())
}

fn create_wheel(world: &mut World) -> Result<BodyId, SceneError> {
    let definition = BodyDef::new(BodyType::Dynamic, HUB, 0.0, true)
        .map_err(|_error| SceneError::Body)?
        .with_angular_damping(ANGULAR_DAMPING)
        .map_err(|_error| SceneError::Body)?
        .with_sleeping_allowed(false);
    let wheel = world
        .create_body(&definition)
        .map_err(|_error| SceneError::Body)?;
    attach_hub(world, wheel)?;
    for vertices in PADDLE_POLYGONS {
        attach_paddle(world, wheel, &vertices)?;
    }
    Ok(wheel)
}

fn attach_hub(world: &mut World, wheel: BodyId) -> Result<(), SceneError> {
    let circle = CircleShape::new(Vec2::ZERO, HUB_RADIUS).map_err(|_error| SceneError::Geometry)?;
    let definition = FixtureDef::new(
        Shape::from(circle),
        WHEEL_DENSITY,
        WALL_FRICTION,
        0.0,
        false,
        FilterData::default(),
    )
    .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(wheel, &definition)
        .map_err(|_error| SceneError::Fixture)?;
    Ok(())
}

fn attach_paddle(world: &mut World, wheel: BodyId, vertices: &[Vec2; 4]) -> Result<(), SceneError> {
    let polygon = PolygonShape::new(vertices).map_err(|_error| SceneError::Geometry)?;
    let definition = FixtureDef::new(
        Shape::from(polygon),
        WHEEL_DENSITY,
        WALL_FRICTION,
        0.0,
        false,
        FilterData::default(),
    )
    .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(wheel, &definition)
        .map_err(|_error| SceneError::Fixture)?;
    Ok(())
}

fn pin_wheel(world: &mut World, ground: BodyId, wheel: BodyId) -> Result<JointId, SceneError> {
    let joint = RevoluteJointDef::new(ground, wheel)
        .map_err(|_error| SceneError::Body)?
        .with_frame(HUB, Vec2::ZERO, 0.0)
        .map_err(|_error| SceneError::Body)?;
    world
        .create_joint(JointDef::from(joint))
        .map_err(|_error| SceneError::Body)
}

fn create_plate(world: &mut World) -> Result<BodyId, SceneError> {
    let definition = BodyDef::new(BodyType::Dynamic, PLATE_CENTER, 0.0, true)
        .map_err(|_error| SceneError::Body)?
        .with_sleeping_allowed(false);
    let plate = world
        .create_body(&definition)
        .map_err(|_error| SceneError::Body)?;
    attach_box(
        world,
        plate,
        PLATE_HALF_WIDTH,
        PLATE_HALF_HEIGHT,
        Vec2::ZERO,
        PLATE_DENSITY,
    )?;
    Ok(plate)
}

fn create_plate_joint(
    world: &mut World,
    ground: BodyId,
    plate: BodyId,
) -> Result<JointId, SceneError> {
    let definition = PrismaticJointDef::new(ground, plate)
        .map_err(|_error| SceneError::Body)?
        .with_collide_connected(true)
        .with_frame(PLATE_CENTER, Vec2::ZERO, Vec2::new(0.0, 1.0), 0.0)
        .map_err(|_error| SceneError::Body)?
        .with_limits(true, LIMIT_LOW, LIMIT_HIGH)
        .map_err(|_error| SceneError::Body)?
        .with_motor(true, 0.0, MAX_MOTOR_FORCE)
        .map_err(|_error| SceneError::Body)?;
    world
        .create_joint(JointDef::from(definition))
        .map_err(|_error| SceneError::Body)
}

fn attach_box(
    world: &mut World,
    body: BodyId,
    half_width: f32,
    half_height: f32,
    center: Vec2,
    density: f32,
) -> Result<(), SceneError> {
    let polygon = PolygonShape::oriented_box(half_width, half_height, center, 0.0)
        .map_err(|_error| SceneError::Geometry)?;
    let definition = FixtureDef::new(
        Shape::from(polygon),
        density,
        WALL_FRICTION,
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

fn create_water_group(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_damping(PARTICLE_DAMPING)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_destruction_by_age(false);
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    let filled =
        Shape::from(PolygonShape::new(&WATER_POLYGON).map_err(|_error| SceneError::Geometry)?);
    let source =
        ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_color(DRIP_COLOR)
        .with_transform(Transform::IDENTITY)
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(system)
}

fn scheduled_plate_speed(elapsed: f32) -> f32 {
    let phase = elapsed.rem_euclid(CYCLE);
    if phase < DWELL {
        0.0
    } else if phase < DWELL + RISE_SECONDS {
        PLATE_SPEED
    } else {
        -PLATE_SPEED
    }
}

fn push_box_outline(
    segments: &mut Vec<RigidSegment>,
    center: Vec2,
    half_width: f32,
    half_height: f32,
) {
    let corners = [
        Vec2::new(center.x - half_width, center.y - half_height),
        Vec2::new(center.x + half_width, center.y - half_height),
        Vec2::new(center.x + half_width, center.y + half_height),
        Vec2::new(center.x - half_width, center.y + half_height),
    ];
    for index in 0..corners.len() {
        segments.push(RigidSegment {
            start: corners[index],
            end: corners[(index + 1) % corners.len()],
        });
    }
}

fn push_transformed_segment(
    segments: &mut Vec<RigidSegment>,
    transform: Transform,
    start: Vec2,
    end: Vec2,
) {
    segments.push(RigidSegment {
        start: transform.apply(start),
        end: transform.apply(end),
    });
}

impl SceneHooks for LiquidBubblerHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        self.elapsed += SIM_DT;
        world
            .revolute_joint_angle(self.revolute)
            .map_err(|_error| SessionError::StepFailed)?;
        world
            .set_prismatic_motor_speed(self.prismatic, scheduled_plate_speed(self.elapsed))
            .map_err(|_error| SessionError::StepFailed)
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
        let mut segments = Vec::new();
        for wall in wall_boxes() {
            push_box_outline(
                &mut segments,
                wall.center,
                wall.half_width,
                wall.half_height,
            );
        }

        let wheel_transform = world
            .body_snapshot(self.wheel)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .transform();
        for (start, end) in PADDLE_CENTERLINES {
            push_transformed_segment(&mut segments, wheel_transform, start, end);
        }

        let plate_transform = world
            .body_snapshot(self.plate)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .transform();
        for index in 0..PLATE_LOCAL_CORNERS.len() {
            push_transformed_segment(
                &mut segments,
                plate_transform,
                PLATE_LOCAL_CORNERS[index],
                PLATE_LOCAL_CORNERS[(index + 1) % PLATE_LOCAL_CORNERS.len()],
            );
        }
        Ok(segments)
    }

    fn collect_circles(&self, world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        let transform = world
            .body_snapshot(self.wheel)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .transform();
        Ok(vec![(transform.apply(Vec2::ZERO), HUB_RADIUS)])
    }
}

#[cfg(test)]
mod tests;
