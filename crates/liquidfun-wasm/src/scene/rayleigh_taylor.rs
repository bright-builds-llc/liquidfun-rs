//! Rayleigh–Taylor instability: a heavy orange layer over a light blue one in
//! a closed tank. Both layers are the same particle liquid in one particle
//! system; the orange group is given extra weight along the live gravity
//! vector once per fixed step, so a seeded ripple grows into sinking fingers
//! and the layers overturn. The engine has no per-group density; this is the
//! extra-weight (Boussinesq) picture.
//!
//! The extra weight is off for a short settle, then ramps in (D-03). The
//! recreating `weight-ratio` preset picks how much extra weight the orange
//! layer carries (D-08). The scene has no pointer interaction or live control.

mod geometry;

use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe,
    ParticleGroupSource,
};
use liquidfun::{BodyDef, ParticleGroupId, ParticleSystemDef, ParticleSystemId, World};

use self::geometry::SeedMode;
use super::particle_force::{self, ForceScale, GravityFrame};
use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks, SceneId,
    attach_basin_fixture,
};
use crate::session::SessionError;

/// Recreating preset: the heavy-to-light weight ratio (D-08).
const WEIGHT_RATIO_CONTROL: &str = "weight-ratio";
/// Warm orange heavy layer (`#FB923C`, UI-D86).
const HEAVY_COLOR: ParticleColor = ParticleColor::new(251, 146, 60, 255);
/// Deep blue light layer (`#3B82F6`, UI-D86).
const LIGHT_COLOR: ParticleColor = ParticleColor::new(59, 130, 246, 255);
/// Particle budget cap for the scene (D-11).
const MAXIMUM_PARTICLE_COUNT: usize = 3_000;

/// Every accepted `weight-ratio` token, in web option order.
#[cfg(test)]
pub(super) const WEIGHT_RATIO_TOKENS: [&str; 4] = ["1.2", "1.5", "2", "3"];
/// The token the web shows on load.
#[cfg(test)]
pub(super) const DEFAULT_WEIGHT_RATIO_TOKEN: &str = "2";

/// Heavy-to-light weight ratio, from the exact `weight-ratio` tokens (D-08).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WeightRatio {
    OnePointTwo,
    OnePointFive,
    Two,
    Three,
}

impl WeightRatio {
    const DEFAULT: Self = Self::Two;

    fn parse(value: &str) -> Option<Self> {
        match value {
            "1.2" => Some(Self::OnePointTwo),
            "1.5" => Some(Self::OnePointFive),
            "2" => Some(Self::Two),
            "3" => Some(Self::Three),
            _ => None,
        }
    }

    /// The weight beyond ordinary gravity, in units of `|g|`: `ratio - 1`,
    /// stored as literals so it is exact.
    fn extra_weight(self) -> f32 {
        match self {
            Self::OnePointTwo => 0.2,
            Self::OnePointFive => 0.5,
            Self::Two => 1.0,
            Self::Three => 2.0,
        }
    }
}

/// Every knob the D-06 spike measured; production uses [`TUNING`].
#[derive(Debug, Clone, Copy, PartialEq)]
struct Tuning {
    radius: f32,
    particle_flags: ParticleFlags,
    viscous_strength: f32,
    repulsive_strength: f32,
    /// Pressure stiffness: stiffer liquid keeps the column from slumping (D-06a).
    pressure_strength: f32,
    /// Particle contact damping (D-06a).
    damping: f32,
    modes: [SeedMode; 2],
    /// Steps with no extra weight while the lattice settles (D-03).
    settle_steps: u16,
    /// Steps over which the extra weight ramps linearly to full (D-03).
    ramp_steps: u16,
}

/// The tuning chosen by the D-06 spike (49-01-SUMMARY): plain water (D-05
/// candidate 1) with stiff pressure, so the column settles without slumping,
/// and light contact damping, so the interface stays mobile enough to
/// overturn at ratio 1.2 (D-06a).
const TUNING: Tuning = Tuning {
    radius: 0.035,
    particle_flags: ParticleFlags::WATER,
    viscous_strength: 0.25,
    repulsive_strength: 1.0,
    pressure_strength: 2.0,
    damping: 0.2,
    modes: [
        SeedMode {
            troughs: 5,
            amplitude: 0.10,
        },
        SeedMode {
            troughs: 9,
            amplitude: 0.03,
        },
    ],
    settle_steps: 36,
    ramp_steps: 24,
};

const _: () = assert!(TUNING.settle_steps >= 18 && TUNING.settle_steps <= 36);
const _: () = assert!(TUNING.ramp_steps >= 12 && TUNING.ramp_steps <= 24);
const _: () = assert!(TUNING.viscous_strength <= 1.5);
// Walls at least three particle diameters thick, so nothing tunnels through.
const _: () = assert!(geometry::WALL_THICKNESS >= 3.0 * 2.0 * TUNING.radius);

/// Ramped extra weight on the heavy group, applied once per fixed step.
struct RayleighTaylorHooks {
    heavy_group: ParticleGroupId,
    ratio: WeightRatio,
    /// Particle mass times particle iterations, from the live system definition.
    scale: ForceScale,
    /// Fixed steps since this build; saturating, and every rebuild starts at 0.
    steps_advanced: u16,
    settle_steps: u16,
    ramp_steps: u16,
    /// Static tank outlines, built once.
    tank_segments: Vec<RigidSegment>,
}

