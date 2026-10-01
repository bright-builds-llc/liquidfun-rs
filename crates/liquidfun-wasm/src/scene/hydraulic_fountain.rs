//! Hydraulic fountain: two kinematic plates squeeze a pool through the gap between them.
//!
//! The plates wait above the water, accelerate downward, pause, then rise slowly so the
//! water can drain back through the hole. Thick outer cheeks cover the side walls, so that
//! gap is the only outlet, and each lid leans two degrees toward the hole. The gap slider
//! moves the inner edges while those cheeks stay across the walls.

use std::f32::consts::TAU;

use liquidfun::collision::{FilterData, PolygonShape, Shape};
use liquidfun::math::{Rotation, Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, ParticleSystemDef, ParticleSystemId, World,
};

use super::{BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks};
use crate::session::SessionError;

/// Three times the previous 1.25 cm radius. The pool still holds 3,200 particles.
const PARTICLE_RADIUS: f32 = 0.0125 * 3.0;
const PARTICLE_COLUMNS: usize = 80;
const PARTICLE_ROWS: usize = 40;
const COLUMN_GAPS: f32 = grid_gaps(PARTICLE_COLUMNS);
const ROW_GAPS: f32 = grid_gaps(PARTICLE_ROWS);
/// Rest spacing is three quarters of the particle diameter, so the pool does not collapse.
const POOL_STRIDE: f32 = PARTICLE_RADIUS * 2.0 * 0.75;
const PARTICLE_DAMPING: f32 = 0.2;
const PRESSURE_STRENGTH: f32 = 0.25;
const GROUP_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const SIM_DT: f32 = 1.0 / 60.0;

// The press must stay under one particle diameter per step.
const _: () = assert!(PEAK_DESCEND_SPEED < PARTICLE_RADIUS * 2.0 / SIM_DT);
const HOLD_HIGH: f32 = 3.0;
const HOLD_LOW: f32 = 1.0;
const DESCEND_DURATION: f32 = 1.5;
const ASCEND_DURATION: f32 = 4.5;
const WALL_FRICTION: f32 = 0.05;
/// Below `tan(2°)`, so particles resting on the lid can drift toward the hole.
const PLATE_FRICTION: f32 = 0.02;
const WALL_HALF_THICKNESS: f32 = 0.1;
const POOL_HALF_SPAN: f32 = COLUMN_GAPS * POOL_STRIDE * 0.5;
const INNER_HALF_WIDTH: f32 = POOL_HALF_SPAN + 0.06;
const FLOOR_TOP_Y: f32 = 0.0;
/// Above the raised cheeks, so the plates stay inside the tank.
const WALL_TOP_Y: f32 = 3.05;
const GAP_CONTROL: &str = "gap";
/// Opening in tenths of a centimeter: 15 is 1.5 cm and 90 is 9.0 cm.
const GAP_TENTHS_MIN: i16 = 15;
const GAP_TENTHS_MAX: i16 = 90;
const GAP_TENTHS_DEFAULT: i16 = 90;
#[cfg(test)]
const GAP_HALF_WIDTH: f32 = 0.045;
const SLAB_HALF_HEIGHT: f32 = 0.09;
const CHEEK_THICKNESS: f32 = 0.16;
const CHEEK_DROP: f32 = 0.06;
const CHEEK_RISE: f32 = 0.06;
/// Outer face of a side wall. The plates end here, so they cover the wall.
const WALL_OUTER_X: f32 = INNER_HALF_WIDTH + 2.0 * WALL_HALF_THICKNESS;
const PLATE_OUTER_X: f32 = WALL_OUTER_X;
const POOL_BOTTOM_Y: f32 = 0.03;
const POOL_TOP_Y: f32 = POOL_BOTTOM_Y + ROW_GAPS * POOL_STRIDE;
const AIR_GAP: f32 = 0.4;
/// The cheeks stop just above the floor, so the press ends as low as the plates allow.
const PRESSED_CHEEK_CLEARANCE: f32 = 0.015;
const PRESSED_SLAB_BOTTOM_Y: f32 = PRESSED_CHEEK_CLEARANCE + CHEEK_DROP;
const RAISED_SLAB_BOTTOM_Y: f32 = POOL_TOP_Y + AIR_GAP;
const STROKE: f32 = RAISED_SLAB_BOTTOM_Y - PRESSED_SLAB_BOTTOM_Y;
const PEAK_DESCEND_SPEED: f32 = 2.0 * STROKE / DESCEND_DURATION;
const ASCEND_SPEED: f32 = STROKE / ASCEND_DURATION;
/// A little above the ease-in peak, and still under one particle diameter per step.
const MAX_PLATE_SPEED: f32 = PEAK_DESCEND_SPEED + 0.15;
const CYCLE: f32 = HOLD_HIGH + DESCEND_DURATION + HOLD_LOW + ASCEND_DURATION;
const CHEEK_HALF_WIDTH: f32 = CHEEK_THICKNESS * 0.5;
const CHEEK_HALF_HEIGHT: f32 = (CHEEK_DROP + CHEEK_RISE) * 0.5 + SLAB_HALF_HEIGHT;
const CHEEK_CENTER_Y: f32 = (CHEEK_RISE - CHEEK_DROP) * 0.5;
#[cfg(test)]
const RAISED_CHEEK_BOTTOM_Y: f32 = RAISED_SLAB_BOTTOM_Y - CHEEK_DROP;
const RAISED_CENTER_Y: f32 = RAISED_SLAB_BOTTOM_Y + SLAB_HALF_HEIGHT;
const PRESSED_CENTER_Y: f32 = PRESSED_SLAB_BOTTOM_Y + SLAB_HALF_HEIGHT;
const SLOPE_ANGLE: f32 = TAU / 180.0;
const SLOPE_EDGE_INSET: f32 = 0.015;
const SLOPE_HALF_THICKNESS: f32 = 0.05;
const SLOPE_OVERLAP: f32 = 0.012;

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

