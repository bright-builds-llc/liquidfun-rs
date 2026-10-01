//! Hydraulic fountain: two kinematic plates slam into a pool and force a jet through their gap.
//!
//! The plates wait above the water, drop together, hold the squeeze, then rise and repeat.
//! The pool has no other outlet, so the displaced water leaves through the gap.

use liquidfun::collision::{FilterData, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, ParticleSystemDef, ParticleSystemId, World,
};

use super::{BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.025;
const PARTICLE_DAMPING: f32 = 0.2;
const PRESSURE_STRENGTH: f32 = 0.12;
const GROUP_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const SIM_DT: f32 = 1.0 / 60.0;
// One step of this speed moves less than a particle diameter, so the plates
// push the pool instead of tunneling through it.
const DESCEND_SPEED: f32 = 2.4;
const ASCEND_SPEED: f32 = 1.5;
const HOLD_HIGH: f32 = 33.0 / 60.0;
const HOLD_LOW: f32 = 75.0 / 60.0;
const WALL_FRICTION: f32 = 0.05;
const WALL_HALF_THICKNESS: f32 = 0.04;
const INNER_HALF_WIDTH: f32 = 1.02;
const FLOOR_TOP_Y: f32 = 0.0;
const WALL_TOP_Y: f32 = 2.85;
const GAP_HALF_WIDTH: f32 = 0.08;
const SIDE_CLEARANCE: f32 = 0.01;
const PLATE_HALF_HEIGHT: f32 = 0.055;
const PLATE_OUTER_X: f32 = INNER_HALF_WIDTH - SIDE_CLEARANCE;
const PLATE_INNER_X: f32 = GAP_HALF_WIDTH;
const PLATE_HALF_WIDTH: f32 = (PLATE_OUTER_X - PLATE_INNER_X) * 0.5;
const PLATE_CENTER_X: f32 = (PLATE_OUTER_X + PLATE_INNER_X) * 0.5;
const PRESSED_BOTTOM_Y: f32 = 0.055;
const RAISED_BOTTOM_Y: f32 = 1.22;
const STROKE: f32 = RAISED_BOTTOM_Y - PRESSED_BOTTOM_Y;
const RAISED_CENTER_Y: f32 = RAISED_BOTTOM_Y + PLATE_HALF_HEIGHT;
const PRESSED_CENTER_Y: f32 = PRESSED_BOTTOM_Y + PLATE_HALF_HEIGHT;
const DESCEND_WINDOW: f32 = STROKE / DESCEND_SPEED + 3.0 * SIM_DT;
const ASCEND_WINDOW: f32 = STROKE / ASCEND_SPEED + 3.0 * SIM_DT;
const CYCLE: f32 = HOLD_HIGH + DESCEND_WINDOW + HOLD_LOW + ASCEND_WINDOW;
const FILL_INSET_X: f32 = 0.045;
const FILL_BOTTOM_Y: f32 = 0.035;
const WATER_DEPTH: f32 = 0.46;
const PLATE_LOCAL_CORNERS: [Vec2; 4] = [
    Vec2::new(-PLATE_HALF_WIDTH, -PLATE_HALF_HEIGHT),
    Vec2::new(PLATE_HALF_WIDTH, -PLATE_HALF_HEIGHT),
    Vec2::new(PLATE_HALF_WIDTH, PLATE_HALF_HEIGHT),
    Vec2::new(-PLATE_HALF_WIDTH, PLATE_HALF_HEIGHT),
];

/// Inner faces of the floor and the two side walls.
const WALL_SEGMENTS: [RigidSegment; 3] = [
    RigidSegment {
        start: Vec2::new(-INNER_HALF_WIDTH, FLOOR_TOP_Y),
        end: Vec2::new(INNER_HALF_WIDTH, FLOOR_TOP_Y),
    },
    RigidSegment {
        start: Vec2::new(-INNER_HALF_WIDTH, FLOOR_TOP_Y),
        end: Vec2::new(-INNER_HALF_WIDTH, WALL_TOP_Y),
    },
    RigidSegment {
        start: Vec2::new(INNER_HALF_WIDTH, FLOOR_TOP_Y),
        end: Vec2::new(INNER_HALF_WIDTH, WALL_TOP_Y),
    },
];

struct HydraulicFountainHooks {
    elapsed: f32,
    plates: [BodyId; 2],
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_hydraulic_fountain().map_err(|_error| SessionError::SceneConstruction)
}

fn build_hydraulic_fountain() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    attach_wall_boxes(&mut world, ground)?;

    let plates = [
        create_plate(&mut world, -PLATE_CENTER_X)?,
        create_plate(&mut world, PLATE_CENTER_X)?,
    ];
    let particle_system = create_water_group(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(HydraulicFountainHooks {
            elapsed: 0.0,
            plates,
        }),
    })
}

