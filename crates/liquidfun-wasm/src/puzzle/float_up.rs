//! "Float Up": a floating ball plugs the drain of its well (Phase 40 D-09,
//! D-12, D-28).
//!
//! A reservoir in the upper left of an 8 m by 11.5 m box drains onto a
//! short gutter, whose end drops the stream into a waste pit on the floor.
//! One plank catches that drop and carries it right into a funnel. The
//! funnel feeds a narrow pipe, and slots at the pipe's foot open into the
//! ball's well.
//!
//! The well's ceiling slopes up to a drain hole. The hole spills over a low
//! weir into the waste pit, and the window from the pipe into the cup sits
//! 0.5 m above that weir. The ball is a circle of density 0.4. As the well
//! fills, the ball floats up, seats in the drain hole and plugs it. The pipe
//! then fills to the window and spills into the cup.
//!
//! A ball of density 2.0 stays on the well floor, so the drain stays open
//! and the water runs to waste (`float_up_needs_buoyancy`). The cup and its
//! count region never move (D-12).
//!
//! A hanging baffle keeps the funnel's stream off the window, and the cup
//! roof is thick where it meets the side wall. Without them, water reached
//! the cup whether or not the ball floated.
//!
//! Walls are literal quads. The gutter, the well ceiling, the waste roof,
//! the funnel wall and the cup roof are tilted, with literal vertex
//! coordinates.

use liquidfun::math::Vec2;
use liquidfun::particle::ParticleColor;

use super::LevelId;
use super::goal::GoalRect;
use super::spec::{FloatBody, LevelSpec, LiquidBlock, LiquidMaterial, SystemTuning};
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

/// Floor, side walls, the reservoir, the gutter and its backstop, the well,
/// the pipe with its slots and trap baffle, the waste roof and the funnel.
///
/// - The reservoir outlet is the 0.4 m gap between x = -2.4 and x = -2.0.
/// - The gutter ends at x = -1.6; the stream drops from there.
/// - The well is x 0.4..1.4 m with its floor at y = 2.0. Its ceiling rises
///   to the 0.5 m drain hole at x 0.65..1.15, y = 3.05. The left wall's top,
///   y = 3.25, is the weir the drain spills over.
/// - The pipe is x 1.6..1.9. Its slots into the well are y 2.2..2.45 and
///   2.6..2.8. The trap baffle at x 1.9..1.96 hangs down to y 3.55.
const WALLS: [[Vec2; 4]; 19] = [
    quad(-4.0, -0.5, 4.0, 0.0),
    quad(-4.0, -0.5, -3.8, 11.0),
    quad(3.8, -0.5, 4.0, 11.0),
    quad(-3.9, 7.0, -2.4, 7.2),
    quad(-2.0, 7.0, -1.1, 7.2),
    quad(-1.2, 7.0, -1.0, 10.8),
    [
        Vec2::new(-2.9, 6.4),
        Vec2::new(-1.6, 6.05),
        Vec2::new(-1.6, 6.25),
        Vec2::new(-2.9, 6.6),
    ],
    quad(-3.1, 6.4, -2.9, 7.05),
    quad(0.2, 1.8, 2.4, 2.0),
    quad(0.2, 1.9, 0.4, 3.25),
    [
        Vec2::new(0.4, 2.85),
        Vec2::new(0.65, 3.05),
        Vec2::new(0.65, 3.25),
        Vec2::new(0.4, 3.25),
    ],
    [
        Vec2::new(1.15, 3.05),
        Vec2::new(1.4, 2.85),
        Vec2::new(1.4, 4.3),
        Vec2::new(1.15, 4.3),
    ],
    quad(1.4, 1.9, 1.6, 2.2),
    quad(1.4, 2.45, 1.6, 2.6),
    quad(1.4, 2.8, 1.6, 4.7),
    quad(1.9, 3.55, 1.96, 4.85),
    [
        Vec2::new(-0.1, 3.7),
        Vec2::new(1.5, 4.4),
        Vec2::new(1.5, 4.6),
        Vec2::new(-0.1, 3.9),
    ],
    [
        Vec2::new(1.9, 4.8),
        Vec2::new(2.1, 4.8),
        Vec2::new(2.9, 5.1),
        Vec2::new(2.7, 5.1),
    ],
    quad(1.96, 4.45, 2.4, 4.85),
];

/// The cup's left wall below the 0.7 m window (y 3.75..4.45), and the roof
/// that closes the cup. The roof drains into the funnel and is 0.64 m thick
/// at the side wall, where a thin roof let pooled water through. The right
/// side wall is the cup's far wall.
const CUP_WALLS: [[Vec2; 4]; 2] = [
    quad(2.2, -0.1, 2.4, 3.75),
    [
        Vec2::new(2.7, 4.95),
        Vec2::new(3.8, 5.36),
        Vec2::new(3.8, 6.0),
        Vec2::new(2.7, 5.15),
    ],
];