struct DrivenPlate {
    body: BodyId,
    toward_gap: f32,
}

struct HydraulicFountainHooks {
    elapsed: f32,
    gap_half: f32,
    plates: [DrivenPlate; 2],
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

    let gap_half = gap_half_from_tenths(GAP_TENTHS_DEFAULT);
    let plates = [
        create_plate(&mut world, 1.0, gap_half, RAISED_CENTER_Y)?,
        create_plate(&mut world, -1.0, gap_half, RAISED_CENTER_Y)?,
    ];
    let particle_system = create_water_group(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(HydraulicFountainHooks {
            elapsed: 0.0,
            gap_half,
            plates,
        }),
    })
}

fn slab_width(gap_half: f32) -> f32 {
    PLATE_OUTER_X - gap_half
}

fn slab_half_width(gap_half: f32) -> f32 {
    slab_width(gap_half) * 0.5
}

fn plate_center_x(gap_half: f32) -> f32 {
    (PLATE_OUTER_X + gap_half) * 0.5
}

fn signed_center_x(toward_gap: f32, gap_half: f32) -> f32 {
    -toward_gap * plate_center_x(gap_half)
}

fn slope_half_length(gap_half: f32) -> f32 {
    slab_half_width(gap_half) - SLOPE_EDGE_INSET
}

fn gap_half_from_tenths(tenths: i16) -> f32 {
    f32::from(tenths) / 2_000.0
}

fn scheduled_center_y(elapsed: f32) -> f32 {
    let cycle = elapsed.rem_euclid(CYCLE);
    if cycle < HOLD_HIGH {
        return RAISED_CENTER_Y;
    }
    let since_high = cycle - HOLD_HIGH;
    if since_high < DESCEND_DURATION {
        let progress = since_high / DESCEND_DURATION;
        let eased = progress * progress;
        return RAISED_CENTER_Y + (PRESSED_CENTER_Y - RAISED_CENTER_Y) * eased;
    }
    let since_descent = since_high - DESCEND_DURATION;
    if since_descent < HOLD_LOW {
        return PRESSED_CENTER_Y;
    }
    let since_low = since_descent - HOLD_LOW;
    if since_low < ASCEND_DURATION {
        return PRESSED_CENTER_Y + ASCEND_SPEED * since_low;
    }
    RAISED_CENTER_Y
}

