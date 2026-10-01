//! Stacked Drip: colored water lands on three motor-off trays from top to bottom.
//!
//! A side-shaft plate stays down through that cascade, then eases up, pauses, and eases back down.

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

/// Twenty millimetres across.
const PARTICLE_DIAMETER: f32 = 0.020;
const PARTICLE_RADIUS: f32 = PARTICLE_DIAMETER * 0.5;
/// Substeps so this diameter still holds against the plate and the shaft walls.
pub(crate) const PARTICLE_ITERATIONS: u32 = 12;
const PARTICLE_DAMPING: f32 = 0.2;
const DRIP_COLOR: ParticleColor = ParticleColor::new(64, 196, 196, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const SIM_DT: f32 = 1.0 / 60.0;
const WALL_HALF: f32 = 0.04;
/// Tall enough that a splash from the reservoir cannot clear the side walls.
const WALL_TOP_Y: f32 = 3.20;
/// Lid below the side-wall tops, so a splash cannot leave the tank.
const CEILING_CENTER_Y: f32 = 3.05;
const WALL_HALF_HEIGHT: f32 = WALL_TOP_Y * 0.5;
const WALL_CENTER_Y: f32 = WALL_HALF_HEIGHT;
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
/// The plate overlaps each shaft wall by one polygon skin. A gap wide enough
/// for the skins to clear would also let these particles fall beside the plate.
const PLATE_WALL_OVERLAP: f32 = 0.01;
/// Same negative group as the divider and the right wall, so the overlap does
/// not jam the slide. The plate still rests on the floor.
const PLATE_FILTER: FilterData = FilterData::new(0x0001, 0xFFFF, -2);
const SHAFT_WALL_FILTER: FilterData = FilterData::new(0x0001, 0xFFFF, -2);
const RIGHT_WALL_CENTER_X: f32 = 1.56;
const RIGHT_WALL_INNER_X: f32 = RIGHT_WALL_CENTER_X - WALL_HALF;
const DIVIDER_INNER_X: f32 = 0.90;
const DIVIDER_CENTER_X: f32 = DIVIDER_INNER_X + WALL_HALF;
const DIVIDER_OUTER_X: f32 = DIVIDER_CENTER_X + WALL_HALF;
const PLATE_LEFT_X: f32 = DIVIDER_OUTER_X - PLATE_WALL_OVERLAP;
const PLATE_RIGHT_X: f32 = RIGHT_WALL_INNER_X + PLATE_WALL_OVERLAP;
const PLATE_HALF_WIDTH: f32 = (PLATE_RIGHT_X - PLATE_LEFT_X) * 0.5;
const PLATE_CENTER_X: f32 = (PLATE_LEFT_X + PLATE_RIGHT_X) * 0.5;
const PLATE_HALF_HEIGHT: f32 = 0.02;
const PLATE_CENTER: Vec2 = Vec2::new(PLATE_CENTER_X, PLATE_HALF_HEIGHT + PLATE_FLOOR_CLEARANCE);
const PLATE_REST_TOP: f32 = PLATE_HALF_HEIGHT + PLATE_FLOOR_CLEARANCE + PLATE_HALF_HEIGHT;
const PLATE_DENSITY: f32 = 1.0;
const STROKE: f32 = 1.90;
/// Peak speed at mid-stroke. Three times the previous 0.45 m/s cruise.
const PLATE_SPEED: f32 = 1.35;
const DWELL: f32 = 6.0;
/// Pause at the top of the stroke before the descent.
const TOP_DWELL: f32 = 3.0;
/// Peak of the smootherstep derivative, so the rise starts and ends at rest.
const EASE_PEAK: f32 = 15.0 / 8.0;
const RISE_SECONDS: f32 = EASE_PEAK * STROKE / PLATE_SPEED;
const CYCLE: f32 = DWELL + RISE_SECONDS + TOP_DWELL + RISE_SECONDS;
const LIMIT_LOW: f32 = -0.02;
const LIMIT_HIGH: f32 = STROKE + 0.02;
const MAX_MOTOR_FORCE: f32 = 1.0e6;
const PROOF_SECONDS: f32 = 5.0;
const PROOF_BATCHES: u32 = 75;
const ANGLE_FLOOR: f32 = 0.05;
const TOP_TRAY_Y: f32 = 1.45;
const BOTTOM_TRAY_Y: f32 = 0.55;
/// Open above the resting plate by more than one particle diameter.
const DIVIDER_BOTTOM_Y: f32 = 0.22;
/// Spill lip. The plate is held fully raised through the pause above this.
const SPILL_LIP_Y: f32 = 1.70;
const DIVIDER_HALF_HEIGHT: f32 = (SPILL_LIP_Y - DIVIDER_BOTTOM_Y) * 0.5;
const DIVIDER_CENTER_Y: f32 = (DIVIDER_BOTTOM_Y + SPILL_LIP_Y) * 0.5;
/// Halfway through the pause at the top, while liquid can still cross the lip.
#[cfg(test)]
const SPILL_SAMPLE_SECONDS: f32 = TOP_DWELL * 0.5;
const SAMPLE_PLATE_TOP: f32 = PLATE_REST_TOP + STROKE;
/// Outer face of the counterweight, in tray-local metres.
const COUNTERWEIGHT_OUTER_X: f32 = 0.22;
/// Air between that face and the divider, so the top tray reads flush.
const DIVIDER_CLEARANCE: f32 = 0.03;
const UPPER_PIVOT_X: f32 = DIVIDER_INNER_X - DIVIDER_CLEARANCE - COUNTERWEIGHT_OUTER_X;
const TRAY_PIVOT_STEP: f32 = 0.30;
/// The left floor falls toward the shaft so drained liquid boards the plate.
const SLOPE_HIGH_X: f32 = -0.70;
const SLOPE_HIGH_Y: f32 = 0.16;
const SLOPE_LOW_Y: f32 = 0.08;
const FLOOR_BOTTOM_Y: f32 = -2.0 * WALL_HALF;
const FLOOR_OUTER_LEFT_X: f32 = -0.78;
const FLOOR_OUTER_RIGHT_X: f32 = RIGHT_WALL_CENTER_X + WALL_HALF;

const _: () = {
    assert!(PROOF_BATCHES == 75);
    assert!(PROOF_SECONDS > ANGLE_FLOOR);
    assert!(DWELL > PROOF_SECONDS);
    assert!(TOP_DWELL >= 2.0);
    assert!(RISE_SECONDS > STROKE / PLATE_SPEED);
    assert!(PLATE_LEFT_X < DIVIDER_OUTER_X);
    assert!(PLATE_RIGHT_X > RIGHT_WALL_INNER_X);
    assert!(DIVIDER_BOTTOM_Y - PLATE_REST_TOP > PARTICLE_DIAMETER);
    assert!(SAMPLE_PLATE_TOP > SPILL_LIP_Y);
    assert!(PLATE_SPEED > 0.0 && PLATE_SPEED <= 2.0);
    assert!(DIVIDER_CLEARANCE > 0.02);
    assert!(UPPER_PIVOT_X + COUNTERWEIGHT_OUTER_X < DIVIDER_INNER_X);
    assert!(SLOPE_LOW_Y > PLATE_REST_TOP);
    assert!(SLOPE_LOW_Y < DIVIDER_BOTTOM_Y);
    assert!(SLOPE_HIGH_Y < BOTTOM_TRAY_Y);
    assert!(WALL_TOP_Y > SAMPLE_PLATE_TOP);
};

const UPPER_PIVOT: Vec2 = Vec2::new(UPPER_PIVOT_X, TOP_TRAY_Y);
const MIDDLE_PIVOT: Vec2 = Vec2::new(UPPER_PIVOT_X - TRAY_PIVOT_STEP, 1.00);
const LOWER_PIVOT: Vec2 = Vec2::new(UPPER_PIVOT_X - 2.0 * TRAY_PIVOT_STEP, BOTTOM_TRAY_Y);

const WATER_POLYGON: [Vec2; 4] = [
    Vec2::new(-0.45, 1.62),
    Vec2::new(0.75, 1.62),
    Vec2::new(0.75, 1.98),
    Vec2::new(-0.45, 1.98),
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
            if wall.shares_plate_group {
                SHAFT_WALL_FILTER
            } else {
                FilterData::default()
            },
        )?;
    }
    attach_polygon(world, ground, &vessel::floor_wedge())?;
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
            FilterData::default(),
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
        PLATE_FILTER,
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

