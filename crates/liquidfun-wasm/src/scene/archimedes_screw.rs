//! Archimedes Screw: slanted vanes slide up a tilted, closed tube and carry
//! separate pockets of water from a low reservoir into a raised basin, which
//! overflows down a chute back into the reservoir.
//!
//! Flat slanted vanes stand in for a turning helix in this 2D side view. All
//! vanes belong to one kinematic train that moves along the tube axis. After
//! one pitch it snaps back exactly one pitch, which leaves the vane lattice
//! unchanged; the vane that appears or disappears at a snap is always inside
//! a solid end housing, out of the water's reach (D-02, D-03). Only the vanes
//! move the water: there are no forces, emitters or teleports.
//!
//! Two live controls set the screw's intent: `screw-speed` (whole-number rpm
//! from 0 through `MAX_SCREW_RPM`) and `screw-direction` (`forward` or
//! `reverse`). Both are applied together in `on_advance` (D-09, D-10).

mod geometry;

use liquidfun::collision::{FilterData, PolygonShape, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, ParticleColor, ParticleDef, ParticleFlags,
    ParticleSystemDef, ParticleSystemId, World,
};

use self::geometry::{
    AXIS, PARTICLE_DIAMETER, PARTICLE_RADIUS, PITCH, TUBE_ORIGIN, VANE_COUNT, push_loop,
    vane_center, visible_vane_outline,
};
use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
    attach_basin_fixture,
};
use crate::session::SessionError;

/// Live control: screw speed in whole revolutions per minute (D-09).
const SCREW_SPEED_CONTROL: &str = "screw-speed";
/// Live control: `forward` lifts water, `reverse` carries it back down (D-10).
const SCREW_DIRECTION_CONTROL: &str = "screw-direction";
/// Starting screw speed in revolutions per minute (D-09).
const DEFAULT_SCREW_RPM: u16 = 30;
/// Fastest screw speed; about twice the default (D-09).
const MAX_SCREW_RPM: u16 = 60;
/// The session's fixed step.
const SIM_DT: f32 = 1.0 / 60.0;
/// Axial vane speed at `MAX_SCREW_RPM`: one pitch per revolution (D-01).
// `f32::from` is not const; the widening u16 to f32 `as` cast is exact here.
const MAX_AXIAL_SPEED: f32 = PITCH * (MAX_SCREW_RPM as f32) / 60.0;
// One step of vane travel stays under half a particle diameter (D-04).
const _: () = assert!(MAX_AXIAL_SPEED < 0.5 * PARTICLE_DIAMETER / SIM_DT);
/// Softer than the default particle damping, as in the diver.
const PARTICLE_DAMPING: f32 = 0.2;
/// Stiffer than the default water so pockets and the reservoir hold their shape.
const PRESSURE_STRENGTH: f32 = 0.25;
/// Vane contact friction against the water.
const VANE_FRICTION: f32 = 0.2;
/// The shared playground water color.
const WATER_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);

/// Whole-number rpm tokens from `0` through [`MAX_SCREW_RPM`].
fn parse_screw_rpm(value: &str) -> Option<u16> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if value.len() > 1 && value.starts_with('0') {
        return None;
    }
    let rpm = value.parse::<u16>().ok()?;
    if rpm > MAX_SCREW_RPM {
        return None;
    }
    Some(rpm)
}

/// Signed axial vane speed in m/s: one pitch per revolution, negative in reverse.
fn axial_speed(rpm: u16, reverse: bool) -> f32 {
    let speed = PITCH * f32::from(rpm) / 60.0;
    if reverse { -speed } else { speed }
}

/// Next lattice phase in `[0, PITCH)` and the snap to apply first: `-PITCH`,
/// `0.0` or `PITCH`. A snap moves the train one whole pitch, so the lattice
/// looks the same before and after it.
fn advance_phase(phase: f32, step: f32) -> (f32, f32) {
    let next = phase + step;
    if next >= PITCH {
        return (next - PITCH, -PITCH);
    }
    if next < 0.0 {
        return (next + PITCH, PITCH);
    }
    (next, 0.0)
}

/// Train body origin at lattice phase `phase`.
fn train_position(phase: f32) -> Vec2 {
    TUBE_ORIGIN + AXIS * phase
}

/// Limits a train velocity to the tunneling-safe maximum (D-04).
fn clamp_speed(velocity: Vec2) -> Vec2 {
    let speed = velocity.length();
    if speed <= MAX_AXIAL_SPEED {
        return velocity;
    }
    velocity * (MAX_AXIAL_SPEED / speed)
}

