//! Cartesian Diver: a closed bottle of particle water with one floating diver.
//!
//! Pressing and holding the bottle, or the `squeeze` action, sinks the diver;
//! letting go raises it again. Switching the diver's fixture density stands
//! in for the squeezed air bubble: the density ramps between a light and a
//! heavy value over a quarter second, and gravity plus the water's buoyancy
//! do the rest. The bottle dent and the bubble inside the diver are drawn
//! only; the walls never move and nothing pushes the water or the diver.

mod geometry;

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, FixtureId, ParticleColor, ParticleDef, ParticleFlags,
    ParticleSystemDef, ParticleSystemId, World,
};

use self::geometry::{COLUMNS, DIVER_RADIUS, DIVER_START, PARTICLE_RADIUS, ROWS};
use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
    attach_basin_fixture,
};
use crate::session::SessionError;

/// Panel action and README/survey cue that toggles the squeeze (D-03).
const SQUEEZE_ACTION: &str = "squeeze";
/// Fixed steps for a full squeeze or release ramp: 0.25 s at 60 Hz (D-06).
const RAMP_STEPS: u8 = 15;
/// Stiffer than the default water so the squeezed diver reaches the floor at high gravity.
const PRESSURE_STRENGTH: f32 = 0.2;
/// Diver density while released: clearly lighter than the water (D-05).
const RELEASED_DENSITY: f32 = 0.5;
/// Diver density while squeezed: clearly heavier than the compressed water (D-05).
const SQUEEZED_DENSITY: f32 = 5.0;
/// Calms the sink and rise without slowing low gravity too much (D-06).
const DIVER_LINEAR_DAMPING: f32 = 1.0;
/// Diver contact friction against the walls and particles.
const DIVER_FRICTION: f32 = 0.2;
/// Water particle density.
const WATER_DENSITY: f32 = 1.0;
/// Room for every grid point; the diver's start disk leaves some unused.
const MAXIMUM_PARTICLE_COUNT: usize = COLUMNS as usize * ROWS as usize;
/// The shared playground water color.
const WATER_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);

/// Squeeze intent and ramp state; physics changes happen only in `on_advance`.
struct CartesianDiverHooks {
    diver: BodyId,
    diver_fixture: FixtureId,
    /// Latest input wins: pointer down in the bottle, up, cancel or the action.
    squeezed: bool,
    /// Ramp position in whole fixed steps, `0..=RAMP_STEPS`, so the squeeze value is exact.
    squeeze_steps: u8,
}

impl CartesianDiverHooks {
    /// Ramped squeeze value in `0.0..=1.0` that drives density, dent and bubble.
    fn squeeze(&self) -> f32 {
        f32::from(self.squeeze_steps) / f32::from(RAMP_STEPS)
    }
}

/// The world, water system and hooks before they are boxed into a [`BuiltScene`].
struct DiverParts {
    world: World,
    particle_system: ParticleSystemId,
    hooks: CartesianDiverHooks,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    // The gravity preset is applied by `build_scene` after construction.
    let _ = presets;
    let parts = build_parts().map_err(|_error| SessionError::SceneConstruction)?;
    Ok(BuiltScene {
        world: parts.world,
        particle_system: parts.particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(parts.hooks),
    })
}

fn build_parts() -> Result<DiverParts, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(Vec2::new(0.0, -10.0))
        .map_err(|_error| SceneError::Gravity)?;

    let bottle = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    for quad in &geometry::wall_quads() {
        attach_basin_fixture(&mut world, bottle, quad)?;
    }

    let particle_system = create_water(&mut world)?;
    let (diver, diver_fixture) = create_diver(&mut world)?;

    Ok(DiverParts {
        world,
        particle_system,
        hooks: CartesianDiverHooks {
            diver,
            diver_fixture,
            squeezed: false,
            squeeze_steps: 0,
        },
    })
}

fn create_water(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_density(WATER_DENSITY)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_pressure_strength(PRESSURE_STRENGTH)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    for position in geometry::water_positions() {
        let definition = ParticleDef::default()
            .with_flags(ParticleFlags::WATER)
            .with_color(WATER_COLOR)
            .with_position(position)
            .map_err(|_error| SceneError::Particle)?;
        let receipt = world
            .create_particle_with_def(system, None, &definition)
            .map_err(|_error| SceneError::Particle)?;
        if !receipt.destruction_occurrences().is_empty() {
            return Err(SceneError::Particle);
        }
    }

    Ok(system)
}

/// One dynamic circle that never sleeps, so a density change always takes effect (D-04).
fn create_diver(world: &mut World) -> Result<(BodyId, FixtureId), SceneError> {
    let definition = BodyDef::new(BodyType::Dynamic, DIVER_START, 0.0, true)
        .map_err(|_error| SceneError::Body)?
        .with_linear_damping(DIVER_LINEAR_DAMPING)
        .map_err(|_error| SceneError::Body)?
        .with_sleeping_allowed(false)
        .with_fixed_rotation(true);
    let body = world
        .create_body(&definition)
        .map_err(|_error| SceneError::Body)?;
    let circle =
        CircleShape::new(Vec2::ZERO, DIVER_RADIUS).map_err(|_error| SceneError::Geometry)?;
    let fixture_definition = FixtureDef::new(
        Shape::from(circle),
        RELEASED_DENSITY,
        DIVER_FRICTION,
        0.0,
        false,
        FilterData::default(),
    )
    .map_err(|_error| SceneError::Fixture)?;
    let fixture = world
        .create_fixture(body, &fixture_definition)
        .map_err(|_error| SceneError::Fixture)?;
    Ok((body, fixture))
}

impl SceneHooks for CartesianDiverHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        let target = if self.squeezed { RAMP_STEPS } else { 0 };
        let next = match self.squeeze_steps.cmp(&target) {
            std::cmp::Ordering::Less => self.squeeze_steps + 1,
            std::cmp::Ordering::Greater => self.squeeze_steps - 1,
            std::cmp::Ordering::Equal => return Ok(()),
        };
        self.squeeze_steps = next;
        let density = RELEASED_DENSITY + (SQUEEZED_DENSITY - RELEASED_DENSITY) * self.squeeze();
        world
            .set_fixture_density(self.diver_fixture, density)
            .map_err(|_error| SessionError::StepFailed)?;
        world
            .reset_body_mass_data(self.diver)
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
        name: &str,
    ) -> Result<(), SessionError> {
        if name != SQUEEZE_ACTION {
            return Err(SessionError::UnknownControl);
        }
        self.squeezed = !self.squeezed;
        Ok(())
    }

    fn apply_pointer(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        kind: PointerKind,
        world_x: f32,
        world_y: f32,
    ) -> Result<(), SessionError> {
        match kind {
            PointerKind::Down if geometry::in_squeeze_region(Vec2::new(world_x, world_y)) => {
                self.squeezed = true;
            }
            PointerKind::Up | PointerKind::Cancel => self.squeezed = false,
            // An ignored pointer must stay Ok: an error would fail the scene in the browser.
            PointerKind::Down | PointerKind::Move => {}
        }
        Ok(())
    }

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        Ok(geometry::outline_segments(self.squeeze()).to_vec())
    }

    fn collect_circles(&self, world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        let center = world
            .body_snapshot(self.diver)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .position();
        let bubble = geometry::bubble_circle(center, DIVER_RADIUS, self.squeeze(), world.gravity());
        Ok(vec![(center, DIVER_RADIUS), bubble])
    }
}

#[cfg(test)]
mod tests;
