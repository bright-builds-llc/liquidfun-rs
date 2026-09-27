//! Drawing Particles and Sparky are experimental native Rust ports of the pinned `LiquidFun` tests. The playground shows recognizable behavior. Catalog previews are static illustrations.

#[cfg(test)]
mod tests;

use liquidfun::collision::{CircleShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupFlags,
    ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{BodyDef, ParticleGroupId, ParticleSystemDef, ParticleSystemId, World};

use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
    attach_basin_fixture,
};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.05;
const BRUSH_RADIUS: f32 = 0.2;
const MAXIMUM_PARTICLE_COUNT: usize = 10240;
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const WATER_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const ELASTIC_COLOR: ParticleColor = ParticleColor::new(61, 220, 151, 255);
const MATERIAL_CONTROL: &str = "material";

const FLOOR: [Vec2; 4] = [
    Vec2::new(-4.0, -2.0),
    Vec2::new(4.0, -2.0),
    Vec2::new(4.0, 0.0),
    Vec2::new(-4.0, 0.0),
];
const LEFT_WALL: [Vec2; 4] = [
    Vec2::new(-4.0, -2.0),
    Vec2::new(-2.0, -2.0),
    Vec2::new(-2.0, 6.0),
    Vec2::new(-4.0, 6.0),
];
const RIGHT_WALL: [Vec2; 4] = [
    Vec2::new(2.0, -2.0),
    Vec2::new(4.0, -2.0),
    Vec2::new(4.0, 6.0),
    Vec2::new(2.0, 6.0),
];
const CEILING: [Vec2; 4] = [
    Vec2::new(-4.0, 4.0),
    Vec2::new(4.0, 4.0),
    Vec2::new(4.0, 6.0),
    Vec2::new(-4.0, 6.0),
];
const WALLS: [[Vec2; 4]; 4] = [FLOOR, LEFT_WALL, RIGHT_WALL, CEILING];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Material {
    Water,
    Elastic,
}

impl Material {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "water" => Some(Self::Water),
            "elastic" => Some(Self::Elastic),
            _ => None,
        }
    }

    const fn particle_flags(self) -> ParticleFlags {
        match self {
            Self::Water => ParticleFlags::WATER,
            Self::Elastic => ParticleFlags::ELASTIC,
        }
    }

    const fn group_flags(self) -> ParticleGroupFlags {
        match self {
            Self::Water => ParticleGroupFlags::empty(),
            Self::Elastic => ParticleGroupFlags::SOLID,
        }
    }

    const fn color(self) -> ParticleColor {
        match self {
            Self::Water => WATER_COLOR,
            Self::Elastic => ELASTIC_COLOR,
        }
    }
}

struct DrawingParticlesHooks {
    wall_segments: [RigidSegment; 16],
    maybe_last_group: Option<ParticleGroupId>,
    stroke_open: bool,
    material: Material,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_drawing().map_err(|_error| SessionError::SceneConstruction)
}

fn build_drawing() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;
    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    for vertices in WALLS {
        attach_basin_fixture(&mut world, ground, &vertices)?;
    }
    let particle_system = create_particle_system(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(DrawingParticlesHooks {
            wall_segments: wall_segments(),
            maybe_last_group: None,
            stroke_open: false,
            material: Material::Water,
        }),
    })
}

fn create_particle_system(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_destruction_by_age(false);
    world
        .create_particle_system_with_def(&definition)
        .map_err(|_error| SceneError::ParticleSystem)
}

fn wall_segments() -> [RigidSegment; 16] {
    let mut segments = [RigidSegment {
        start: Vec2::ZERO,
        end: Vec2::ZERO,
    }; 16];
    let mut index = 0usize;
    for wall in WALLS {
        for edge in 0..4 {
            segments[index] = RigidSegment {
                start: wall[edge],
                end: wall[(edge + 1) % 4],
            };
            index += 1;
        }
    }
    segments
}

fn brush_shape(center: Vec2) -> Result<Shape, SessionError> {
    let circle =
        CircleShape::new(center, BRUSH_RADIUS).map_err(|_error| SessionError::StepFailed)?;
    Ok(Shape::from(circle))
}

impl DrawingParticlesHooks {
    fn should_stamp(&self, kind: PointerKind) -> bool {
        matches!(kind, PointerKind::Down | PointerKind::Move)
            || (kind == PointerKind::Up && !self.stroke_open)
    }

    fn drop_stale_last(&mut self, world: &World) {
        let Some(group) = self.maybe_last_group else {
            return;
        };
        let still_live = world
            .particle_group_view(group)
            .is_ok_and(|view| view.member_count() > 0);
        if !still_live {
            self.maybe_last_group = None;
        }
    }

    fn join_destination(&self, world: &World) -> ParticleGroupDestination {
        let Some(group) = self.maybe_last_group else {
            return ParticleGroupDestination::New;
        };
        let flags_match = world
            .particle_group_view(group)
            .is_ok_and(|view| view.flags() == self.material.group_flags());
        if flags_match {
            ParticleGroupDestination::AppendTo(group)
        } else {
            ParticleGroupDestination::New
        }
    }

    fn stamp(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
        world_x: f32,
        world_y: f32,
    ) -> Result<(), SessionError> {
        let brush = brush_shape(Vec2::new(world_x, world_y))?;
        world
            .destroy_particles_in_shape(system, &brush, Transform::IDENTITY)
            .map_err(|_error| SessionError::StepFailed)?;
        self.drop_stale_last(world);
        let source = ParticleGroupSource::filled_shapes(vec![brush])
            .map_err(|_error| SessionError::StepFailed)?;
        let recipe = ParticleGroupRecipe::new(source, self.join_destination(world))
            .with_particle_flags(self.material.particle_flags())
            .with_group_flags(self.material.group_flags())
            .with_color(self.material.color());
        if let Ok(group) = world.create_particle_group(system, &recipe) {
            self.maybe_last_group = Some(group);
        }
        Ok(())
    }
}

impl SceneHooks for DrawingParticlesHooks {
    fn on_advance(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        Ok(())
    }

    fn apply_control(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        name: &str,
        value: &str,
    ) -> Result<ControlEffect, SessionError> {
        if name != MATERIAL_CONTROL {
            return Err(SessionError::UnknownControl);
        }
        let Some(material) = Material::parse(value) else {
            return Err(SessionError::UnknownControl);
        };
        self.material = material;
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
        world: &mut World,
        system: ParticleSystemId,
        kind: PointerKind,
        world_x: f32,
        world_y: f32,
    ) -> Result<(), SessionError> {
        if self.should_stamp(kind) {
            self.stamp(world, system, world_x, world_y)?;
        }
        if kind == PointerKind::Down {
            self.stroke_open = true;
        }
        if matches!(kind, PointerKind::Up | PointerKind::Cancel) {
            self.maybe_last_group = None;
            self.stroke_open = false;
        }
        Ok(())
    }

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        Ok(self.wall_segments.to_vec())
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}
