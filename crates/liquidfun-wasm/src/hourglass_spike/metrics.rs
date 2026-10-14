//! Pure hourglass spike metrics (Phase 45 D-25), fixed in
//! `45-HOURGLASS-SPIKE.md` before the first measurement.

use liquidfun::math::Vec2;

use super::geometry::{BULB_HALF_WIDTH, LOWER_FLOOR};

/// Deposit particles lie below this height.
const DEPOSIT_MAX_Y: f32 = -0.2;
/// Deposit particles move slower than this (m/s); excludes the stream.
const DEPOSIT_MAX_SPEED: f32 = 0.5;
const BIN_WIDTH: f32 = 0.2;
const BIN_COUNT: usize = 15;
/// A bin is occupied with at least this many deposit particles.
const BIN_MIN_PARTICLES: usize = 3;
/// Jam: `lower` unchanged for more than this many consecutive steps.
const JAM_STEPS: u32 = 60;
pub(super) const PASS_MIN_DEGREES: f32 = 20.0;
pub(super) const PASS_BAND_DEGREES: f32 = 5.0;
pub(super) const WATER_MARGIN_DEGREES: f32 = 15.0;

/// Nearest-rank percentile: the value at 1-based rank `⌈fraction · N⌉`
/// of the sorted values (rank at least 1). `NaN` for no values.
pub(super) fn nearest_rank(values: &[f32], fraction: f32) -> f32 {
    if values.is_empty() {
        return f32::NAN;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    let rank = ((fraction * sorted.len() as f32).ceil() as usize).clamp(1, sorted.len());
    sorted[rank - 1]
}

/// One occupied 0.2 m deposit bin.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Bin {
    index: usize,
    /// Highest deposit particle y in the bin.
    top: f32,
    /// x of that highest particle.
    top_x: f32,
}

impl Bin {
    fn left_edge(self) -> f32 {
        -BULB_HALF_WIDTH + self.index as f32 * BIN_WIDTH
    }

    fn center(self) -> f32 {
        self.left_edge() + 0.5 * BIN_WIDTH
    }
}

/// Pile shape readings for one sample; every field is in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Pile {
    /// Edge-relative slope (decision metric).
    pub(super) slope: f32,
    /// Literal D-25 reading against the lower-bulb floor (informational).
    pub(super) literal: f32,
    /// Mean least-squares side slope (informational).
    pub(super) side: f32,
}

impl Pile {
    const FLAT: Self = Self {
        slope: 0.0,
        literal: 0.0,
        side: 0.0,
    };
}

fn is_deposit(position: Vec2, velocity: Vec2) -> bool {
    position.y < DEPOSIT_MAX_Y && velocity.length() < DEPOSIT_MAX_SPEED
}

fn occupied_bins(deposit: &[Vec2]) -> Vec<Bin> {
    let mut counts = [0_usize; BIN_COUNT];
    let mut tops: [Option<Vec2>; BIN_COUNT] = [None; BIN_COUNT];
    for position in deposit {
        let offset = (position.x + BULB_HALF_WIDTH) / BIN_WIDTH;
        if !(0.0..BIN_COUNT as f32).contains(&offset) {
            continue;
        }
        let index = offset as usize;
        counts[index] += 1;
        if tops[index].is_none_or(|top| position.y > top.y) {
            tops[index] = Some(*position);
        }
    }
    (0..BIN_COUNT)
        .filter(|&index| counts[index] >= BIN_MIN_PARTICLES)
        .filter_map(|index| {
            tops[index].map(|top| Bin {
                index,
                top: top.y,
                top_x: top.x,
            })
        })
        .collect()
}

fn degrees(rise: f32, run: f32) -> f32 {
    if run <= 0.0 {
        return 0.0;
    }
    (rise / run).atan().to_degrees()
}

/// Least-squares slope of `(x, y)` points; `None` below two points.
fn fit_slope(points: &[(f32, f32)]) -> Option<f32> {
    if points.len() < 2 {
        return None;
    }
    let count = points.len() as f32;
    let mean_x = points.iter().map(|point| point.0).sum::<f32>() / count;
    let mean_y = points.iter().map(|point| point.1).sum::<f32>() / count;
    let covariance: f32 = points
        .iter()
        .map(|point| (point.0 - mean_x) * (point.1 - mean_y))
        .sum();
    let variance: f32 = points.iter().map(|point| (point.0 - mean_x).powi(2)).sum();
    (variance > 0.0).then(|| covariance / variance)
}