fn attach_polygon(world: &mut World, body: BodyId, points: &[Vec2]) -> Result<(), SceneError> {
    let polygon = PolygonShape::new(points).map_err(|_error| SceneError::Geometry)?;
    let definition = FixtureDef::new(
        Shape::from(polygon),
        0.0,
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

fn attach_box(
    world: &mut World,
    body: BodyId,
    half_width: f32,
    half_height: f32,
    center: Vec2,
    density: f32,
    filter: FilterData,
) -> Result<(), SceneError> {
    let polygon = PolygonShape::oriented_box(half_width, half_height, center, 0.0)
        .map_err(|_error| SceneError::Geometry)?;
    let definition = FixtureDef::new(
        Shape::from(polygon),
        density,
        WALL_FRICTION,
        0.0,
        false,
        filter,
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
        return 0.0;
    }
    let after_bottom = phase - DWELL;
    if after_bottom < RISE_SECONDS {
        return eased_travel_speed(after_bottom / RISE_SECONDS);
    }
    let after_rise = after_bottom - RISE_SECONDS;
    if after_rise < TOP_DWELL {
        return 0.0;
    }
    let descent = after_rise - TOP_DWELL;
    -eased_travel_speed(descent / RISE_SECONDS)
}

/// Smootherstep speed: zero speed and zero acceleration at each end of the stroke.
fn eased_travel_speed(progress: f32) -> f32 {
    let unit = progress.clamp(0.0, 1.0);
    let toward_end = unit * (1.0 - unit);
    let shape = 30.0 * toward_end * toward_end;
    PLATE_SPEED * shape / EASE_PEAK
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

fn push_polygon_outline(segments: &mut Vec<RigidSegment>, corners: &[Vec2]) {
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
        push_polygon_outline(&mut segments, &vessel::floor_wedge());
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
