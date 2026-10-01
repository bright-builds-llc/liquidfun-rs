//! Liquid Bubbler: colored water crosses three shelves, then a slanted shaft plate lifts it.

use std::f32::consts::TAU;

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

mod elevator_motion;
mod elevator_plate;
use elevator_motion::{plate_local_corners, scheduled_plate_offset, scheduled_plate_speed};
use elevator_plate::{attach_plate_fixtures, push_plate_segments};

/// Finer than the original 0.025 m drip. The default stride then fills the upper chamber with 3,000 particles.
const PARTICLE_RADIUS: f32 = 0.008;
const PARTICLE_DAMPING: f32 = 0.2;
const DRIP_COLOR: ParticleColor = ParticleColor::new(242, 176, 64, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const SIM_DT: f32 = 1.0 / 60.0;
const WALL_HALF: f32 = 0.04;
const WALL_FRICTION: f32 = 0.2;
const LEVEL_COUNT: usize = 3;
const SHELF_THICKNESS: f32 = 0.08;
const HOLE_HALF_WIDTH: f32 = 0.10;
const HUB_RADIUS: f32 = 0.05;
const PADDLE_INNER: f32 = 0.05;
const PADDLE_OUTER: f32 = 0.11;
const PADDLE_HALF_WIDTH: f32 = 0.028;
const WHEEL_DENSITY: f32 = 0.03;
const ANGULAR_DAMPING: f32 = 0.05;
const PLATE_FLOOR_CLEARANCE: f32 = 2.0 * 0.01;
/// Reaches both shaft faces, leaving about a 1 mm seam.
const PLATE_HALF_WIDTH: f32 = 0.198;
const PLATE_HALF_HEIGHT: f32 = 0.02;
const PLATE_SLANT: f32 = TAU * 2.0 / 360.0;
const PLATE_CENTER: Vec2 = Vec2::new(0.76, 0.047);
const PLATE_DENSITY: f32 = 1.0;
const WALL_TOP_Y: f32 = 2.40;
const WALL_HALF_HEIGHT: f32 = WALL_TOP_Y * 0.5;
const WALL_CENTER_Y: f32 = WALL_HALF_HEIGHT;
const DIVIDER_BOTTOM_Y: f32 = 0.074;
const DIVIDER_TOP_Y: f32 = 2.00;
const DIVIDER_HALF_HEIGHT: f32 = (DIVIDER_TOP_Y - DIVIDER_BOTTOM_Y) * 0.5;
const DIVIDER_CENTER_Y: f32 = (DIVIDER_BOTTOM_Y + DIVIDER_TOP_Y) * 0.5;
const STROKE: f32 = 2.20;
const PLATE_SPEED: f32 = 0.15 * 4.0;
const DWELL: f32 = 3.0;
const TOP_DWELL: f32 = 3.0;
const RISE_SECONDS: f32 = STROKE / PLATE_SPEED;
const CYCLE: f32 = DWELL + RISE_SECONDS + TOP_DWELL + RISE_SECONDS;
const LIMIT_LOW: f32 = -0.02;
const LIMIT_HIGH: f32 = STROKE + 0.02;
const MAX_MOTOR_FORCE: f32 = 1.0e6;

const LEFT_CHAMBER_INNER_X: f32 = -0.55;
const DIVIDER_LEFT_X: f32 = 0.48;

const WATER_POLYGON: [Vec2; 4] = [
    Vec2::new(-0.486, 1.506),
    Vec2::new(0.414, 1.506),
    Vec2::new(0.414, 1.986),
    Vec2::new(-0.486, 1.986),
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

/// One horizontal shelf, the gap the liquid must find, and the spinner under that gap.
#[derive(Clone, Copy)]
struct Level {
    shelf_top: f32,
    hole_center_x: f32,
    spinner_center: Vec2,
}

impl Level {
    const fn shelf_bottom(self) -> f32 {
        self.shelf_top - SHELF_THICKNESS
    }

    const fn hole_left(self) -> f32 {
        self.hole_center_x - HOLE_HALF_WIDTH
    }

    const fn hole_right(self) -> f32 {
        self.hole_center_x + HOLE_HALF_WIDTH
    }
}

const _: () = {
    let top = LEVELS[0];
    let middle = LEVELS[1];
    let bottom = LEVELS[2];
    assert!(top.shelf_bottom() > middle.shelf_top);
    assert!(middle.shelf_bottom() > bottom.shelf_top);
    assert!(top.spinner_center.y + PADDLE_OUTER < top.shelf_bottom());
    assert!(middle.spinner_center.y + PADDLE_OUTER < middle.shelf_bottom());
    assert!(bottom.spinner_center.y + PADDLE_OUTER < bottom.shelf_bottom());
    assert!(top.hole_left() > LEFT_CHAMBER_INNER_X);
    assert!(bottom.hole_right() < DIVIDER_LEFT_X);
    assert!(PADDLE_OUTER - PADDLE_INNER >= 0.05);
    assert!(PADDLE_HALF_WIDTH * 2.0 >= 0.05);
    assert!(WATER_POLYGON[0].y > top.shelf_top);
    assert!(WATER_POLYGON[0].x > LEFT_CHAMBER_INNER_X);
    assert!(WATER_POLYGON[1].x < DIVIDER_LEFT_X);
    assert!(WATER_POLYGON[2].y + PARTICLE_RADIUS < DIVIDER_TOP_Y);
    assert!(PLATE_SLANT > 0.0);
    assert!(PLATE_CENTER.y + STROKE > DIVIDER_TOP_Y);
    assert!(PLATE_CENTER.y >= PLATE_HALF_HEIGHT + PLATE_FLOOR_CLEARANCE + PLATE_HALF_WIDTH * 0.035);
};

const LEVELS: [Level; LEVEL_COUNT] = [
    Level {
        shelf_top: 1.49,
        hole_center_x: -0.32,
        spinner_center: Vec2::new(-0.32, 1.23),
    },
    Level {
        shelf_top: 1.05,
        hole_center_x: 0.18,
        spinner_center: Vec2::new(0.18, 0.79),
    },
    Level {
        shelf_top: 0.61,
        hole_center_x: -0.32,
        spinner_center: Vec2::new(-0.32, 0.35),
    },
];

#[derive(Clone, Copy)]
struct BoxSpec {
    half_width: f32,
    half_height: f32,
    center: Vec2,
}

struct SpinnerRig {
    revolute: JointId,
    body: BodyId,
}

struct LiquidBubblerHooks {
    elapsed: f32,
    spinners: [SpinnerRig; LEVEL_COUNT],
    prismatic: JointId,
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

    let spinners = create_spinners(&mut world, ground)?;
    let plate = create_plate(&mut world)?;
    let prismatic = create_plate_joint(&mut world, ground, plate)?;
    let particle_system = create_water_group(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(LiquidBubblerHooks {
            elapsed: 0.0,
            spinners,
            prismatic,
            plate,
        }),
    })
}

fn chamber_boxes() -> [BoxSpec; 4] {
    [
        BoxSpec {
            half_width: WALL_HALF,
            half_height: WALL_HALF_HEIGHT,
            center: Vec2::new(-0.59, WALL_CENTER_Y),
        },
        BoxSpec {
            half_width: 0.835,
            half_height: WALL_HALF,
            center: Vec2::new(0.205, -WALL_HALF),
        },
        BoxSpec {
            half_width: WALL_HALF,
            half_height: WALL_HALF_HEIGHT,
            center: Vec2::new(1.00, WALL_CENTER_Y),
        },
        BoxSpec {
            half_width: WALL_HALF,
            half_height: DIVIDER_HALF_HEIGHT,
            center: Vec2::new(0.52, DIVIDER_CENTER_Y),
        },
    ]
}

fn shelf_boxes() -> [BoxSpec; LEVEL_COUNT * 2] {
    std::array::from_fn(|index| {
        let level = LEVELS[index / 2];
        if index % 2 == 0 {
            span_box(
                LEFT_CHAMBER_INNER_X,
                level.hole_left(),
                level.shelf_bottom(),
                level.shelf_top,
            )
        } else {
            span_box(
                level.hole_right(),
                DIVIDER_LEFT_X,
                level.shelf_bottom(),
                level.shelf_top,
            )
        }
    })
}

fn wall_boxes() -> Vec<BoxSpec> {
    let mut walls = chamber_boxes()
        .into_iter()
        .chain(shelf_boxes())
        .collect::<Vec<_>>();
    walls.push(span_box(0.48, 0.56, 0.0, 0.050));
    walls
}

fn span_box(left: f32, right: f32, bottom: f32, top: f32) -> BoxSpec {
    BoxSpec {
        half_width: (right - left) * 0.5,
        half_height: (top - bottom) * 0.5,
        center: Vec2::new((left + right) * 0.5, (bottom + top) * 0.5),
    }
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
            0.0,
        )?;
    }
    Ok(())
}

