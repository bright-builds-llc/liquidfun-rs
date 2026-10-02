//! A 4 m pool: a dynamic end platform lifts one end and sends a wave along it.
//!
//! Motor speed is a sine of simulation time. Limits sit outside that stroke.
//! Width and slant rebuild the tank. Speed and amplitude change the live motor.
//! The plate slopes down toward the pool so water runs off the spill edge.

use liquidfun::collision::{FilterData, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, JointDef, JointId, ParticleSystemDef, ParticleSystemId,
    PrismaticJointDef, World,
};

use super::{BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks};
use crate::session::SessionError;

/// Lengths are the former 50 m pool, shrunk so the inner width is 4 m.
const LENGTH_SCALE: f32 = 4.0 / 50.0;
/// About 13 mm, the previous droplets at the 4 m scale.
const PARTICLE_RADIUS: f32 = 0.11 * 1.5 * LENGTH_SCALE;
/// `LiquidFun`'s usual spacing: 0.75 of the particle diameter.
const PARTICLE_SPACING: f32 = 1.5 * PARTICLE_RADIUS;
/// Full rows across the pool, then a short top row, for 2500 particles.
const PARTICLE_COLUMNS: u16 = 199;
const PARTICLE_ROWS: u16 = 13;
const PARTICLE_COUNT: usize = 2500;
const PARTICLE_DAMPING: f32 = 0.2;
const GROUP_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const SIM_DT: f32 = 1.0 / 60.0;
/// Five times the earlier period, so 0.4× rises in about three seconds.
const BASE_PERIOD: f32 = 2.4;
const DEFAULT_STROKE: f32 = 0.168;
/// Four tenths of the original unit rate.
const DEFAULT_SPEED: f32 = 0.4;
const LIMIT_MARGIN: f32 = 0.05 * LENGTH_SCALE;
const MAX_MOTOR_FORCE: f32 = 1.0e6;
const PLATFORM_DENSITY: f32 = 1.0;
const WALL_FRICTION: f32 = 0.2;
/// Several particle diameters, so water stays in the pool.
const WALL_HALF: f32 = 0.5 * LENGTH_SCALE;
/// The moving pad stays a thin plate on top of the floor.
const PAD_HALF: f32 = 0.1 * LENGTH_SCALE;
const DEFAULT_PLATFORM_WIDTH: f32 = 0.92;
const AMPLITUDE_MAX: f32 = 4.0 * LENGTH_SCALE;
const DEFAULT_SLANT_DEGREES: u16 = 5;
const SLANT_MAX_DEGREES: u16 = 30;
const SLAB_THICKNESS: f32 = PAD_HALF * 2.0;
const PLATFORM_WIDTH_CONTROL: &str = "platform-width";
const PLATFORM_SLANT_CONTROL: &str = "platform-slant";
const PLATFORM_SPEED_CONTROL: &str = "platform-speed";
const PLATFORM_AMPLITUDE_CONTROL: &str = "platform-amplitude";
const NEAR_WALL_INNER_X: f32 = 0.0;
/// The platform and the tank share a negative group so the slab can overlap
/// the near wall and the fixed floor without sticking.
const TANK_FILTER: FilterData = FilterData::new(0x0001, 0xffff, -1);
/// Far wall stays put while the platform width slider moves the joint anchor.
const TANK_RIGHT: f32 = 50.0 * LENGTH_SCALE;
const FLOOR_BOTTOM_Y: f32 = -(WALL_HALF * 2.0);
/// Side walls clear the deeper pool plus a full stroke and some splash.
const WALL_TOP_Y: f32 = 12.0 * LENGTH_SCALE;
const FILL_INSET: f32 = 2.0 * PARTICLE_SPACING;

struct WallBox {
    half_width: f32,
    half_height: f32,
    center: Vec2,
}

struct WaveLayout {
    anchor: Vec2,
    pad_center: Vec2,
    collision_center: Vec2,
    half_width: f32,
    collision_half_height: f32,
    /// Positive radians raise the back wall and leave the spill edge at local origin.
    slant: f32,
}

struct PlatePose {
    center: Vec2,
    angle: f32,
}

struct WaveTankHooks {
    phase: f32,
    speed: f32,
    stroke: f32,
    joint: JointId,
    platform: BodyId,
    layout: WaveLayout,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let (width, slant) = construction_from_presets(presets)?;
    build_wave_tank(width, slant).map_err(|_error| SessionError::SceneConstruction)
}

