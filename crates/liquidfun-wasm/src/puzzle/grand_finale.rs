//! "Grand Finale": three earlier ideas in one level, a jelly pad, a timed
//! valve and syrup (Phase 40 D-09).
//!
//! A reservoir in the upper right of a 9 m by 11.2 m box drains into a waste
//! shaft. The player's plank catches the stream and drops it onto a long
//! roof that runs down to a chimney in the upper left. At the chimney's foot
//! an uncounted jelly pad (`LiquidMaterial::Elastic`) fills a slotted cradle
//! and turns the water sideways under a baffle, so it falls onto a 45° ramp.
//! The ramp and its launch lip turn the stream into a jet that flies right,
//! over the cup and under a syrup tank, to waste.
//!
//! The lower half is the Timed Gate tent (`timed_gate.rs`). The player's
//! valve has one slot, upright between the jet and the cup's far wall.
//! While closed, its lower part stops the jet, so the water drops into the
//! cup, and its upper part seals the syrup tank's side opening (viscous
//! strength 1.0, D-26). Opening it frees the syrup into the cup. Opened too
//! early, the jet flies past it to waste; opened too late, the slow syrup
//! cannot fill the cup in time. Without the valve the jet is lost and the
//! syrup alone falls short (`grand_finale_needs_the_valve`); without the pad
//! the stream drains through the cradle slots.
//!
//! A ceiling over the cup, a wall beside the ramp and the roof over the
//! syrup tank keep every other route out of the cup, and authored no-place
//! zones leave tools only the plank area and the valve slot.
//!
//! Walls are literal quads; the roof, the syrup tank floor, the ramp and the
//! lip are tilted quads with literal vertex coordinates.

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

/// Floor and side walls; the syrup tank (45° floor, sill dam, left and right
/// walls); the ramp and launch lip; the ceiling over the cup and the wall
/// beside the ramp; the slotted cradle with its right wall and the chimney
/// baffle; the roof; the waste shaft wall; and the reservoir.
///
/// The syrup tank's side opening is the gap at x = 0.41 between the dam top
/// (y = 3.12) and the left wall (y = 3.5). The cradle base has two 0.16 m
/// drain slots, at x = -4.15..-3.99 and x = -3.85..-3.69. The baffle leaves
/// a 0.28 m gap above the pad. The reservoir outlet is the 0.6 m gap between
/// x = 2.5 and x = 3.1.
const WALLS: [[Vec2; 4]; 20] = [
    quad(-4.5, -0.5, 4.5, 0.0),
    quad(-4.5, 0.0, -4.3, 11.2),
    quad(4.3, 0.0, 4.5, 11.2),
    [
        Vec2::new(0.41, 2.8),
        Vec2::new(1.81, 4.2),
        Vec2::new(1.81, 4.4),
        Vec2::new(0.41, 3.0),
    ],
    quad(0.41, 3.0, 0.5, 3.12),
    quad(0.41, 3.5, 0.61, 7.9),
    quad(1.81, 4.2, 2.01, 8.3),
    [
        Vec2::new(-3.0, 4.75),
        Vec2::new(-1.3, 3.05),
        Vec2::new(-1.3, 3.25),
        Vec2::new(-3.0, 4.95),
    ],
    [
        Vec2::new(-1.3, 3.05),
        Vec2::new(-0.8, 2.8),
        Vec2::new(-0.8, 3.0),
        Vec2::new(-1.3, 3.25),
    ],
    quad(-1.1, 3.9, 0.41, 4.1),
    quad(-1.3, 3.9, -1.1, 7.4),
    quad(-4.3, 5.0, -4.15, 5.2),
    quad(-3.99, 5.0, -3.85, 5.2),
    quad(-3.69, 5.0, -2.7, 5.2),
    quad(-2.9, 5.2, -2.7, 5.55),
    quad(-3.7, 5.9, -3.5, 6.7),
    [
        Vec2::new(-3.7, 6.6),
        Vec2::new(2.21, 8.3),
        Vec2::new(2.21, 8.5),
        Vec2::new(-3.7, 6.8),
    ],
    quad(2.21, 0.6, 2.41, 8.3),
    quad(2.3, 9.6, 2.5, 11.2),
    quad(3.1, 9.6, 4.3, 9.8),
];

