//! Fake Ferrofluid: a dark liquid puddle in a wide shallow dish rises into a
//! row of separate spikes over a magnet resting under the dish.
//!
//! The physics engine has no magnetism. The spikes come from a scripted
//! magnetic pull ([`field`]): once per fixed step, `on_advance` gives every
//! particle the gradient of one bounded polynomial potential plus velocity
//! drag, through the shared gravity-aware force helper. The magnet itself is
//! only a drawn marker; it never collides with anything.
//!
//! Pressing anywhere switches the magnet on and moves it to the pointer, and
//! dragging moves it; the spikes follow. Releasing or cancelling the press
//! switches it off and parks it back under the dish, so the spikes slump.
//! The `magnet` action toggles it at its current position, and the live
//! `magnet-strength` control sets the pull from 0 to 100 percent. These
//! inputs only store intent; the pull is applied in `on_advance`.

mod field;
mod geometry;

use liquidfun::collision::{PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe,
    ParticleGroupSource,
};
use liquidfun::{BodyDef, ParticleSystemDef, ParticleSystemId, World};

use self::field::FIELD;
use self::geometry::{
    MAGNET_REST, PARTICLE_RADIUS, PUDDLE_CENTER, PUDDLE_HALF_HEIGHT, PUDDLE_HALF_WIDTH,
};
use super::particle_force::{self, ForceScale, GravityFrame, ParticleScratch};
use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks, SceneId,
    attach_basin_fixture, parse_whole_rpm,
};
use crate::session::SessionError;

/// Live control: pull strength in whole percent, `0..=100` (D-09, D-11).
const MAGNET_STRENGTH_CONTROL: &str = "magnet-strength";
/// Action: toggles the magnet on or off at its current position (D-10).
const MAGNET_ACTION: &str = "magnet";
/// Strength on load, in percent of the tuned maximum: five to nine spikes (D-04, D-09).
const DEFAULT_STRENGTH_PERCENT: u16 = 65;
/// The tuned safe maximum strength, in percent (D-09).
const MAX_STRENGTH_PERCENT: u16 = 100;
/// Neighbor viscosity that keeps spikes cohesive; the step fails above about 2.0 (D-03).
const VISCOUS_STRENGTH: f32 = 0.25;
/// Liquid particle density.
const LIQUID_DENSITY: f32 = 1.0;
/// Graphite violet: dark, but readable on the canvas at the density-shade floor (D-16, UI-D73).
const LIQUID_COLOR: ParticleColor = ParticleColor::new(138, 132, 173, 255);
/// Particle budget cap for the scene (D-15).
const MAXIMUM_PARTICLE_COUNT: usize = 3_000;

const _: () = assert!(DEFAULT_STRENGTH_PERCENT >= 60 && DEFAULT_STRENGTH_PERCENT <= 70);
const _: () = assert!(VISCOUS_STRENGTH <= 1.5);

/// Magnet intent from the pointer, action and control seams; the pull is
/// applied only in `on_advance`, once per fixed step (D-08).
struct FakeFerrofluidHooks {
    /// Whether the magnet pulls; on at load (D-06).
    magnet_on: bool,
    /// Field origin; rests under the dish floor center at load (D-06).
    magnet_position: Vec2,
    /// Whether a press is held, so pointer moves drag the magnet (D-07).
    pointer_held: bool,
    /// Pull strength in percent; 0 means no pull while drag still holds the puddle (D-11).
    strength_percent: u16,
    /// Particle mass times particle iterations, from the live system definition.
    scale: ForceScale,
    /// Reused particle lanes, so stepping allocates nothing after the first step.
    scratch: ParticleScratch,
    /// Static dish outlines, built once.
    dish_segments: Vec<RigidSegment>,
}

/// The world, liquid system and hooks before they are boxed into a [`BuiltScene`].
struct FerrofluidParts {
    world: World,
    particle_system: ParticleSystemId,
    hooks: FakeFerrofluidHooks,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    // `build_scene` strips and applies the gravity preset; nothing else is accepted.
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    let parts = build_parts().map_err(|_error| SessionError::SceneConstruction)?;
    Ok(BuiltScene {
        world: parts.world,
        particle_system: parts.particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(parts.hooks),
    })
}

