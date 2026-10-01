//! Sinusoidal wave tank: a dynamic end platform lifts one end of a still pool.
//!
//! Motor speed is a sine of simulation time. Limits sit outside that stroke.
//! The pool is one meter wide, one default water group, and the rest of the
//! floor stays fixed. Lengths keep the original 1.4 m layout at that scale.

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

/// Inner width from the near wall face to the far wall face.
const POOL_WIDTH: f32 = 1.0;
/// Uniform scale from the original 1.4 m pool onto [`POOL_WIDTH`].
const LENGTH_SCALE: f32 = POOL_WIDTH / 1.4;
const PARTICLE_RADIUS: f32 = 0.025 * LENGTH_SCALE;
const PARTICLE_DAMPING: f32 = 0.2;
const GROUP_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const SIM_DT: f32 = 1.0 / 60.0;
const PERIOD: f32 = 2.0;
const STROKE: f32 = 0.16 * LENGTH_SCALE;
const PEAK_SPEED: f32 = STROKE * std::f32::consts::TAU / (2.0 * PERIOD);
const LIMIT_LOW: f32 = -0.02 * LENGTH_SCALE;
const LIMIT_HIGH: f32 = STROKE + 0.02 * LENGTH_SCALE;
const MAX_MOTOR_FORCE: f32 = 1.0e6;
const PLATFORM_DENSITY: f32 = 1.0;
const WALL_FRICTION: f32 = 0.2;
const WALL_HALF_THICKNESS: f32 = 0.02 * LENGTH_SCALE;
const PLATFORM_WIDTH: f32 = 0.50 * LENGTH_SCALE;
const CHANNEL_LENGTH: f32 = 0.90 * LENGTH_SCALE;
const WATER_DEPTH: f32 = 0.32 * LENGTH_SCALE;
const ANCHOR_X: f32 = 0.50 * LENGTH_SCALE;
const ANCHOR: Vec2 = Vec2::new(ANCHOR_X, 0.0);
const NEAR_WALL_INNER_X: f32 = 0.0;
const FAR_WALL_INNER_X: f32 = ANCHOR_X + CHANNEL_LENGTH;
const FLOOR_TOP_Y: f32 = 0.0;
const WALL_TOP_Y: f32 = 0.90 * LENGTH_SCALE;
const FILL_INSET: f32 = 0.06 * LENGTH_SCALE;
const WALL_HALF_HEIGHT: f32 = WALL_TOP_Y * 0.5;
const SLAB_LEFT_LOCAL_X: f32 = NEAR_WALL_INNER_X + WALL_HALF_THICKNESS - ANCHOR_X;
const PLATFORM_RIGHT_LOCAL_X: f32 = SLAB_LEFT_LOCAL_X + PLATFORM_WIDTH;
const SLAB_RIGHT_LOCAL_X: f32 = -WALL_HALF_THICKNESS;
const SLAB_CENTER: Vec2 = Vec2::new(
    (SLAB_LEFT_LOCAL_X + SLAB_RIGHT_LOCAL_X) * 0.5,
    -WALL_HALF_THICKNESS,
);
const SLAB_HALF_WIDTH: f32 = (SLAB_RIGHT_LOCAL_X - SLAB_LEFT_LOCAL_X) * 0.5;
const FACE_CENTER: Vec2 = Vec2::new(
    PLATFORM_RIGHT_LOCAL_X - WALL_HALF_THICKNESS,
    -0.08 * LENGTH_SCALE,
);
const FACE_HALF_HEIGHT: f32 = 0.12 * LENGTH_SCALE;

const WALL_SEGMENTS: [RigidSegment; 3] = [
    RigidSegment {
        start: Vec2::new(NEAR_WALL_INNER_X, FLOOR_TOP_Y),
        end: Vec2::new(NEAR_WALL_INNER_X, WALL_TOP_Y),
    },
    RigidSegment {
        start: Vec2::new(ANCHOR_X, FLOOR_TOP_Y),
        end: Vec2::new(FAR_WALL_INNER_X, FLOOR_TOP_Y),
    },
    RigidSegment {
        start: Vec2::new(FAR_WALL_INNER_X, FLOOR_TOP_Y),
        end: Vec2::new(FAR_WALL_INNER_X, WALL_TOP_Y),
    },
];