/// Mean `|atan|` of the least-squares slopes of the bin tops left and right
/// of the highest bin (each side includes the highest bin).
fn side_fit_deg(bins: &[Bin]) -> f32 {
    let Some(highest) = bins
        .iter()
        .enumerate()
        .fold(None::<(usize, Bin)>, |best, (position, bin)| match best {
            Some((_, kept)) if kept.top >= bin.top => best,
            _ => Some((position, *bin)),
        })
        .map(|(position, _)| position)
    else {
        return 0.0;
    };
    let points: Vec<(f32, f32)> = bins.iter().map(|bin| (bin.center(), bin.top)).collect();
    let sides: Vec<f32> = [&points[..=highest], &points[highest..]]
        .into_iter()
        .filter_map(fit_slope)
        .map(|slope| slope.atan().abs().to_degrees())
        .collect();
    if sides.is_empty() {
        return 0.0;
    }
    sides.iter().sum::<f32>() / sides.len() as f32
}

/// Pile readings for the lower-bulb deposit (rules in the spike doc).
pub(super) fn pile(positions: &[Vec2], velocities: &[Vec2]) -> Pile {
    let deposit: Vec<Vec2> = positions
        .iter()
        .zip(velocities)
        .filter(|(position, velocity)| is_deposit(**position, **velocity))
        .map(|(position, _)| *position)
        .collect();
    let bins = occupied_bins(&deposit);
    let (Some(first), Some(last)) = (bins.first(), bins.last()) else {
        return Pile::FLAT;
    };
    let tops: Vec<f32> = bins.iter().map(|bin| bin.top).collect();
    let peak = nearest_rank(&tops, 0.95);
    let edge = 0.5 * (first.top + last.top);
    let half_run = 0.5 * (last.top_x - first.top_x);
    let base_width = last.left_edge() + BIN_WIDTH - first.left_edge();
    let deposit_ys: Vec<f32> = deposit.iter().map(|position| position.y).collect();
    let literal_height = nearest_rank(&deposit_ys, 0.95) - LOWER_FLOOR;
    Pile {
        slope: degrees(peak - edge, half_run),
        literal: degrees(literal_height, 0.5 * base_width),
        side: side_fit_deg(&bins),
    }
}

/// Streaming D-25 jam detector, fed once per step.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct JamTracker {
    maybe_last_lower: Option<usize>,
    stalled_steps: u32,
    maybe_jam_step: Option<u32>,
}

impl JamTracker {
    /// Records the counts after `step`; returns the first jam step, if any.
    pub(super) fn observe(&mut self, step: u32, upper: usize, lower: usize) -> Option<u32> {
        let unchanged = self.maybe_last_lower == Some(lower);
        self.stalled_steps = if unchanged && upper > 0 {
            self.stalled_steps + 1
        } else {
            0
        };
        self.maybe_last_lower = Some(lower);
        if self.stalled_steps > JAM_STEPS && self.maybe_jam_step.is_none() {
            self.maybe_jam_step = Some(step);
        }
        self.maybe_jam_step
    }

    pub(super) const fn maybe_jam_step(&self) -> Option<u32> {
        self.maybe_jam_step
    }
}

/// D-25 verdict over the 25-30 s window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Window {
    pub(super) median: f32,
    pub(super) min: f32,
    pub(super) max: f32,
    pub(super) pass: bool,
}

