//! "Timed Gate": the valve's open time decides the outcome (Phase 40 D-13,
//! D-26).
//!
//! A water tank in the upper left of a 9 m by 9 m box drains down a ramp. A
//! launch lip turns the stream into a jet that flies right, over the cup and
//! under a syrup tank, to waste on the floor beyond. The player's one valve
//! has a single slot, standing upright between the jet and the cup's far
//! wall. While closed it does two jobs:
//!
//! - its lower part stops the jet, so that water drops into the cup;
//! - its upper part covers the side opening of the syrup tank (viscous
//!   strength 1.0, D-26), whose 45° floor runs down to that opening.
//!
//! Opening the valve frees the syrup, which falls into the cup, and lets any
//! remaining jet fly past. Opened too early, most of the jet is lost. Opened
//! too late, the slow syrup cannot fill the cup before the step limit. The
//! window sweep `timed_gate_open_time_decides_the_outcome` proves both bands.
//!
//! The plan's preferred diverter (two water tanks sharing a chute) was
//! spiked first and gave a monotone sweep, because a diverted stream is
//! wasted whether or not the chute overflows. A syrup seal is the D-26
//! fallback. The valve keeps the D-04 clearance, so the seal's slit leaks
//! about 50 syrup particles over a run. A dam at the opening's sill halves
//! that leak, and the margins stay well clear of it.
//!
//! Walls are literal quads; the tank floor, the ramp and the lip are tilted
//! quads with literal vertex coordinates.

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

/// Floor, side walls, the syrup tank (45° floor, sill dam, left wall above
/// the opening, right wall), the water tank and its ramp and launch lip.
///
/// The syrup tank's side opening is the gap at x = 0.41 between the dam top
/// (y = 3.12) and the left wall (y = 3.5). The water tank's outlet is the
/// 0.5 m gap between x = -2.4 and x = -1.9.
const WALLS: [[Vec2; 4]; 11] = [
    quad(-4.5, -0.5, 4.5, 0.0),
    quad(-4.5, 0.0, -4.3, 9.0),
    quad(4.3, 0.0, 4.5, 9.0),
    [
        Vec2::new(0.41, 2.8),
        Vec2::new(1.81, 4.2),
        Vec2::new(1.81, 4.4),
        Vec2::new(0.41, 3.0),
    ],
    quad(0.41, 3.0, 0.5, 3.12),
    quad(0.41, 3.5, 0.61, 6.6),
    quad(1.81, 4.2, 2.01, 6.8),
    quad(-4.3, 5.2, -2.4, 5.4),
    quad(-1.9, 5.2, -1.7, 8.0),
    [
        Vec2::new(-2.6, 4.35),
        Vec2::new(-1.3, 3.05),
        Vec2::new(-1.3, 3.25),
        Vec2::new(-2.6, 4.55),
    ],
    [
        Vec2::new(-1.3, 3.05),
        Vec2::new(-0.8, 2.8),
        Vec2::new(-0.8, 3.0),
        Vec2::new(-1.3, 3.25),
    ],
];

/// Cup side walls; the floor closes the bottom.
const CUP_WALLS: [[Vec2; 4]; 2] = [quad(-1.4, 0.0, -1.2, 1.6), quad(0.6, 0.0, 0.8, 1.6)];

/// Tank water: about 400 particles at 0.75 × diameter spacing.
const WATER: LiquidBlock = LiquidBlock {
    min: Vec2::new(-4.26, 5.44),
    max: Vec2::new(-2.44, 6.22),
    spacing: 0.06,
    color: ParticleColor::new(77, 163, 255, 255),
    counted: true,
    material: LiquidMaterial::Water,
};

/// Tank syrup: about 400 particles. It drops onto the 45° floor at step 0
/// and settles against the closed valve.
const SYRUP: LiquidBlock = LiquidBlock {
    min: Vec2::new(0.65, 4.45),
    max: Vec2::new(1.77, 5.71),
    spacing: 0.06,
    color: ParticleColor::new(214, 140, 60, 255),
    counted: true,
    material: LiquidMaterial::Viscous,
};

const LIQUID_BLOCKS: [LiquidBlock; 2] = [WATER, SYRUP];

const TOOL_BUDGET: [ToolAllowance; 1] = [ToolAllowance {
    kind: ToolKind::Valve,
    count: 1,
}];

