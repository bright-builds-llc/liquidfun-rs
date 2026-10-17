//! Test-only helpers shared by the ferrofluid unit, spike and flow tests: the
//! fixed step, the binned surface-peak metric (Research "Surface-Peak
//! Metric"), the hold positions and P3 hold rules, and session helpers.

use liquidfun::StepConfiguration;
use liquidfun::math::Vec2;

use super::geometry::{
    DISH_INNER_HALF_WIDTH, FRAME_MAX_X, FRAME_MAX_Y, FRAME_MIN_X, FRAME_MIN_Y, MAGNET_MAX_Y,
    PARTICLE_DIAMETER, PARTICLE_RADIUS, out_of_dish,
};
use crate::scene::{SceneId, particle_iterations};
use crate::session::{MAX_ADVANCE_STEPS, SessionCore};

/// Least topographic prominence of a counted surface peak: about two diameters.
pub(super) const PEAK_PROMINENCE: f32 = 0.15;
/// Largest smoothed surface range that still reads as flat: 1.5 diameters.
pub(super) const FLAT_RANGE: f32 = 0.12;
/// Profile bins stay two diameters clear of each inner wall face.
const BIN_START: f32 = -DISH_INNER_HALF_WIDTH + 2.0 * PARTICLE_DIAMETER;
/// One bin per particle diameter across the binned span.
const BIN_COUNT: u16 = 71;
/// How far past the frame a particle may fly before it counts as escaped.
const ESCAPE_MARGIN: f32 = 0.5;
/// A particle with no neighbor center within this distance is isolated (flung).
const NEIGHBOR_DISTANCE: f32 = 3.0 * PARTICLE_RADIUS;

/// The session's fixed step: 1/60 s, 8 velocity and 3 position iterations,
/// and the scene's particle iterations.
pub(super) fn step_configuration() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3)
        .expect("valid step")
        .with_particle_iterations(particle_iterations(SceneId::FakeFerrofluid))
        .expect("particle iterations")
}

/// A 3-bin-median smoothed max-height profile across the dish (gravity straight down).
#[derive(Debug, Clone, PartialEq)]
pub(super) struct SurfaceProfile {
    tops: Vec<f32>,
}

/// Bins particle centers one diameter wide, keeps each bin's highest center
/// (an empty bin reads as the floor, 0), then smooths with a 3-bin median.
pub(super) fn surface_profile(positions: &[Vec2]) -> SurfaceProfile {
    let bins = usize::from(BIN_COUNT);
    let mut raw = vec![0.0_f32; bins];
    for position in positions {
        let Some(index) = bin_index(position.x) else {
            continue;
        };
        raw[index] = raw[index].max(position.y);
    }
    let tops = (0..bins)
        .map(|index| {
            let left = raw[index.saturating_sub(1)];
            let right = raw[(index + 1).min(bins - 1)];
            median3(left, raw[index], right)
        })
        .collect();
    SurfaceProfile { tops }
}

/// The bin holding `x`, or `None` outside the binned span.
fn bin_index(x: f32) -> Option<usize> {
    let offset = ((x - BIN_START) / PARTICLE_DIAMETER).floor();
    if !(0.0..f32::from(BIN_COUNT)).contains(&offset) {
        return None;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "offset is a whole number inside 0..BIN_COUNT"
    )]
    let index = offset as usize;
    Some(index)
}

fn median3(a: f32, b: f32, c: f32) -> f32 {
    a.max(b).min(a.min(b).max(c))
}

impl SurfaceProfile {
    /// Bins that are local maxima (`top[i-1] < top[i] >= top[i+1]`, edges
    /// allowed) with a prominence of at least `PEAK_PROMINENCE`.
    fn peak_bins(&self) -> Vec<usize> {
        let tops = &self.tops;
        (0..tops.len())
            .filter(|&index| {
                let rises = index == 0 || tops[index - 1] < tops[index];
                let holds = index + 1 == tops.len() || tops[index] >= tops[index + 1];
                rises && holds && self.prominence(index) >= PEAK_PROMINENCE
            })
            .collect()
    }

