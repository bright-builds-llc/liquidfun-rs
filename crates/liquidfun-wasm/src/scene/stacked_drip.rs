//! Stacked Drip: colored water lands on three motor-off trays from top to bottom.
//!
//! A side-shaft plate stays down through that cascade, then creeps upward later.

mod vessel;

use liquidfun::collision::{FilterData, PolygonShape, Shape};
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
const DRIP_COLOR: ParticleColor = ParticleColor::new(64, 196, 196, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const SIM_DT: f32 = 1.0 / 60.0;
const WALL_HALF: f32 = 0.04;
const WALL_FRICTION: f32 = 0.2;
const POUR_ANGLE: f32 = std::f32::consts::TAU / 8.0;
/// Slow enough that a poured tray is still past the angle floor at the 5 s sample.
const ANGULAR_DAMPING: f32 = 20.0;
const DECK_DENSITY: f32 = 1.0;
/// Just above the deck density, so an empty tray rests at the lower limit and a
/// loaded tray still pours.
const COUNTERWEIGHT_DENSITY: f32 = 1.05;
/// Rests one polygon-skin pair above the floor so a flush contact does not
/// pop the plate off translation 0 during the dwell.
const PLATE_FLOOR_CLEARANCE: f32 = 2.0 * 0.01;
const PLATE_HALF_WIDTH: f32 = 0.12;
const PLATE_HALF_HEIGHT: f32 = 0.04;
const PLATE_CENTER: Vec2 = Vec2::new(1.20, 0.04 + PLATE_FLOOR_CLEARANCE);
const PLATE_DENSITY: f32 = 1.0;
const STROKE: f32 = 1.90;
const PLATE_SPEED: f32 = 0.15;
const DWELL: f32 = 6.0;
const RISE_SECONDS: f32 = STROKE / PLATE_SPEED;
const CYCLE: f32 = DWELL + RISE_SECONDS + RISE_SECONDS;
const LIMIT_LOW: f32 = -0.02;
const LIMIT_HIGH: f32 = STROKE + 0.02;
const MAX_MOTOR_FORCE: f32 = 1.0e6;
const PROOF_SECONDS: f32 = 5.0;
const PROOF_BATCHES: u32 = 75;
const ANGLE_FLOOR: f32 = 0.05;
const DIVIDER_INNER_X: f32 = 0.90;
const TOP_TRAY_Y: f32 = 1.45;
const BOTTOM_TRAY_Y: f32 = 0.55;

const _: () = {
    assert!(PROOF_BATCHES == 75);
    assert!(PROOF_SECONDS > ANGLE_FLOOR);
};

const UPPER_PIVOT: Vec2 = Vec2::new(0.55, TOP_TRAY_Y);
const MIDDLE_PIVOT: Vec2 = Vec2::new(0.25, 1.00);
const LOWER_PIVOT: Vec2 = Vec2::new(-0.05, BOTTOM_TRAY_Y);

const WATER_POLYGON: [Vec2; 4] = [
    Vec2::new(0.20, 1.62),
    Vec2::new(0.55, 1.62),
    Vec2::new(0.55, 1.78),
    Vec2::new(0.20, 1.78),
];

const PLATE_LOCAL: vessel::LocalBox = vessel::LocalBox {
    half_width: PLATE_HALF_WIDTH,
    half_height: PLATE_HALF_HEIGHT,
    center: Vec2::ZERO,
    density: PLATE_DENSITY,
};

struct TrayRig {
    joint: JointId,
    body: BodyId,
}

struct StackedDripHooks {
    elapsed: f32,
    upper: TrayRig,
    middle: TrayRig,
    lower: TrayRig,
    prismatic: JointId,
    plate: BodyId,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_stacked_drip().map_err(|_error| SessionError::SceneConstruction)
}

fn build_stacked_drip() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    attach_wall_boxes(&mut world, ground)?;

    let upper = create_tray_rig(&mut world, ground, UPPER_PIVOT)?;
    let middle = create_tray_rig(&mut world, ground, MIDDLE_PIVOT)?;
    let lower = create_tray_rig(&mut world, ground, LOWER_PIVOT)?;
    let plate = create_plate(&mut world)?;
    let prismatic = create_plate_joint(&mut world, ground, plate)?;
    let particle_system = create_water_group(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(StackedDripHooks {
            elapsed: 0.0,
            upper,
            middle,
            lower,
            prismatic,
            plate,
        }),
    })
}

fn attach_wall_boxes(world: &mut World, ground: BodyId) -> Result<(), SceneError> {
    for wall in vessel::wall_boxes() {
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

fn create_tray_rig(world: &mut World, ground: BodyId, pivot: Vec2) -> Result<TrayRig, SceneError> {
    let definition = BodyDef::new(BodyType::Dynamic, pivot, 0.0, true)
        .map_err(|_error| SceneError::Body)?
        .with_angular_damping(ANGULAR_DAMPING)
        .map_err(|_error| SceneError::Body)?
        .with_sleeping_allowed(false);
    let body = world
        .create_body(&definition)
        .map_err(|_error| SceneError::Body)?;
    for fixture in vessel::tray_fixtures() {
        attach_box(
            world,
            body,
            fixture.half_width,
            fixture.half_height,
            fixture.center,
            fixture.density,
        )?;
    }
    let joint = RevoluteJointDef::new(ground, body)
        .map_err(|_error| SceneError::Body)?
        .with_frame(pivot, Vec2::ZERO, 0.0)
        .map_err(|_error| SceneError::Body)?
        .with_limits(true, 0.0, POUR_ANGLE)
        .map_err(|_error| SceneError::Body)?;
    let joint = world
        .create_joint(JointDef::from(joint))
        .map_err(|_error| SceneError::Body)?;
    Ok(TrayRig { joint, body })
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
        PLATE_LOCAL.half_width,
        PLATE_LOCAL.half_height,
        PLATE_LOCAL.center,
        PLATE_LOCAL.density,
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

fn push_body_boxes(
    segments: &mut Vec<RigidSegment>,
    world: &World,
    body: BodyId,
    fixtures: &[vessel::LocalBox],
) -> Result<(), SessionError> {
    let transform = world
        .body_snapshot(body)
        .map_err(|_error| SessionError::FrameCaptureFailed)?
        .transform();
    for fixture in fixtures {
        let corners = vessel::local_corners(*fixture);
        for index in 0..corners.len() {
            segments.push(RigidSegment {
                start: transform.apply(corners[index]),
                end: transform.apply(corners[(index + 1) % corners.len()]),
            });
        }
    }
    Ok(())
}

impl SceneHooks for StackedDripHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        self.elapsed += SIM_DT;
        world
            .revolute_joint_angle(self.upper.joint)
            .map_err(|_error| SessionError::StepFailed)?;
        world
            .revolute_joint_angle(self.middle.joint)
            .map_err(|_error| SessionError::StepFailed)?;
        world
            .revolute_joint_angle(self.lower.joint)
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
        for wall in vessel::wall_boxes() {
            push_box_outline(
                &mut segments,
                wall.center,
                wall.half_width,
                wall.half_height,
            );
        }
        let trays = vessel::tray_fixtures();
        push_body_boxes(&mut segments, world, self.upper.body, &trays)?;
        push_body_boxes(&mut segments, world, self.middle.body, &trays)?;
        push_body_boxes(&mut segments, world, self.lower.body, &trays)?;
        push_body_boxes(&mut segments, world, self.plate, &[PLATE_LOCAL])?;
        Ok(segments)
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests;