enum PlateCommand {
    Hold,
    Move { target_y: f32, max_speed: f32 },
}

fn plate_command(elapsed: f32) -> PlateCommand {
    let cycle = elapsed.rem_euclid(CYCLE);
    if cycle < HOLD_HIGH {
        return PlateCommand::Hold;
    }
    if cycle < HOLD_HIGH + DESCEND_WINDOW {
        return PlateCommand::Move {
            target_y: PRESSED_CENTER_Y,
            max_speed: DESCEND_SPEED,
        };
    }
    if cycle < HOLD_HIGH + DESCEND_WINDOW + HOLD_LOW {
        return PlateCommand::Hold;
    }
    PlateCommand::Move {
        target_y: RAISED_CENTER_Y,
        max_speed: ASCEND_SPEED,
    }
}

fn plate_velocity(elapsed: f32, center_y: f32) -> f32 {
    let PlateCommand::Move {
        target_y,
        max_speed,
    } = plate_command(elapsed)
    else {
        return 0.0;
    };
    let delta = target_y - center_y;
    let max_step = max_speed * SIM_DT;
    delta.clamp(-max_step, max_step) / SIM_DT
}

fn attach_wall_boxes(world: &mut World, ground: BodyId) -> Result<(), SceneError> {
    let wall_half_height = WALL_TOP_Y * 0.5;
    let boxes = [
        (
            INNER_HALF_WIDTH + WALL_HALF_THICKNESS,
            WALL_HALF_THICKNESS,
            Vec2::new(0.0, -WALL_HALF_THICKNESS),
        ),
        (
            WALL_HALF_THICKNESS,
            wall_half_height,
            Vec2::new(-INNER_HALF_WIDTH - WALL_HALF_THICKNESS, wall_half_height),
        ),
        (
            WALL_HALF_THICKNESS,
            wall_half_height,
            Vec2::new(INNER_HALF_WIDTH + WALL_HALF_THICKNESS, wall_half_height),
        ),
    ];
    for (half_width, half_height, center) in boxes {
        attach_box(world, ground, half_width, half_height, center)?;
    }
    Ok(())
}

fn create_plate(world: &mut World, center_x: f32) -> Result<BodyId, SceneError> {
    let definition = BodyDef::new(
        BodyType::Kinematic,
        Vec2::new(center_x, RAISED_CENTER_Y),
        0.0,
        true,
    )
    .map_err(|_error| SceneError::Body)?
    .with_sleeping_allowed(false)
    .with_fixed_rotation(true);
    let plate = world
        .create_body(&definition)
        .map_err(|_error| SceneError::Body)?;
    attach_box(
        world,
        plate,
        PLATE_HALF_WIDTH,
        PLATE_HALF_HEIGHT,
        Vec2::ZERO,
    )?;
    Ok(plate)
}

fn attach_box(
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

fn water_polygon() -> [Vec2; 4] {
    let left = -INNER_HALF_WIDTH + FILL_INSET_X;
    let right = INNER_HALF_WIDTH - FILL_INSET_X;
    let top = FILL_BOTTOM_Y + WATER_DEPTH;
    [
        Vec2::new(left, FILL_BOTTOM_Y),
        Vec2::new(right, FILL_BOTTOM_Y),
        Vec2::new(right, top),
        Vec2::new(left, top),
    ]
}

fn create_water_group(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_damping(PARTICLE_DAMPING)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_pressure_strength(PRESSURE_STRENGTH)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_strict_contact_check(true);
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    let filled =
        Shape::from(PolygonShape::new(&water_polygon()).map_err(|_error| SceneError::Geometry)?);
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

impl SceneHooks for HydraulicFountainHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        self.elapsed += SIM_DT;
        for plate in self.plates {
            let center_y = world
                .body_snapshot(plate)
                .map_err(|_error| SessionError::StepFailed)?
                .transform()
                .position()
                .y;
            world
                .set_body_linear_velocity(
                    plate,
                    Vec2::new(0.0, plate_velocity(self.elapsed, center_y)),
                )
                .map_err(|_error| SessionError::StepFailed)?;
        }
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
        let mut segments = WALL_SEGMENTS.to_vec();
        for plate in self.plates {
            let transform = world
                .body_snapshot(plate)
                .map_err(|_error| SessionError::FrameCaptureFailed)?
                .transform();
            let world_corners = PLATE_LOCAL_CORNERS.map(|corner| transform.apply(corner));
            for index in 0..world_corners.len() {
                segments.push(RigidSegment {
                    start: world_corners[index],
                    end: world_corners[(index + 1) % world_corners.len()],
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