struct WaveTankHooks {
    elapsed: f32,
    joint: JointId,
    platform: BodyId,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_wave_tank().map_err(|_error| SessionError::SceneConstruction)
}

fn build_wave_tank() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    attach_static_walls(&mut world, ground)?;

    let platform = create_platform(&mut world)?;
    let joint = create_platform_joint(&mut world, ground, platform)?;
    let particle_system = create_water_group(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(WaveTankHooks {
            elapsed: 0.0,
            joint,
            platform,
        }),
    })
}

fn sinusoidal_motor_speed(elapsed: f32) -> f32 {
    let phase = elapsed * std::f32::consts::TAU / PERIOD;
    PEAK_SPEED * phase.sin()
}

fn attach_static_walls(world: &mut World, ground: BodyId) -> Result<(), SceneError> {
    let boxes = [
        (
            WALL_HALF_THICKNESS,
            WALL_HALF_HEIGHT,
            Vec2::new(NEAR_WALL_INNER_X - WALL_HALF_THICKNESS, WALL_HALF_HEIGHT),
        ),
        (
            CHANNEL_LENGTH * 0.5,
            WALL_HALF_THICKNESS,
            Vec2::new(
                ANCHOR_X + CHANNEL_LENGTH * 0.5,
                FLOOR_TOP_Y - WALL_HALF_THICKNESS,
            ),
        ),
        (
            WALL_HALF_THICKNESS,
            WALL_HALF_HEIGHT,
            Vec2::new(FAR_WALL_INNER_X + WALL_HALF_THICKNESS, WALL_HALF_HEIGHT),
        ),
    ];
    for (half_width, half_height, center) in boxes {
        attach_box(world, ground, half_width, half_height, center, 0.0)?;
    }
    Ok(())
}

fn create_platform(world: &mut World) -> Result<BodyId, SceneError> {
    let definition = BodyDef::new(BodyType::Dynamic, ANCHOR, 0.0, true)
        .map_err(|_error| SceneError::Body)?
        .with_sleeping_allowed(false);
    let platform = world
        .create_body(&definition)
        .map_err(|_error| SceneError::Body)?;
    attach_box(
        world,
        platform,
        SLAB_HALF_WIDTH,
        WALL_HALF_THICKNESS,
        SLAB_CENTER,
        PLATFORM_DENSITY,
    )?;
    attach_box(
        world,
        platform,
        WALL_HALF_THICKNESS,
        FACE_HALF_HEIGHT,
        FACE_CENTER,
        PLATFORM_DENSITY,
    )?;
    Ok(platform)
}

fn create_platform_joint(
    world: &mut World,
    ground: BodyId,
    platform: BodyId,
) -> Result<JointId, SceneError> {
    let definition = PrismaticJointDef::new(ground, platform)
        .map_err(|_error| SceneError::Body)?
        .with_collide_connected(true)
        .with_frame(ANCHOR, Vec2::ZERO, Vec2::new(0.0, 1.0), 0.0)
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

fn water_polygon() -> [Vec2; 4] {
    let left = NEAR_WALL_INNER_X + FILL_INSET;
    let right = FAR_WALL_INNER_X - FILL_INSET;
    let bottom = FILL_INSET;
    let top = FILL_INSET + WATER_DEPTH;
    [
        Vec2::new(left, bottom),
        Vec2::new(right, bottom),
        Vec2::new(right, top),
        Vec2::new(left, top),
    ]
}

fn create_water_group(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_damping(PARTICLE_DAMPING)
        .map_err(|_error| SceneError::ParticleSystem)?;
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

impl SceneHooks for WaveTankHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        self.elapsed += SIM_DT;
        world
            .set_prismatic_motor_speed(self.joint, sinusoidal_motor_speed(self.elapsed))
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
        let mut segments = WALL_SEGMENTS.to_vec();
        let transform = world
            .body_snapshot(self.platform)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .transform();
        append_box_segments(
            &mut segments,
            transform,
            box_corners(SLAB_CENTER, SLAB_HALF_WIDTH, WALL_HALF_THICKNESS),
        );
        append_box_segments(
            &mut segments,
            transform,
            box_corners(FACE_CENTER, WALL_HALF_THICKNESS, FACE_HALF_HEIGHT),
        );
        Ok(segments)
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests;