fn build_parts() -> Result<FerrofluidParts, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(Vec2::new(0.0, -10.0))
        .map_err(|_error| SceneError::Gravity)?;

    let dish = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    for quad in &geometry::dish_quads() {
        attach_basin_fixture(&mut world, dish, quad)?;
    }

    let particle_system = create_puddle(&mut world)?;
    let scale = ForceScale::for_system(
        &world,
        particle_system,
        super::particle_iterations(SceneId::FakeFerrofluid),
    )
    .map_err(|_error| SceneError::ParticleSystem)?;

    Ok(FerrofluidParts {
        world,
        particle_system,
        hooks: FakeFerrofluidHooks {
            magnet_on: true,
            magnet_position: MAGNET_REST,
            pointer_held: false,
            strength_percent: DEFAULT_STRENGTH_PERCENT,
            scale,
            scratch: ParticleScratch::default(),
            dish_segments: geometry::dish_segments(),
        },
    })
}

/// One `TENSILE | VISCOUS` group filling the dish floor (D-03).
fn create_puddle(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_density(LIQUID_DENSITY)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_viscous_strength(VISCOUS_STRENGTH)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    let filled = Shape::from(
        PolygonShape::oriented_box(PUDDLE_HALF_WIDTH, PUDDLE_HALF_HEIGHT, PUDDLE_CENTER, 0.0)
            .map_err(|_error| SceneError::Geometry)?,
    );
    let source =
        ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(ParticleFlags::TENSILE | ParticleFlags::VISCOUS)
        .with_color(LIQUID_COLOR)
        .with_transform(Transform::IDENTITY)
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(system)
}

impl SceneHooks for FakeFerrofluidHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        if !self.magnet_on {
            return Ok(());
        }
        // Read every step: tilt changes gravity without a rebuild (D-14).
        let Some(frame) = GravityFrame::from_gravity(world.gravity()) else {
            return Ok(());
        };
        let strength = f32::from(self.strength_percent) / 100.0;
        let magnet = self.magnet_position;
        particle_force::apply_accelerations(
            world,
            system,
            self.scale,
            &mut self.scratch,
            |position, velocity| {
                let drag = FIELD.drag_per_second * field::envelope(&FIELD, frame, magnet, position);
                field::magnet_acceleration(&FIELD, frame, strength, magnet, position)
                    - velocity * drag
            },
        )
    }

    fn apply_control(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        name: &str,
        value: &str,
    ) -> Result<ControlEffect, SessionError> {
        if name != MAGNET_STRENGTH_CONTROL {
            return Err(SessionError::UnknownControl);
        }
        let Some(percent) = parse_whole_rpm(value, MAX_STRENGTH_PERCENT) else {
            return Err(SessionError::UnknownControl);
        };
        self.strength_percent = percent;
        Ok(ControlEffect::Live)
    }

    fn apply_action(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        name: &str,
    ) -> Result<(), SessionError> {
        if name != MAGNET_ACTION {
            return Err(SessionError::UnknownControl);
        }
        self.magnet_on = !self.magnet_on;
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
        let pointer = Vec2::new(world_x, world_y);
        match kind {
            PointerKind::Down => {
                self.pointer_held = true;
                self.magnet_on = true;
                self.magnet_position = geometry::clamp_magnet(pointer);
            }
            PointerKind::Move if self.pointer_held => {
                self.magnet_position = geometry::clamp_magnet(pointer);
            }
            // Release and cancel always park the magnet, pressed or not (D-07).
            PointerKind::Up | PointerKind::Cancel => {
                self.pointer_held = false;
                self.magnet_on = false;
                self.magnet_position = MAGNET_REST;
            }
            // An ignored pointer must stay Ok: an error would fail the scene in the browser.
            PointerKind::Move => {}
        }
        Ok(())
    }

    fn collect_segments(&self, world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        // Dish first, so dish line indices never change when the marker toggles.
        let mut segments = self.dish_segments.clone();
        segments.extend(geometry::magnet_marker(
            self.magnet_position,
            GravityFrame::from_gravity(world.gravity()),
            self.magnet_on,
        ));
        Ok(segments)
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod control_tests;
#[cfg(test)]
mod flow_tests;
#[cfg(test)]
mod spike_tests;
#[cfg(test)]
mod sync_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
