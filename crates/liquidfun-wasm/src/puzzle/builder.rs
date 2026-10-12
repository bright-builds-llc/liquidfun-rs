//! One generic builder that turns a `LevelSpec` plus placements into the
//! existing `BuiltScene` (D-06, D-09).
//!
//! Creation order is fixed: static level geometry, then planks in canonical
//! order, then the liquid. Every engine error maps to a typed `SceneError`.

use liquidfun::collision::{PolygonShape, Shape};
use liquidfun::math::Vec2;
use liquidfun::particle::{
    ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{BodyDef, ParticleGroupId, ParticleSystemDef, ParticleSystemId, World};

use super::goal::{GoalTracker, count_in_goal};
use super::hooks::PuzzleHooks;
use super::spec::{LevelSpec, LiquidBlock};
use super::tools::Placements;
use crate::scene::{BuiltScene, RigidSegment, SceneError, attach_basin_fixture};
use crate::session::SessionError;

/// Builds `spec` with `placements` at step 0.
pub(crate) fn build_level(
    spec: &'static LevelSpec,
    placements: &Placements,
) -> Result<BuiltScene, SessionError> {
    build(spec, placements).map_err(|_error| SessionError::SceneConstruction)
}

fn build(spec: &'static LevelSpec, placements: &Placements) -> Result<BuiltScene, SceneError> {
    let tracker = validated_tracker(spec)?;
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(spec.gravity)
        .map_err(|_error| SceneError::Gravity)?;
    create_geometry(&mut world, spec)?;
    create_planks(&mut world, placements)?;
    let (particle_system, counted_groups) = create_liquid(&mut world, spec)?;
    let segments = outline_segments(spec, placements);
    let initial_count = {
        let view = world
            .particle_system_view(particle_system)
            .map_err(|_error| SceneError::ParticleSystem)?;
        count_in_goal(
            view.positions(),
            view.group_ids(),
            &counted_groups,
            spec.cup_interior,
        )
    };
    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: spec.particle_radius,
        hooks: Box::new(PuzzleHooks::new(
            spec,
            counted_groups,
            tracker,
            initial_count,
            segments,
        )),
    })
}

/// Rejects an inconsistent spec before any world is created.
fn validated_tracker(spec: &LevelSpec) -> Result<GoalTracker, SceneError> {
    let radius_is_valid = spec.particle_radius.is_finite() && spec.particle_radius > 0.0;
    if !spec.cup_interior.is_valid() || spec.target_count > spec.full_count || !radius_is_valid {
        return Err(SceneError::Geometry);
    }
    GoalTracker::new(spec.target_count, spec.settle_steps, spec.step_limit)
        .ok_or(SceneError::Geometry)
}

/// One static body for every wall, then every cup wall, in spec order.
fn create_geometry(world: &mut World, spec: &LevelSpec) -> Result<(), SceneError> {
    let body = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    for quad in spec.walls.iter().chain(spec.cup_walls) {
        attach_basin_fixture(world, body, quad)?;
    }
    Ok(())
}

/// One static body per plank at angle 0; the vertices are already in world space.
fn create_planks(world: &mut World, placements: &Placements) -> Result<(), SceneError> {
    for placement in placements.as_slice() {
        let body = world
            .create_body(&BodyDef::default())
            .map_err(|_error| SceneError::Body)?;
        attach_basin_fixture(world, body, &placement.vertices())?;
    }
    Ok(())
}

/// Creates the particle system and one group per liquid block, returning the
/// counted group ids.
fn create_liquid(
    world: &mut World,
    spec: &LevelSpec,
) -> Result<(ParticleSystemId, Vec<ParticleGroupId>), SceneError> {
    let definition = ParticleSystemDef::default()
        .with_radius(spec.particle_radius)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(spec.maximum_particle_count)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&definition)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let mut counted_groups = Vec::new();
    for block in spec.liquid_blocks {
        let group = create_block(world, system, block)?;
        if block.counted {
            counted_groups.push(group);
        }
    }
    Ok((system, counted_groups))
}

fn create_block(
    world: &mut World,
    system: ParticleSystemId,
    block: &LiquidBlock,
) -> Result<ParticleGroupId, SceneError> {
    let LiquidBlock { min, max, .. } = *block;
    let polygon = PolygonShape::new(&[min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)])
        .map_err(|_error| SceneError::Geometry)?;
    let source = ParticleGroupSource::filled_shapes(vec![Shape::from(polygon)])
        .map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(ParticleFlags::WATER)
        .with_color(block.color)
        .with_stride(block.spacing)
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)
}

/// Four edges per wall, cup wall and plank quad, in creation order.
fn outline_segments(spec: &LevelSpec, placements: &Placements) -> Vec<RigidSegment> {
    let planks = placements
        .as_slice()
        .iter()
        .map(|placement| placement.vertices());
    spec.walls
        .iter()
        .chain(spec.cup_walls)
        .copied()
        .chain(planks)
        .flat_map(|[v0, v1, v2, v3]| {
            [(v0, v1), (v1, v2), (v2, v3), (v3, v0)].map(|(start, end)| RigidSegment { start, end })
        })
        .collect()
}

#[cfg(test)]
mod tests;