fn create_spinners(
    world: &mut World,
    ground: BodyId,
) -> Result<[SpinnerRig; LEVEL_COUNT], SceneError> {
    let mut built = Vec::with_capacity(LEVEL_COUNT);
    for level in LEVELS {
        built.push(create_spinner(world, ground, level.spinner_center)?);
    }
    built.try_into().map_err(|_built| SceneError::Body)
}

fn create_spinner(
    world: &mut World,
    ground: BodyId,
    center: Vec2,
) -> Result<SpinnerRig, SceneError> {
    let definition = BodyDef::new(BodyType::Dynamic, center, 0.0, true)
        .map_err(|_error| SceneError::Body)?
        .with_angular_damping(ANGULAR_DAMPING)
        .map_err(|_error| SceneError::Body)?
        .with_sleeping_allowed(false);
    let body = world
        .create_body(&definition)
        .map_err(|_error| SceneError::Body)?;
    attach_hub(world, body)?;
    for vertices in PADDLE_POLYGONS {
        attach_paddle(world, body, &vertices)?;
    }
    let revolute = pin_spinner(world, ground, body, center)?;
    Ok(SpinnerRig { revolute, body })
}

fn attach_hub(world: &mut World, spinner: BodyId) -> Result<(), SceneError> {
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
        .create_fixture(spinner, &definition)
        .map_err(|_error| SceneError::Fixture)?;
    Ok(())
}

