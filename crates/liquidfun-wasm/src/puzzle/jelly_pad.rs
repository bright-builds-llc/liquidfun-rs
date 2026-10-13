//! "Jelly Pad": a soft elastic pad that turns the stream toward the cup and
//! never counts (Phase 40 D-09, D-11).
//!
//! A reservoir in the upper left of an 8 m by 11 m box drains onto the floor.
//! The cup sits raised in the lower right, under a roof, so water can reach
//! it only along a low channel. The channel starts at a slotted cradle at the
//! foot of a chimney. An uncounted jelly pad (`LiquidMaterial::Elastic`, a
//! `SOLID` group) fills the cradle level with the channel floor. One plank
//! drops the stream down the chimney onto the pad. The pad turns the water
//! sideways under the chimney baffle, along the channel and into the cup.
//! Without the pad the same stream falls through the cradle slots and the
//! level is lost (`jelly_pad_needs_the_pad`).
//!
//! Walls are literal quads; the roof is the only tilted one, with literal
//! vertex coordinates.

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

/// Floor, side walls, reservoir, the slotted cradle with its chimney, the
/// channel floor, the roof over the cup and the chimney baffle.
///
/// The reservoir outlet is the 0.4 m gap between x = -2.4 and x = -2.0. The
/// cradle base has two 0.16 m drain slots, at x = -0.7..-0.54 and
/// x = 0.3..0.46. The roof rises 0.4 m toward the right wall. The baffle
/// leaves a 0.28 m gap above the pad.
const WALLS: [[Vec2; 4]; 12] = [
    quad(-4.0, -0.5, 4.0, 0.0),
    quad(-4.0, 0.0, -3.8, 11.0),
    quad(3.8, 0.0, 4.0, 11.0),
    quad(-3.8, 8.0, -2.4, 8.2),
    quad(-2.0, 8.0, -1.8, 10.8),
    quad(-1.4, 3.2, -0.7, 3.4),
    quad(-0.54, 3.2, 0.3, 3.4),
    quad(0.46, 3.2, 1.2, 3.4),
    quad(-1.4, 3.4, -1.2, 4.6),
    quad(1.2, 3.2, 2.0, 3.8),
    [
        Vec2::new(0.0, 4.4),
        Vec2::new(3.8, 4.8),
        Vec2::new(3.8, 5.0),
        Vec2::new(0.0, 4.6),
    ],
    quad(0.0, 4.1, 0.2, 6.4),
];

/// Raised cup floor and left wall; the right side wall is the backstop and
/// the roof closes the top.
const CUP_WALLS: [[Vec2; 4]; 2] = [quad(1.8, 1.8, 3.8, 2.0), quad(1.8, 2.0, 2.0, 3.2)];

/// Reservoir water: about 1,130 particles at 0.75 × diameter spacing.
const WATER: LiquidBlock = LiquidBlock {
    min: Vec2::new(-3.76, 8.24),
    max: Vec2::new(-2.04, 10.6),
    spacing: 0.06,
    color: ParticleColor::new(77, 163, 255, 255),
    counted: true,
    material: LiquidMaterial::Water,
};

/// The jelly pad: about 230 uncounted elastic particles filling the cradle.
/// Its top sits level with the channel floor (y = 3.8).
const PAD: LiquidBlock = LiquidBlock {
    min: Vec2::new(-1.18, 3.42),
    max: Vec2::new(1.18, 3.82),
    spacing: 0.06,
    color: ParticleColor::new(236, 112, 178, 255),
    counted: false,
    material: LiquidMaterial::Elastic,
};

const LIQUID_BLOCKS: [LiquidBlock; 2] = [WATER, PAD];

const TOOL_BUDGET: [ToolAllowance; 1] = [ToolAllowance {
    kind: ToolKind::Plank,
    count: 2,
}];

/// No-build zone over the cradle and the chimney foot: x -1.4..0.0 m,
/// y 3.2..4.6 m.
const NO_PLACE: [GridRect; 1] = [GridRect::new(-28, 64, 0, 92)];

