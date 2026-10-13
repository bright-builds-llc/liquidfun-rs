//! A reservoir drains through a gap onto the floor beside the cup; one plank
//! ramp turns the stream into the cup.
//!
//! All geometry is literal and axis-aligned. Plan 37-06 tunes the numbers
//! against measured outcomes and records the final values.

use liquidfun::math::Vec2;
use liquidfun::particle::ParticleColor;

use super::LevelId;
use super::goal::GoalRect;
use super::spec::{LevelSpec, LiquidBlock, LiquidMaterial, SystemTuning};
use super::tools::{GridRect, ToolAllowance, ToolKind};

/// Counter-clockwise corners of the axis-aligned rectangle `min..max`.
const fn quad(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> [Vec2; 4] {
    [
        Vec2::new(min_x, min_y),
        Vec2::new(max_x, min_y),
        Vec2::new(max_x, max_y),
        Vec2::new(min_x, max_y),
    ]
}

/// Floor, side walls and the reservoir. The 0.4 m gap between x = -1.0 and
/// x = -0.6 under the reservoir is the outlet.
const WALLS: [[Vec2; 4]; 6] = [
    quad(-6.0, -0.5, 6.0, 0.0),
    quad(-6.0, 0.0, -5.8, 9.0),
    quad(5.8, 0.0, 6.0, 9.0),
    quad(-3.2, 4.6, -1.0, 4.8),
    quad(-3.2, 4.8, -3.0, 8.2),
    quad(-0.6, 4.6, -0.4, 8.2),
];

/// Cup left wall (rim line y = 1.8) and a tall backstop so the stream cannot
/// overshoot.
const CUP_WALLS: [[Vec2; 4]; 2] = [quad(1.4, 0.0, 1.6, 1.8), quad(3.4, 0.0, 3.6, 3.4)];

/// Reservoir water: about 1,970 particles at 0.75 × diameter spacing.
const LIQUID_BLOCKS: [LiquidBlock; 1] = [LiquidBlock {
    min: Vec2::new(-2.96, 4.84),
    max: Vec2::new(-0.64, 7.9),
    spacing: 0.06,
    color: ParticleColor::new(77, 163, 255, 255),
    counted: true,
    material: LiquidMaterial::Water,
}];

const TOOL_BUDGET: [ToolAllowance; 1] = [ToolAllowance {
    kind: ToolKind::Plank,
    count: 2,
}];

/// "First Pour": plain water, up to two planks, par one.
pub(crate) const FIRST_POUR: LevelSpec = LevelSpec {
    id: LevelId::FirstPour,
    title: "First Pour",
    gravity: Vec2::new(0.0, -10.0),
    particle_radius: 0.04,
    maximum_particle_count: 4096,
    particle_iterations: 2,
    system_tuning: SystemTuning::NONE,
    walls: &WALLS,
    cup_walls: &CUP_WALLS,
    cup_interior: GoalRect::new(Vec2::new(1.6, 0.0), Vec2::new(3.4, 1.8)),
    liquid_blocks: &LIQUID_BLOCKS,
    floats: &[],
    target_count: 540,
    // The interior holds about 3.24 m² / 0.0036 m² = 900 particles at the fill spacing.
    full_count: 900,
    settle_steps: 30,
    step_limit: 720,
    tool_budget: &TOOL_BUDGET,
    // x -1.0..2.0 m, y 2.2..4.2 m.
    build_area: GridRect::new(-20, 44, 40, 84),
    // 90% of full_count; D-07: ≥ target + 10 pp and the reference peak held clears it by ≥ 5% of full.
    brim_count: 810,
    par_tools: 1,
    no_place: &[],
};

/// Reference solution pose: centre (0.4, 3.2) m, angle -15° (step 23).
#[cfg(test)]
pub(crate) const REFERENCE_PLANK_POSE: [i32; 4] = [8, 64, 23, 0];