/// Reservoir water: 2,394 particles at 0.75 × diameter spacing.
const WATER: LiquidBlock = LiquidBlock {
    min: Vec2::new(-3.76, 7.24),
    max: Vec2::new(-1.24, 10.66),
    spacing: 0.06,
    color: ParticleColor::new(77, 163, 255, 255),
    counted: true,
    material: LiquidMaterial::Water,
};

const LIQUID_BLOCKS: [LiquidBlock; 1] = [WATER];

/// The ball: it rests on the well floor and floats up 0.44 m to seat in the
/// drain hole, centre (0.9, 2.8).
pub(crate) const FLOAT: FloatBody = FloatBody {
    center: Vec2::new(0.9, 2.36),
    radius: 0.35,
    density: 0.4,
};

const FLOATS: [FloatBody; 1] = [FLOAT];

const TOOL_BUDGET: [ToolAllowance; 1] = [ToolAllowance {
    kind: ToolKind::Plank,
    count: 2,
}];

/// No-build zone over the well, where the ball rises: x 0.4..1.4 m,
/// y 2.0..3.25 m.
const NO_PLACE: [GridRect; 1] = [GridRect::new(8, 40, 28, 65)];

/// "Float Up": water and a floating ball, up to two planks, par one.
pub(crate) const FLOAT_UP: LevelSpec = LevelSpec {
    id: LevelId::FloatUp,
    title: "Float Up",
    gravity: Vec2::new(0.0, -10.0),
    particle_radius: 0.04,
    maximum_particle_count: 4096,
    particle_iterations: 2,
    system_tuning: SystemTuning::NONE,
    walls: &WALLS,
    cup_walls: &CUP_WALLS,
    cup_interior: GoalRect::new(Vec2::new(2.4, 0.0), Vec2::new(3.8, 1.0)),
    liquid_blocks: &LIQUID_BLOCKS,
    floats: &FLOATS,
    // 60% of full_count.
    target_count: 233,
    // The interior holds about 1.4 m² / 0.0036 m² = 389 particles at the fill spacing.
    full_count: 389,
    settle_steps: 30,
    step_limit: 720,
    tool_budget: &TOOL_BUDGET,
    // x -3.0..1.2 m, y 3.6..6.2 m.
    build_area: GridRect::new(-60, 72, 24, 124),
    // D-07: ≥ target + 10% of full, and the reference peak held clears it by ≥ 5% of full.
    brim_count: 272,
    par_tools: 1,
    no_place: &NO_PLACE,
};

/// Reference solution pose: centre (-0.2, 5.2) m, angle -15° (step 23).
#[cfg(test)]
pub(crate) const REFERENCE_PLANK_POSE: [i32; 4] = [-4, 104, 23, 0];

/// Idea control: `FLOAT_UP` with a ball of density 2.0, which sinks
/// (D-12, D-18).
#[cfg(test)]
pub(crate) const FLOAT_UP_DENSE: LevelSpec = LevelSpec {
    floats: &[FloatBody {
        density: 2.0,
        ..FLOAT
    }],
    ..FLOAT_UP
};

#[cfg(test)]
mod tests {
    use super::super::LevelId;
    use super::super::probe::{print_report, probe, probe_nudges, probe_spec};
    use super::{FLOAT_UP_DENSE, REFERENCE_PLANK_POSE};

    const PLANK_CODE: i32 = 1;

    fn reference_rows() -> [[i32; 5]; 1] {
        let [x, y, angle, open_tick] = REFERENCE_PLANK_POSE;
        [[PLANK_CODE, x, y, angle, open_tick]]
    }

    #[test]
    #[ignore = "tuning probe (D-18)"]
    fn probe_float_up() {
        // Arrange
        let reference = reference_rows();

        // Act
        let empty = probe(LevelId::FloatUp, &[]).expect("empty level probes");
        let solved = probe(LevelId::FloatUp, &reference).expect("reference probes");
        let dense = probe_spec(&FLOAT_UP_DENSE, &reference).expect("dense control probes");
        let tally = probe_nudges(LevelId::FloatUp, &reference);

        // Assert
        print_report("float-up empty", &empty);
        print_report("float-up reference", &solved);
        print_report("float-up dense-float", &dense);
        println!(
            "PROBE float-up nudges wins {} valid {} skipped {} generated {}",
            tally.wins, tally.valid, tally.skipped, tally.generated
        );
        assert_eq!((empty.outcome, solved.outcome), (2, 1));
        assert_ne!(dense.outcome, 1);
    }
}
