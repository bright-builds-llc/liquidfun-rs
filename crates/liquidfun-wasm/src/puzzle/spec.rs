//! Declarative, const-constructible puzzle level data (D-06).

use liquidfun::math::Vec2;
use liquidfun::particle::ParticleColor;

use super::LevelId;
use super::goal::GoalRect;
use super::tools::{GridRect, ToolAllowance};
use super::valve::STEPS_PER_OPEN_TICK;

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
    /// Whether this group counts toward the cup (false for a future jelly pad).
    pub(crate) counted: bool,
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
    /// Static level walls.
    pub(crate) walls: &'static [[Vec2; 4]],
    /// Static cup walls.
    pub(crate) cup_walls: &'static [[Vec2; 4]],
    /// Region whose strictly contained counted particles fill the cup.
    pub(crate) cup_interior: GoalRect,
    /// Liquid created after the tools, in order.
    pub(crate) liquid_blocks: &'static [LiquidBlock],
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
