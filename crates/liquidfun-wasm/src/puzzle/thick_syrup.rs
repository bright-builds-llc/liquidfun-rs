//! "Thick Syrup": viscous liquid needs a steeper path (Phase 40 D-09, D-10,
//! D-29).
//!
//! A reservoir in the upper left drains syrup through a 0.6 m outlet onto the
//! floor of an 8 m by 10 m box. The cup sits in the lower right, with the box's
//! right wall as its backstop. Two planks sloped at -30° and -45° carry the
//! syrup into the cup in time. The same planks at a shallow -15°, which pour
//! water into the cup by step 207, leave the syrup short of the target at the
//! step limit (`thick_syrup_needs_a_steep_path`).
//!
//! All geometry is literal and axis-aligned.

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

/// Floor, side walls and the reservoir. The 0.6 m gap between x = -2.0 and
/// x = -1.4 under the reservoir is the outlet.
const WALLS: [[Vec2; 4]; 5] = [
    quad(-4.0, -0.5, 4.0, 0.0),
    quad(-4.0, 0.0, -3.8, 10.0),
    quad(3.8, 0.0, 4.0, 10.0),
    quad(-3.8, 7.0, -2.0, 7.2),
    quad(-1.4, 7.0, -1.2, 9.8),
];

/// Cup left wall (rim line y = 1.7); the right side wall is the backstop.
const CUP_WALLS: [[Vec2; 4]; 1] = [quad(1.6, 0.0, 1.8, 1.7)];

/// Reservoir syrup: about 1,520 particles at 0.75 × diameter spacing.
const LIQUID_BLOCKS: [LiquidBlock; 1] = [LiquidBlock {
    min: Vec2::new(-3.76, 7.24),
    max: Vec2::new(-1.44, 9.6),
    spacing: 0.06,
    color: ParticleColor::new(214, 140, 60, 255),
    counted: true,
    material: LiquidMaterial::Viscous,
}];

const TOOL_BUDGET: [ToolAllowance; 1] = [ToolAllowance {
    kind: ToolKind::Plank,
    count: 2,
}];

/// "Thick Syrup": viscous liquid, up to two planks, par two.
pub(crate) const THICK_SYRUP: LevelSpec = LevelSpec {
    id: LevelId::ThickSyrup,
    title: "Thick Syrup",
    gravity: Vec2::new(0.0, -10.0),
    particle_radius: 0.04,
    maximum_particle_count: 4096,
    particle_iterations: 2,
    // Top of the stable 1.0..=1.5 band (D-29); weaker syrup flows too well
    // on the shallow control slope.
    system_tuning: SystemTuning {
        maybe_viscous_strength: Some(1.5),
        ..SystemTuning::NONE
    },
    walls: &WALLS,
    cup_walls: &CUP_WALLS,
    cup_interior: GoalRect::new(Vec2::new(1.8, 0.0), Vec2::new(3.8, 1.7)),
    liquid_blocks: &LIQUID_BLOCKS,
    floats: &[],
    // About 65% of full_count.
    target_count: 610,
    // The interior holds about 3.4 m² / 0.0036 m² = 944 particles at the fill spacing.
    full_count: 944,
    settle_steps: 30,
    step_limit: 720,
    tool_budget: &TOOL_BUDGET,
    // x -1.8..1.6 m, y 1.8..6.8 m.
    build_area: GridRect::new(-36, 36, 32, 136),
    // D-07: ≥ target + 10% of full, and the reference peak held clears it by ≥ 5% of full.
    brim_count: 705,
    par_tools: 2,
    no_place: &[],
};

/// Reference solution poses: centres (-0.9, 5.0) m at -30° (step 22) and
/// (0.9, 3.0) m at -45° (step 21).
#[cfg(test)]
pub(crate) const REFERENCE_PLANK_POSES: [[i32; 4]; 2] = [[-18, 100, 22, 0], [18, 60, 21, 0]];

/// Angle step 23: a shallow -15° slope.
#[cfg(test)]
pub(crate) const SHALLOW_ANGLE: i32 = 23;

#[cfg(test)]
mod tests {
    use super::super::LevelId;
    use super::super::probe::{print_report, probe, probe_nudges};
    use super::{REFERENCE_PLANK_POSES, SHALLOW_ANGLE};

    const PLANK_CODE: i32 = 1;

    fn rows(angle_override: Option<i32>) -> Vec<[i32; 5]> {
        REFERENCE_PLANK_POSES
            .iter()
            .map(|&[x, y, angle, open_tick]| {
                [PLANK_CODE, x, y, angle_override.unwrap_or(angle), open_tick]
            })
            .collect()
    }

    #[test]
    #[ignore = "tuning probe (D-18)"]
    fn probe_thick_syrup() {
        // Arrange
        let reference = rows(None);
        let shallow = rows(Some(SHALLOW_ANGLE));

        // Act
        let empty = probe(LevelId::ThickSyrup, &[]).expect("empty level probes");
        let solved = probe(LevelId::ThickSyrup, &reference).expect("reference probes");
        let slow = probe(LevelId::ThickSyrup, &shallow).expect("shallow control probes");
        let tally = probe_nudges(LevelId::ThickSyrup, &reference);

        // Assert
        print_report("thick-syrup empty", &empty);
        print_report("thick-syrup reference", &solved);
        print_report("thick-syrup shallow-15", &slow);
        println!(
            "PROBE thick-syrup nudges wins {} valid {} skipped {} generated {}",
            tally.wins, tally.valid, tally.skipped, tally.generated
        );
        assert_eq!((empty.outcome, solved.outcome), (2, 1));
        assert_ne!(slow.outcome, 1);
    }
}