/// "Jelly Pad": water and an uncounted jelly pad, up to two planks, par one.
pub(crate) const JELLY_PAD: LevelSpec = LevelSpec {
    id: LevelId::JellyPad,
    title: "Jelly Pad",
    gravity: Vec2::new(0.0, -10.0),
    particle_radius: 0.04,
    maximum_particle_count: 4096,
    particle_iterations: 2,
    // Mid-band elastic strength; the pad centroid stays within about 0.15 m.
    system_tuning: SystemTuning {
        maybe_elastic_strength: Some(0.5),
        ..SystemTuning::NONE
    },
    walls: &WALLS,
    cup_walls: &CUP_WALLS,
    cup_interior: GoalRect::new(Vec2::new(2.0, 2.0), Vec2::new(3.8, 3.4)),
    liquid_blocks: &LIQUID_BLOCKS,
    floats: &[],
    // 60% of full_count.
    target_count: 420,
    // The interior holds about 2.52 m² / 0.0036 m² = 700 particles at the fill spacing.
    full_count: 700,
    settle_steps: 30,
    step_limit: 720,
    tool_budget: &TOOL_BUDGET,
    // x -3.6..-0.4 m, y 4.8..7.6 m.
    build_area: GridRect::new(-72, 96, -8, 152),
    // 90% of full_count; D-07: ≥ target + 10% of full, and the reference peak held clears it by ≥ 5% of full.
    brim_count: 630,
    par_tools: 1,
    no_place: &NO_PLACE,
};

/// Reference solution pose: centre (-2.0, 6.0) m, angle -30° (step 22).
#[cfg(test)]
pub(crate) const REFERENCE_PLANK_POSE: [i32; 4] = [-40, 120, 22, 0];

/// Idea control: `JELLY_PAD` with the pad block removed (D-18).
#[cfg(test)]
pub(crate) const JELLY_PAD_WITHOUT_PAD: LevelSpec = LevelSpec {
    liquid_blocks: &[WATER],
    ..JELLY_PAD
};

/// Mean position of the jelly pad's particles. The pad is the level's only
/// elastic group, so its particles are exactly those with the `ELASTIC` flag.
///
/// # Panics
///
/// Panics when the session has no particle view or no pad particles.
#[cfg(test)]
pub(crate) fn pad_centroid(core: &crate::session::SessionCore) -> Vec2 {
    use liquidfun::particle::ParticleFlags;

    core.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("live particle system has a view");
        let (sum, count) = view
            .positions()
            .iter()
            .zip(view.flags())
            .filter(|(_, flags)| flags.contains(ParticleFlags::ELASTIC))
            .fold(
                (Vec2::new(0.0, 0.0), 0_u16),
                |(sum, count), (position, _)| (sum + *position, count + 1),
            );
        assert!(count > 0, "the jelly pad has particles");
        sum * (1.0 / f32::from(count))
    })
}

#[cfg(test)]
mod tests {
    use super::super::LevelId;
    use super::super::probe::{print_report, probe, probe_nudges, probe_spec};
    use super::super::tools::Placements;
    use super::{JELLY_PAD, JELLY_PAD_WITHOUT_PAD, REFERENCE_PLANK_POSE, pad_centroid};
    use crate::puzzle::validate::validate_for_build;
    use crate::session::SessionCore;

    const PLANK_CODE: i32 = 1;

    fn reference_rows() -> [[i32; 5]; 1] {
        let [x, y, angle, open_tick] = REFERENCE_PLANK_POSE;
        [[PLANK_CODE, x, y, angle, open_tick]]
    }

    /// Largest pad centroid displacement from step 0 over the step limit.
    fn max_pad_drift(placements: Placements) -> f32 {
        let mut core =
            SessionCore::create_puzzle(LevelId::JellyPad, placements).expect("reference builds");
        let start = pad_centroid(&core);
        let mut drift: f32 = 0.0;
        for _ in 0..JELLY_PAD.step_limit {
            core.advance(1).expect("level steps");
            drift = drift.max((pad_centroid(&core) - start).length());
        }
        drift
    }

    #[test]
    #[ignore = "tuning probe (D-18)"]
    fn probe_jelly_pad() {
        // Arrange
        let reference = reference_rows();
        let [_, x, y, angle, open_tick] = reference[0];
        let validated = validate_for_build(&JELLY_PAD, &[1], &[x, y, angle, open_tick])
            .expect("reference placement is valid");

        // Act
        let empty = probe(LevelId::JellyPad, &[]).expect("empty level probes");
        let solved = probe(LevelId::JellyPad, &reference).expect("reference probes");
        let without_pad =
            probe_spec(&JELLY_PAD_WITHOUT_PAD, &reference).expect("no-pad control probes");
        let tally = probe_nudges(LevelId::JellyPad, &reference);
        let drift = max_pad_drift(validated);

        // Assert
        print_report("jelly-pad empty", &empty);
        print_report("jelly-pad reference", &solved);
        print_report("jelly-pad without-pad", &without_pad);
        println!("PROBE jelly-pad pad-drift max {drift:.3} m");
        println!(
            "PROBE jelly-pad nudges wins {} valid {} skipped {} generated {}",
            tally.wins, tally.valid, tally.skipped, tally.generated
        );
        assert_eq!((empty.outcome, solved.outcome), (2, 1));
        assert_ne!(without_pad.outcome, 1);
    }
}