    /// Topographic prominence: the peak height above the higher of its two
    /// bases, each the lowest bin before a higher bin or the profile edge.
    fn prominence(&self, index: usize) -> f32 {
        let tops = &self.tops;
        let top = tops[index];
        let left_min = tops[..=index]
            .iter()
            .rev()
            .take_while(|height| **height <= top)
            .fold(top, |low, height| low.min(*height));
        let right_min = tops[index..]
            .iter()
            .take_while(|height| **height <= top)
            .fold(top, |low, height| low.min(*height));
        top - left_min.max(right_min)
    }

    /// Number of separate surface peaks.
    pub(super) fn peak_count(&self) -> usize {
        self.peak_bins().len()
    }

    /// Mean x of the peak bin centers, or `None` without peaks.
    pub(super) fn peak_centroid_x(&self) -> Option<f32> {
        let peaks = self.peak_bins();
        if peaks.is_empty() {
            return None;
        }
        let sum: f32 = peaks.iter().map(|&index| bin_center_x(index)).sum();
        Some(sum / count_as_f32(peaks.len()))
    }

    /// No peaks and a smoothed range of at most `FLAT_RANGE`.
    pub(super) fn is_flat(&self) -> bool {
        let highest = self.tops.iter().copied().fold(f32::MIN, f32::max);
        let lowest = self.tops.iter().copied().fold(f32::MAX, f32::min);
        self.peak_count() == 0 && highest - lowest <= FLAT_RANGE
    }
}

fn bin_center_x(index: usize) -> f32 {
    BIN_START + (count_as_f32(index) + 0.5) * PARTICLE_DIAMETER
}

/// A small count as `f32`; test counts stay far below `u16::MAX`.
pub(super) fn count_as_f32(count: usize) -> f32 {
    f32::from(u16::try_from(count).expect("test counts fit in u16"))
}

/// Kinetic-energy proxy: the sum of `0.5 * |v|^2` (unit mass).
pub(super) fn kinetic_energy(velocities: &[Vec2]) -> f32 {
    velocities
        .iter()
        .map(|velocity| 0.5 * velocity.length_squared())
        .sum()
}

/// Fastest particle speed.
pub(super) fn max_speed(velocities: &[Vec2]) -> f32 {
    velocities
        .iter()
        .map(|velocity| velocity.length())
        .fold(0.0, f32::max)
}

/// Particle centers outside the dish interior.
pub(super) fn out_of_dish_count(positions: &[Vec2]) -> usize {
    positions
        .iter()
        .filter(|position| out_of_dish(**position))
        .count()
}

/// Non-finite particles and particles beyond the frame grown by `ESCAPE_MARGIN`.
pub(super) fn escaped_count(positions: &[Vec2]) -> usize {
    positions
        .iter()
        .filter(|position| {
            !position.x.is_finite()
                || !position.y.is_finite()
                || position.x < FRAME_MIN_X - ESCAPE_MARGIN
                || position.x > FRAME_MAX_X + ESCAPE_MARGIN
                || position.y < FRAME_MIN_Y - ESCAPE_MARGIN
                || position.y > FRAME_MAX_Y + ESCAPE_MARGIN
        })
        .count()
}

/// Particles with no other particle center within `NEIGHBOR_DISTANCE` (O(n^2)).
pub(super) fn isolated_count(positions: &[Vec2]) -> usize {
    let limit = NEIGHBOR_DISTANCE * NEIGHBOR_DISTANCE;
    positions
        .iter()
        .enumerate()
        .filter(|(index, position)| {
            !positions.iter().enumerate().any(|(other_index, other)| {
                other_index != *index && (*other - **position).length_squared() <= limit
            })
        })
        .count()
}

/// Highest particle center: the tallest column top.
pub(super) fn column_top(positions: &[Vec2]) -> f32 {
    positions
        .iter()
        .map(|position| position.y)
        .fold(f32::MIN, f32::max)
}

/// Fewest peaks the roadmap floor accepts.
pub(super) const MIN_PEAKS: usize = 4;
/// D-04 target peak count at the default strength.
pub(super) const TARGET_PEAKS: std::ops::RangeInclusive<usize> = 5..=9;
/// Step at which the crown is counted against the D-04 target.
pub(super) const CROWN_STEP: u32 = 180;