fn construction_from_presets(presets: &[(String, String)]) -> Result<(f32, f32), SessionError> {
    let mut width = DEFAULT_PLATFORM_WIDTH;
    let mut slant = default_slant();
    for (name, value) in presets {
        match name.as_str() {
            PLATFORM_WIDTH_CONTROL => {
                width = slider::parse_platform_width(value).ok_or(SessionError::UnknownControl)?;
            }
            PLATFORM_SLANT_CONTROL => {
                let degrees = slider::parse_degrees(value, SLANT_MAX_DEGREES)
                    .ok_or(SessionError::UnknownControl)?;
                slant = slant_radians(degrees);
            }
            _ => return Err(SessionError::UnknownControl),
        }
    }
    Ok((width, slant))
}

fn default_slant() -> f32 {
    slant_radians(f32::from(DEFAULT_SLANT_DEGREES))
}

fn slant_radians(degrees: f32) -> f32 {
    degrees * std::f32::consts::TAU / 360.0
}

fn build_wave_tank(platform_width: f32, slant: f32) -> Result<BuiltScene, SceneError> {
    let layout = wave_layout(platform_width, slant);
    if layout.half_width <= 0.0 {
        return Err(SceneError::Geometry);
    }

    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    attach_static_walls(&mut world, ground)?;

    let platform = create_platform(&mut world, &layout)?;
    let joint = create_platform_joint(&mut world, ground, platform, layout.anchor)?;
    let particle_system = create_water_group(&mut world, platform_width, slant)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(WaveTankHooks {
            phase: 0.0,
            speed: DEFAULT_SPEED,
            stroke: DEFAULT_STROKE,
            joint,
            platform,
            layout,
        }),
    })
}

/// The pad stays solid down through the floor at the tallest slider stroke.
fn skirt_depth() -> f32 {
    AMPLITUDE_MAX + LIMIT_MARGIN + (WALL_HALF * 3.0)
}

fn wave_layout(platform_width: f32, slant: f32) -> WaveLayout {
    // Local x is relative to the anchor. At rest the spill edge is the floor top.
    // A skirt below that stays inside the floor so water spills off only when raised.
    let slab_left = -platform_width;
    let slab_right = 0.0;
    let half_width = (slab_right - slab_left) * 0.5;
    let center_x = (slab_left + slab_right) * 0.5;
    let skirt_depth = skirt_depth();
    let collision_depth = skirt_depth + SLAB_THICKNESS;
    WaveLayout {
        anchor: Vec2::new(platform_width, 0.0),
        pad_center: Vec2::new(center_x, -SLAB_THICKNESS * 0.5),
        collision_center: Vec2::new(center_x, -collision_depth * 0.5),
        half_width,
        collision_half_height: collision_depth * 0.5,
        slant,
    }
}

fn tank_boxes() -> [WallBox; 3] {
    let wall_half_height = (WALL_TOP_Y - FLOOR_BOTTOM_Y) * 0.5;
    let wall_center_y = (WALL_TOP_Y + FLOOR_BOTTOM_Y) * 0.5;
    let outer_left = NEAR_WALL_INNER_X - WALL_HALF;
    let outer_right = TANK_RIGHT + WALL_HALF;
    [
        WallBox {
            half_width: WALL_HALF,
            half_height: wall_half_height,
            center: Vec2::new(outer_left + WALL_HALF, wall_center_y),
        },
        WallBox {
            half_width: (outer_right - outer_left) * 0.5,
            half_height: WALL_HALF,
            center: Vec2::new((outer_left + outer_right) * 0.5, FLOOR_BOTTOM_Y + WALL_HALF),
        },
        WallBox {
            half_width: WALL_HALF,
            half_height: wall_half_height,
            center: Vec2::new(outer_right - WALL_HALF, wall_center_y),
        },
    ]
}

fn motor_speed(phase: f32, speed: f32, stroke: f32) -> f32 {
    let peak = stroke * speed * std::f32::consts::TAU / (2.0 * BASE_PERIOD);
    let angle = phase * std::f32::consts::TAU / BASE_PERIOD;
    peak * angle.sin()
}