/// Cup side walls; the floor closes the bottom.
const CUP_WALLS: [[Vec2; 4]; 2] = [quad(-1.4, 0.0, -1.2, 1.6), quad(0.6, 0.0, 0.8, 1.6)];

/// Reservoir water: about 470 particles at 0.75 × diameter spacing.
const WATER: LiquidBlock = LiquidBlock {
    min: Vec2::new(2.54, 9.84),
    max: Vec2::new(4.26, 10.82),
    spacing: 0.06,
    color: ParticleColor::new(77, 163, 255, 255),
    counted: true,
    material: LiquidMaterial::Water,
};

/// The jelly pad: about 150 uncounted elastic particles filling the cradle.
/// Its top sits just above the cradle's right wall (y = 5.55).
const PAD: LiquidBlock = LiquidBlock {
    min: Vec2::new(-4.28, 5.22),
    max: Vec2::new(-2.92, 5.62),
    spacing: 0.06,
    color: ParticleColor::new(236, 112, 178, 255),
    counted: false,
    material: LiquidMaterial::Elastic,
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

const LIQUID_BLOCKS: [LiquidBlock; 3] = [WATER, PAD, SYRUP];

const TOOL_BUDGET: [ToolAllowance; 2] = [
    ToolAllowance {
        kind: ToolKind::Plank,
        count: 2,
    },
    ToolAllowance {
        kind: ToolKind::Valve,
        count: 1,
    },
];

/// No-build zones that leave only the plank area under the reservoir and
/// the valve slot:
///
/// - the cup chamber left of the valve slot, x -1.3..-0.15 m, y 1.6..3.9 m;
/// - the space under the roof right of the slot (the syrup tank, the jet's
///   escape and the waste shaft), x 0.6..4.3 m, y 0..8.1 m;
/// - the enclosed space above the cup ceiling, x -4.3..0.6 m, y 3.9..8.2 m.
const NO_PLACE: [GridRect; 3] = [
    GridRect::new(-26, 32, -3, 78),
    GridRect::new(12, 0, 86, 162),
    GridRect::new(-86, 78, 12, 164),
];

/// "Grand Finale": water, a jelly pad and syrup; up to two planks and one
/// valve, par two.
pub(crate) const GRAND_FINALE: LevelSpec = LevelSpec {
    id: LevelId::GrandFinale,
    title: "Grand Finale",
    gravity: Vec2::new(0.0, -10.0),
    particle_radius: 0.04,
    maximum_particle_count: 4096,
    particle_iterations: 2,
    // D-26 caps a valve-held syrup at 1.0 and D-29 keeps levels at 1.0 or
    // more, so 1.0 is the only allowed strength. The pad uses jelly-pad's
    // mid-band elastic strength.
    system_tuning: SystemTuning {
        maybe_viscous_strength: Some(1.0),
        maybe_elastic_strength: Some(0.5),
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
    // The water's long route and the slow syrup need 15 s.
    step_limit: 900,
    tool_budget: &TOOL_BUDGET,
    // x 0.1..4.2 m, y 2.95..9.5 m; the no-place zones carve it down to the
    // plank area and the valve slot (x 0.1..0.2 m, y 2.95..3.15 m).
    build_area: GridRect::new(2, 59, 84, 190),
    // D-07: ≥ target + 10% of full, and the reference peak held clears it by ≥ 5% of full.
    brim_count: 582,
    par_tools: 2,
    no_place: &NO_PLACE,
};

/// Reference plank pose: centre (2.3, 8.95) m at 15° (angle step 1), under
/// the reservoir outlet and sloping down toward the roof.
#[cfg(test)]
pub(crate) const REFERENCE_PLANK_POSE: [i32; 4] = [46, 179, 1, 0];

/// Reference valve pose: upright (angle step 6), centred at (0.2, 3.05) m,
/// opening at tick 14 (7.0 s, step 420).
#[cfg(test)]
pub(crate) const REFERENCE_VALVE_POSE: [i32; 4] = [4, 61, 6, 14];

/// Idea control: `GRAND_FINALE` with the jelly pad removed (D-18).
#[cfg(test)]
pub(crate) const GRAND_FINALE_WITHOUT_PAD: LevelSpec = LevelSpec {
    liquid_blocks: &[WATER, SYRUP],
    ..GRAND_FINALE
};

/// Idea control: a copy that allows a third plank, so a plank can be tried
/// in the valve's place (D-18).
#[cfg(test)]
pub(crate) const GRAND_FINALE_WITH_PLANKS: LevelSpec = LevelSpec {
    tool_budget: &[ToolAllowance {
        kind: ToolKind::Plank,
        count: 3,
    }],
    ..GRAND_FINALE
};

#[cfg(test)]
mod tests {
    use super::super::LevelId;
    use super::super::probe::{print_report, probe, probe_nudges, probe_open_ticks, probe_spec};
    use super::super::replay::fixture::FixtureOutcome;
    use super::{
        GRAND_FINALE, GRAND_FINALE_WITH_PLANKS, GRAND_FINALE_WITHOUT_PAD, REFERENCE_PLANK_POSE,
        REFERENCE_VALVE_POSE,
    };

    const PLANK_CODE: i32 = 1;
    const VALVE_CODE: i32 = 2;

    fn plank_row() -> [i32; 5] {
        let [x, y, angle, open_tick] = REFERENCE_PLANK_POSE;
        [PLANK_CODE, x, y, angle, open_tick]
    }

    fn reference_rows() -> [[i32; 5]; 2] {
        let [x, y, angle, open_tick] = REFERENCE_VALVE_POSE;
        [plank_row(), [VALVE_CODE, x, y, angle, open_tick]]
    }

    fn outcome_token(maybe_outcome: Option<FixtureOutcome>) -> &'static str {
        maybe_outcome.map_or("invalid", FixtureOutcome::token)
    }

    #[test]
    #[ignore = "tuning probe (D-18)"]
    fn probe_grand_finale() {
        // Arrange
        let reference = reference_rows();
        let plank_only = [plank_row()];
        let [x, y, angle, _] = REFERENCE_VALVE_POSE;
        let plank_instead = [plank_row(), [PLANK_CODE, x, y, angle, 0]];
        let mut never_opens = reference;
        never_opens[1][4] = i32::from(GRAND_FINALE.max_open_tick());

        // Act
        let empty = probe(LevelId::GrandFinale, &[]).expect("empty level probes");
        let solved = probe(LevelId::GrandFinale, &reference).expect("reference probes");
        let no_valve = probe(LevelId::GrandFinale, &plank_only).expect("no-valve control probes");
        let maybe_plank = probe_spec(&GRAND_FINALE_WITH_PLANKS, &plank_instead);
        let closed = probe(LevelId::GrandFinale, &never_opens).expect("closed valve probes");
        let without_pad =
            probe_spec(&GRAND_FINALE_WITHOUT_PAD, &reference).expect("no-pad control probes");
        let sweep = probe_open_ticks(LevelId::GrandFinale, &reference, 1);
        let tally = probe_nudges(LevelId::GrandFinale, &reference);

        // Assert
        print_report("grand-finale empty", &empty);
        print_report("grand-finale reference", &solved);
        print_report("grand-finale no-valve", &no_valve);
        match &maybe_plank {
            Ok(report) => print_report("grand-finale plank-instead", report),
            Err(error) => println!("PROBE grand-finale plank-instead invalid {error:?}"),
        }
        print_report("grand-finale valve-never-opens", &closed);
        print_report("grand-finale without-pad", &without_pad);
        for (tick, maybe_outcome) in (1..).zip(&sweep) {
            println!(
                "PROBE grand-finale open_tick {tick} {}",
                outcome_token(*maybe_outcome)
            );
        }
        println!(
            "PROBE grand-finale nudges wins {} valid {} skipped {} generated {}",
            tally.wins, tally.valid, tally.skipped, tally.generated
        );
        assert_eq!((empty.outcome, solved.outcome), (2, 1));
        assert_ne!(no_valve.outcome, 1);
        assert_ne!(closed.outcome, 1);
        assert_ne!(without_pad.outcome, 1);
    }
}