fn attach_paddle(
    world: &mut World,
    spinner: BodyId,
    vertices: &[Vec2; 4],
) -> Result<(), SceneError> {
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
        .create_fixture(spinner, &definition)
        .map_err(|_error| SceneError::Fixture)?;
    Ok(())
}

fn pin_spinner(
    world: &mut World,
    ground: BodyId,
    spinner: BodyId,
    center: Vec2,
) -> Result<JointId, SceneError> {
    let joint = RevoluteJointDef::new(ground, spinner)
        .map_err(|_error| SceneError::Body)?
        .with_frame(center, Vec2::ZERO, 0.0)
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
    attach_plate_fixtures(world, plate)?;
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
    angle: f32,
) -> Result<(), SceneError> {
    let polygon = PolygonShape::oriented_box(half_width, half_height, center, angle)
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
        for spinner in &self.spinners {
            world
                .revolute_joint_angle(spinner.revolute)
                .map_err(|_error| SessionError::StepFailed)?;
        }
        world
            .set_prismatic_motor_speed(self.prismatic, scheduled_plate_speed(self.elapsed))
            .map_err(|_error| SessionError::StepFailed)
    }

    fn on_after_step(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
        _transitions: &[liquidfun::ContactTransition],
    ) -> Result<(), SessionError> {
        // Contact with the lowered wall can nudge the deck. Put it back on the schedule.
        let offset = scheduled_plate_offset(self.elapsed);
        world
            .set_body_transform(
                self.plate,
                Vec2::new(PLATE_CENTER.x, PLATE_CENTER.y + offset),
                0.0,
            )
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

        for spinner in &self.spinners {
            let transform = world
                .body_snapshot(spinner.body)
                .map_err(|_error| SessionError::FrameCaptureFailed)?
                .transform();
            for (start, end) in PADDLE_CENTERLINES {
                push_transformed_segment(&mut segments, transform, start, end);
            }
        }

        let plate_transform = world
            .body_snapshot(self.plate)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .transform();
        push_plate_segments(&mut segments, plate_transform);
        Ok(segments)
    }

    fn collect_circles(&self, world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        let mut circles = Vec::with_capacity(self.spinners.len());
        for spinner in &self.spinners {
            let transform = world
                .body_snapshot(spinner.body)
                .map_err(|_error| SessionError::FrameCaptureFailed)?
                .transform();
            circles.push((transform.apply(Vec2::ZERO), HUB_RADIUS));
        }
        Ok(circles)
    }
}

#[cfg(test)]
mod tests;