/// "Timed Gate": water and syrup, one valve, par one.
pub(crate) const TIMED_GATE: LevelSpec = LevelSpec {
    id: LevelId::TimedGate,
    title: "Timed Gate",
    gravity: Vec2::new(0.0, -10.0),
    particle_radius: 0.04,
    maximum_particle_count: 4096,
    particle_iterations: 2,
    // D-26 caps a valve-held syrup at 1.0 and D-29 keeps levels at 1.0 or
    // more, so 1.0 is the only allowed strength.
    system_tuning: SystemTuning {
        maybe_viscous_strength: Some(1.0),
        ..SystemTuning::NONE
    },
    walls: &WALLS,
    cup_walls: &CUP_WALLS,
    cup_interior: GoalRect::new(Vec2::new(-1.2, 0.0), Vec2::new(0.6, 1.6)),
    liquid_blocks: &LIQUID_BLOCKS,
    floats: &[],
    // About 63% of full_count.
    target_count: 502,
    // The interior holds about 2.88 m² / 0.0036 m² = 800 particles at the fill spacing.
    full_count: 800,
    settle_steps: 30,
    step_limit: 720,
    tool_budget: &TOOL_BUDGET,
    // The valve slot, x 0.1..0.2 m and y 2.95..3.15 m: every centre in it
    // both stops the jet and covers the syrup opening.
    build_area: GridRect::new(2, 59, 4, 63),
    // D-07: ≥ target + 10% of full, and the reference peak held clears it by ≥ 5% of full.
    brim_count: 582,
    par_tools: 1,
    no_place: &[],
};

/// Reference solution pose: an upright valve (angle step 6) centred at
/// (0.2, 3.05) m, opening at tick 9 (4.5 s, step 270).
#[cfg(test)]
pub(crate) const REFERENCE_VALVE_POSE: [i32; 4] = [4, 61, 6, 9];

/// Idea control: a copy that allows one plank instead of the valve, with a
/// build area large enough for the 3 m plank (D-18).
#[cfg(test)]
pub(crate) const TIMED_GATE_WITH_PLANK: LevelSpec = LevelSpec {
    tool_budget: &[ToolAllowance {
        kind: ToolKind::Plank,
        count: 1,
    }],
    build_area: GridRect::new(2, 59, 4, 66),
    ..TIMED_GATE
};

/// A permanent upright plank in the valve slot, raised clear of the cup.
#[cfg(test)]
pub(crate) const PERMANENT_PLANK_POSE: [i32; 4] = [4, 64, 6, 0];

#[cfg(test)]
mod tests {
    use super::super::LevelId;
    use super::super::probe::{print_report, probe, probe_nudges, probe_open_ticks, probe_spec};
    use super::super::replay::fixture::FixtureOutcome;
    use super::{PERMANENT_PLANK_POSE, REFERENCE_VALVE_POSE, TIMED_GATE_WITH_PLANK};

    const PLANK_CODE: i32 = 1;
    const VALVE_CODE: i32 = 2;

    fn reference_rows() -> [[i32; 5]; 1] {
        let [x, y, angle, open_tick] = REFERENCE_VALVE_POSE;
        [[VALVE_CODE, x, y, angle, open_tick]]
    }

    fn outcome_token(maybe_outcome: Option<FixtureOutcome>) -> &'static str {
        maybe_outcome.map_or("invalid", FixtureOutcome::token)
    }

    #[test]
    #[ignore = "tuning probe (D-18)"]
    fn probe_timed_gate() {
        // Arrange
        let reference = reference_rows();
        let [x, y, angle, _] = PERMANENT_PLANK_POSE;
        let plank = [[PLANK_CODE, x, y, angle, 0]];

        // Act
        let empty = probe(LevelId::TimedGate, &[]).expect("empty level probes");
        let solved = probe(LevelId::TimedGate, &reference).expect("reference probes");
        let permanent = probe_spec(&TIMED_GATE_WITH_PLANK, &plank).expect("permanent plank probes");
        let sweep = probe_open_ticks(LevelId::TimedGate, &reference, 0);
        let tally = probe_nudges(LevelId::TimedGate, &reference);

        // Assert
        print_report("timed-gate empty", &empty);
        print_report("timed-gate reference", &solved);
        print_report("timed-gate permanent-plank", &permanent);
        for (tick, maybe_outcome) in (1..).zip(&sweep) {
            println!(
                "PROBE timed-gate open_tick {tick} {}",
                outcome_token(*maybe_outcome)
            );
        }
        println!(
            "PROBE timed-gate nudges wins {} valid {} skipped {} generated {}",
            tally.wins, tally.valid, tally.skipped, tally.generated
        );
        assert_eq!((empty.outcome, solved.outcome), (2, 1));
        assert_ne!(permanent.outcome, 1);
    }
}