/// Screw intent and the vane lattice phase; motion is applied only in `on_advance`.
struct ArchimedesScrewHooks {
    train: BodyId,
    /// Lattice phase in `[0, PITCH)`: how far the train sits from its start.
    phase: f32,
    rpm: u16,
    reverse: bool,
    /// Static wall outlines, built once.
    wall_segments: Vec<RigidSegment>,
}

/// The world, water system and hooks before they are boxed into a [`BuiltScene`].
struct ScrewParts {
    world: World,
    particle_system: ParticleSystemId,
    hooks: ArchimedesScrewHooks,
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

fn build_parts() -> Result<ScrewParts, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(Vec2::new(0.0, -10.0))
        .map_err(|_error| SceneError::Gravity)?;

    let walls = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    for quad in &geometry::wall_quads() {
        attach_basin_fixture(&mut world, walls, quad)?;
    }

    let train = create_train(&mut world)?;
    let particle_system = create_water(&mut world)?;

    Ok(ScrewParts {
        world,
        particle_system,
        hooks: ArchimedesScrewHooks {
            train,
            phase: 0.0,
            rpm: DEFAULT_SCREW_RPM,
            reverse: false,
            wall_segments: geometry::wall_segments(),
        },
    })
}

/// One kinematic body carrying every vane; it never sleeps or turns (D-01).
fn create_train(world: &mut World) -> Result<BodyId, SceneError> {
    let definition = BodyDef::new(BodyType::Kinematic, train_position(0.0), 0.0, true)
        .map_err(|_error| SceneError::Body)?
        .with_sleeping_allowed(false)
        .with_fixed_rotation(true);
    let train = world
        .create_body(&definition)
        .map_err(|_error| SceneError::Body)?;
    for quad in &geometry::vane_quads() {
        let polygon = PolygonShape::new(quad).map_err(|_error| SceneError::Geometry)?;
        let fixture = FixtureDef::new(
            Shape::from(polygon),
            0.0,
            VANE_FRICTION,
            0.0,
            false,
            FilterData::default(),
        )
        .map_err(|_error| SceneError::Fixture)?;
        world
            .create_fixture(train, &fixture)
            .map_err(|_error| SceneError::Fixture)?;
    }
    Ok(train)
}

fn create_water(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let positions = geometry::water_positions();
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_damping(PARTICLE_DAMPING)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_pressure_strength(PRESSURE_STRENGTH)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(positions.len())
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_strict_contact_check(true);
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    for position in positions {
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

impl SceneHooks for ArchimedesScrewHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        let (next, snap) = advance_phase(self.phase, axial_speed(self.rpm, self.reverse) * SIM_DT);
        if snap != 0.0 {
            // The vane this adds or removes lies inside a solid housing (geometry tests).
            world
                .set_body_transform(self.train, train_position(self.phase + snap), 0.0)
                .map_err(|_error| SessionError::StepFailed)?;
        }
        let current = world
            .body_snapshot(self.train)
            .map_err(|_error| SessionError::StepFailed)?
            .position();
        let velocity = clamp_speed((train_position(next) - current) * (1.0 / SIM_DT));
        world
            .set_body_linear_velocity(self.train, velocity)
            .map_err(|_error| SessionError::StepFailed)?;
        self.phase = next;
        Ok(())
    }

    fn apply_control(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        name: &str,
        value: &str,
    ) -> Result<ControlEffect, SessionError> {
        // Intent only: `on_advance` turns it into vane motion on the next step.
        match (name, value) {
            (SCREW_SPEED_CONTROL, _) => {
                let Some(rpm) = parse_screw_rpm(value) else {
                    return Err(SessionError::UnknownControl);
                };
                self.rpm = rpm;
            }
            (SCREW_DIRECTION_CONTROL, "forward") => self.reverse = false,
            (SCREW_DIRECTION_CONTROL, "reverse") => self.reverse = true,
            _ => return Err(SessionError::UnknownControl),
        }
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
        // Dragging the screw is an anti-feature (D-11); an error would fail the scene.
        Ok(())
    }

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        let mut segments = self.wall_segments.clone();
        for index in 0..VANE_COUNT {
            if let Some(outline) = visible_vane_outline(vane_center(index, self.phase)) {
                push_loop(&mut segments, &outline);
            }
        }
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
mod hooks_tests;
#[cfg(test)]
mod spike_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
