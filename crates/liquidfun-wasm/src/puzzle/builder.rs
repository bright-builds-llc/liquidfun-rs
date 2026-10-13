//! One generic builder that turns a `LevelSpec` plus placements into the
//! existing `BuiltScene` (D-06, D-09).
//!
//! Creation order is fixed: static geometry, then tools in canonical order,
//! then the liquid; valves are destroyed later by the hooks (Phase 40 D-03).
//! Every engine error maps to a typed `SceneError`.

use liquidfun::collision::{PolygonShape, Shape};
use liquidfun::math::Vec2;
use liquidfun::particle::{
    ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{BodyDef, ParticleGroupId, ParticleSystemDef, ParticleSystemId, World};

use super::goal::{GoalTracker, count_in_goal};
use super::hooks::PuzzleHooks;
use super::peak::PeakHeldTracker;
use super::spec::{LevelSpec, LiquidBlock};
use super::tools::{Placement, Placements, ToolKind};
use super::valve::ValveGate;
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
    let (tracker, peak) = validated_trackers(spec)?;
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(spec.gravity)
        .map_err(|_error| SceneError::Gravity)?;
    create_geometry(&mut world, spec)?;
    let valves = create_tools(&mut world, placements)?;
    let (particle_system, counted_groups) = create_liquid(&mut world, spec)?;
    let segments = static_segments(spec, placements);
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
            peak,
            initial_count,
            segments,
            valves,
        )),
    })
}

/// Rejects an inconsistent spec before any world is created.
fn validated_trackers(spec: &LevelSpec) -> Result<(GoalTracker, PeakHeldTracker), SceneError> {
    let radius_is_valid = spec.particle_radius.is_finite() && spec.particle_radius > 0.0;
    let counts_are_ordered = spec.target_count < spec.brim_count
        && spec.brim_count <= spec.full_count
        && spec.par_tools <= total_budget(spec);
    if !spec.cup_interior.is_valid() || !counts_are_ordered || !radius_is_valid {
        return Err(SceneError::Geometry);
    }
    let tracker = GoalTracker::new(spec.target_count, spec.settle_steps, spec.step_limit)
        .ok_or(SceneError::Geometry)?;
    let peak = PeakHeldTracker::new(spec.settle_steps, spec.step_limit, spec.brim_count)
        .ok_or(SceneError::Geometry)?;
    Ok((tracker, peak))
}

/// Placements the level allows across every kind.
fn total_budget(spec: &LevelSpec) -> u32 {
    spec.tool_budget
        .iter()
        .map(|allowance| u32::from(allowance.count))
        .sum()
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

/// One static body per tool at angle 0, in canonical order; the vertices are
/// already in world space. Returns one closed gate per valve.
fn create_tools(world: &mut World, placements: &Placements) -> Result<Vec<ValveGate>, SceneError> {
    let mut valves = Vec::new();
    for placement in placements.as_slice() {
        let body = world
            .create_body(&BodyDef::default())
            .map_err(|_error| SceneError::Body)?;
        let vertices = placement.vertices();
        attach_basin_fixture(world, body, &vertices)?;
        if placement.kind() == ToolKind::Valve {
            valves.push(ValveGate::new(placement.open_tick(), body, edges(vertices)));
        }
    }
    Ok(valves)
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

/// Four edges per wall, cup wall and non-valve tool quad, in creation order.
/// Valves contribute their edges through their gates while closed.
fn static_segments(spec: &LevelSpec, placements: &Placements) -> Vec<RigidSegment> {
    let fixed_tools = placements
        .as_slice()
        .iter()
        .copied()
        .filter(|placement| placement.kind() != ToolKind::Valve)
        .map(Placement::vertices);
    spec.walls
        .iter()
        .chain(spec.cup_walls)
        .copied()
        .chain(fixed_tools)
        .flat_map(edges)
        .collect()
}

/// The four edges of a counter-clockwise quad.
fn edges([v0, v1, v2, v3]: [Vec2; 4]) -> [RigidSegment; 4] {
    [(v0, v1), (v1, v2), (v2, v3), (v3, v0)].map(|(start, end)| RigidSegment { start, end })
}

#[cfg(test)]
mod tests;