/// Median ≥ 20°, every sample within ±5° of it and no jam; with
/// `maybe_water_median`, the median must also beat water by 15°.
pub(super) fn window_verdict(
    readings: &[f32],
    jammed: bool,
    maybe_water_median: Option<f32>,
) -> Window {
    let median = nearest_rank(readings, 0.5);
    let min = readings.iter().copied().fold(f32::INFINITY, f32::min);
    let max = readings.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let steady = readings
        .iter()
        .all(|reading| (reading - median).abs() <= PASS_BAND_DEGREES);
    let beats_water = maybe_water_median.is_none_or(|water| median - water >= WATER_MARGIN_DEGREES);
    Window {
        median,
        min,
        max,
        pass: !readings.is_empty()
            && median >= PASS_MIN_DEGREES
            && steady
            && !jammed
            && beats_water,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRID: f32 = 0.075;

    /// Columns across the lower bulb, each filled downward from `top(x)` to
    /// the floor at grid spacing, at rest.
    fn deposit(top: impl Fn(f32) -> f32) -> (Vec<Vec2>, Vec<Vec2>) {
        let mut positions = Vec::new();
        for column in -20..20 {
            let x = (column as f32 + 0.5) * GRID;
            let mut y = top(x);
            while y > LOWER_FLOOR {
                positions.push(Vec2::new(x, y));
                y -= GRID;
            }
        }
        let velocities = vec![Vec2::new(0.0, 0.0); positions.len()];
        (positions, velocities)
    }

    #[test]
    fn flat_bin_tops_give_zero_slope() {
        // Arrange
        let (positions, velocities) = deposit(|_| -2.5);

        // Act
        let reading = pile(&positions, &velocities);

        // Assert
        assert!(reading.slope.abs() < 1e-3, "{reading:?}");
    }

    #[test]
    fn symmetric_twenty_degree_triangle_gives_twenty_degrees() {
        // Arrange
        let tangent = 20.0_f32.to_radians().tan();
        let (positions, velocities) = deposit(|x| -2.0 - tangent * x.abs());

        // Act
        let reading = pile(&positions, &velocities);

        // Assert
        assert!((reading.slope - 20.0).abs() <= 1.0, "{reading:?}");
    }

    #[test]
    fn plan_formula_reads_the_twenty_degree_triangle_low() {
        // Arrange: the plan's run (half the outer-edge base width) with the
        // same rise, kept to document the pre-measurement correction.
        let tangent = 20.0_f32.to_radians().tan();
        let (positions, _) = deposit(|x| -2.0 - tangent * x.abs());
        let bins = occupied_bins(&positions);
        let tops: Vec<f32> = bins.iter().map(|bin| bin.top).collect();
        let rise = nearest_rank(&tops, 0.95) - 0.5 * (tops[0] + tops[tops.len() - 1]);

        // Act
        let plan_degrees = degrees(rise, 0.5 * 3.0);

        // Assert
        assert!(plan_degrees < 19.0, "{plan_degrees}");
    }

    #[test]
    fn stalled_lower_for_sixty_one_steps_is_a_jam() {
        // Arrange
        let mut tracker = JamTracker::default();
        tracker.observe(0, 100, 7);

        // Act
        let maybe_jam = (1..=61).fold(None, |_, step| tracker.observe(step, 100, 7));

        // Assert
        assert_eq!(maybe_jam, Some(61));
    }

    #[test]
    fn stalled_lower_for_sixty_steps_is_not_a_jam() {
        // Arrange
        let mut tracker = JamTracker::default();
        tracker.observe(0, 100, 7);

        // Act
        let maybe_jam = (1..=60).fold(None, |_, step| tracker.observe(step, 100, 7));

        // Assert
        assert_eq!(maybe_jam, None);
    }

    #[test]
    fn an_empty_upper_bulb_never_jams() {
        // Arrange
        let mut tracker = JamTracker::default();

        // Act
        let maybe_jam = (0..=200).fold(None, |_, step| tracker.observe(step, 0, 600));

        // Assert
        assert_eq!(maybe_jam, None);
    }

    #[test]
    fn a_steady_pile_must_beat_water_by_fifteen_degrees() {
        // Arrange
        let readings = [24.0_f32; 21];

        // Act
        let against_flat_water = window_verdict(&readings, false, Some(2.0));
        let against_steep_water = window_verdict(&readings, false, Some(12.0));

        // Assert
        assert!(against_flat_water.pass);
        assert!(!against_steep_water.pass);
    }

    #[test]
    fn a_reading_outside_five_degrees_of_the_median_fails() {
        // Arrange
        let mut readings = [24.0_f32; 21];
        readings[20] = 29.5;

        // Act
        let window = window_verdict(&readings, false, Some(0.0));

        // Assert
        assert!(!window.pass);
    }
}