/// Magnet resting under the dish floor.
pub(super) const UNDER_THE_DISH: Vec2 = Vec2::new(0.0, -0.35);
/// Magnet held above the puddle, as high as the clamp allows.
pub(super) const ABOVE_THE_PUDDLE: Vec2 = Vec2::new(0.0, MAGNET_MAX_Y);
/// Magnet under the dish, far off-center.
pub(super) const OFF_CENTER: Vec2 = Vec2::new(2.0, -0.35);
/// Magnet tapped into the middle of the puddle.
pub(super) const INSIDE_THE_PUDDLE: Vec2 = Vec2::new(0.0, 0.3);
/// Magnet tapped into the puddle next to the right wall.
pub(super) const INSIDE_THE_PUDDLE_AT_THE_WALL: Vec2 = Vec2::new(2.7, 0.3);
/// Magnet held where it gathers the tallest measured column (quick 261016-u0s probe).
pub(super) const HIGH_ABOVE_THE_PUDDLE: Vec2 = Vec2::new(1.5, MAGNET_MAX_Y);
/// Magnet held at the top right corner of the clamp, outside the inner face.
pub(super) const HIGH_AT_THE_EDGE: Vec2 = Vec2::new(3.2, MAGNET_MAX_Y);
/// Every hold position, plus the two tallest-column positions, for the
/// containment margin check.
pub(super) const HOLD_POSITIONS: [Vec2; 7] = [
    UNDER_THE_DISH,
    ABOVE_THE_PUDDLE,
    OFF_CENTER,
    INSIDE_THE_PUDDLE,
    INSIDE_THE_PUDDLE_AT_THE_WALL,
    HIGH_ABOVE_THE_PUDDLE,
    HIGH_AT_THE_EDGE,
];

/// Particle placement samples are taken every this many hold steps.
const HOLD_SAMPLE_EVERY: u32 = 30;
/// Highest particle speed any hold may reach, in m/s (clamp 9.6).
const MAX_HOLD_SPEED: f32 = 4.0;
/// Early window for the max-speed growth check, in steps.
const EARLY_WINDOW: std::ops::Range<u32> = 120..720;
/// Late window for the max-speed growth check, in steps.
const LATE_WINDOW: std::ops::Range<u32> = 3_000..3_600;
/// How much faster the late window may get than the early one, in m/s.
const SPEED_GROWTH_ALLOWANCE: f32 = 0.25;
/// First kinetic-energy half of a hold, after the 2 s transient.
const FIRST_HALF: std::ops::Range<u32> = 120..1_860;
/// Second kinetic-energy half of a hold.
const SECOND_HALF: std::ops::Range<u32> = 1_860..3_600;
/// The second-half kinetic-energy mean may be at most this times the first...
const KE_GROWTH_FACTOR: f32 = 1.25;
/// ...plus this absolute allowance.
const KE_GROWTH_ALLOWANCE: f32 = 0.5;
/// Most isolated (flung) particles any hold sample may show.
const MAX_ISOLATED: usize = 3;

/// Measurements from one hold run.
#[derive(Debug, Clone, Copy)]
pub(super) struct HoldReport {
    pub(super) max_out_of_dish: usize,
    pub(super) max_escaped: usize,
    pub(super) max_isolated: usize,
    pub(super) max_speed: f32,
    pub(super) early_max_speed: f32,
    pub(super) late_max_speed: f32,
    pub(super) first_half_mean_energy: f32,
    pub(super) second_half_mean_energy: f32,
    pub(super) max_column_top: f32,
}

/// Accumulates a [`HoldReport`] from per-step particle reads.
pub(super) struct HoldRecorder {
    report: HoldReport,
    first_half: Vec<f32>,
    second_half: Vec<f32>,
}

impl HoldRecorder {
    pub(super) fn new() -> Self {
        Self {
            report: HoldReport {
                max_out_of_dish: 0,
                max_escaped: 0,
                max_isolated: 0,
                max_speed: 0.0,
                early_max_speed: 0.0,
                late_max_speed: 0.0,
                first_half_mean_energy: 0.0,
                second_half_mean_energy: 0.0,
                max_column_top: f32::MIN,
            },
            first_half: Vec::new(),
            second_half: Vec::new(),
        }
    }

