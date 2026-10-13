//! Declarative, const-constructible puzzle level data (D-06).

use liquidfun::math::Vec2;
use liquidfun::particle::{ParticleColor, ParticleFlags, ParticleGroupFlags};

use super::LevelId;
use super::goal::GoalRect;
use super::tools::{GridRect, ToolAllowance};
use super::valve::STEPS_PER_OPEN_TICK;

/// Particle material of one liquid block (D-10). Flags are per particle, so
/// materials can share a level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))] // The teaching levels (Phase 40) author the other materials.
pub(crate) enum LiquidMaterial {
    /// Plain water: `ParticleFlags::WATER`, no group flags.
    Water,
    /// Slow, clinging liquid: `ParticleFlags::VISCOUS`.
    Viscous,
    /// Granular particles: `ParticleFlags::POWDER`.
    Powder,
    /// Soft jelly: `ParticleFlags::ELASTIC` in a `SOLID` group.
    Elastic,
}

impl LiquidMaterial {
    /// Particle and group flags; Elastic must be a `SOLID` group or water
    /// soaks into it (research Pitfall 4).
    pub(crate) const fn flags(self) -> (ParticleFlags, ParticleGroupFlags) {
        match self {
            Self::Water => (ParticleFlags::WATER, ParticleGroupFlags::empty()),
            Self::Viscous => (ParticleFlags::VISCOUS, ParticleGroupFlags::empty()),
            Self::Powder => (ParticleFlags::POWDER, ParticleGroupFlags::empty()),
            Self::Elastic => (ParticleFlags::ELASTIC, ParticleGroupFlags::SOLID),
        }
    }
}

/// Optional particle-system strengths; `None` leaves the engine default
/// untouched and adds no builder call (D-29).
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(
    clippy::struct_field_names,
    reason = "maybe_ prefixes make the three optional strengths explicit"
)]
pub(crate) struct SystemTuning {
    /// Viscous strength; levels stay in 1.0..=1.5 (values near 2.5 fail the step).
    pub(crate) maybe_viscous_strength: Option<f32>,
    /// Elastic strength for jelly groups.
    pub(crate) maybe_elastic_strength: Option<f32>,
    /// Powder strength for granular groups.
    pub(crate) maybe_powder_strength: Option<f32>,
}

impl SystemTuning {
    /// No overrides.
    pub(crate) const NONE: Self = Self {
        maybe_viscous_strength: None,
        maybe_elastic_strength: None,
        maybe_powder_strength: None,
    };
}

/// Level-authored dynamic circle for buoyancy (D-12, D-28). A circle keeps
/// runtime trig out of contact geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FloatBody {
    /// Start centre in meters.
    pub(crate) center: Vec2,
    /// Radius in meters.
    pub(crate) radius: f32,
    /// Fixture density; below the particle density of 1.0 it floats.
    pub(crate) density: f32,
}

/// One rectangular block of liquid created as its own particle group.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LiquidBlock {
    /// Lower-left corner in meters.
    pub(crate) min: Vec2,
    /// Upper-right corner in meters.
    pub(crate) max: Vec2,
    /// Group stride in meters.
    pub(crate) spacing: f32,
    /// Particle color.
    pub(crate) color: ParticleColor,
    /// Whether this group counts toward the cup (false for a jelly pad, D-11).
    pub(crate) counted: bool,
    /// Particle material (D-10).
    pub(crate) material: LiquidMaterial,
}

/// Declarative, const-constructible puzzle level (D-06).
///
/// Every quad is the four counter-clockwise corners of a static polygon in
/// world space, so building a level never calls `sin` or `cos`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LevelSpec {
    /// Allowlisted level id.
    pub(crate) id: LevelId,
    /// Player-facing title.
    pub(crate) title: &'static str,
    /// World gravity, set once at build.
    pub(crate) gravity: Vec2,
    /// Particle radius in meters.
    pub(crate) particle_radius: f32,
    /// Particle system cap.
    pub(crate) maximum_particle_count: usize,
    /// Particle substeps; 2 is the shared scene default (D-15).
    pub(crate) particle_iterations: u32,
    /// Optional particle-system strengths (D-29).
    pub(crate) system_tuning: SystemTuning,
    /// Static level walls.
    pub(crate) walls: &'static [[Vec2; 4]],
    /// Static cup walls.
    pub(crate) cup_walls: &'static [[Vec2; 4]],
    /// Region whose strictly contained counted particles fill the cup.
    pub(crate) cup_interior: GoalRect,
    /// Liquid created after the tools, in order.
    pub(crate) liquid_blocks: &'static [LiquidBlock],
    /// Dynamic circles created after the level geometry and before the
    /// tools; their start area should be an authored `no_place` zone.
    pub(crate) floats: &'static [FloatBody],
    /// Count that must hold for `settle_steps` consecutive steps to win.
    pub(crate) target_count: u32,
    /// Authored count for 100% fill.
    pub(crate) full_count: u32,
    /// Consecutive steps at or above target needed to win.
    pub(crate) settle_steps: u32,
    /// Last step at which a win may complete; the run loses after it.
    pub(crate) step_limit: u32,
    /// Allowed tool kinds and counts.
    pub(crate) tool_budget: &'static [ToolAllowance],
    /// Inclusive grid rectangle where tools may be placed.
    pub(crate) build_area: GridRect,
    /// Count whose peak held fill earns ★ Brim.
    pub(crate) brim_count: u32,
    /// Most placements at Play that still earn ★ Thrifty.
    pub(crate) par_tools: u32,
    /// Extra authored no-build zones in grid cells; the cup interior and the
    /// liquid blocks are always implicit no-place zones.
    pub(crate) no_place: &'static [GridRect],
}

impl LevelSpec {
    /// Largest valve open tick: `step_limit / STEPS_PER_OPEN_TICK`, saturated
    /// to `u8`.
    pub(crate) fn max_open_tick(&self) -> u8 {
        u8::try_from(self.step_limit / STEPS_PER_OPEN_TICK).unwrap_or(u8::MAX)
    }
}