fn plate_velocity(elapsed: f32, center_y: f32) -> f32 {
    let delta = scheduled_center_y(elapsed) - center_y;
    let max_step = MAX_PLATE_SPEED * SIM_DT;
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
        let polygon = PolygonShape::oriented_box(half_width, half_height, center, 0.0)
            .map_err(|_error| SceneError::Geometry)?;
        attach_polygon(world, ground, polygon, WALL_FRICTION)?;
    }
    Ok(())
}

fn create_plate(
    world: &mut World,
    toward_gap: f32,
    gap_half: f32,
    center_y: f32,
) -> Result<DrivenPlate, SceneError> {
    let definition = BodyDef::new(
        BodyType::Kinematic,
        Vec2::new(signed_center_x(toward_gap, gap_half), center_y),
        0.0,
        true,
    )
    .map_err(|_error| SceneError::Body)?
    .with_sleeping_allowed(false)
    .with_fixed_rotation(true);
    let body = world
        .create_body(&definition)
        .map_err(|_error| SceneError::Body)?;
    let slab =
        PolygonShape::oriented_box(slab_half_width(gap_half), SLAB_HALF_HEIGHT, Vec2::ZERO, 0.0)
            .map_err(|_error| SceneError::Geometry)?;
    attach_polygon(world, body, slab, PLATE_FRICTION)?;
    let cheek = PolygonShape::oriented_box(
        CHEEK_HALF_WIDTH,
        CHEEK_HALF_HEIGHT,
        cheek_center(toward_gap, gap_half),
        0.0,
    )
    .map_err(|_error| SceneError::Geometry)?;
    attach_polygon(world, body, cheek, PLATE_FRICTION)?;
    attach_polygon(
        world,
        body,
        slope_polygon(toward_gap, gap_half)?,
        PLATE_FRICTION,
    )?;
    Ok(DrivenPlate { body, toward_gap })
}

fn cheek_center(toward_gap: f32, gap_half: f32) -> Vec2 {
    let outer_x = -toward_gap * slab_half_width(gap_half);
    Vec2::new(outer_x + toward_gap * CHEEK_HALF_WIDTH, CHEEK_CENTER_Y)
}

fn slope_polygon(toward_gap: f32, gap_half: f32) -> Result<PolygonShape, SceneError> {
    let angle = -toward_gap * SLOPE_ANGLE;
    let rotation = Rotation::from_angle(angle);
    let half_length = slope_half_length(gap_half);
    let gap_bottom = rotation.apply(Vec2::new(toward_gap * half_length, -SLOPE_HALF_THICKNESS));
    let overlap = slab_width(gap_half) * SLOPE_ANGLE.sin() + SLOPE_OVERLAP;
    let lip = Vec2::new(
        toward_gap * (slab_half_width(gap_half) - SLOPE_EDGE_INSET),
        SLAB_HALF_HEIGHT - overlap,
    );
    PolygonShape::oriented_box(half_length, SLOPE_HALF_THICKNESS, lip - gap_bottom, angle)
        .map_err(|_error| SceneError::Geometry)
}