/// The world, particle system and hooks before they are boxed into a [`BuiltScene`].
struct RayleighTaylorParts {
    world: World,
    particle_system: ParticleSystemId,
    hooks: RayleighTaylorHooks,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let ratio = ratio_from_presets(presets)?;
    let parts = build_parts(ratio, &TUNING).map_err(|_error| SessionError::SceneConstruction)?;
    Ok(BuiltScene {
        world: parts.world,
        particle_system: parts.particle_system,
        particle_radius: TUNING.radius,
        hooks: Box::new(parts.hooks),
    })
}

/// The weight ratio from the scene presets: the last `weight-ratio` value,
/// or the default when absent. `build_scene` strips and applies the gravity
/// preset first, so any other name or an unknown token fails closed (D-08).
fn ratio_from_presets(presets: &[(String, String)]) -> Result<WeightRatio, SessionError> {
    if presets
        .iter()
        .any(|(name, _value)| name != WEIGHT_RATIO_CONTROL)
    {
        return Err(SessionError::UnknownControl);
    }
    preset_value(presets, WEIGHT_RATIO_CONTROL)
        .map(WeightRatio::parse)
        .map_or(Ok(WeightRatio::DEFAULT), |maybe_ratio| {
            maybe_ratio.ok_or(SessionError::UnknownControl)
        })
}

/// The last value stored for `name`, matching the session's replace semantics.
fn preset_value<'a>(presets: &'a [(String, String)], name: &str) -> Option<&'a str> {
    presets
        .iter()
        .rev()
        .find(|(key, _value)| key == name)
        .map(|(_key, value)| value.as_str())
}

fn build_parts(ratio: WeightRatio, tuning: &Tuning) -> Result<RayleighTaylorParts, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(Vec2::new(0.0, -10.0))
        .map_err(|_error| SceneError::Gravity)?;

    let tank = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    for quad in &geometry::tank_quads() {
        attach_basin_fixture(&mut world, tank, quad)?;
    }

    let particle_system = create_system(&mut world, tuning)?;
    let (light, heavy) = geometry::split_layers(geometry::lattice(tuning.radius), &tuning.modes);
    create_layer(&mut world, particle_system, light, LIGHT_COLOR, tuning)?;
    let heavy_group = create_layer(&mut world, particle_system, heavy, HEAVY_COLOR, tuning)?;
    let scale = ForceScale::for_system(
        &world,
        particle_system,
        super::particle_iterations(SceneId::RayleighTaylor),
    )
    .map_err(|_error| SceneError::ParticleSystem)?;

    Ok(RayleighTaylorParts {
        world,
        particle_system,
        hooks: RayleighTaylorHooks {
            heavy_group,
            ratio,
            scale,
            steps_advanced: 0,
            settle_steps: tuning.settle_steps,
            ramp_steps: tuning.ramp_steps,
            tank_segments: geometry::tank_segments(),
        },
    })
}

fn create_system(world: &mut World, tuning: &Tuning) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(tuning.radius)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_viscous_strength(tuning.viscous_strength)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_repulsive_strength(tuning.repulsive_strength)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_pressure_strength(tuning.pressure_strength)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_damping(tuning.damping)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)
}

/// One layer as a new group, one particle per lattice point, in order.
fn create_layer(
    world: &mut World,
    system: ParticleSystemId,
    points: Vec<Vec2>,
    color: ParticleColor,
    tuning: &Tuning,
) -> Result<ParticleGroupId, SceneError> {
    let source = ParticleGroupSource::positions(points).map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(tuning.particle_flags)
        .with_color(color)
        .with_transform(Transform::IDENTITY)
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)
}

/// 0 during the settle window, then a linear ramp that reaches 1 after
/// `ramp_steps` steps and stays there (D-03).
fn ramp_factor(step: u16, settle_steps: u16, ramp_steps: u16) -> f32 {
    let Some(into_ramp) = step.checked_sub(settle_steps) else {
        return 0.0;
    };
    (f32::from(into_ramp.saturating_add(1)) / f32::from(ramp_steps)).min(1.0)
}

impl SceneHooks for RayleighTaylorHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        let step = self.steps_advanced;
        self.steps_advanced = step.saturating_add(1);
        let ramp = ramp_factor(step, self.settle_steps, self.ramp_steps);
        if ramp <= 0.0 {
            return Ok(());
        }
        // Read every step: tilt changes gravity without a rebuild (D-02).
        let Some(frame) = GravityFrame::from_gravity(world.gravity()) else {
            return Ok(());
        };
        let extra = self.ratio.extra_weight() * frame.magnitude * ramp;
        particle_force::apply_group_acceleration(
            world,
            system,
            self.heavy_group,
            self.scale,
            frame.up * -extra,
        )
    }

    fn apply_control(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        name: &str,
        value: &str,
    ) -> Result<ControlEffect, SessionError> {
        // The session stores the preset and rebuilds; the hooks never mutate here.
        if name == WEIGHT_RATIO_CONTROL && WeightRatio::parse(value).is_some() {
            return Ok(ControlEffect::Recreated);
        }
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
        // An ignored pointer must stay Ok: an error would fail the scene in the browser.
        Ok(())
    }

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        Ok(self.tank_segments.clone())
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod flow_tests;
#[cfg(test)]
mod force_lane_tests;
#[cfg(test)]
mod hook_tests;
#[cfg(test)]
mod spike_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