fn attach_static_walls(world: &mut World, ground: BodyId) -> Result<(), SceneError> {
    for wall in tank_boxes() {
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

fn create_platform(world: &mut World, layout: &WaveLayout) -> Result<BodyId, SceneError> {
    let definition = BodyDef::new(BodyType::Dynamic, layout.anchor, 0.0, true)
        .map_err(|_error| SceneError::Body)?
        .with_sleeping_allowed(false);
    let platform = world
        .create_body(&definition)
        .map_err(|_error| SceneError::Body)?;
    attach_box(
        world,
        platform,
        layout.half_width,
        layout.collision_half_height,
        layout.collision_center,
        0.0,
        PLATFORM_DENSITY,
    )?;
    let pose = plate_pose(layout);
    attach_box(
        world,
        platform,
        layout.half_width,
        PAD_HALF,
        pose.center,
        pose.angle,
        PLATFORM_DENSITY,
    )?;
    Ok(platform)
}

fn create_platform_joint(
    world: &mut World,
    ground: BodyId,
    platform: BodyId,
    anchor: Vec2,
) -> Result<JointId, SceneError> {
    let definition = PrismaticJointDef::new(ground, platform)
        .map_err(|_error| SceneError::Body)?
        .with_collide_connected(true)
        .with_frame(anchor, Vec2::ZERO, Vec2::new(0.0, 1.0), 0.0)
        .map_err(|_error| SceneError::Body)?
        .with_limits(true, 0.0, DEFAULT_STROKE + LIMIT_MARGIN)
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
    angle: f32,
    density: f32,
) -> Result<(), SceneError> {
    let polygon = PolygonShape::oriented_box(half_width, half_height, center, angle)
        .map_err(|_error| SceneError::Geometry)?;
    let definition = FixtureDef::new(
        Shape::from(polygon),
        density,
        WALL_FRICTION,
        0.0,
        false,
        TANK_FILTER,
    )
    .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(body, &definition)
        .map_err(|_error| SceneError::Fixture)?;
    Ok(())
}

fn pool_positions(platform_width: f32, slant: f32) -> Result<Vec<Vec2>, SceneError> {
    let left = NEAR_WALL_INNER_X + FILL_INSET;
    // Sit one diameter above the surface. On the plate that surface is the slope.
    let clearance = PARTICLE_RADIUS * 2.0;
    let tan_slant = slant.tan();
    let mut positions = Vec::new();
    positions
        .try_reserve_exact(PARTICLE_COUNT)
        .map_err(|_error| SceneError::Particle)?;
    for row in 0..PARTICLE_ROWS {
        for column in 0..PARTICLE_COLUMNS {
            if positions.len() == PARTICLE_COUNT {
                return Ok(positions);
            }
            let x = left + f32::from(column) * PARTICLE_SPACING;
            let slope = if x < platform_width {
                (platform_width - x) * tan_slant
            } else {
                0.0
            };
            positions.push(Vec2::new(
                x,
                clearance + slope + f32::from(row) * PARTICLE_SPACING,
            ));
        }
    }
    Ok(positions)
}

fn create_water_group(
    world: &mut World,
    platform_width: f32,
    slant: f32,
) -> Result<ParticleSystemId, SceneError> {
    let particle_count = PARTICLE_COUNT;
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_damping(PARTICLE_DAMPING)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(particle_count)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    let source = ParticleGroupSource::positions(pool_positions(platform_width, slant)?)
        .map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_color(GROUP_COLOR)
        .with_transform(Transform::IDENTITY)
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(system)
}

/// The plate slopes down to the spill edge. The right-hand wall stays vertical
/// and as thick as the tank walls, with its top on that edge.
fn append_platform_segments(
    segments: &mut Vec<RigidSegment>,
    transform: Transform,
    layout: &WaveLayout,
    rise: f32,
) {
    append_box_segments(segments, transform, plate_corners(layout));
    let face = platform_face(layout, rise);
    append_box_segments(
        segments,
        transform,
        box_corners(face.center, face.half_width, face.half_height),
    );
}

/// Clockwise about the spill edge, so a positive slant raises the back.
fn plate_pose(layout: &WaveLayout) -> PlatePose {
    let angle = -layout.slant;
    PlatePose {
        center: rotate_about_origin(Vec2::new(-layout.half_width, -PAD_HALF), angle),
        angle,
    }
}

fn plate_corners(layout: &WaveLayout) -> [Vec2; 4] {
    let pose = plate_pose(layout);
    oriented_corners(pose.center, layout.half_width, PAD_HALF, pose.angle)
}

fn oriented_corners(center: Vec2, half_width: f32, half_height: f32, angle: f32) -> [Vec2; 4] {
    box_corners(Vec2::ZERO, half_width, half_height)
        .map(|corner| center + rotate_about_origin(corner, angle))
}

fn rotate_about_origin(point: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    Vec2::new(point.x * cos - point.y * sin, point.x * sin + point.y * cos)
}

fn platform_face(layout: &WaveLayout, rise: f32) -> WallBox {
    let slab_right = layout.pad_center.x + layout.half_width;
    let slab_left = layout.pad_center.x - layout.half_width;
    let thickness = (WALL_HALF * 2.0).min(slab_right - slab_left);
    let face_bottom = FLOOR_BOTTOM_Y - rise.max(0.0);
    WallBox {
        half_width: thickness * 0.5,
        half_height: (0.0 - face_bottom) * 0.5,
        center: Vec2::new(slab_right - (thickness * 0.5), face_bottom * 0.5),
    }
}

fn append_box_segments(segments: &mut Vec<RigidSegment>, transform: Transform, corners: [Vec2; 4]) {
    let world_corners = corners.map(|corner| transform.apply(corner));
    for index in 0..world_corners.len() {
        segments.push(RigidSegment {
            start: world_corners[index],
            end: world_corners[(index + 1) % world_corners.len()],
        });
    }
}

fn box_corners(center: Vec2, half_width: f32, half_height: f32) -> [Vec2; 4] {
    [
        Vec2::new(center.x - half_width, center.y - half_height),
        Vec2::new(center.x + half_width, center.y - half_height),
        Vec2::new(center.x + half_width, center.y + half_height),
        Vec2::new(center.x - half_width, center.y + half_height),
    ]
}

fn write_motor(
    world: &mut World,
    joint: JointId,
    phase: f32,
    speed: f32,
    stroke: f32,
) -> Result<(), SessionError> {
    let mut commanded = motor_speed(phase, speed, stroke);
    let translation = world
        .prismatic_joint_translation(joint)
        .map_err(|_error| SessionError::StepFailed)?;
    // The slab sits on the floor. A downward motor would push water through it.
    if translation <= 0.0 && commanded < 0.0 {
        commanded = 0.0;
    }
    world
        .set_prismatic_motor_speed(joint, commanded)
        .map_err(|_error| SessionError::StepFailed)
}

impl SceneHooks for WaveTankHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        self.phase += self.speed * SIM_DT;
        write_motor(world, self.joint, self.phase, self.speed, self.stroke)
    }

    fn apply_control(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
        name: &str,
        value: &str,
    ) -> Result<ControlEffect, SessionError> {
        match name {
            PLATFORM_WIDTH_CONTROL => {
                if slider::parse_platform_width(value).is_none() {
                    return Err(SessionError::UnknownControl);
                }
                return Ok(ControlEffect::Recreated);
            }
            PLATFORM_SLANT_CONTROL => {
                if slider::parse_degrees(value, SLANT_MAX_DEGREES).is_none() {
                    return Err(SessionError::UnknownControl);
                }
                return Ok(ControlEffect::Recreated);
            }
            PLATFORM_SPEED_CONTROL => {
                let Some(speed) = slider::parse_platform_speed(value) else {
                    return Err(SessionError::UnknownControl);
                };
                self.speed = speed;
            }
            PLATFORM_AMPLITUDE_CONTROL => {
                let Some(stroke) = slider::parse_platform_amplitude(value) else {
                    return Err(SessionError::UnknownControl);
                };
                self.stroke = stroke;
                world
                    .set_prismatic_limits(self.joint, 0.0, stroke + LIMIT_MARGIN)
                    .map_err(|_error| SessionError::StepFailed)?;
            }
            _ => return Err(SessionError::UnknownControl),
        }

        write_motor(world, self.joint, self.phase, self.speed, self.stroke)?;
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
        let mut segments = Vec::new();
        for wall in tank_boxes() {
            append_box_segments(
                &mut segments,
                Transform::IDENTITY,
                box_corners(wall.center, wall.half_width, wall.half_height),
            );
        }
        let transform = world
            .body_snapshot(self.platform)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .transform();
        append_platform_segments(
            &mut segments,
            transform,
            &self.layout,
            transform.position().y,
        );
        Ok(segments)
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

mod slider;

#[cfg(test)]
mod tests;