fn attach_polygon(
    world: &mut World,
    body: BodyId,
    polygon: PolygonShape,
    friction: f32,
) -> Result<(), SceneError> {
    let definition = FixtureDef::new(
        Shape::from(polygon),
        0.0,
        friction,
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

fn pool_positions() -> Vec<Vec2> {
    let mut positions = Vec::with_capacity(PARTICLE_COLUMNS * PARTICLE_ROWS);
    let mut y = POOL_BOTTOM_Y;
    for _row in 0..PARTICLE_ROWS {
        let mut x = -POOL_HALF_SPAN;
        for _column in 0..PARTICLE_COLUMNS {
            positions.push(Vec2::new(x, y));
            x += POOL_STRIDE;
        }
        y += POOL_STRIDE;
    }
    positions
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
    let source =
        ParticleGroupSource::positions(pool_positions()).map_err(|_error| SceneError::Particle)?;
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
        for plate in &self.plates {
            let center_y = world
                .body_snapshot(plate.body)
                .map_err(|_error| SessionError::StepFailed)?
                .transform()
                .position()
                .y;
            world
                .set_body_linear_velocity(
                    plate.body,
                    Vec2::new(0.0, plate_velocity(self.elapsed, center_y)),
                )
                .map_err(|_error| SessionError::StepFailed)?;
        }
        Ok(())
    }

    fn apply_control(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
        name: &str,
        value: &str,
    ) -> Result<ControlEffect, SessionError> {
        if name != GAP_CONTROL {
            return Err(SessionError::UnknownControl);
        }
        let Some(tenths) = parse_gap_tenths(value) else {
            return Err(SessionError::UnknownControl);
        };
        let gap_half = gap_half_from_tenths(tenths);
        if gap_half.to_bits() == self.gap_half.to_bits() {
            return Ok(ControlEffect::Live);
        }
        let left = replace_plate(world, &self.plates[0], gap_half)?;
        let right = replace_plate(world, &self.plates[1], gap_half)?;
        self.plates = [left, right];
        self.gap_half = gap_half;
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
        let mut segments = WALL_SEGMENTS.to_vec();
        for plate in &self.plates {
            let transform = world
                .body_snapshot(plate.body)
                .map_err(|_error| SessionError::FrameCaptureFailed)?
                .transform();
            push_loop(&mut segments, transform, &slab_corners(self.gap_half));
            push_loop(
                &mut segments,
                transform,
                &cheek_corners(plate.toward_gap, self.gap_half),
            );
            push_loop(
                &mut segments,
                transform,
                slope_polygon(plate.toward_gap, self.gap_half)
                    .map_err(|_error| SessionError::FrameCaptureFailed)?
                    .vertices(),
            );
        }
        Ok(segments)
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

fn slab_corners(gap_half: f32) -> [Vec2; 4] {
    box_corners(Vec2::ZERO, slab_half_width(gap_half), SLAB_HALF_HEIGHT)
}

fn cheek_corners(toward_gap: f32, gap_half: f32) -> [Vec2; 4] {
    box_corners(
        cheek_center(toward_gap, gap_half),
        CHEEK_HALF_WIDTH,
        CHEEK_HALF_HEIGHT,
    )
}

fn parse_gap_tenths(value: &str) -> Option<i16> {
    let (whole, fraction) = value.split_once('.')?;
    if whole.len() != 1 || fraction.len() != 1 {
        return None;
    }
    if !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let whole_digit = whole.parse::<i16>().ok()?;
    let fraction_digit = fraction.parse::<i16>().ok()?;
    let tenths = whole_digit * 10 + fraction_digit;
    if !(GAP_TENTHS_MIN..=GAP_TENTHS_MAX).contains(&tenths) {
        return None;
    }
    Some(tenths)
}

fn replace_plate(
    world: &mut World,
    plate: &DrivenPlate,
    gap_half: f32,
) -> Result<DrivenPlate, SessionError> {
    let snapshot = world
        .body_snapshot(plate.body)
        .map_err(|_error| SessionError::StepFailed)?;
    let center_y = snapshot.position().y;
    let velocity_y = snapshot.linear_velocity().y;
    world
        .destroy_body(plate.body)
        .map_err(|_error| SessionError::StepFailed)?;
    let created = create_plate(world, plate.toward_gap, gap_half, center_y)
        .map_err(|_error| SessionError::StepFailed)?;
    world
        .set_body_linear_velocity(created.body, Vec2::new(0.0, velocity_y))
        .map_err(|_error| SessionError::StepFailed)?;
    Ok(created)
}

fn box_corners(center: Vec2, half_width: f32, half_height: f32) -> [Vec2; 4] {
    [
        center + Vec2::new(-half_width, -half_height),
        center + Vec2::new(half_width, -half_height),
        center + Vec2::new(half_width, half_height),
        center + Vec2::new(-half_width, half_height),
    ]
}

fn push_loop(segments: &mut Vec<RigidSegment>, transform: Transform, corners: &[Vec2]) {
    for index in 0..corners.len() {
        let next = (index + 1) % corners.len();
        segments.push(RigidSegment {
            start: transform.apply(corners[index]),
            end: transform.apply(corners[next]),
        });
    }
}

#[cfg(test)]
mod tests;

const fn grid_gaps(count: usize) -> f32 {
    let mut gaps = 0.0;
    let mut remaining = count.saturating_sub(1);
    while remaining > 0 {
        gaps += 1.0;
        remaining -= 1;
    }
    gaps
}