    /// Records the particles after fixed step `step` (1-based).
    pub(super) fn record(&mut self, step: u32, placed: &[Vec2], lanes: &[Vec2]) {
        let report = &mut self.report;
        let speed = max_speed(lanes);
        report.max_speed = report.max_speed.max(speed);
        report.max_column_top = report.max_column_top.max(column_top(placed));
        if EARLY_WINDOW.contains(&step) {
            report.early_max_speed = report.early_max_speed.max(speed);
        }
        if LATE_WINDOW.contains(&step) {
            report.late_max_speed = report.late_max_speed.max(speed);
        }
        if FIRST_HALF.contains(&step) {
            self.first_half.push(kinetic_energy(lanes));
        }
        if SECOND_HALF.contains(&step) {
            self.second_half.push(kinetic_energy(lanes));
        }
        if step.is_multiple_of(HOLD_SAMPLE_EVERY) {
            report.max_out_of_dish = report.max_out_of_dish.max(out_of_dish_count(placed));
            report.max_escaped = report.max_escaped.max(escaped_count(placed));
            report.max_isolated = report.max_isolated.max(isolated_count(placed));
        }
    }

    pub(super) fn finish(mut self) -> HoldReport {
        self.report.first_half_mean_energy = mean(&self.first_half);
        self.report.second_half_mean_energy = mean(&self.second_half);
        self.report
    }
}

fn mean(values: &[f32]) -> f32 {
    values.iter().sum::<f32>() / count_as_f32(values.len())
}

/// The P3 hold rules: nothing leaves the dish, speed stays bounded and does
/// not grow, kinetic energy does not grow and nothing is flung.
pub(super) fn assert_hold_is_bounded(report: &HoldReport) {
    assert_eq!(report.max_out_of_dish, 0, "{report:?}");
    assert_eq!(report.max_escaped, 0, "{report:?}");
    assert!(report.max_speed <= MAX_HOLD_SPEED, "{report:?}");
    assert!(
        report.late_max_speed <= report.early_max_speed + SPEED_GROWTH_ALLOWANCE,
        "{report:?}"
    );
    assert!(
        report.second_half_mean_energy
            <= KE_GROWTH_FACTOR * report.first_half_mean_energy + KE_GROWTH_ALLOWANCE,
        "{report:?}"
    );
    assert!(report.max_isolated <= MAX_ISOLATED, "{report:?}");
}

/// Advances a session `steps` fixed steps in `MAX_ADVANCE_STEPS` chunks.
pub(super) fn advance(session: &mut SessionCore, steps: u32) {
    let mut remaining = steps;
    while remaining > 0 {
        let chunk = remaining.min(MAX_ADVANCE_STEPS);
        session.advance(chunk).expect("the ferrofluid scene steps");
        remaining -= chunk;
    }
}

/// Live particle positions and velocities of a session.
pub(super) fn session_lanes(session: &SessionCore) -> (Vec<Vec2>, Vec<Vec2>) {
    session.read_particles(|world, system| {
        let view = world.particle_system_view(system).expect("liquid");
        (view.positions().to_vec(), view.velocities().to_vec())
    })
}

/// The session's current surface profile.
pub(super) fn profile(session: &SessionCore) -> SurfaceProfile {
    surface_profile(&session_lanes(session).0)
}

/// Advances `every` steps at a time up to `deadline` and returns the first
/// sampled step (counted from the call) at which `predicate` holds.
pub(super) fn first_step_where(
    session: &mut SessionCore,
    deadline: u32,
    every: u32,
    predicate: impl Fn(&SessionCore) -> bool,
) -> Option<u32> {
    let mut step = 0;
    while step + every <= deadline {
        advance(session, every);
        step += every;
        if predicate(session) {
            return Some(step);
        }
    }
    None
}

/// Runs a session one fixed step at a time for `steps`, recording the P3 hold
/// measurements.
pub(super) fn run_session_hold(session: &mut SessionCore, steps: u32) -> HoldReport {
    let mut recorder = HoldRecorder::new();
    for step in 1..=steps {
        advance(session, 1);
        let (placed, lanes) = session_lanes(session);
        recorder.record(step, &placed, &lanes);
    }
    recorder.finish()
}
